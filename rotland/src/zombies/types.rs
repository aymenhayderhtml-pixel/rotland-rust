//! Data types and animation clip mapping for the live zombies.

use engine::anim::Rig;
use engine::render::GpuMesh;
use game::zombie::{AnimClip, AttackHit, BrainOutput, Noise, ZombieBrain, ZombieDef};
use game::{DamageInfo, Health};
use glam::Vec3;

pub const GRAVITY: f32 = 9.8;
/// The flow field is rebuilt this often (it costs ~10 ms; zombies walk ~2.6 m/s, so 0.5 s is plenty).
pub const FLOW_EVERY: f32 = 0.5;
/// Dead bodies stay this long, then the slot is freed.
pub const CORPSE_SECONDS: f32 = 12.0;
/// Body half-width for melee and gun hits.
pub const BODY_RADIUS: f32 = 0.35;
/// Zombie ids start here (the player is 1).
pub const FIRST_ID: u64 = 100;
/// Distant zombies farther than this from player have throttled brain updates (Godot Zombie.cs:17).
pub const FAR_DISTANCE: f32 = 60.0;
/// Distant zombies update thinking every 0.5 s instead of every frame (Godot Zombie.cs:18).
pub const FAR_THINK_INTERVAL: f32 = 0.5;

/// One zombie look: mesh, skeleton and clips.
pub struct Look {
    pub name: String,
    pub mesh: GpuMesh,
    pub rig: Rig,
    pub clips: Vec<pack::Clip>,
}

impl Look {
    pub fn clip(&self, name: &str) -> Option<&pack::Clip> {
        self.clips.iter().find(|c| c.name == name)
    }
}

pub struct Zombie {
    pub id: u64,
    /// The zombies.json type (walker, runner...), for bounties.
    pub kind: String,
    pub look: usize,
    pub def: ZombieDef,
    pub brain: ZombieBrain,
    pub health: Health,
    pub position: Vec3,
    pub velocity: Vec3,
    /// Push from hits, fading out over a fraction of a second.
    pub knock: Vec3,
    pub yaw: f32,
    pub on_floor: bool,
    pub gait: String,
    pub anim: AnimClip,
    pub anim_time: f32,
    pub dead_for: f32,
    pub pending_hit: Option<DamageInfo>,
    /// Accumulated delta time for distant throttled thinking (Godot Zombie.cs:285).
    pub think_accumulator: f32,
    pub last_out: Option<BrainOutput>,
}

/// What happened to the world / player this frame.
#[derive(Default)]
pub struct ZombieFrame {
    pub bites: Vec<AttackHit>,
    /// Sounds emitted by zombies (e.g. screams).
    pub noises: Vec<Noise>,
    /// Positions where zombies started screaming this frame, for audio trigger.
    pub screams: Vec<Vec3>,
}

/// The clip name for a zombie's current animation (ZombieVisual: gait for walking, the type's run clip).
pub fn clip_name(z: &Zombie) -> &str {
    let anim_key = match z.anim {
        AnimClip::Idle => "idle",
        AnimClip::Walk => "walk",
        AnimClip::Run => "run",
        AnimClip::Attack => "attack",
        AnimClip::Hit => "hit",
        AnimClip::Die => "die",
        AnimClip::Scream => "scream",
        AnimClip::Eat => "eat",
        AnimClip::Bang => "bang",
        AnimClip::Crawl => "crawl",
    };
    if let Some(c) = z.def.clips.get(anim_key) {
        return c.as_str();
    }
    let default_name = match z.anim {
        AnimClip::Idle => "idle",
        AnimClip::Walk => z.gait.as_str(),
        AnimClip::Run => z.def.run_clip.as_deref().unwrap_or(z.gait.as_str()),
        AnimClip::Attack => "zombie_attack",
        AnimClip::Hit => "zombie_stagger",
        AnimClip::Die => "die",
        AnimClip::Scream => "zombie_scream",
        AnimClip::Eat => "zombie_eat",
        AnimClip::Bang => "zombie_bang",
        AnimClip::Crawl => "zombie_crawl",
    };
    if let Some(c) = z.def.clips.get(default_name) {
        return c.as_str();
    }
    default_name
}
