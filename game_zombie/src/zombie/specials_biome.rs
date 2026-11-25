//! Pure biome-special zombie behaviours (frozen, husk, bloater) and toxic cloud tracking.

use glam::Vec3;
use serde::Deserialize;

use super::specials::{
    CrawlerConfig, SpecialConfig, SpecialDirective, SpecialKind, SpecialPose, SpecialState, TickCtx,
};

// new: DECISIONS 2026-11-25
const CLOUD_EXPIRE_EPS: f32 = 1e-4;

/// Tuning numbers for the frozen biome zombie (`data/special_zombies.json` `frozen`).
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FrozenConfig {
    pub chill_on_bite: f32,
    pub speed_mult: f32,
    pub health_mult: f32,
}

/// Tuning numbers for the desert husk biome zombie (`data/special_zombies.json` `husk`).
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HuskConfig {
    pub lurk_range: f32,
    pub lunge_trigger_range: f32,
    pub lunge_speed: f32,
    pub lunge_seconds: f32,
    pub hit_range: f32,
    pub lunge_damage: f32,
    pub lunge_chance: f32,
    pub recover_seconds: f32,
    pub retry_seconds: f32,
    pub speed_mult: f32,
    pub health_mult: f32,
}

impl HuskConfig {
    /// Converts husk lunge numbers into a [`CrawlerConfig`] so crawler lunge logic is reused directly.
    pub fn to_crawler_config(&self) -> CrawlerConfig {
        CrawlerConfig {
            lurk_range: self.lurk_range,
            lunge_trigger_range: self.lunge_trigger_range,
            lunge_speed: self.lunge_speed,
            lunge_seconds: self.lunge_seconds,
            hit_range: self.hit_range,
            lunge_damage: self.lunge_damage,
            lunge_chance: self.lunge_chance,
            recover_seconds: self.recover_seconds,
            retry_seconds: self.retry_seconds,
        }
    }
}

/// Tuning numbers for the swamp bloater biome zombie (`data/special_zombies.json` `bloater`).
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BloaterConfig {
    pub cloud_radius: f32,
    pub cloud_seconds: f32,
    pub player_dps: f32,
    pub zombie_dps: f32,
    pub speed_mult: f32,
    pub health_mult: f32,
    pub burst_on_fire: bool,
}

/// Extra status effects carried when a special zombie's melee bite connects.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BiteEffect {
    /// Warmth reduction applied to the player on a frozen zombie bite.
    pub chill: Option<f32>,
}

/// One-frame toxic burst emitted when a bloater dies or is hit by fire.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ToxicBurst {
    pub pos: Vec3,
    pub radius: f32,
    pub seconds: f32,
    pub player_dps: f32,
    pub zombie_dps: f32,
}

/// Special zombie events reported outside per-frame movement directives.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SpecialEvent {
    ToxicBurst(ToxicBurst),
}

/// Computes extra effects when a zombie of `kind` lands a melee bite.
pub fn bite_effect(kind: SpecialKind, cfg: &SpecialConfig) -> BiteEffect {
    match kind {
        SpecialKind::Frozen => BiteEffect {
            chill: Some(cfg.frozen.chill_on_bite),
        },
        _ => BiteEffect::default(),
    }
}

impl SpecialState {
    /// Husk reuses the crawler lunge state machine with husk's own tuning numbers.
    pub(super) fn tick_husk(
        &mut self,
        cfg: &HuskConfig,
        ctx: TickCtx,
        roll: f32,
    ) -> SpecialDirective {
        let lunge_cfg = cfg.to_crawler_config();
        let mut directive = self.tick_crawler(&lunge_cfg, ctx, roll);
        if directive.pose == SpecialPose::Lurk {
            directive.pose = SpecialPose::Normal;
            directive.velocity = None;
        }
        directive
    }

    /// Returns extra effects when this zombie's melee attack lands (e.g., frozen chill).
    pub fn on_attack_landed(&self, cfg: &SpecialConfig) -> BiteEffect {
        bite_effect(self.kind(), cfg)
    }

    /// Handles a hit at `pos`, returning any triggered [`SpecialEvent`] (such as bloater burst).
    pub fn on_hit_at(
        &mut self,
        cfg: &SpecialConfig,
        pos: Vec3,
        killed: bool,
        is_fire: bool,
    ) -> Option<SpecialEvent> {
        if killed {
            self.on_hit(true);
        }
        if self.kind() == SpecialKind::Bloater
            && !self.burst_emitted
            && (killed || (is_fire && cfg.bloater.burst_on_fire))
        {
            self.burst_emitted = true;
            return Some(SpecialEvent::ToxicBurst(ToxicBurst {
                pos,
                radius: cfg.bloater.cloud_radius,
                seconds: cfg.bloater.cloud_seconds,
                player_dps: cfg.bloater.player_dps,
                zombie_dps: cfg.bloater.zombie_dps,
            }));
        }
        None
    }

    /// Handles death at `pos`, returning a [`ToxicBurst`] once if this zombie is a bloater.
    pub fn on_death(&mut self, cfg: &SpecialConfig, pos: Vec3) -> Option<ToxicBurst> {
        self.on_hit_at(cfg, pos, true, false).map(|SpecialEvent::ToxicBurst(burst)| burst)
    }
}

/// One active toxic cloud spawned by a bloater burst.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ToxicCloud {
    pub pos: Vec3,
    pub radius: f32,
    pub remaining: f32,
    pub duration: f32,
    pub player_dps: f32,
    pub zombie_dps: f32,
}

impl ToxicCloud {
    /// Remaining lifetime fraction in `0.0..=1.0`.
    pub fn alpha(&self) -> f32 {
        if self.duration <= 0.0 {
            0.0
        } else {
            (self.remaining / self.duration).clamp(0.0, 1.0)
        }
    }

    /// True if `point` lies within this cloud's horizontal radius.
    pub fn contains(&self, point: Vec3) -> bool {
        if !point.is_finite() || self.radius <= 0.0 {
            return false;
        }
        let dx = point.x - self.pos.x;
        let dz = point.z - self.pos.z;
        dx * dx + dz * dz <= self.radius * self.radius
    }
}

/// Pure list of active toxic clouds with spawn, expiry, and DPS lookup.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ToxicClouds {
    clouds: Vec<ToxicCloud>,
}

impl ToxicClouds {
    pub fn new() -> Self {
        Self { clouds: Vec::new() }
    }

    /// Spawns a new cloud from a [`ToxicBurst`] event.
    pub fn spawn(&mut self, burst: ToxicBurst) {
        if !burst.pos.is_finite() || burst.radius <= 0.0 || burst.seconds <= 0.0 {
            return;
        }
        self.clouds.push(ToxicCloud {
            pos: burst.pos,
            radius: burst.radius,
            remaining: burst.seconds,
            duration: burst.seconds,
            player_dps: burst.player_dps.max(0.0),
            zombie_dps: burst.zombie_dps.max(0.0),
        });
    }

    /// Advances cloud lifetimes by `dt` seconds and removes expired clouds.
    pub fn update(&mut self, dt: f32) {
        if !dt.is_finite() || dt <= 0.0 {
            return;
        }
        for cloud in &mut self.clouds {
            cloud.remaining -= dt;
        }
        self.clouds.retain(|cloud| cloud.remaining > CLOUD_EXPIRE_EPS);
    }

    /// Returns `(player_dps, zombie_dps)` at `pos`, taking the highest DPS among overlapping clouds.
    pub fn damage_at(&self, pos: Vec3) -> (f32, f32) {
        let mut player_dps = 0.0_f32;
        let mut zombie_dps = 0.0_f32;
        for cloud in &self.clouds {
            if cloud.contains(pos) {
                player_dps = player_dps.max(cloud.player_dps);
                zombie_dps = zombie_dps.max(cloud.zombie_dps);
            }
        }
        (player_dps, zombie_dps)
    }

    /// Read-only slice of active clouds for rendering.
    pub fn active(&self) -> &[ToxicCloud] {
        &self.clouds
    }

    pub fn len(&self) -> usize {
        self.clouds.len()
    }

    pub fn is_empty(&self) -> bool {
        self.clouds.is_empty()
    }

    pub fn clear(&mut self) {
        self.clouds.clear();
    }
}
