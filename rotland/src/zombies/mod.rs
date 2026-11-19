//! The live zombies: each has a brain (game::zombie), health, a body moved by the physics mover, and an
//! animated look. One flow field around the player guides every chasing zombie around obstacles.

mod combat_glue;
mod rendering;
mod spawning;
pub mod types;

use game::zombie::{
    AnimClip, BrainInput, BrainOutput, HerdZombie, Noise, SpatialGrid, ZombieBrain,
    ZombieCorpses, ZombieDefs,
};
use game::FlowField;
use glam::Vec3;
use physics::{CharacterMover, PhysicsWorld};

pub use types::{
    clip_name, Look, Zombie, ZombieFrame, BODY_RADIUS, CORPSE_SECONDS, FAR_DISTANCE,
    FAR_THINK_INTERVAL, FIRST_ID, FLOW_EVERY, GRAVITY,
};

pub struct Zombies {
    pub(super) looks: Vec<Look>,
    pub list: Vec<Zombie>,
    pub(super) defs: ZombieDefs,
    mover: CharacterMover,
    crawler_mover: CharacterMover,
    flow: FlowField,
    flow_timer: f32,
    pub(super) next_id: u64,
    pub(super) rng: u64,
    /// Types of the zombies the player killed since the last `take_kills`.
    pub(super) kills: Vec<(String, bool)>,
    spatial_grid: SpatialGrid,
    nearby_buf: Vec<HerdZombie>,
    pub(super) corpses: ZombieCorpses,
    corpse_buf: Vec<Vec3>,
    pub(super) game_time: f64,
    /// Safe zone input: while true, zombies do not target the player (Godot ZombiePlayerLink / SealsOff).
    pub player_in_safe_zone: bool,
    /// Unreachable input: true when player is on a ladder or roof (Godot ZombieHideouts).
    pub player_unreachable: bool,
}

impl Zombies {
    pub fn new(device: &wgpu::Device, pack: &pack::ModelPack, defs: ZombieDefs) -> Self {
        let looks = ["zombie_01", "zombie_02", "zombie_03"]
            .iter()
            .filter_map(|name| {
                let m = pack.find(name)?;
                let skel = m.skeleton.as_ref()?;
                Some(Look {
                    mesh: engine::render::GpuMesh::new(device, name, &m.vertices),
                    rig: engine::anim::Rig::new(skel),
                    clips: m.clips.clone(),
                })
            })
            .collect();
        let herd_rad = defs.behaviour.herd_radius;
        let crawler_mover = CharacterMover::new(
            defs.behaviour.crawler_radius,
            defs.behaviour.crawler_height,
        );
        Self {
            looks,
            list: Vec::new(),
            defs,
            mover: CharacterMover::default(),
            crawler_mover,
            flow: FlowField::new(1.0, 64),
            flow_timer: 0.0,
            next_id: FIRST_ID,
            rng: 0x9E37_79B9_7F4A_7C15,
            kills: Vec::new(),
            spatial_grid: SpatialGrid::new(herd_rad, 256),
            nearby_buf: Vec::with_capacity(32),
            corpses: ZombieCorpses::new(),
            corpse_buf: Vec::with_capacity(16),
            game_time: 0.0,
            player_in_safe_zone: false,
            player_unreachable: false,
        }
    }

    /// Runs every brain and body for one frame using stored safe zone / unreachable settings.
    pub fn update(
        &mut self,
        dt: f32,
        player: Option<Vec3>,
        noises: &[Noise],
        physics: &PhysicsWorld,
    ) -> ZombieFrame {
        let unreachable = self.player_unreachable;
        let safe = self.player_in_safe_zone;
        self.update_full(dt, player, noises, physics, unreachable, safe)
    }

    /// Full update taking explicit unreachable and safe-zone flags.
    pub fn update_full(
        &mut self,
        dt: f32,
        player: Option<Vec3>,
        noises: &[Noise],
        physics: &PhysicsWorld,
        player_unreachable: bool,
        player_in_safe_zone: bool,
    ) -> ZombieFrame {
        let mut frame = ZombieFrame::default();
        self.game_time += dt as f64;
        self.player_unreachable = player_unreachable;
        self.player_in_safe_zone = player_in_safe_zone;

        let effective_player = if player_in_safe_zone { None } else { player };

        self.flow_timer -= dt;
        if let (Some(p), true) = (effective_player, self.flow_timer <= 0.0) {
            self.flow.rebuild(physics, p);
            self.flow_timer = FLOW_EVERY;
        }

        // 1. Rebuild spatial grid of active zombies once per frame into reused buffer
        self.spatial_grid.clear();
        for z in &self.list {
            if !z.health.is_dead() {
                self.spatial_grid.insert(HerdZombie::new(
                    z.position,
                    z.brain.state(),
                    z.brain.last_seen(),
                ));
            }
        }

        // 2. Collect recent corpse positions into reused buffer
        self.corpses.recent_positions(
            self.defs.behaviour.corpse_spot_seconds,
            self.game_time,
            &mut self.corpse_buf,
        );

        let mut nearby_buf = std::mem::take(&mut self.nearby_buf);

        for z in &mut self.list {
            let dead = z.health.is_dead();
            let distance = effective_player.map_or(0.0, |p| z.position.distance(p));
            let far = effective_player.is_some() && distance > FAR_DISTANCE;
            z.think_accumulator += dt;

            let out = if far && z.think_accumulator < FAR_THINK_INTERVAL && !dead {
                z.last_out.unwrap_or(BrainOutput {
                    velocity: Vec3::ZERO,
                    yaw: z.yaw,
                    state: z.brain.state(),
                    attack_hit: None,
                    anim: AnimClip::Walk,
                    staggered: false,
                    scream: false,
                    teleport: None,
                })
            } else {
                let elapsed = if far { z.think_accumulator } else { dt };
                z.think_accumulator = 0.0;

                self.spatial_grid.query_nearby(z.position, &mut nearby_buf);

                let took_hit = z.pending_hit.take();
                let is_leg_hit = took_hit.as_ref().map_or(false, |h| h.leg_hit);

                let input = BrainInput {
                    position: z.position,
                    yaw: z.yaw,
                    on_floor: z.on_floor,
                    target: effective_player,
                    heard: noises,
                    flow_dir: effective_player.and_then(|_| self.flow.direction(z.position)),
                    took_damage: took_hit,
                    is_dead: dead,
                    world: physics,
                    nearby_zombies: &nearby_buf,
                    corpses: &self.corpse_buf,
                    target_unreachable: player_unreachable,
                    leg_hit: is_leg_hit,
                    player_in_safe_zone,
                };
                let o = z.brain.update(&input, elapsed);
                z.last_out = Some(o);
                o
            };

            if let Some(bite) = out.attack_hit {
                frame.bites.push(bite);
            }
            if out.scream {
                frame.noises.push(Noise {
                    position: z.position,
                    radius: self.defs.behaviour.scream_alert_radius,
                });
                frame.screams.push(z.position);
            }

            let walk = if dead {
                Vec3::ZERO
            } else {
                Vec3::new(out.velocity.x, 0.0, out.velocity.z)
            };
            z.knock *= (1.0 - 6.0 * dt).max(0.0);
            let fall = if z.on_floor {
                -1.0
            } else {
                z.velocity.y - GRAVITY * dt
            };
            let wanted = walk + z.knock + Vec3::new(0.0, fall, 0.0);

            let mover = if z.brain.is_crawler() {
                &self.crawler_mover
            } else {
                &self.mover
            };
            let moved = mover.move_and_slide(physics, z.position, wanted, dt);
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

        self.nearby_buf = nearby_buf;
        self.list.retain(|z| z.dead_for < CORPSE_SECONDS);
        frame
    }
}
