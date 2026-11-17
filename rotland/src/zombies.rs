//! The live zombies: each has a brain (game::zombie), health, a body moved by the physics mover, and an
//! animated look. One flow field around the player guides every chasing zombie around obstacles.

use engine::anim::Rig;
use engine::render::{GpuMesh, SkinInstance, SkinnedBatch};
use game::zombie::{AnimClip, AttackHit, BrainInput, Noise, ZombieBehaviourDef, ZombieBrain, ZombieDef, ZombieDefs};
use game::{DamageInfo, FlowField, Health};
use glam::{Mat4, Vec3};
use physics::{CharacterMover, PhysicsWorld};

const GRAVITY: f32 = 9.8;
/// The flow field is rebuilt this often (it costs ~10 ms; zombies walk ~2.6 m/s, so 0.5 s is plenty).
const FLOW_EVERY: f32 = 0.5;
/// Dead bodies stay this long, then the slot is freed.
const CORPSE_SECONDS: f32 = 12.0;
/// Body half-width for melee and gun hits.
pub const BODY_RADIUS: f32 = 0.35;
/// Zombie ids start here (the player is 1).
const FIRST_ID: u64 = 100;

/// One zombie look: mesh, skeleton and clips.
struct Look {
    mesh: GpuMesh,
    rig: Rig,
    clips: Vec<pack::Clip>,
}

impl Look {
    fn clip(&self, name: &str) -> Option<&pack::Clip> {
        self.clips.iter().find(|c| c.name == name)
    }
}

pub struct Zombie {
    pub id: u64,
    /// The zombies.json type (walker, runner...), for bounties.
    pub kind: String,
    look: usize,
    def: ZombieDef,
    pub brain: ZombieBrain,
    pub health: Health,
    pub position: Vec3,
    velocity: Vec3,
    /// Push from hits, fading out over a fraction of a second.
    knock: Vec3,
    yaw: f32,
    on_floor: bool,
    gait: String,
    anim: AnimClip,
    anim_time: f32,
    dead_for: f32,
    pending_hit: Option<DamageInfo>,
}

pub struct Zombies {
    looks: Vec<Look>,
    pub list: Vec<Zombie>,
    defs: ZombieDefs,
    mover: CharacterMover,
    flow: FlowField,
    flow_timer: f32,
    next_id: u64,
    rng: u64,
    /// Types of the zombies the player killed since the last `take_kills`.
    kills: Vec<(String, bool)>,
}

/// What happened to the player this frame.
#[derive(Default)]
pub struct ZombieFrame {
    pub bites: Vec<AttackHit>,
}

impl Zombies {
    pub fn new(device: &wgpu::Device, pack: &pack::ModelPack, defs: ZombieDefs) -> Self {
        let looks = ["zombie_01", "zombie_02", "zombie_03"]
            .iter()
            .filter_map(|name| {
                let m = pack.find(name)?;
                let skel = m.skeleton.as_ref()?;
                Some(Look { mesh: GpuMesh::new(device, name, &m.vertices), rig: Rig::new(skel), clips: m.clips.clone() })
            })
            .collect();
        Self {
            looks,
            list: Vec::new(),
            defs,
            mover: CharacterMover::default(),
            flow: FlowField::new(1.0, 64),
            flow_timer: 0.0,
            next_id: FIRST_ID,
            rng: 0x9E37_79B9_7F4A_7C15,
            kills: Vec::new(),
        }
    }

    fn rand(&mut self) -> f32 {
        // xorshift64: deterministic, no global randomness
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 7;
        self.rng ^= self.rng << 17;
        (self.rng >> 40) as f32 / (1u64 << 24) as f32
    }

    /// `count` zombies of type `kind` on a ring `min..max` metres around `center` (like BenchmarkZombies).
    pub fn spawn_ring(&mut self, kind: &str, center: Vec3, count: usize, min: f32, max: f32, physics: &PhysicsWorld) {
        let Some(def) = self.defs.get(kind).cloned() else { return };
        for i in 0..count {
            let angle = std::f32::consts::TAU * i as f32 / count as f32 + (self.rand() - 0.5) * 0.4;
            let dist = min + (max - min) * self.rand();
            let (x, z) = (center.x + angle.cos() * dist, center.z + angle.sin() * dist);
            let y = physics.ground_height(x, z).unwrap_or(center.y);
            self.spawn(kind, &def, Vec3::new(x, y, z));
        }
    }

    fn spawn(&mut self, kind: &str, def: &ZombieDef, at: Vec3) {
        let look_names: Vec<&str> = def.models.iter().map(String::as_str).collect();
        let pick = (self.rand() * look_names.len() as f32) as usize;
        let look = ["zombie_01", "zombie_02", "zombie_03"].iter().position(|n| Some(n) == look_names.get(pick)).unwrap_or(0);
        let roll = self.rand();
        let gaits = &self.defs.behaviour.gaits;
        let gait = gaits.get((roll * gaits.len() as f32) as usize).cloned().unwrap_or_else(|| "zombie_shamble".into());
        let yaw = self.rand() * std::f32::consts::TAU;
        // Random start phase so a group does not step in unison (ZombieVisual)
        let phase = self.rand() * 2.0;
        let id = self.next_id;
        self.next_id += 1;
        let tuning: &ZombieBehaviourDef = &self.defs.behaviour;
        self.list.push(Zombie {
            id,
            kind: kind.to_string(),
            look: look.min(self.looks.len().saturating_sub(1)),
            def: def.clone(),
            brain: ZombieBrain::new(def, tuning, at, id),
            health: Health::new(def.max_health),
            position: at,
            velocity: Vec3::ZERO,
            knock: Vec3::ZERO,
            yaw,
            on_floor: true,
            gait,
            anim: AnimClip::Idle,
            anim_time: phase,
            dead_for: 0.0,
            pending_hit: None,
        });
    }

    /// One zombie of type `kind` (e.g. "walker", "runner") at `at`, for the spawner. Returns its id.
    pub fn spawn_kind(&mut self, kind: &str, at: Vec3) -> Option<u64> {
        let def = self.defs.get(kind).cloned()?;
        let id = self.next_id;
        self.spawn(kind, &def, at);
        Some(id)
    }

    /// Removes zombies the spawner says are too far away.
    pub fn despawn(&mut self, ids: &[u64]) {
        self.list.retain(|z| !ids.contains(&z.id));
    }

    /// Living zombies as the spawner sees them (a chasing zombie is kept farther away).
    pub fn active(&self) -> Vec<game::spawner_config::ActiveZombie> {
        use game::zombie::ZombieState;
        self.list
            .iter()
            .filter(|z| !z.health.is_dead())
            .map(|z| {
                let mut a = game::spawner_config::ActiveZombie::new(z.id, z.position);
                a.is_chasing = matches!(z.brain.state(), ZombieState::Chase | ZombieState::Attack);
                a
            })
            .collect()
    }

    /// Applies a hit from the player. Returns true if it killed the zombie.
    pub fn hit(&mut self, id: u64, damage: DamageInfo, knockback: Vec3) -> bool {
        let Some(z) = self.list.iter_mut().find(|z| z.id == id) else { return false };
        let headshot = damage.headshot;
        let result = z.health.take_damage(damage.clone());
        if result.dealt > 0.0 {
            z.knock += Vec3::new(knockback.x, 0.0, knockback.z);
            z.pending_hit = Some(damage);
        }
        if result.died_now {
            self.kills.push((z.kind.clone(), headshot));
        }
        result.died_now
    }

    /// Runs every brain and body for one frame. `player` is None while the player is dead.
    pub fn update(&mut self, dt: f32, player: Option<Vec3>, noises: &[Noise], physics: &PhysicsWorld) -> ZombieFrame {
        let mut frame = ZombieFrame::default();
        self.flow_timer -= dt;
        if let (Some(p), true) = (player, self.flow_timer <= 0.0) {
            self.flow.rebuild(physics, p);
            self.flow_timer = FLOW_EVERY;
        }
        for z in &mut self.list {
            let dead = z.health.is_dead();
            let input = BrainInput {
                position: z.position,
                yaw: z.yaw,
                on_floor: z.on_floor,
                target: player,
                heard: noises,
                flow_dir: player.and_then(|_| self.flow.direction(z.position)),
                took_damage: z.pending_hit.take(),
                is_dead: dead,
                world: physics,
                nearby_zombies: &[],
                corpses: &[],
                target_unreachable: false,
                leg_hit: false,
            };
            let out = z.brain.update(&input, dt);
            if let Some(bite) = out.attack_hit {
                frame.bites.push(bite);
            }
            // The brain's walk plus any knockback (which fades out), plus gravity
            let walk = if dead { Vec3::ZERO } else { Vec3::new(out.velocity.x, 0.0, out.velocity.z) };
            z.knock *= (1.0 - 6.0 * dt).max(0.0);
            let fall = if z.on_floor { -1.0 } else { z.velocity.y - GRAVITY * dt };
            let wanted = walk + z.knock + Vec3::new(0.0, fall, 0.0);
            let moved = self.mover.move_and_slide(physics, z.position, wanted, dt);
            z.position = moved.position;
            z.velocity = moved.velocity;
            z.on_floor = moved.on_floor;
            z.yaw = out.yaw;
            if out.anim != z.anim {
                z.anim = out.anim;
                z.anim_time = 0.0;
            }
            z.anim_time += dt;
            if dead {
                z.dead_for += dt;
            }
        }
        self.list.retain(|z| z.dead_for < CORPSE_SECONDS);
        frame
    }

    /// The types of zombies killed since the last call.
    /// A hit that is not the player's (camp guards): a kill does not count for the player.
    pub fn hit_unclaimed(&mut self, id: u64, damage: DamageInfo, knockback: Vec3) -> bool {
        let killed = self.hit(id, damage, knockback);
        if killed {
            self.kills.pop();
        }
        killed
    }

    /// (type, was a headshot) of each zombie the player killed since the last call.
    pub fn take_kills(&mut self) -> Vec<(String, bool)> {
        std::mem::take(&mut self.kills)
    }

    /// Living zombies as melee targets.
    pub fn targets(&self) -> Vec<game::melee::MeleeTarget> {
        self.list
            .iter()
            .filter(|z| !z.health.is_dead())
            .map(|z| game::melee::MeleeTarget { id: z.id, position: z.position, radius: BODY_RADIUS })
            .collect()
    }

    pub fn alive(&self) -> usize {
        self.list.iter().filter(|z| !z.health.is_dead()).count()
    }

    /// Adds every zombie's skin matrices to `bones` and returns one instanced batch per look.
    pub fn pose(&self, bones: &mut Vec<Mat4>) -> Vec<SkinnedBatch<'_>> {
        let mut per_look: Vec<Vec<SkinInstance>> = vec![Vec::new(); self.looks.len()];
        for z in &self.list {
            let look = &self.looks[z.look];
            let name = clip_name(z);
            let Some(clip) = look.clip(name).or_else(|| look.clip("idle")) else { continue };
            // Death plays once and holds its last pose; everything else loops
            let t = if z.anim == AnimClip::Die { z.anim_time.min(clip.duration - 0.01).max(0.0) } else { z.anim_time };
            let base = bones.len();
            bones.resize(base + look.rig.joint_count(), Mat4::IDENTITY);
            look.rig.sample(clip, t, &mut bones[base..]);
            // The brain's yaw is Godot's Rotation.Y and the models already face that way (no half turn:
            // with one they walked backwards, seen in the owner's screenshot)
            let model = Mat4::from_translation(z.position) * Mat4::from_rotation_y(z.yaw);
            per_look[z.look].push(SkinInstance::new(model, base as u32));
        }
        per_look
            .into_iter()
            .zip(&self.looks)
            .filter(|(v, _)| !v.is_empty())
            .map(|(instances, look)| SkinnedBatch { mesh: &look.mesh, instances })
            .collect()
    }
}

/// The clip name for a zombie's current animation (ZombieVisual: gait for walking, the type's run clip).
fn clip_name(z: &Zombie) -> &str {
    match z.anim {
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
    }
}
