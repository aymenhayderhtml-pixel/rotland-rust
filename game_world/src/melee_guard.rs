use glam::Vec3;
use serde::Deserialize;

use crate::melee::MeleeTarget;
use game_player::player::Stamina;

/// Below this length a horizontal direction is treated as zero. // new: DECISIONS 2026-11-24
const EPS: f32 = 0.001;

/// Tuning parameters for melee shove.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShoveConfig {
    pub range: f32,
    pub arc_deg: f32,
    pub max_targets: usize,
    pub stamina_cost: f32,
    pub push: f32,
    pub stagger_seconds: f32,
    pub cooldown: f32,
    pub clip: String,
    pub clip_speed: f32,
    pub immune_kinds: Vec<String>,
}

impl Default for ShoveConfig {
    fn default() -> Self {
        Self {
            range: 1.8,                             // data: melee_guard.json range
            arc_deg: 100.0,                         // data: melee_guard.json arcDeg
            max_targets: 3,                         // data: melee_guard.json maxTargets
            stamina_cost: 15.0,                     // data: melee_guard.json staminaCost
            push: 4.0,                              // data: melee_guard.json push
            stagger_seconds: 0.9,                   // data: melee_guard.json staggerSeconds
            cooldown: 0.8,                          // data: melee_guard.json cooldown
            clip: "shove".to_string(),              // data: melee_guard.json clip
            clip_speed: 1.0,                        // data: melee_guard.json clipSpeed
            immune_kinds: vec!["brute".to_string()], // data: melee_guard.json immuneKinds
        }
    }
}

/// Tuning parameters for melee block.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockConfig {
    pub arc_deg: f32,
    pub damage_scale: f32,
    pub stamina_per_hit: f32,
    pub drain_per_second: f32,
    pub min_stamina: f32,
    pub break_seconds: f32,
    pub hit_clip: String,
}

impl Default for BlockConfig {
    fn default() -> Self {
        Self {
            arc_deg: 120.0,                   // data: melee_guard.json arcDeg
            damage_scale: 0.3,                // data: melee_guard.json damageScale
            stamina_per_hit: 12.0,            // data: melee_guard.json staminaPerHit
            drain_per_second: 3.0,            // data: melee_guard.json drainPerSecond
            min_stamina: 5.0,                 // data: melee_guard.json minStamina
            break_seconds: 1.5,               // data: melee_guard.json breakSeconds
            hit_clip: "block_hit".to_string(), // data: melee_guard.json hitClip
        }
    }
}

/// Tuning parameters for melee shove and block.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GuardConfig {
    pub version: u32,
    pub tap_seconds: f32,
    pub shove: ShoveConfig,
    pub block: BlockConfig,
}

impl Default for GuardConfig {
    fn default() -> Self {
        Self {
            version: 1,       // data: melee_guard.json version
            tap_seconds: 0.2, // data: melee_guard.json tapSeconds
            shove: ShoveConfig::default(),
            block: BlockConfig::default(),
        }
    }
}

impl GuardConfig {
    /// Parses a guard configuration from a JSON string.
    pub fn from_json(s: &str) -> Result<Self, String> {
        serde_json::from_str(s).map_err(|e| format!("invalid melee_guard.json: {e}"))
    }

    /// Loads `data/melee_guard.json` from the repository root.
    pub fn from_repo() -> Result<Self, String> {
        let path = paths::repo_path("data/melee_guard.json");
        let raw = std::fs::read_to_string(&path)
            .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
        Self::from_json(&raw)
    }
}

/// Per-frame guard input from the player.
#[derive(Clone, Copy, Debug, Default)]
pub struct GuardInput {
    pub held: bool,
    pub can_guard: bool,
}

/// Event emitted by `GuardState::update`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GuardEvent {
    None,
    Shove,
    BlockStart,
    BlockEnd,
    GuardBroken,
}

/// Player melee shove and block state.
#[derive(Clone, Debug, Default)]
pub struct GuardState {
    held_time: f32,
    blocking: bool,
    shove_cooldown: f32,
    break_left: f32,
    was_held: bool,
}

impl GuardState {
    /// One frame. Press starts timing; held past `tap_seconds` starts blocking (if stamina >=
    /// `min_stamina` and not guard-broken); released before `tap_seconds` shoves (pays stamina,
    /// starts cooldown) if allowed. Blocking drains stamina and resets `stamina.time_since_use`;
    /// reaching 0 triggers `GuardBroken`. `can_guard` false ends any active block and starts nothing.
    pub fn update(
        &mut self,
        dt: f32,
        input: GuardInput,
        stamina: &mut Stamina,
        cfg: &GuardConfig,
    ) -> GuardEvent {
        let dt = if dt > 0.0 && dt.is_finite() { dt } else { 0.0 }; // new: DECISIONS 2026-11-24
        self.shove_cooldown = (self.shove_cooldown - dt).max(0.0);
        self.break_left = (self.break_left - dt).max(0.0);

        if !input.can_guard {
            self.held_time = 0.0;
            self.was_held = false;
            if self.blocking {
                self.blocking = false;
                return GuardEvent::BlockEnd;
            }
            return GuardEvent::None;
        }

        if input.held {
            if self.blocking {
                stamina.current = (stamina.current - cfg.block.drain_per_second * dt).max(0.0);
                stamina.time_since_use = 0.0;
                self.was_held = true;
                if stamina.current <= 0.0 {
                    self.blocking = false;
                    self.break_left = cfg.block.break_seconds;
                    self.held_time = 0.0;
                    return GuardEvent::GuardBroken;
                }
                return GuardEvent::None;
            }
            if !self.was_held {
                self.held_time = 0.0;
            }
            self.held_time += dt;
            self.was_held = true;
            if self.held_time >= cfg.tap_seconds
                && self.break_left <= 0.0
                && stamina.current >= cfg.block.min_stamina
            {
                self.blocking = true;
                return GuardEvent::BlockStart;
            }
            return GuardEvent::None;
        }

        if self.blocking {
            self.blocking = false;
            self.held_time = 0.0;
            self.was_held = false;
            return GuardEvent::BlockEnd;
        }

        if self.was_held {
            let was_tap = self.held_time < cfg.tap_seconds;
            self.held_time = 0.0;
            self.was_held = false;
            if was_tap
                && self.shove_cooldown <= 0.0
                && stamina.current >= cfg.shove.stamina_cost
            {
                stamina.current -= cfg.shove.stamina_cost;
                stamina.time_since_use = 0.0;
                self.shove_cooldown = cfg.shove.cooldown;
                return GuardEvent::Shove;
            }
            return GuardEvent::None;
        }

        self.held_time = 0.0;
        GuardEvent::None
    }

    /// True while the player is actively holding a block.
    pub fn is_blocking(&self) -> bool {
        self.blocking
    }

    /// Filters one zombie bite on the player. Returns `(damage_to_take, blocked)`.
    /// Blocked only when blocking and the attacker is inside the front arc (flat, Godot yaw
    /// forward = `(-sin yaw, 0, -cos yaw)`); unknown attacker is not blocked.
    pub fn filter_bite(
        &mut self,
        damage: f32,
        player_pos: Vec3,
        player_yaw: f32,
        attacker: Option<Vec3>,
        stamina: &mut Stamina,
        cfg: &GuardConfig,
    ) -> (f32, bool) {
        if !self.blocking {
            return (damage, false);
        }
        let Some(attacker_pos) = attacker else {
            return (damage, false);
        };
        let (sy, cy) = player_yaw.sin_cos();
        let forward = Vec3::new(-sy, 0.0, -cy); // new: DECISIONS 2026-11-24
        let offset = Vec3::new(
            attacker_pos.x - player_pos.x,
            0.0,
            attacker_pos.z - player_pos.z,
        );
        let dist = offset.length();
        let cos_half_arc = (cfg.block.arc_deg * 0.5).to_radians().cos(); // new: DECISIONS 2026-11-24
        if dist > EPS && forward.dot(offset / dist) < cos_half_arc {
            return (damage, false);
        }

        if stamina.current < cfg.block.stamina_per_hit {
            stamina.current = 0.0;
            stamina.time_since_use = 0.0;
            self.blocking = false;
            self.break_left = cfg.block.break_seconds;
            return (damage, false);
        }

        stamina.current -= cfg.block.stamina_per_hit;
        stamina.time_since_use = 0.0;
        if stamina.current <= 0.0 {
            self.blocking = false;
            self.break_left = cfg.block.break_seconds;
        }
        (damage * cfg.block.damage_scale, true)
    }
}

/// Targets a shove reaches, nearest first, at most `max_targets`: pushes `(id, push_velocity)`
/// into `out` (cleared first, no allocation per frame).
pub fn shove_targets(
    position: Vec3,
    facing: Vec3,
    targets: &[MeleeTarget],
    cfg: &GuardConfig,
    out: &mut Vec<(u64, Vec3)>,
) {
    out.clear();
    if cfg.shove.max_targets == 0 {
        return;
    }
    let dir = Vec3::new(facing.x, 0.0, facing.z).normalize_or_zero();
    if dir.length_squared() <= EPS * EPS {
        return;
    }
    let cos_half_arc = (cfg.shove.arc_deg * 0.5).to_radians().cos(); // new: DECISIONS 2026-11-24

    for _ in 0..cfg.shove.max_targets {
        let mut best: Option<(f32, u64, Vec3)> = None;
        for target in targets {
            if out.iter().any(|&(id, _)| id == target.id) {
                continue;
            }
            let offset = Vec3::new(
                target.position.x - position.x,
                0.0,
                target.position.z - position.z,
            );
            let dist = offset.length();
            if dist - target.radius > cfg.shove.range {
                continue;
            }
            let away = if dist > EPS { offset / dist } else { dir };
            if dist > EPS && dir.dot(away) < cos_half_arc {
                continue;
            }
            if best.is_none_or(|(best_dist, _, _)| dist < best_dist) {
                best = Some((dist, target.id, away * cfg.shove.push));
            }
        }
        let Some((_, id, push)) = best else {
            break;
        };
        out.push((id, push));
    }
}
