//! The live zombies: each has a brain (game::zombie), health, a body moved by the physics mover, and an
//! animated look. One flow field around the player guides every chasing zombie around obstacles.

mod combat_glue;
mod rendering;
mod specials;
mod spawning;
pub mod types;

use std::sync::Arc;

use game::zombie::{
    AnimClip, BrainInput, BrainOutput, HerdZombie, Noise, SpatialGrid, SpecialConfig,
    ZombieCorpses, ZombieDefs,
};
use game::FlowField;
use glam::Vec3;
use physics::{CharacterMover, PhysicsWorld};
use specials::PendingCall;

pub use types::{
    Look, Zombie, ZombieEvent, ZombieEventTracker, ZombieFrame, BODY_RADIUS,
    CORPSE_SECONDS, FAR_DISTANCE, FAR_THINK_INTERVAL, FIRST_ID, FLOW_EVERY, GRAVITY,
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
    /// Active zombie hordes moving in formation.
    pub hordes: game::horde::HordeSet,
    /// Lifetime kills counter for director input.
    pub total_kills: u64,
    /// Alert tracking for hordes, specials, and camp attacks.
    pub tracker: ZombieEventTracker,
    /// Center of the camp/settlement safe zone if known.
    pub camp_center: Option<Vec3>,
    /// Camp wall radius for attack detection.
    pub camp_wall_radius: f32,
    /// data/special_zombies.json, loaded once; None if it failed (specials then act as plain zombies).
    pub special_cfg: Option<Arc<SpecialConfig>>,
    /// data/zombie_anim.json: crossfade times, speed-matched rates and death angles.
    pub anim_cfg: game::zombie::anim_fade::ZombieAnimConfig,
}

impl Zombies {
    pub fn new(device: &wgpu::Device, pack: &pack::ModelPack, defs: ZombieDefs) -> Self {
        let mut look_names = Vec::new();
        for def in defs.types.values() {
            for m in &def.models {
                if !look_names.contains(m) { look_names.push(m.clone()); }
            }
        }
        for fb in ["zombie_01", "zombie_02", "zombie_03"] {
            if !look_names.iter().any(|n| n == fb) { look_names.push(fb.to_string()); }
        }
        // Looks only named by POI buildings (data/poi.json `zombieModels`, e.g. the R42 police/medic
        // looks) are loaded too, or a chosen model would fall back to the first look.
        for name in spawning::extra_look_names() {
            if !look_names.contains(&name) { look_names.push(name); }
        }
        let looks = look_names.iter().filter_map(|name| {
            let m = pack.find(name)?;
            let skel = m.skeleton.as_ref()?;
            Some(Look {
                name: name.clone(),
                mesh: engine::render::GpuMesh::new(device, name, &m.vertices),
                rig: engine::anim::Rig::new(skel),
                clips: m.clips.clone(),
            })
        }).collect();
        let herd_rad = defs.behaviour.herd_radius;
        let crawler_mover = CharacterMover::new(defs.behaviour.crawler_radius, defs.behaviour.crawler_height);
        let special_cfg = match SpecialConfig::from_repo() {
            Ok(cfg) => Some(Arc::new(cfg)),
            Err(e) => {
                log::warn!("data/special_zombies.json not loaded, specials use the old behaviour: {e}");
                None
            }
        };
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
            hordes: game::horde::HordeSet::new(),
            total_kills: 0,
            tracker: ZombieEventTracker::new(),
            camp_center: None,
            camp_wall_radius: 20.0, // data: hud.json horde_events camp_wall_radius_m
            special_cfg,
            anim_cfg: game::zombie::anim_fade::ZombieAnimConfig::from_repo().unwrap_or_default(),
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

        // 0. Update active hordes
        self.hordes.remove_dead_with(|id| {
            self.list.iter().any(|z| z.id == id && !z.health.is_dead())
        });
        for horde in self.hordes.hordes_mut() {
            if let Some(leader) = self.list.iter().find(|z| z.id == horde.leader && !z.health.is_dead()) {
                horde.set_leader_pos(leader.position);
            }
        }
        self.hordes.merge_meeting(self.defs.behaviour.horde_spacing * 4.0);

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
        let mut calls: Vec<PendingCall> = Vec::new();
        let mut roll_state = (self.game_time.to_bits() ^ self.rng) | 1;

        for z in &mut self.list {
            let dead = z.health.is_dead();
            let distance = effective_player.map_or(0.0, |p| z.position.distance(p));
            let far = effective_player.is_some() && distance > FAR_DISTANCE;
            z.think_accumulator += dt;

            let mut out = if far && z.think_accumulator < FAR_THINK_INTERVAL && !dead {
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
                let is_leg_hit = took_hit.as_ref().is_some_and(|h| h.leg_hit);

                // Compute horde steering if member of a horde
                let horde_steer = self.hordes.hordes().iter()
                    .find(|h| h.contains(z.id))
                    .map(|h| h.steer_member(z.id, z.position, |pos| self.flow.direction(pos)));
                z.brain.set_horde_steer(horde_steer);

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
                let mut o = z.brain.update(&input, elapsed);
                if z.herd_route.is_some()
                    && !dead
                    && matches!(
                        z.brain.state(),
                        game::zombie::ZombieState::Idle | game::zombie::ZombieState::Wander
                    )
                    && let Some(h) = self.hordes.hordes().iter().find(|h| h.contains(z.id))
                {
                    let steer = h.steer_member(z.id, z.position, |_| None);
                    o.velocity = steer;
                    o.anim = AnimClip::Walk;
                    if steer.x * steer.x + steer.z * steer.z > 1e-4 {
                        o.yaw = (-steer.x).atan2(-steer.z);
                    }
                }
                z.last_out = Some(o);
                o
            };

            if z.idle_lock_timer > 0.0 && !dead {
                // Showcase spawns (`--spawn-zombie`) stand still for a few seconds (R32-07).
                z.idle_lock_timer = (z.idle_lock_timer - dt).max(0.0);
                out.velocity = Vec3::ZERO;
                out.yaw = z.yaw;
                out.anim = AnimClip::Idle;
                out.attack_hit = None;
                out.scream = false;
            } else if let (Some(cfg), Some(p), false, true) =
                (self.special_cfg.as_deref(), effective_player, dead, z.special.is_some())
            {
                let roll = specials::next_roll(&mut roll_state);
                specials::step(z, cfg, physics, p, dt, roll, &mut out, &mut frame, &mut calls);
            }
            // Bloaters burst into a toxic cloud on death (R32-07).
            specials::maybe_bloater_burst(z, false, &mut frame);
            if let Some(bite) = out.attack_hit {
                frame.bites.push(bite);
            }
            if out.scream {
                frame.noises.push(Noise {
                    position: z.position,
                    radius: self.defs.behaviour.scream_alert_radius,
                });
                frame.screams.push(z.position);
                if z.kind == "screamer" || z.brain.is_screamer() {
                    calls.push(PendingCall {
                        source: z.id,
                        origin: z.position,
                        target: effective_player.unwrap_or(z.position),
                        radius: specials::LEGACY_CALL_RADIUS,
                        max: usize::MAX,
                    });
                }
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
            specials::note_wall(z, walk, moved.position, dt);
            z.position = moved.position;
            z.velocity = moved.velocity;
            z.on_floor = moved.on_floor;
            z.yaw = out.yaw;
            let prev_anim = z.anim;
            if out.anim != z.anim {
                z.anim = out.anim;
                z.anim_time = 0.0;
            }
            z.anim_time += dt;
            let look = &self.looks[z.look];
            let name = types::resolved_clip_name(z, look);
            let speed = glam::Vec2::new(z.velocity.x, z.velocity.z).length();
            let rate = self
                .anim_cfg
                .ref_speed(name)
                .map_or(1.0, |ref_s| {
                    game::zombie::anim_fade::locomotion_rate(speed, ref_s, &self.anim_cfg)
                });
            let fade_secs = self.anim_cfg.fade_seconds_for(prev_anim, z.anim);
            z.fade.update(dt, game::zombie::anim_fade::clip_key(name), rate, fade_secs);
            if dead {
                z.dead_for += dt;
            }
        }

        specials::apply_calls(&mut self.list, &calls);

        self.nearby_buf = nearby_buf;
        self.list.retain(|z| z.dead_for < CORPSE_SECONDS);

        self.tracker.evaluate(
            self.hordes.hordes(),
            &self.list,
            player,
            player_in_safe_zone,
            self.camp_center,
            self.camp_wall_radius,
            60.0, // data: hud.json horde_events near_distance_m
            30.0, // data: hud.json horde_events special_seen_distance_m
            5,    // data: hud.json horde_events horde_min_size
            &mut frame.events,
        );

        frame
    }
}
