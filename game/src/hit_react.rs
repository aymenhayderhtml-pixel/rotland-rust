//! Directional player hit flinches and full-body stagger reactions.

use glam::Vec3;
use serde::Deserialize;

const DIR_EPSILON_SQ: f32 = 1e-6; // new: DECISIONS 2026-11-24
const ZERO_FADE: f32 = 0.0; // new: DECISIONS 2026-11-24

/// Tuning parameters for player hit reactions.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HitReactConfig {
    pub version: u32,
    pub min_gap_seconds: f32,
    pub stagger_knockback: f32,
    pub flinch_speed: f32,
    pub stagger_fade_in: f32,
    pub stagger_fade_out: f32,
}

impl Default for HitReactConfig {
    fn default() -> Self {
        Self {
            version: 1,              // data: hit_react.json version
            min_gap_seconds: 0.3,    // data: hit_react.json minGapSeconds
            stagger_knockback: 4.0,  // data: hit_react.json staggerKnockback
            flinch_speed: 1.0,       // data: hit_react.json flinchSpeed
            stagger_fade_in: 0.08,   // data: hit_react.json staggerFadeIn
            stagger_fade_out: 0.25,  // data: hit_react.json staggerFadeOut
        }
    }
}

impl HitReactConfig {
    /// Parses a hit reaction configuration from a JSON string.
    pub fn from_json(s: &str) -> Result<Self, String> {
        serde_json::from_str(s).map_err(|e| format!("invalid hit_react.json: {e}"))
    }

    /// Loads `data/hit_react.json` from the repository root.
    pub fn from_repo() -> Result<Self, String> {
        let path = paths::repo_path("data/hit_react.json");
        let raw = std::fs::read_to_string(&path)
            .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
        Self::from_json(&raw)
    }
}

/// Relative horizontal side of the player an attack came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HitSide {
    Front,
    Back,
    Left,
    Right,
}

impl HitSide {
    fn flinch_clip(self) -> &'static str {
        match self {
            Self::Front => "hit_front",
            Self::Back => "hit_back",
            Self::Left => "hit_left",
            Self::Right => "hit_right",
        }
    }
}

/// Side of the player the attacker is on.
/// Uses Godot yaw convention: forward = (-sin yaw, 0, -cos yaw), right = (cos yaw, 0, -sin yaw).
/// Divided into four 90-degree flat sectors; zero distance returns `HitSide::Front`.
pub fn hit_side(player_pos: Vec3, player_yaw: f32, attacker_pos: Vec3) -> HitSide {
    let to_attacker = Vec3::new(
        attacker_pos.x - player_pos.x,
        0.0, // new: DECISIONS 2026-11-24
        attacker_pos.z - player_pos.z,
    );
    if !to_attacker.is_finite()
        || !player_yaw.is_finite()
        || to_attacker.length_squared() < DIR_EPSILON_SQ
    {
        return HitSide::Front;
    }

    let (sy, cy) = player_yaw.sin_cos();
    let forward = Vec3::new(-sy, 0.0, -cy); // new: DECISIONS 2026-11-24
    let right = Vec3::new(cy, 0.0, -sy); // new: DECISIONS 2026-11-24
    let fwd_dot = to_attacker.dot(forward);
    let right_dot = to_attacker.dot(right);

    if fwd_dot.abs() >= right_dot.abs() {
        if fwd_dot >= 0.0 {
            HitSide::Front
        } else {
            HitSide::Back
        }
    } else if right_dot >= 0.0 {
        HitSide::Right
    } else {
        HitSide::Left
    }
}

/// Animation clip and playback parameters selected for a player hit.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HitReaction {
    pub clip: &'static str,
    pub full_body: bool,
    pub speed: f32,
    pub fade_in: f32,
    pub fade_out: f32,
}

/// Per-frame hit context passed into `HitReactState::on_hit`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HitInput {
    pub time: f64,
    pub player_pos: Vec3,
    pub player_yaw: f32,
    pub attacker: Option<Vec3>,
    pub knockback: Vec3,
}

/// Tracks the last flinch timestamp to throttle overlapping normal hits.
#[derive(Clone, Debug, Default)]
pub struct HitReactState {
    last_flinch_time: Option<f64>,
}

impl HitReactState {
    /// Returns the clip to play for this hit, or `None` when inside `min_gap_seconds`
    /// of the previous reaction (big hits always play).
    pub fn on_hit(&mut self, input: &HitInput, cfg: &HitReactConfig) -> Option<HitReaction> {
        let knock_len = input.knockback.length();
        let is_big = knock_len.is_finite() && knock_len >= cfg.stagger_knockback;

        if is_big {
            self.last_flinch_time = Some(input.time);
            let (sy, cy) = input.player_yaw.sin_cos();
            let forward = Vec3::new(-sy, 0.0, -cy); // new: DECISIONS 2026-11-24
            let push_flat = Vec3::new(input.knockback.x, 0.0, input.knockback.z); // new: DECISIONS 2026-11-24
            let push_forward = push_flat.dot(forward);

            if push_forward > 0.0 {
                return Some(HitReaction {
                    clip: "hit_back",
                    full_body: false,
                    speed: cfg.flinch_speed,
                    fade_in: ZERO_FADE,
                    fade_out: ZERO_FADE,
                });
            }
            return Some(HitReaction {
                clip: "stagger_back",
                full_body: true,
                speed: cfg.flinch_speed,
                fade_in: cfg.stagger_fade_in,
                fade_out: cfg.stagger_fade_out,
            });
        }

        if let Some(last) = self.last_flinch_time
            && input.time - last < f64::from(cfg.min_gap_seconds)
        {
            return None;
        }

        self.last_flinch_time = Some(input.time);
        let side = input
            .attacker
            .map_or(HitSide::Front, |pos| hit_side(input.player_pos, input.player_yaw, pos));

        Some(HitReaction {
            clip: side.flinch_clip(),
            full_body: false,
            speed: cfg.flinch_speed,
            fade_in: ZERO_FADE,
            fade_out: ZERO_FADE,
        })
    }
}
