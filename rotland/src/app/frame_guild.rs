//! Guild task hooks in the world (R24-N6): fetch hand-ins at the guild, explore places from the real map,
//! the "ready to claim" notice and the tracker rows. The pure rules live in game::guild::task_hooks.

use game::guild::{
    GuildManager, GuildTaskEvent, GuildTaskHookConfig, GuildTaskHooks, NamedPlace, TaskRow,
    tracker_rows,
};
use game::Inventory;
use glam::{Vec2, Vec3};

use super::state::{PlaySide, WorldState};
use crate::hud::{Hud, NotificationKind};

/// Explore place names, as written in data/guild.json `place_name`.
pub const ASHFORD_CORE: &str = "Ashford core"; // data: guild.json explore_ashford
pub const THE_LAKE: &str = "the lake"; // data: guild.json explore_lake
pub const WATCHTOWER: &str = "watchtower"; // data: guild.json explore_watchtower
/// Building type of a watchtower in data/poi.json (a scatter POI's `building_id`; its own id is `poi_<x>_<z>`).
const WATCHTOWER_BUILDING: &str = "watchtower"; // data: poi.json watchtower
/// Tracker rows are rebuilt at most this often. // new: DECISIONS 2026-11-23 (task spec: 0.5 s)
const ROW_REFRESH_SECONDS: f32 = 0.5; // new: DECISIONS 2026-11-23

/// Tracker rows and named places, kept between frames so nothing is rebuilt every frame.
#[derive(Default)]
pub struct GuildTaskPanel {
    /// Rows the HUD tracker draws (at most three).
    pub rows: Vec<TaskRow>,
    age: f32,
    refresh: bool,
    places: Vec<NamedPlace<'static>>,
    places_built: bool,
    places_generation: Option<u64>,
}

impl GuildTaskPanel {
    /// Forgets the rows and the places, so the next frame rebuilds them.
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Forces a row rebuild on the next frame (after a hand-in or other guild change).
    pub fn request_refresh(&mut self) {
        self.refresh = true;
    }

    /// Rebuilds the named places from the world when the loaded chunks changed (or on first use).
    fn refresh_places(&mut self, world: &WorldState) {
        let generation = world.streamer.as_ref().map(|s| s.generation);
        if self.places_built && self.places_generation == generation {
            return;
        }
        self.places = build_places(world);
        self.places_generation = generation;
        self.places_built = true;
    }
}

/// Named places for the explore objectives, built from the real world: the city core, the lake shore
/// (first dock, like the dev panel's "Lake" spot) and every watchtower in the loaded chunks.
fn build_places(world: &WorldState) -> Vec<NamedPlace<'static>> {
    let mut places = Vec::new();
    if let Some(wgen) = world.world_gen.as_deref() {
        if let Some(city) = wgen.city() {
            places.push(NamedPlace { name: ASHFORD_CORE, position: flat(game::city_center(city)) });
        }
        if let Some(lake) = wgen.lake() {
            if lake.docks.is_empty() && lake.shore.is_empty() {
                places.push(NamedPlace { name: THE_LAKE, position: flat(lake.center) });
            } else {
                for dock in &lake.docks {
                    places.push(NamedPlace { name: THE_LAKE, position: flat(dock.start) });
                }
                for &shore_pt in &lake.shore {
                    places.push(NamedPlace { name: THE_LAKE, position: flat(shore_pt) });
                }
            }
        }
    }
    if let Some(streamer) = world.streamer.as_ref() {
        for chunk in streamer.chunks.values() {
            for poi in chunk.pois.iter().filter(|p| p.building_id == WATCHTOWER_BUILDING) {
                places.push(NamedPlace { name: WATCHTOWER, position: poi.position });
            }
        }
    }
    places
}

fn flat(p: Vec2) -> Vec3 {
    Vec3::new(p.x, 0.0, p.y)
}

/// Loads the guild task tuning. A failure logs a warning and turns the hooks off (never a crash).
pub fn load_hooks() -> Option<GuildTaskHooks> {
    match GuildTaskHookConfig::from_repo() {
        Ok(cfg) => Some(GuildTaskHooks::new(cfg)),
        Err(e) => {
            log::warn!("guild task hooks not loaded: {e}");
            None
        }
    }
}

/// A new game or a loaded save: the hooks forget their timer and announced tasks, the tracker starts empty.
pub fn reset(side: &mut PlaySide) {
    clear_hooks(&mut side.guild_hooks);
    side.guild_panel.reset();
}

/// Death: the hooks forget their timer and announced tasks.
pub fn clear_hooks(hooks: &mut Option<GuildTaskHooks>) {
    if let Some(h) = hooks.as_mut() {
        h.clear();
    }
}

/// Once a frame: explore places and clear/ready checks (throttled inside the hooks), then the tracker rows.
pub fn update(side: &mut PlaySide, world: &WorldState, player: Vec3, dt: f32, hud: &mut Hud) {
    let PlaySide { play: play_state, guild_hooks, guild_panel, .. } = side;
    guild_panel.refresh_places(world);
    let Some(pl) = play_state.as_mut() else {
        guild_panel.rows.clear();
        return;
    };
    let towns = world.world_gen.as_deref().map_or(&[][..], |g| g.towns());
    let events = match guild_hooks.as_mut() {
        Some(hooks) => hooks.tick_with_towns(pl.camp.guild.manager_mut(), &guild_panel.places, towns, player, dt),
        None => Vec::new(),
    };
    guild_panel.age += dt;
    if !events.is_empty() || guild_panel.refresh || guild_panel.age >= ROW_REFRESH_SECONDS {
        guild_panel.rows = tracker_rows(pl.camp.guild.manager());
        guild_panel.age = 0.0;
        guild_panel.refresh = false;
    }
    announce(&events, hud);
}

/// Shows a notice for each task that just became ready to claim. Ordinary progress shows nothing.
pub fn announce(events: &[GuildTaskEvent], hud: &mut Hud) {
    for event in events {
        if let GuildTaskEvent::Ready { .. } = event {
            hud.notify("Task ready: return to the guild", NotificationKind::Info);
        }
    }
}

/// Hands in what one fetch task still needs from the bag. Only that task is served; the items are
/// removed exactly as planned. Returns the status line and the events.
pub fn hand_in(
    hooks: &mut GuildTaskHooks,
    manager: &mut GuildManager,
    bag: &mut Inventory,
    guild_pos: Vec3,
    player: Vec3,
    task_id: &str,
) -> (String, Vec<GuildTaskEvent>) {
    if !hooks.is_in_hand_in_range(guild_pos, player) {
        return ("Stand closer to the guild to hand in items".into(), Vec::new());
    }
    let Some(entry) = hooks.plan_hand_in_for(manager, task_id, |id| bag.count(id), guild_pos, player) else {
        return ("You don't have the items needed for this task".into(), Vec::new());
    };
    if !bag.remove(&entry.item_id, entry.count) {
        return ("You don't have the items needed for this task".into(), Vec::new());
    }
    let item = entry.item_id.replace('_', " ");
    let count = entry.count;
    let events = hooks.apply_hand_in(manager, &[entry]);
    let ready = events.iter().any(|e| matches!(e, GuildTaskEvent::Ready { .. }));
    let msg = if ready {
        format!("Handed in {count} {item} — ready to claim!")
    } else {
        format!("Handed in {count} {item}")
    };
    (msg, events)
}
