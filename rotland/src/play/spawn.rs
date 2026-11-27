//! Zombie spawning around the player: Godot's spawner rules, director waves and the test horde.

use super::PlayState;
use glam::{Vec2, Vec3};
use game::director::{DirectorInput, HordeEvent};
use game::spawner::SpawnContext;
use physics::PhysicsWorld;
use world::WorldGenerator;

use crate::zombies::Zombies;

impl PlayState {
    fn tier_zombie_mult(&self, tier: u8) -> f32 {
        if tier == 0 {
            return 1.0; // new: DECISIONS 2026-11-25
        }
        self.tier_zombie_mults
            .get((tier - 1) as usize)
            .copied()
            .unwrap_or(1.0) // new: DECISIONS 2026-11-25
    }

    /// Zombies appear and leave around the player by Godot's spawner rules and director waves.
    #[allow(clippy::too_many_arguments)]
    pub fn spawn_tick(
        &mut self,
        dt: f32,
        player: Vec3,
        forward: Vec3,
        night: bool,
        zombies: &mut Zombies,
        physics: &PhysicsWorld,
        world_gen: &WorldGenerator,
    ) -> Vec<HordeEvent> {
        if !self.horde_test_spawned && std::env::args().any(|a| a == "--horde-test") {
            self.horde_test_spawned = true;
            self.spawn_test_horde(player, forward, zombies, physics);
        }

        let alive = zombies.active();
        let alive_non_herd: Vec<_> = zombies
            .list
            .iter()
            .filter(|z| !z.health.is_dead() && z.herd_route.is_none())
            .map(|z| {
                let mut a = game::spawner_config::ActiveZombie::new(z.id, z.position);
                a.is_chasing = matches!(
                    z.brain.state(),
                    game::zombie::ZombieState::Chase | game::zombie::ZombieState::Attack
                );
                a
            })
            .collect();
        let p = Vec2::new(player.x, player.z);
        let in_city = world_gen.city().is_some_and(|c| c.contains(p));
        let in_city_core = world_gen.city().is_some_and(|c| c.district_at(p) == Some(world::District::Core));
        let player_in_camp = world_gen.settlements().first().is_some_and(|s| s.contains(player, 0.0, world_gen.config()));
        let zombies_near = alive.iter().filter(|z| (z.position - player).length_squared() < 900.0).count();
        let kills_since = (zombies.total_kills - self.last_total_kills) as u32;
        self.last_total_kills = zombies.total_kills;

        let dir_input = DirectorInput {
            dt,
            is_night: night,
            player_health: self.player_health,
            recent_kills: kills_since,
            zombies_near,
            in_city,
            player_in_camp,
        };
        let prev_dir_state = self.director.state();
        let dir_out = self.director.update(&dir_input);
        let cur_dir_state = self.director.state();

        let tier = world_gen.tier(player.x, player.z);
        let tier_mult = self.tier_zombie_mult(tier);
        let in_town = world_gen.town_at(player.x, player.z).is_some();
        let metro = self.spawner.config.metro.as_ref().and_then(|cfg| {
            crate::play_metro::metro_profile_at(world_gen, player, night, cfg)
        });
        let metro_ground = |p: Vec3| crate::play_metro::metro_spawn_ground(world_gen, physics, p);
        let in_metro = metro.is_some();
        let ctx = SpawnContext {
            player_position: player,
            player_forward: forward,
            is_night: night,
            alive_zombies: &alive_non_herd,
            settlements: world_gen.settlements(),
            settlement_config: Some(world_gen.config()),
            physics,
            ground_check: if in_metro { Some(&metro_ground) } else { None },
            zombie_amount: 1.0,
            biome_name: Some(world_gen.biome(player.x, player.z).key()),
            tier_mult,
            in_city,
            in_city_core,
            in_town,
            spawn_rate_multiplier: dir_out.spawn_rate_multiplier,
            max_active_multiplier: dir_out.max_active_multiplier,
            special_chance: dir_out.special_chance,
            metro,
        };
        let orders = self.spawner.update(dt, &ctx);
        zombies.despawn(&orders.despawns);
        for order in orders.spawns {
            zombies.spawn_kind(&order.type_id, order.position);
        }

        // Street herds steer themselves (play_metro.rs); only director hordes hunt the player here.
        let herd_ids: std::collections::HashSet<u64> =
            zombies.list.iter().filter(|z| z.herd_route.is_some()).map(|z| z.id).collect();
        for horde in zombies.hordes.hordes_mut() {
            if horde.target_pos.is_some() && !herd_ids.contains(&horde.leader) {
                let near_camp = world_gen.settlements().iter().any(|s| {
                    horde.target_pos.is_some_and(|t| (t - s.center).length_squared() < 900.0)
                });
                if !near_camp {
                    horde.set_target_pos(Some(player));
                }
            }
        }

        let mut frame_events = Vec::new();
        if prev_dir_state != cur_dir_state {
            let phase_evt = HordeEvent::PhaseChange { from: prev_dir_state, to: cur_dir_state };
            frame_events.push(phase_evt);
            self.pending_horde_events.push(phase_evt);
        }
        for event in dir_out.events {
            let (event_size, is_camp_attack, hud_event) = match event {
                game::director::DirectorEvent::WaveStart { size, .. } => (size, false, HordeEvent::HordeIncoming { size }),
                game::director::DirectorEvent::CampAttack { size } => (size, true, HordeEvent::CampAttack { size }),
            };
            frame_events.push(hud_event);
            self.pending_horde_events.push(hud_event);

            let cap = self.spawner.calculate_cap(&ctx);
            let current_alive = zombies.alive();
            let available_slots = cap.saturating_sub(current_alive);
            let count = event_size.min(available_slots);

            if count > 0 {
                let target = if is_camp_attack {
                    let camp_gate = world_gen.settlements().first().map(|site| {
                        let half = world_gen.config().safe_zone_half;
                        site.center + site.local(0.0, 0.0, -half)
                    }).unwrap_or(player);
                    Some(camp_gate)
                } else {
                    Some(player)
                };

                let angle_mag = self.spawner.rng.range(self.spawner.config.out_of_view_angle_deg, 180.0);
                let sign = if self.spawner.rng.next_bool() { 1.0 } else { -1.0 };
                let angle = angle_mag * sign;
                let dist = self.spawner.rng.range(70.0, 90.0);
                let mut flat_fwd = Vec3::new(forward.x, 0.0, forward.z);
                if flat_fwd.length_squared() < 1e-4 { flat_fwd = Vec3::NEG_Z; }
                let rot = glam::Quat::from_rotation_y(angle.to_radians());
                let spawn_center = player + (rot * flat_fwd.normalize()) * dist;

                let mut spawned_ids = Vec::with_capacity(count);
                for i in 0..count {
                    let offset_angle = i as f32 * 2.4;
                    let offset_dist = (i as f32).sqrt() * 1.5;
                    let pt = spawn_center + Vec3::new(offset_angle.cos() * offset_dist, 0.0, offset_angle.sin() * offset_dist);
                    let ground_pt = physics.ground_height(pt.x, pt.z).map(|y| Vec3::new(pt.x, y + 0.1, pt.z)).unwrap_or(pt);

                    let kind = if (night || in_city)
                        && dir_out.special_chance > 0.0
                        && self.spawner.rng.range(0.0, 1.0) < dir_out.special_chance
                    {
                        let specials = ["brute", "screamer", "crawler"];
                        let idx = (self.spawner.rng.next_f32() * specials.len() as f32).min(specials.len() as f32 - 1.0) as usize;
                        specials[idx]
                    } else {
                        self.spawner.config.pick_type_id_core(&mut self.spawner.rng, night, in_city_core)
                    };

                    if let Some(id) = zombies.spawn_kind(kind, ground_pt) {
                        spawned_ids.push(id);
                    }
                }

                if !spawned_ids.is_empty() {
                    let leader_pos = spawn_center;
                    if let Some(h_idx) = zombies.hordes.create_from_wave(&event, &spawned_ids, leader_pos)
                        && let Some(h) = zombies.hordes.hordes_mut().get_mut(h_idx)
                    {
                        h.set_target_pos(target);
                    }
                    for &id in &spawned_ids {
                        if let Some(z) = zombies.list.iter_mut().find(|z| z.id == id)
                            && let Some(t) = target
                        {
                            z.brain.alert_chase(t);
                        }
                    }
                }
            }
        }

        frame_events
    }

    /// Spawns a test horde of 12 zombies 60m away for testing/benchmarking.
    pub fn spawn_test_horde(&mut self, player: Vec3, forward: Vec3, zombies: &mut Zombies, physics: &PhysicsWorld) {
        let count = 12;
        let dist = 60.0;
        let mut flat_fwd = Vec3::new(forward.x, 0.0, forward.z);
        if flat_fwd.length_squared() < 1e-4 { flat_fwd = Vec3::NEG_Z; }
        let rot = glam::Quat::from_rotation_y(std::f32::consts::PI);
        let center = player + (rot * flat_fwd.normalize()) * dist;
        let mut spawned = Vec::with_capacity(count);
        for i in 0..count {
            let offset_angle = i as f32 * 2.4;
            let offset_dist = (i as f32).sqrt() * 1.5;
            let pt = center + Vec3::new(offset_angle.cos() * offset_dist, 0.0, offset_angle.sin() * offset_dist);
            let ground_pt = physics.ground_height(pt.x, pt.z).map(|y| Vec3::new(pt.x, y + 0.1, pt.z)).unwrap_or(pt);
            if let Some(id) = zombies.spawn_kind("walker", ground_pt) {
                spawned.push(id);
            }
        }
        if !spawned.is_empty() {
            let leader_id = spawned[0];
            let members = spawned[1..].to_vec();
            let mut horde = game::horde::Horde::new(leader_id, members);
            horde.set_leader_pos(center);
            horde.set_target_pos(Some(player));
            zombies.hordes.hordes.push(horde);
            for &id in &spawned {
                if let Some(z) = zombies.list.iter_mut().find(|z| z.id == id) {
                    z.brain.alert_chase(player);
                }
            }
        }
    }
}
