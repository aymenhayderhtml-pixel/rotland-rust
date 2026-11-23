//! Pure gameplay hooks for advancing guild fetch/explore tasks and tracking readiness.

use glam::{Vec2, Vec3};
use serde::Deserialize;

use super::manager::GuildManager;
use super::tasks::GuildTaskKind;

const DEFAULT_HAND_IN_RADIUS: f32 = 6.0; // new: DECISIONS 2026-11-23
const DEFAULT_PLACE_RADIUS: f32 = 12.0; // new: DECISIONS 2026-11-23
const DEFAULT_HOOK_CHECK_SECONDS: f32 = 0.5; // data: guild.json hook_check_seconds
const MAX_TRACKER_ROWS: usize = 3; // data: guild.json max_active

fn default_hand_in_radius() -> f32 {
    DEFAULT_HAND_IN_RADIUS
}

fn default_place_radius() -> f32 {
    DEFAULT_PLACE_RADIUS
}

fn default_hook_check_seconds() -> f32 {
    DEFAULT_HOOK_CHECK_SECONDS
}

/// Distances and polling cadence for guild task hooks, read from `data/guild.json`.
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct GuildTaskHookConfig {
    #[serde(default = "default_hand_in_radius")]
    pub hand_in_radius: f32,
    #[serde(default = "default_place_radius")]
    pub place_radius: f32,
    #[serde(default = "default_hook_check_seconds")]
    pub hook_check_seconds: f32,
}

impl Default for GuildTaskHookConfig {
    fn default() -> Self {
        Self {
            hand_in_radius: DEFAULT_HAND_IN_RADIUS,
            place_radius: DEFAULT_PLACE_RADIUS,
            hook_check_seconds: DEFAULT_HOOK_CHECK_SECONDS,
        }
    }
}

impl GuildTaskHookConfig {
    /// Reads the three top-level hook keys from `guild.json`, ignoring all other keys.
    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }

    /// Loads task-hook configuration from `data/guild.json` in the repository.
    pub fn from_repo() -> Result<Self, std::io::Error> {
        let path = paths::repo_path("data/guild.json");
        let text = std::fs::read_to_string(path)?;
        Self::from_json(&text)
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))
    }
}

/// A named world location used to evaluate explore objectives.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NamedPlace<'a> {
    pub name: &'a str,
    pub position: Vec3,
}

/// One planned item hand-in targeted at a single active fetch task.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HandIn {
    pub task_id: String,
    pub item_id: String,
    pub count: u32,
}

/// Events emitted by guild task hooks when progress advances or a task becomes ready to claim.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GuildTaskEvent {
    Progress {
        task_id: String,
        progress: u32,
        target: u32,
    },
    Ready {
        task_id: String,
    },
}

/// Stateful throttle and one-shot ready-announcement tracker for active guild tasks.
#[derive(Clone, Debug, PartialEq)]
pub struct GuildTaskHooks {
    cfg: GuildTaskHookConfig,
    check_timer: f32,
    announced_ready: Vec<String>,
}

impl GuildTaskHooks {
    /// Creates hooks with the supplied configuration and zeroed state.
    pub fn new(cfg: GuildTaskHookConfig) -> Self {
        Self {
            cfg,
            check_timer: 0.0,
            announced_ready: Vec::new(),
        }
    }

    /// Death or load: forget the timer and the announced ids.
    pub fn clear(&mut self) {
        self.check_timer = 0.0;
        self.announced_ready.clear();
    }

    /// Pure plan, nothing is changed. Empty when the player is farther than `hand_in_radius` from `guild_pos`.
    /// `count` returns how many of an item the bag holds. Walks the active tasks in order.
    pub fn plan_hand_in(
        &self,
        manager: &GuildManager,
        count: impl Fn(&str) -> u32,
        guild_pos: Vec3,
        player: Vec3,
    ) -> Vec<HandIn> {
        if !within_horizontal_radius(player, guild_pos, self.cfg.hand_in_radius) {
            return Vec::new();
        }

        let mut plan: Vec<HandIn> = Vec::new();
        for task in manager.board().active() {
            if task.is_done() {
                continue;
            }
            let GuildTaskKind::Fetch { item_id } = &task.kind else {
                continue;
            };
            let need = task.target.saturating_sub(task.progress);
            if need == 0 {
                continue;
            }
            let reserved = plan
                .iter()
                .filter(|entry| entry.item_id == *item_id)
                .fold(0_u32, |acc, entry| acc.saturating_add(entry.count));
            let bag_left = count(item_id).saturating_sub(reserved);
            let take = need.min(bag_left);
            if take > 0 {
                plan.push(HandIn {
                    task_id: task.id.clone(),
                    item_id: item_id.clone(),
                    count: take,
                });
            }
        }
        plan
    }

    /// Call AFTER the caller removed each `HandIn`'s items from the bag.
    /// Applies progress per task and returns events (`Ready` included).
    pub fn apply_hand_in(
        &mut self,
        manager: &mut GuildManager,
        plan: &[HandIn],
    ) -> Vec<GuildTaskEvent> {
        let mut events = Vec::new();
        for entry in plan {
            if entry.count == 0 {
                continue;
            }
            for task in &mut manager.board_mut().active {
                if task.id != entry.task_id {
                    continue;
                }
                let prev = task.progress;
                if task.on_item_delivered(&entry.item_id, entry.count) && task.progress != prev {
                    events.push(GuildTaskEvent::Progress {
                        task_id: task.id.clone(),
                        progress: task.progress,
                        target: task.target,
                    });
                }
                break;
            }
        }
        self.collect_ready_events(manager, &mut events);
        events
    }

    /// Throttled: does nothing until `hook_check_seconds` has passed (then subtracts it).
    /// Checks explore objectives and `Ready` notifications.
    pub fn tick(
        &mut self,
        manager: &mut GuildManager,
        places: &[NamedPlace<'_>],
        player: Vec3,
        dt: f32,
    ) -> Vec<GuildTaskEvent> {
        if !dt.is_finite() || dt <= 0.0 {
            return Vec::new();
        }
        let interval = self.cfg.hook_check_seconds;
        if interval.is_finite() && interval > 0.0 {
            self.check_timer += dt;
            if self.check_timer < interval {
                return Vec::new();
            }
            self.check_timer -= interval;
            if self.check_timer >= interval {
                self.check_timer = 0.0;
            }
        }

        let mut events = Vec::new();
        for task in &mut manager.board_mut().active {
            if task.is_done() {
                continue;
            }
            let matched_name = match &task.kind {
                GuildTaskKind::Explore { place_name } => places
                    .iter()
                    .find(|place| {
                        place.name.eq_ignore_ascii_case(place_name)
                            && within_horizontal_radius(
                                player,
                                place.position,
                                self.cfg.place_radius,
                            )
                    })
                    .map(|place| place.name),
                _ => None,
            };
            if let Some(name) = matched_name {
                let prev = task.progress;
                if task.on_place_reached(name) && task.progress != prev {
                    events.push(GuildTaskEvent::Progress {
                        task_id: task.id.clone(),
                        progress: task.progress,
                        target: task.target,
                    });
                }
            }
        }
        self.collect_ready_events(manager, &mut events);
        events
    }

    fn collect_ready_events(&mut self, manager: &GuildManager, events: &mut Vec<GuildTaskEvent>) {
        let active = manager.board().active();
        self.announced_ready
            .retain(|id| active.iter().any(|task| task.id == *id && task.is_done()));
        for task in active {
            if task.is_done() && !self.announced_ready.iter().any(|id| id == &task.id) {
                self.announced_ready.push(task.id.clone());
                events.push(GuildTaskEvent::Ready {
                    task_id: task.id.clone(),
                });
            }
        }
    }
}

/// Compact display snapshot of one active guild task for the HUD tracker.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskRow {
    pub title: String,
    pub progress: u32,
    pub target: u32,
    pub ready: bool,
}

/// One row per active task, in board order, at most 3.
/// Reads the board only, so abandoned tasks vanish by themselves.
pub fn tracker_rows(manager: &GuildManager) -> Vec<TaskRow> {
    manager
        .board()
        .active()
        .iter()
        .take(MAX_TRACKER_ROWS)
        .map(|task| TaskRow {
            title: task.title.clone(),
            progress: task.progress,
            target: task.target,
            ready: task.is_done(),
        })
        .collect()
}

fn horizontal_pos(v: Vec3) -> Vec2 {
    Vec2::new(v.x, v.z)
}

fn within_horizontal_radius(a: Vec3, b: Vec3, radius: f32) -> bool {
    if radius < 0.0 || !radius.is_finite() {
        return false;
    }
    let dist_sq = horizontal_pos(a).distance_squared(horizontal_pos(b));
    dist_sq.is_finite() && dist_sq <= radius * radius
}
