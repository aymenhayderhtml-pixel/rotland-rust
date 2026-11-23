//! Pure state machines for special zombie behaviours (brute, screamer, crawler).

use glam::Vec3;
use serde::Deserialize;

// new: DECISIONS 2026-11-23
const DIR_EPSILON: f32 = 1e-4;
// new: DECISIONS 2026-11-23
const TIMER_EPS_FRACTION: f64 = 1e-2;

/// The three special zombie archetypes that carry a [`SpecialState`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpecialKind {
    Brute,
    Screamer,
    Crawler,
}

impl SpecialKind {
    /// Returns the special kind for `"brute"`, `"screamer"`, or `"crawler"`; walkers and runners return `None`.
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "brute" => Some(Self::Brute),
            "screamer" => Some(Self::Screamer),
            "crawler" => Some(Self::Crawler),
            _ => None,
        }
    }
}

/// Visual pose hint produced by [`SpecialState::tick`] for animation mapping.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SpecialPose {
    #[default]
    Normal,
    Windup,
    Charge,
    Stunned,
    Lurk,
    Lunge,
    Recover,
    Scream,
    Retreat,
}

/// One-frame melee/impact hit emitted when a brute charge or crawler lunge connects.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpecialHit {
    pub amount: f32,
    /// Horizontal knockback vector in m/s (same units as combat knockback).
    pub knockback: Vec3,
}

/// One-frame horde call emitted when a screamer finishes its wind-up.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpecialCall {
    pub target: Vec3,
    pub radius: f32,
    pub max_zombies: u32,
}

/// Per-frame output from [`SpecialState::tick`]. Plain `Copy`, zero allocations.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SpecialDirective {
    /// Horizontal velocity override in m/s; `None` lets the normal brain decide.
    pub velocity: Option<Vec3>,
    /// Unit horizontal facing direction; `None` lets the normal brain decide.
    pub face: Option<Vec3>,
    pub pose: SpecialPose,
    /// Set only on the single tick a special hit lands.
    pub damage: Option<SpecialHit>,
    /// Set only on the single tick a screamer call is emitted.
    pub call: Option<SpecialCall>,
    pub stunned: bool,
    /// True while open to extra damage (stunned brute or recovering crawler).
    pub vulnerable: bool,
}

/// Tuning numbers loaded from `data/special_zombies.json`.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpecialConfig {
    pub vulnerable_multiplier: f32,
    pub brute: BruteConfig,
    pub screamer: ScreamerConfig,
    pub crawler: CrawlerConfig,
}

/// Tuning numbers for the brute charge behaviour.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BruteConfig {
    pub charge_min_range: f32,
    pub charge_max_range: f32,
    pub windup_seconds: f32,
    pub charge_seconds: f32,
    pub charge_speed: f32,
    pub turn_rate_deg: f32,
    pub hit_range: f32,
    pub hit_damage: f32,
    pub knockback_speed: f32,
    pub stun_seconds: f32,
    pub cooldown_seconds: f32,
}

/// Tuning numbers for the screamer call and retreat behaviour.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ScreamerConfig {
    pub trigger_range: f32,
    pub windup_seconds: f32,
    pub call_radius: f32,
    pub call_max_zombies: u32,
    pub call_cooldown_seconds: f32,
    pub retreat_seconds: f32,
    pub retreat_speed: f32,
}

/// Tuning numbers for the crawler lurk, lunge and recovery behaviour.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CrawlerConfig {
    pub lurk_range: f32,
    pub lunge_trigger_range: f32,
    pub lunge_speed: f32,
    pub lunge_seconds: f32,
    pub hit_range: f32,
    pub lunge_damage: f32,
    pub lunge_chance: f32,
    pub recover_seconds: f32,
    pub retry_seconds: f32,
}

impl SpecialConfig {
    /// Parses `special_zombies.json` text.
    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }

    /// Loads `data/special_zombies.json` from the repository root.
    pub fn from_repo() -> Result<Self, std::io::Error> {
        let text = std::fs::read_to_string(paths::repo_path("data/special_zombies.json"))?;
        Self::from_json(&text)
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Idle,
    BruteWindup,
    BruteCharge,
    BruteStunned,
    ScreamerWindup,
    ScreamerScream,
    ScreamerRetreat,
    CrawlerLunge,
    CrawlerRecover,
}

/// Per-zombie special behaviour state, allocated only for [`SpecialKind`] zombies.
#[derive(Clone, Copy, Debug)]
pub struct SpecialState {
    kind: SpecialKind,
    phase: Phase,
    timer: f64,
    cooldown: f64,
    dir: Vec3,
    hit_done: bool,
    killed: bool,
}

#[derive(Clone, Copy)]
struct TickCtx {
    player: Vec3,
    to_player: Vec3,
    dist: f32,
    los: bool,
    hit_wall: bool,
    dt: f32,
    dt_f64: f64,
    eps: f64,
}

impl SpecialState {
    /// Creates a fresh state locked to `kind`.
    pub fn new(kind: SpecialKind) -> Self {
        Self {
            kind,
            phase: Phase::Idle,
            timer: 0.0,
            cooldown: 0.0,
            dir: Vec3::ZERO,
            hit_done: false,
            killed: false,
        }
    }

    /// Returns the special archetype this state belongs to.
    pub fn kind(&self) -> SpecialKind {
        self.kind
    }

    /// Notifies the special state of an incoming hit; lethal hits cancel an active screamer wind-up.
    pub fn on_hit(&mut self, killed: bool) {
        if !killed {
            return;
        }
        self.killed = true;
        self.phase = Phase::Idle;
        self.timer = 0.0;
    }

    /// Advances the special state machine by `dt` seconds without allocating or panicking.
    #[allow(clippy::too_many_arguments)]
    pub fn tick(
        &mut self,
        cfg: &SpecialConfig,
        own: Vec3,
        player: Vec3,
        los: bool,
        hit_wall: bool,
        dt: f32,
        roll: f32,
    ) -> SpecialDirective {
        if self.killed {
            return SpecialDirective::default();
        }
        let dt = if dt.is_finite() && dt > 0.0 { dt } else { 0.0 };
        let dt_f64 = f64::from(dt);
        let eps = dt_f64 * TIMER_EPS_FRACTION;
        if self.cooldown > 0.0 {
            self.cooldown = (self.cooldown - dt_f64).max(0.0);
            if self.cooldown <= eps {
                self.cooldown = 0.0;
            }
        }
        let offset = horizontal_offset(own, player);
        let dist = offset.length();
        let to_player = if dist < DIR_EPSILON {
            Vec3::ZERO
        } else {
            offset / dist
        };
        let ctx = TickCtx {
            player,
            to_player,
            dist,
            los,
            hit_wall,
            dt,
            dt_f64,
            eps,
        };
        match self.kind {
            SpecialKind::Brute => self.tick_brute(&cfg.brute, ctx),
            SpecialKind::Screamer => self.tick_screamer(&cfg.screamer, ctx),
            SpecialKind::Crawler => self.tick_crawler(&cfg.crawler, ctx, roll),
        }
    }

    fn tick_brute(&mut self, cfg: &BruteConfig, ctx: TickCtx) -> SpecialDirective {
        match self.phase {
            Phase::Idle => {
                if ctx.los
                    && self.cooldown <= 0.0
                    && ctx.dist >= cfg.charge_min_range
                    && ctx.dist <= cfg.charge_max_range
                {
                    self.phase = Phase::BruteWindup;
                    self.timer = f64::from(cfg.windup_seconds);
                    return self.step_brute_windup(cfg, ctx);
                }
                SpecialDirective::default()
            }
            Phase::BruteWindup => self.step_brute_windup(cfg, ctx),
            Phase::BruteCharge => self.step_brute_charge(cfg, ctx),
            Phase::BruteStunned => self.step_brute_stunned(ctx),
            _ => SpecialDirective::default(),
        }
    }

    fn step_brute_windup(&mut self, cfg: &BruteConfig, ctx: TickCtx) -> SpecialDirective {
        self.timer -= ctx.dt_f64;
        if self.timer <= ctx.eps {
            self.dir = ctx.to_player;
            self.hit_done = false;
            self.phase = Phase::BruteCharge;
            self.timer = f64::from(cfg.charge_seconds);
        }
        SpecialDirective {
            velocity: Some(Vec3::ZERO),
            face: opt_dir(ctx.to_player),
            pose: SpecialPose::Windup,
            ..SpecialDirective::default()
        }
    }

    fn step_brute_charge(&mut self, cfg: &BruteConfig, ctx: TickCtx) -> SpecialDirective {
        let max_turn = cfg.turn_rate_deg.to_radians() * ctx.dt;
        self.dir = turn_toward(self.dir, ctx.to_player, max_turn);
        if !self.hit_done && ctx.dist <= cfg.hit_range {
            self.hit_done = true;
            self.phase = Phase::Idle;
            self.timer = 0.0;
            self.cooldown = f64::from(cfg.cooldown_seconds);
            let kb_dir = if ctx.to_player != Vec3::ZERO {
                ctx.to_player
            } else {
                self.dir
            };
            return SpecialDirective {
                velocity: Some(Vec3::ZERO),
                face: opt_dir(self.dir),
                pose: SpecialPose::Charge,
                damage: Some(SpecialHit {
                    amount: cfg.hit_damage,
                    knockback: kb_dir * cfg.knockback_speed,
                }),
                ..SpecialDirective::default()
            };
        }
        if ctx.hit_wall {
            self.phase = Phase::BruteStunned;
            self.timer = f64::from(cfg.stun_seconds);
            self.cooldown = f64::from(cfg.cooldown_seconds);
            return self.step_brute_stunned(ctx);
        }
        self.timer -= ctx.dt_f64;
        if self.timer <= ctx.eps {
            self.phase = Phase::BruteStunned;
            self.timer = f64::from(cfg.stun_seconds);
            self.cooldown = f64::from(cfg.cooldown_seconds);
        }
        SpecialDirective {
            velocity: Some(self.dir * cfg.charge_speed),
            face: opt_dir(self.dir),
            pose: SpecialPose::Charge,
            ..SpecialDirective::default()
        }
    }

    fn step_brute_stunned(&mut self, ctx: TickCtx) -> SpecialDirective {
        self.timer -= ctx.dt_f64;
        if self.timer <= ctx.eps {
            self.phase = Phase::Idle;
            self.timer = 0.0;
        }
        SpecialDirective {
            velocity: Some(Vec3::ZERO),
            pose: SpecialPose::Stunned,
            stunned: true,
            vulnerable: true,
            ..SpecialDirective::default()
        }
    }

    fn tick_screamer(&mut self, cfg: &ScreamerConfig, ctx: TickCtx) -> SpecialDirective {
        match self.phase {
            Phase::Idle => {
                if ctx.los && self.cooldown <= 0.0 && ctx.dist <= cfg.trigger_range {
                    self.phase = Phase::ScreamerWindup;
                    self.timer = f64::from(cfg.windup_seconds);
                    return self.step_screamer_windup(ctx);
                }
                SpecialDirective::default()
            }
            Phase::ScreamerWindup => self.step_screamer_windup(ctx),
            Phase::ScreamerScream => {
                self.cooldown = f64::from(cfg.call_cooldown_seconds);
                self.phase = Phase::ScreamerRetreat;
                self.timer = f64::from(cfg.retreat_seconds);
                SpecialDirective {
                    velocity: Some(Vec3::ZERO),
                    face: opt_dir(ctx.to_player),
                    pose: SpecialPose::Scream,
                    call: Some(SpecialCall {
                        target: ctx.player,
                        radius: cfg.call_radius,
                        max_zombies: cfg.call_max_zombies,
                    }),
                    ..SpecialDirective::default()
                }
            }
            Phase::ScreamerRetreat => {
                let away = if ctx.to_player == Vec3::ZERO {
                    Vec3::ZERO
                } else {
                    -ctx.to_player
                };
                self.timer -= ctx.dt_f64;
                if self.timer <= ctx.eps {
                    self.phase = Phase::Idle;
                    self.timer = 0.0;
                }
                SpecialDirective {
                    velocity: Some(away * cfg.retreat_speed),
                    face: opt_dir(away),
                    pose: SpecialPose::Retreat,
                    ..SpecialDirective::default()
                }
            }
            _ => SpecialDirective::default(),
        }
    }

    fn step_screamer_windup(&mut self, ctx: TickCtx) -> SpecialDirective {
        self.timer -= ctx.dt_f64;
        if self.timer <= ctx.eps {
            self.phase = Phase::ScreamerScream;
            self.timer = 0.0;
        }
        SpecialDirective {
            velocity: Some(Vec3::ZERO),
            face: opt_dir(ctx.to_player),
            pose: SpecialPose::Windup,
            ..SpecialDirective::default()
        }
    }

    fn tick_crawler(&mut self, cfg: &CrawlerConfig, ctx: TickCtx, roll: f32) -> SpecialDirective {
        match self.phase {
            Phase::Idle => {
                if !ctx.los || ctx.dist > cfg.lurk_range {
                    return SpecialDirective {
                        velocity: Some(Vec3::ZERO),
                        pose: SpecialPose::Lurk,
                        ..SpecialDirective::default()
                    };
                }
                if ctx.dist <= cfg.lunge_trigger_range && self.cooldown <= 0.0 {
                    if roll < cfg.lunge_chance {
                        self.dir = ctx.to_player;
                        self.hit_done = false;
                        self.phase = Phase::CrawlerLunge;
                        self.timer = f64::from(cfg.lunge_seconds);
                        return self.step_crawler_lunge(cfg, ctx);
                    }
                    self.cooldown = f64::from(cfg.retry_seconds);
                }
                SpecialDirective::default()
            }
            Phase::CrawlerLunge => self.step_crawler_lunge(cfg, ctx),
            Phase::CrawlerRecover => {
                self.timer -= ctx.dt_f64;
                if self.timer <= ctx.eps {
                    self.phase = Phase::Idle;
                    self.timer = 0.0;
                    self.cooldown = f64::from(cfg.retry_seconds);
                }
                SpecialDirective {
                    velocity: Some(Vec3::ZERO),
                    pose: SpecialPose::Recover,
                    vulnerable: true,
                    ..SpecialDirective::default()
                }
            }
            _ => SpecialDirective::default(),
        }
    }

    fn step_crawler_lunge(&mut self, cfg: &CrawlerConfig, ctx: TickCtx) -> SpecialDirective {
        if !self.hit_done && ctx.dist <= cfg.hit_range {
            self.hit_done = true;
            self.phase = Phase::CrawlerRecover;
            self.timer = f64::from(cfg.recover_seconds);
            return SpecialDirective {
                velocity: Some(Vec3::ZERO),
                face: opt_dir(self.dir),
                pose: SpecialPose::Lunge,
                damage: Some(SpecialHit {
                    amount: cfg.lunge_damage,
                    knockback: Vec3::ZERO,
                }),
                ..SpecialDirective::default()
            };
        }
        self.timer -= ctx.dt_f64;
        if self.timer <= ctx.eps {
            self.phase = Phase::CrawlerRecover;
            self.timer = f64::from(cfg.recover_seconds);
        }
        SpecialDirective {
            velocity: Some(self.dir * cfg.lunge_speed),
            face: opt_dir(self.dir),
            pose: SpecialPose::Lunge,
            ..SpecialDirective::default()
        }
    }
}

fn horizontal_offset(from: Vec3, to: Vec3) -> Vec3 {
    let dx = to.x - from.x;
    let dz = to.z - from.z;
    if dx.is_finite() && dz.is_finite() {
        Vec3::new(dx, 0.0, dz)
    } else {
        Vec3::ZERO
    }
}

fn opt_dir(dir: Vec3) -> Option<Vec3> {
    if dir == Vec3::ZERO { None } else { Some(dir) }
}

fn turn_toward(current: Vec3, target: Vec3, max_turn: f32) -> Vec3 {
    if target.length() < DIR_EPSILON {
        return current;
    }
    if current.length() < DIR_EPSILON {
        return target;
    }
    if !max_turn.is_finite() || max_turn <= 0.0 {
        return current;
    }
    let dot = current.x * target.x + current.z * target.z;
    let cross = current.x * target.z - current.z * target.x;
    let step = cross.atan2(dot).clamp(-max_turn, max_turn);
    let (sin, cos) = step.sin_cos();
    let rotated = Vec3::new(
        current.x * cos - current.z * sin,
        0.0,
        current.x * sin + current.z * cos,
    );
    let len = rotated.length();
    if len < DIR_EPSILON {
        current
    } else {
        rotated / len
    }
}
