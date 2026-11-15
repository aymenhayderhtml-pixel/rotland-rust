//! The zombie's brain (port of ZombieAI.cs, the movement in ZombieMotion.cs and the hit rules in Zombie.cs).
//! Pure: it decides where to walk, when to bite and how a hit changes things. It never moves the body.
//! The caller moves the body by `BrainOutput::velocity`, turns it to `yaw` and applies the damage of
//! `attack_hit`. Scream, eat, bang, herd and horde are TODO(R3-07): their states exist but are never entered.

use std::f32::consts::{PI, TAU};

use glam::Vec3;
use physics::PhysicsWorld;

use super::attack::{AttackHit, ZombieAttack};
use super::defs::{ZombieBehaviourDef, ZombieDef};
use super::rng::Rng;
use super::senses::{Noise, Senses};
use super::state::ZombieState;
use crate::damage::DamageInfo;

/// An idle pause lasts a random time in this range, in seconds.
const IDLE_MIN: f32 = 1.0;
const IDLE_MAX: f32 = 3.0;
/// A wander spot is this far from where the wander starts (ZombieNavigation: 2 to 8 m).
const WANDER_MIN: f32 = 2.0;
const WANDER_MAX: f32 = 8.0;
/// How fast the walk eases toward its wish, per second (ZombieMotion.SpeedBlendRate). The easing is
/// why a zombie does not change speed in one frame.
const SPEED_BLEND_RATE: f32 = 8.0;
/// How fast the body turns toward where it looks, per second (ZombieMotion.TurnSpeed).
const TURN_SPEED: f32 = 8.0;
/// Walk speeds below this count as standing still for the animation.
const MOVING_SPEED: f32 = 0.1;

/// What the caller knows this frame. The brain reads it and returns a `BrainOutput`.
pub struct BrainInput<'a> {
    /// Feet on the ground.
    pub position: Vec3,
    /// Body yaw in radians, as Godot's Rotation.Y (0 faces -Z).
    pub yaw: f32,
    /// Not read by the brain: gravity belongs to the body. Passed so callers send what the brief asks.
    pub on_floor: bool,
    /// The target's feet, or None when there is no living target.
    pub target: Option<Vec3>,
    /// Sounds made this frame. Each one is checked against this zombie's hearing.
    pub heard: &'a [Noise],
    /// Flat unit direction toward the target from the flow field, or None when it has none.
    pub flow_dir: Option<Vec3>,
    /// A hit that landed this frame. The caller has already applied its damage.
    pub took_damage: Option<DamageInfo>,
    /// Health is zero. Death is final for this brain.
    pub is_dead: bool,
    /// Walls block sight through this world.
    pub world: &'a PhysicsWorld,
}

/// Which body clip should play. The caller maps it to the animation name.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimClip {
    Idle,
    Walk,
    Run,
    Attack,
    /// Played on a flinch and during a stagger (the stagger clip in Godot).
    Hit,
    Die,
}

/// What the body should do this frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BrainOutput {
    /// Horizontal walk velocity in m/s, already eased toward the wish.
    pub velocity: Vec3,
    /// Yaw to turn the body to this frame, in radians.
    pub yaw: f32,
    pub state: ZombieState,
    /// A bite that landed this frame. The caller applies `damage` to the target.
    pub attack_hit: Option<AttackHit>,
    pub anim: AnimClip,
    /// True while a strong hit has stopped the zombie.
    pub staggered: bool,
}

/// The numbers the brain reads from the zombie type and the shared behaviour block. Copied once so the
/// brain holds no references to the data.
#[derive(Clone, Copy, Debug)]
pub(super) struct Numbers {
    pub(super) wander_speed: f32,
    pub(super) chase_speed: f32,
    pub(super) attack_range: f32,
    pub(super) hearing_multiplier: f32,
    pub(super) strong_hit_damage: f32,
    pub(super) stagger_seconds: f32,
    pub(super) look_around_seconds: f32,
}

/// What one frame of thinking asks for before the easing.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Step {
    /// Walk velocity the zombie wants (flat), before easing.
    pub(super) wish: Vec3,
    pub(super) hit: Option<AttackHit>,
}

/// One zombie's brain. Build with `new`, then call `update` once per frame.
#[derive(Clone, Debug)]
pub struct ZombieBrain {
    pub(super) state: ZombieState,
    pub(super) numbers: Numbers,
    home: Vec3,
    pub(super) senses: Senses,
    pub(super) attack: ZombieAttack,
    rng: Rng,
    /// Seconds spent in the current state.
    pub(super) time_in_state: f32,
    /// Countdown of the current state: an idle pause, or the look-around.
    pub(super) timer: f32,
    /// Investigate: true once at the spot, looking about.
    pub(super) looking_around: bool,
    /// Where a wander or an investigate walk is heading.
    pub(super) goal: Vec3,
    /// Where the body turns to face this frame, if not simply its walk.
    pub(super) look_point: Option<Vec3>,
    /// The eased walk velocity.
    walk_velocity: Vec3,
    /// Seconds of stagger left after a strong hit.
    pub(super) stagger_timer: f32,
}

impl ZombieBrain {
    /// A fresh zombie of type `def`, idle at `home`. `tuning` is the shared `behaviour` block of
    /// data/zombies.json. The same `seed` always gives the same wander path.
    pub fn new(def: &ZombieDef, tuning: &ZombieBehaviourDef, home: Vec3, seed: u64) -> Self {
        let numbers = Numbers {
            wander_speed: def.wander_speed,
            chase_speed: def.chase_speed,
            attack_range: def.attack_range,
            hearing_multiplier: def.hearing_multiplier,
            strong_hit_damage: tuning.strong_hit_damage,
            stagger_seconds: tuning.stagger_fallback_seconds,
            look_around_seconds: tuning.look_around_seconds,
        };
        let mut brain = Self {
            state: ZombieState::Idle,
            numbers,
            home,
            senses: Senses::new(def),
            attack: ZombieAttack::new(def),
            rng: Rng::new(seed),
            time_in_state: 0.0,
            timer: 0.0,
            looking_around: false,
            goal: home,
            look_point: None,
            walk_velocity: Vec3::ZERO,
            stagger_timer: 0.0,
        };
        brain.enter_idle();
        brain
    }

    pub fn state(&self) -> ZombieState {
        self.state
    }

    /// Where the zombie was placed. Kept for the guard and horde rules (R3-07).
    pub fn home(&self) -> Vec3 {
        self.home
    }

    /// Advances the brain by `dt` seconds and returns what the body should do this frame.
    pub fn update(&mut self, ctx: &BrainInput<'_>, dt: f32) -> BrainOutput {
        // Death is final. A dead zombie stops at once (C# Kill zeroes the wish).
        if ctx.is_dead || self.state == ZombieState::Dead {
            self.enter_dead();
            return BrainOutput {
                velocity: Vec3::ZERO,
                yaw: ctx.yaw,
                state: ZombieState::Dead,
                attack_hit: None,
                anim: AnimClip::Die,
                staggered: false,
            };
        }

        self.stagger_timer = (self.stagger_timer - dt).max(0.0);
        let flinched = match &ctx.took_damage {
            Some(info) => self.on_hit(info, ctx.target),
            None => false,
        };
        if let Some(spot) = self.last_audible(ctx) {
            self.hear_noise(spot);
        }
        self.senses
            .update(dt, ctx.position, forward(ctx.yaw), ctx.target, ctx.world);

        // A staggered zombie does not think (C# skips Think), but its walk still eases to a stop
        let step = if self.is_staggered() {
            Step::default()
        } else {
            self.think(ctx, dt)
        };

        let blend = 1.0 - (-SPEED_BLEND_RATE * dt).exp();
        self.walk_velocity = self.walk_velocity.lerp(step.wish, blend);
        let yaw = self.turn(ctx, dt);
        BrainOutput {
            velocity: self.walk_velocity,
            yaw,
            state: self.state,
            attack_hit: step.hit,
            anim: self.anim_clip(flinched),
            staggered: self.is_staggered(),
        }
    }

    fn is_staggered(&self) -> bool {
        self.stagger_timer > 0.0
    }

    /// One frame of the state machine (ZombieAI.Think).
    fn think(&mut self, ctx: &BrainInput<'_>, dt: f32) -> Step {
        self.time_in_state += dt;
        // A chase or attack with no target gives up and wanders (C# Think)
        if ctx.target.is_none() && matches!(self.state, ZombieState::Chase | ZombieState::Attack) {
            self.enter_wander(ctx.position);
            return Step::default();
        }
        match self.state {
            ZombieState::Idle => self.think_idle(ctx, dt),
            ZombieState::Wander => self.think_wander(ctx),
            ZombieState::Investigate => self.think_investigate(ctx, dt),
            ZombieState::Chase => self.think_chase(ctx),
            ZombieState::Attack => self.think_attack(ctx, dt),
            // TODO(R3-07): Scream, Eat and Bang (ZombieAI.Actions.cs). Never entered yet.
            ZombieState::Scream | ZombieState::Eat | ZombieState::Bang | ZombieState::Dead => {
                Step::default()
            }
        }
    }

    /// Turns the body toward its look point, else toward its walk (ZombieMotion.TurnBody). Returns the new yaw.
    fn turn(&self, ctx: &BrainInput<'_>, dt: f32) -> f32 {
        let look = match self.look_point {
            Some(p) => p - ctx.position,
            None => self.walk_velocity,
        };
        let flat = Vec3::new(look.x, 0.0, look.z);
        // Standing still with nothing to look at: keep the yaw
        if flat.length_squared() < 1e-4 {
            return ctx.yaw;
        }
        let target = (-flat.x).atan2(-flat.z);
        let blend = 1.0 - (-TURN_SPEED * dt).exp();
        ctx.yaw + wrap_angle(target - ctx.yaw) * blend
    }

    fn anim_clip(&self, flinched: bool) -> AnimClip {
        if self.is_staggered() || flinched {
            return AnimClip::Hit;
        }
        let moving = self.walk_velocity.length() > MOVING_SPEED;
        match self.state {
            ZombieState::Dead => AnimClip::Die,
            ZombieState::Attack => AnimClip::Attack,
            ZombieState::Chase if moving => AnimClip::Run,
            ZombieState::Wander | ZombieState::Investigate if moving => AnimClip::Walk,
            _ => AnimClip::Idle,
        }
    }

    fn enter(&mut self, state: ZombieState) {
        self.state = state;
        self.time_in_state = 0.0;
        self.looking_around = false;
        self.look_point = None;
    }

    pub(super) fn enter_idle(&mut self) {
        self.enter(ZombieState::Idle);
        self.timer = self.rng.range(IDLE_MIN, IDLE_MAX);
    }

    /// Picks a random spot 2 to 8 m from `from` and walks there.
    pub(super) fn enter_wander(&mut self, from: Vec3) {
        self.enter(ZombieState::Wander);
        let angle = self.rng.range(0.0, TAU);
        let dist = self.rng.range(WANDER_MIN, WANDER_MAX);
        self.goal = from + Vec3::new(angle.cos(), 0.0, angle.sin()) * dist;
    }

    pub(super) fn enter_investigate(&mut self, spot: Vec3) {
        self.enter(ZombieState::Investigate);
        self.goal = spot;
    }

    pub(super) fn enter_chase(&mut self) {
        self.enter(ZombieState::Chase);
    }

    pub(super) fn enter_attack(&mut self) {
        self.enter(ZombieState::Attack);
        self.attack.start();
    }

    fn enter_dead(&mut self) {
        self.enter(ZombieState::Dead);
        self.walk_velocity = Vec3::ZERO;
    }
}

/// The way a zombie with this yaw faces (flat, unit). Yaw 0 faces -Z, as in Godot.
fn forward(yaw: f32) -> Vec3 {
    Vec3::new(-yaw.sin(), 0.0, -yaw.cos())
}

/// Wraps an angle into (-PI, PI], so a turn always takes the short way round.
fn wrap_angle(a: f32) -> f32 {
    (a + PI).rem_euclid(TAU) - PI
}
