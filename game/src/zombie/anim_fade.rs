//! Per-zombie animation crossfade, speed-matched playback rate, and directional death clips.

use std::collections::BTreeMap;

use glam::Vec3;
use serde::{Deserialize, Serialize};

use super::AnimClip;

const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325; // new: DECISIONS 2026-11-24
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3; // new: DECISIONS 2026-11-24
const DEATH_ANGLE_DEG: f32 = 60.0; // data: zombie_anim.json deathAngleDeg
const DIR_EPSILON_SQ: f32 = 1e-6; // new: DECISIONS 2026-11-24

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZombieAnimConfig {
    pub fade_default: f32,
    pub fade_into_attack: f32,
    pub fade_into_hit: f32,
    pub fade_into_death: f32,
    pub fade_between_locomotion: f32,
    pub min_rate: f32,
    pub max_rate: f32,
    pub blend_max_distance: f32,
    pub death_angle_deg: f32,
    pub ref_speeds: BTreeMap<String, f32>,
}

impl Default for ZombieAnimConfig {
    fn default() -> Self {
        let mut ref_speeds = BTreeMap::new();
        ref_speeds.insert("walk".into(), 1.4); // data: zombie_anim.json refSpeeds.walk
        ref_speeds.insert("zombie_shamble".into(), 1.1); // data: zombie_anim.json refSpeeds.zombie_shamble
        ref_speeds.insert("zombie_lurch".into(), 1.25); // data: zombie_anim.json refSpeeds.zombie_lurch
        ref_speeds.insert("run".into(), 3.2); // data: zombie_anim.json refSpeeds.run
        ref_speeds.insert("zombie_runner_sprint".into(), 4.2); // data: zombie_anim.json refSpeeds.zombie_runner_sprint
        ref_speeds.insert("zombie_crawl".into(), 0.75); // data: zombie_anim.json refSpeeds.zombie_crawl
        ref_speeds.insert("zombie_crawler_crawl".into(), 0.85); // data: zombie_anim.json refSpeeds.zombie_crawler_crawl
        Self {
            fade_default: 0.22, // data: zombie_anim.json fadeDefault
            fade_into_attack: 0.1, // data: zombie_anim.json fadeIntoAttack
            fade_into_hit: 0.08, // data: zombie_anim.json fadeIntoHit
            fade_into_death: 0.12, // data: zombie_anim.json fadeIntoDeath
            fade_between_locomotion: 0.25, // data: zombie_anim.json fadeBetweenLocomotion
            min_rate: 0.6, // data: zombie_anim.json minRate
            max_rate: 1.6, // data: zombie_anim.json maxRate
            blend_max_distance: 25.0, // data: zombie_anim.json blendMaxDistance
            death_angle_deg: DEATH_ANGLE_DEG, // data: zombie_anim.json deathAngleDeg
            ref_speeds,
        }
    }
}

impl ZombieAnimConfig {
    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }

    pub fn from_repo() -> Option<Self> {
        let paths = ["data/zombie_anim.json", "../data/zombie_anim.json"];
        for path in paths {
            if let Ok(text) = std::fs::read_to_string(path)
                && let Ok(cfg) = Self::from_json(&text)
            {
                return Some(cfg);
            }
        }
        None
    }

    pub fn ref_speed(&self, clip: &str) -> Option<f32> {
        self.ref_speeds.get(clip).copied()
    }

    pub fn fade_seconds_for(&self, prev: AnimClip, next: AnimClip) -> f32 {
        match next {
            AnimClip::Die => self.fade_into_death,
            AnimClip::Hit => self.fade_into_hit,
            AnimClip::Attack => self.fade_into_attack,
            AnimClip::Idle | AnimClip::Walk | AnimClip::Run | AnimClip::Crawl
                if matches!(
                    prev,
                    AnimClip::Idle | AnimClip::Walk | AnimClip::Run | AnimClip::Crawl
                ) =>
            {
                self.fade_between_locomotion
            }
            _ => self.fade_default,
        }
    }
}

/// Per-zombie visual animation state (Copy). Clips are identified by a u64 key (hash of the clip name).
#[derive(Clone, Copy, Debug, Default)]
pub struct AnimFade {
    initialized: bool,
    current_key: u64,
    current_time: f32,
    prev_key: u64,
    prev_time: f32,
    fade_left: f32,
    fade_total: f32,
}

impl AnimFade {
    /// Call every frame with the clip chosen this frame. Starts a fade when the key changes.
    pub fn update(&mut self, dt: f32, key: u64, rate: f32, fade_seconds: f32) {
        let dt = if dt.is_finite() && dt > 0.0 { dt } else { 0.0 };
        let rate = if rate.is_finite() && rate > 0.0 { rate } else { 0.0 };
        let fade_seconds = if fade_seconds.is_finite() && fade_seconds > 0.0 {
            fade_seconds
        } else {
            0.0
        };

        if !self.initialized {
            self.initialized = true;
            self.current_key = key;
            self.current_time = dt * rate;
            self.prev_key = 0;
            self.prev_time = 0.0;
            self.fade_left = 0.0;
            self.fade_total = 0.0;
            return;
        }

        if key != self.current_key {
            let (dom_key, dom_time) = match self.blend() {
                Some((pk, pt, w)) if w < 0.5 => (pk, pt), // new: DECISIONS 2026-11-24
                _ => (self.current_key, self.current_time),
            };
            self.prev_key = dom_key;
            self.prev_time = dom_time;
            self.current_key = key;
            self.current_time = 0.0;
            self.fade_total = fade_seconds;
            self.fade_left = fade_seconds;
        }

        self.current_time += dt * rate;
        if self.fade_left > 0.0 {
            self.prev_time += dt;
            self.fade_left = (self.fade_left - dt).max(0.0);
            if self.fade_left == 0.0 {
                self.fade_total = 0.0;
            }
        }
    }

    pub fn time(&self) -> f32 {
        self.current_time
    }

    /// (previous key, previous time, weight of the CURRENT clip 0..1) while fading, else None.
    pub fn blend(&self) -> Option<(u64, f32, f32)> {
        if self.fade_left > 0.0 && self.fade_total > 0.0 {
            let w = (1.0 - self.fade_left / self.fade_total).clamp(0.0, 1.0);
            Some((self.prev_key, self.prev_time, w))
        } else {
            None
        }
    }
}

/// Stable FNV-1a 64-bit hash for clip names.
pub fn clip_key(name: &str) -> u64 {
    let mut hash = FNV_OFFSET_BASIS;
    for b in name.bytes() {
        hash ^= u64::from(b);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

/// Playback rate for a locomotion clip from ground speed (clamped by config).
pub fn locomotion_rate(speed: f32, ref_speed: f32, cfg: &ZombieAnimConfig) -> f32 {
    if !speed.is_finite() || !ref_speed.is_finite() || ref_speed <= 0.0 {
        return 1.0; // new: DECISIONS 2026-11-24
    }
    (speed.max(0.0) / ref_speed).clamp(cfg.min_rate, cfg.max_rate)
}

/// Death clip from the hit direction (flat knockback) and the zombie's yaw (forward = (-sin yaw, 0, -cos yaw)).
pub fn death_clip(knock_dir: Vec3, yaw: f32) -> &'static str {
    let flat = Vec3::new(knock_dir.x, 0.0, knock_dir.z);
    if !flat.is_finite() || !yaw.is_finite() || flat.length_squared() < DIR_EPSILON_SQ {
        return "die";
    }
    let dir = flat.normalize();
    let forward = Vec3::new(-yaw.sin(), 0.0, -yaw.cos());
    let threshold = DEATH_ANGLE_DEG.to_radians().cos();
    let dot = dir.dot(forward);
    if dot <= -threshold {
        "zombie_die_back"
    } else if dot >= threshold {
        "zombie_die_forward"
    } else {
        "die"
    }
}
