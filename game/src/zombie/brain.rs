//! The zombie's brain (port of ZombieAI.cs, ZombieMotion.cs, ZombieWounds.cs, ZombieCrawl.cs).
//! Pure: it decides where to walk, when to bite and how a hit changes things. It never moves the body.
//! The caller moves the body by `BrainOutput::velocity`, turns it to `yaw` and applies the damage of
//! `attack_hit`.

use std::f32::consts::PI;
use std::sync::{Arc, Mutex};

use glam::Vec3;
use physics::PhysicsWorld;

use super::attack::{AttackHit, ZombieAttack};
use super::defs::{ZombieBehaviourDef, ZombieDef};
use super::herd::{HerdZombie, ZombieHorde};
use super::rng::Rng;
use super::senses::{Noise, Senses};
use super::state::ZombieState;
use super::steering::ZombieSteering;
use crate::damage::DamageInfo;

const SPEED_BLEND_RATE: f32 = 8.0;
const TURN_SPEED: f32 = 8.0;
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
    /// Walls block sight and feeler rays through this world.
    pub world: &'a PhysicsWorld,
    /// Other active zombies nearby, so an idle or wandering zombie can join a chase.
    pub nearby_zombies: &'a [HerdZombie],
    /// Recent corpse positions nearby that an idle zombie might stop to eat.
    pub corpses: &'a [Vec3],
    /// Whether the target is on a roof, ladder or high box out of reach.
    pub target_unreachable: bool,
    /// Explicit leg hit flag (or inferred from hit_point / damage info).
    pub leg_hit: bool,
    /// Whether the player is inside the safe zone (Godot ZombiePlayerLink.cs / SealsOff).
    pub player_in_safe_zone: bool,
}

impl<'a> BrainInput<'a> {
    pub fn new(position: Vec3, yaw: f32, world: &'a PhysicsWorld) -> Self {
        Self {
            position,
            yaw,
            on_floor: true,
            target: None,
            heard: &[],
            flow_dir: None,
            took_damage: None,
            is_dead: false,
            world,
            nearby_zombies: &[],
            corpses: &[],
            target_unreachable: false,
            leg_hit: false,
            player_in_safe_zone: false,
        }
    }
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
    /// Spotted the player and screaming to alert others.
    Scream,
    /// Kneeling or leaning at a corpse eating.
    Eat,
    /// Banging at the foot of a ladder or wall.
    Bang,
    /// Crawling on hands and dragging legs.
    Crawl,
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
    /// True on the single frame a scream starts, alerting others and playing sound.
    pub scream: bool,
    /// Last resort displacement when completely stuck.
    pub teleport: Option<Vec3>,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Numbers {
    pub(super) wander_speed: f32,
    pub(super) chase_speed: f32,
    pub(super) attack_range: f32,
    pub(super) hearing_multiplier: f32,
    pub(super) strong_hit_damage: f32,
    pub(super) stagger_seconds: f32,
    pub(super) look_around_seconds: f32,
    pub(super) crawler_speed: f32,
    pub(super) leg_hit_height: f32,
    pub(super) leg_hits_to_crawl: u32,
    pub(super) scream_seconds: f32,
    pub(super) eat_min_seconds: f32,
    pub(super) eat_max_seconds: f32,
    pub(super) eat_random_chance: f32,
    pub(super) eat_near_corpse_chance: f32,
    pub(super) corpse_spot_radius: f32,
    pub(super) herd_radius: f32,
    pub(super) herd_check_seconds: f32,
    pub(super) bang_seconds: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Step {
    pub(super) wish: Vec3,
    pub(super) hit: Option<AttackHit>,
}

/// One zombie's brain. Build with `new`, then call `update` once per frame.
#[derive(Clone, Debug)]
pub struct ZombieBrain {
    pub(super) state: ZombieState,
    pub(super) numbers: Numbers,
    pub(super) home: Vec3,
    pub(super) senses: Senses,
    pub(super) attack: ZombieAttack,
    pub(super) steering: ZombieSteering,
    pub(super) rng: Rng,
    pub(super) time_in_state: f32,
    pub(super) timer: f32,
    pub(super) looking_around: bool,
    pub(super) goal: Vec3,
    pub(super) look_point: Option<Vec3>,
    pub(super) walk_velocity: Vec3,
    pub(super) stagger_timer: f32,
    pub(super) crawler: bool,
    pub(super) leg_hits: u32,
    pub(super) scream_chance: f32,
    pub(super) hideout_override: Option<bool>,
    pub(super) bang_on_arrive: bool,
    pub(super) herd_timer: f32,
    pub(super) horde: Option<Arc<Mutex<ZombieHorde>>>,
    pub(super) just_screamed: bool,
    pub(super) eat_chance_override: Option<f32>,
    pub(super) is_screamer: bool,
    pub(super) has_screamed: bool,
    pub(super) horde_steer: Option<Vec3>,
}

impl ZombieBrain {
    pub fn new(def: &ZombieDef, tuning: &ZombieBehaviourDef, home: Vec3, seed: u64) -> Self {
        let is_screamer = def.models.iter().any(|m| m.contains("screamer"));
        let is_crawler = def.models.iter().any(|m| m.contains("crawler"));
        let numbers = Numbers {
            wander_speed: def.wander_speed,
            chase_speed: def.chase_speed,
            attack_range: def.attack_range,
            hearing_multiplier: def.hearing_multiplier,
            strong_hit_damage: tuning.strong_hit_damage,
            stagger_seconds: tuning.stagger_fallback_seconds,
            look_around_seconds: tuning.look_around_seconds,
            crawler_speed: tuning.crawler_speed,
            leg_hit_height: tuning.leg_hit_height,
            leg_hits_to_crawl: tuning.leg_hits_to_crawl,
            scream_seconds: tuning.scream_seconds,
            eat_min_seconds: tuning.eat_min_seconds,
            eat_max_seconds: tuning.eat_max_seconds,
            eat_random_chance: tuning.eat_random_chance,
            eat_near_corpse_chance: tuning.eat_near_corpse_chance,
            corpse_spot_radius: tuning.corpse_spot_radius,
            herd_radius: tuning.herd_radius,
            herd_check_seconds: tuning.herd_check_seconds,
            bang_seconds: tuning.bang_seconds,
        };
        let steering = ZombieSteering::new(
            seed.wrapping_add(1),
            tuning.stuck_seconds,
            tuning.stuck_distance,
            tuning.escape_seconds,
            tuning.escape_tries_before_teleport,
            tuning.teleport_metres,
            tuning.blocked_memory as usize,
            tuning.blocked_radius,
        );
        let mut brain = Self {
            state: ZombieState::Idle,
            numbers,
            home,
            senses: Senses::new(def),
            attack: ZombieAttack::new(def),
            steering,
            rng: Rng::new(seed),
            time_in_state: 0.0,
            timer: 0.0,
            looking_around: false,
            goal: home,
            look_point: None,
            walk_velocity: Vec3::ZERO,
            stagger_timer: 0.0,
            crawler: is_crawler,
            leg_hits: 0,
            scream_chance: tuning.scream_chance,
            hideout_override: None,
            bang_on_arrive: false,
            herd_timer: 0.0,
            horde: None,
            just_screamed: false,
            eat_chance_override: None,
            is_screamer,
            has_screamed: false,
            horde_steer: None,
        };
        brain.enter_idle();
        brain
    }

    pub fn state(&self) -> ZombieState {
        self.state
    }

    pub fn home(&self) -> Vec3 {
        self.home
    }

    /// Last seen target spot known to this zombie's senses.
    pub fn last_seen(&self) -> Vec3 {
        self.senses.last_seen()
    }

    pub fn update(&mut self, ctx: &BrainInput<'_>, dt: f32) -> BrainOutput {
        self.just_screamed = false;
        if ctx.is_dead || self.state == ZombieState::Dead {
            self.enter_dead();
            return BrainOutput {
                velocity: Vec3::ZERO,
                yaw: ctx.yaw,
                state: ZombieState::Dead,
                attack_hit: None,
                anim: AnimClip::Die,
                staggered: false,
                scream: false,
                teleport: None,
            };
        }

        let effective_target = if ctx.player_in_safe_zone {
            None
        } else {
            ctx.target
        };

        self.stagger_timer = (self.stagger_timer - dt).max(0.0);
        let flinched = match &ctx.took_damage {
            Some(info) => self.on_hit(info, effective_target, ctx.position.y, ctx.leg_hit),
            None => false,
        };
        if let Some(spot) = self.last_audible(ctx) {
            self.hear_noise(spot);
        }
        self.senses
            .update(dt, ctx.position, forward(ctx.yaw), effective_target, ctx.world);

        let step = if self.is_staggered() {
            Step::default()
        } else {
            self.think(ctx, dt)
        };

        let blend = 1.0 - (-SPEED_BLEND_RATE * dt).exp();
        self.walk_velocity = self.walk_velocity.lerp(step.wish, blend);
        let yaw = self.turn(ctx, dt);
        let teleport = self.steering.take_teleport();
        BrainOutput {
            velocity: self.walk_velocity,
            yaw,
            state: self.state,
            attack_hit: step.hit,
            anim: self.anim_clip(flinched, step.wish),
            staggered: self.is_staggered(),
            scream: self.just_screamed,
            teleport,
        }
    }

    fn is_staggered(&self) -> bool {
        self.stagger_timer > 0.0
    }

    fn think(&mut self, ctx: &BrainInput<'_>, dt: f32) -> Step {
        self.time_in_state += dt;
        let effective_target = if ctx.player_in_safe_zone {
            None
        } else {
            ctx.target
        };
        if effective_target.is_none() && matches!(self.state, ZombieState::Chase | ZombieState::Attack) {
            self.enter_wander(ctx.position);
            return Step::default();
        }
        match self.state {
            ZombieState::Idle => self.think_idle(ctx, dt),
            ZombieState::Wander => self.think_wander(ctx, dt),
            ZombieState::Investigate => self.think_investigate(ctx, dt),
            ZombieState::Chase => self.think_chase(ctx, dt),
            ZombieState::Attack => self.think_attack(ctx, dt),
            ZombieState::Scream => self.think_scream(ctx, dt),
            ZombieState::Eat => self.think_eat(ctx, dt),
            ZombieState::Bang => self.think_bang(ctx, dt),
            ZombieState::Dead => Step::default(),
        }
    }

    fn turn(&self, ctx: &BrainInput<'_>, dt: f32) -> f32 {
        let look = match self.look_point {
            Some(p) => p - ctx.position,
            None => self.walk_velocity,
        };
        let flat = Vec3::new(look.x, 0.0, look.z);
        if flat.length_squared() < 1e-4 {
            return ctx.yaw;
        }
        let target = (-flat.x).atan2(-flat.z);
        let blend = 1.0 - (-TURN_SPEED * dt).exp();
        ctx.yaw + wrap_angle(target - ctx.yaw) * blend
    }

    fn anim_clip(&self, flinched: bool, wish: Vec3) -> AnimClip {
        if self.state == ZombieState::Dead {
            return AnimClip::Die;
        }
        if self.is_staggered() || flinched {
            return AnimClip::Hit;
        }
        if self.state == ZombieState::Attack {
            return AnimClip::Attack;
        }
        if self.crawler {
            return AnimClip::Crawl;
        }
        let moving = self.walk_velocity.length() > MOVING_SPEED || wish.length() > MOVING_SPEED;
        match self.state {
            ZombieState::Dead => AnimClip::Die,
            ZombieState::Attack => AnimClip::Attack,
            ZombieState::Scream => AnimClip::Scream,
            ZombieState::Eat => AnimClip::Eat,
            ZombieState::Bang => AnimClip::Bang,
            ZombieState::Chase if moving => AnimClip::Run,
            ZombieState::Wander | ZombieState::Investigate if moving => AnimClip::Walk,
            _ => AnimClip::Idle,
        }
    }
}

fn forward(yaw: f32) -> Vec3 {
    Vec3::new(-yaw.sin(), 0.0, -yaw.cos())
}

fn wrap_angle(a: f32) -> f32 {
    (a + PI).rem_euclid(2.0 * PI) - PI
}
