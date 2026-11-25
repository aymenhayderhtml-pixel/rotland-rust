//! Update simulation step for Animals.
// Godot: AnimalSpawner.cs, Animal.cs

use glam::Vec3;
use physics::PhysicsWorld;
use super::manager::{AnimalFrame, AnimalUpdateContext, Animals};
use super::motion::AnimalMotion;
use super::spawner::{pick_spawn_spot, plan_herd};

impl Animals {
    // Godot: AnimalSpawner.cs _PhysicsProcess & Animal.cs _PhysicsProcess
    pub fn update(
        &mut self,
        dt: f32,
        ctx: &AnimalUpdateContext,
        physics: &PhysicsWorld,
    ) -> AnimalFrame {
        let mut attacks = Vec::new();
        let mut sounds = Vec::new();
        let mut alarms: Vec<(u64, Vec3)> = Vec::new();

        // 1. Spawning & Despawning
        if self.enabled && let Some(player_pos) = ctx.player_position {
            self.despawn_far(player_pos);
            self.play_time += dt;
            if self.play_time >= self.config.spawn.start_delay {
                self.timer -= dt;
                if self.timer <= 0.0 {
                    self.timer = self.config.spawn.interval;
                    let cap = if ctx.is_night {
                        self.config.spawn.night_cap
                    } else {
                        self.config.spawn.day_cap
                    };
                    let room = cap.saturating_sub(self.live_count());
                    if room > 0 {
                        let biome = ctx.biome_name.unwrap_or("forest");
                        let no_spawn = |p| ctx.in_no_spawn_zone.is_some_and(|f| f(p));
                        if let Some((species_id, feet)) = pick_spawn_spot(
                            player_pos,
                            ctx.player_forward,
                            &self.config,
                            biome,
                            &mut self.rng,
                            physics,
                            no_spawn,
                        ) && let Some(sp) = self.config.species(&species_id).cloned() {
                            let herd_spots = plan_herd(
                                &species_id,
                                feet,
                                room,
                                &sp,
                                &mut self.rng,
                                physics,
                                no_spawn,
                            );
                            let herd_id = if herd_spots.len() > 1 {
                                let hid = self.next_herd_id;
                                self.next_herd_id += 1;
                                Some(hid)
                            } else {
                                None
                            };
                            for (pos, model) in herd_spots {
                                self.spawn_at(&species_id, pos, herd_id, Some(&model));
                            }
                        }
                    }
                }
            }
        }

        // 2. Process noises
        let b = self.config.behaviour.clone();
        for animal in &mut self.list {
            if animal.is_valid_and_alive() {
                for &(noise_pos, noise_radius) in ctx.noises {
                    if let Some(out) = animal.ai.on_noise(
                        noise_pos,
                        noise_radius,
                        animal.position,
                        &animal.species_data,
                        &b,
                    ) {
                        if let Some(threat) = out.herd_alarm
                            && let Some(hid) = animal.herd_id {
                                alarms.push((hid, threat));
                            }
                        if let Some(snd) = out.sound_id {
                            sounds.push((snd, animal.position));
                        }
                    }
                }
            }
        }

        // 3. Animal AI, combat, and movement
        for animal in &mut self.list {
            if animal.is_dead() {
                animal.ai.body_left -= dt;
                animal.tick_visual(dt);
                let moved = AnimalMotion::move_and_slide(
                    &self.mover,
                    physics,
                    animal.position,
                    Vec3::ZERO,
                    Vec3::ZERO,
                    animal.velocity.y,
                    animal.on_floor,
                    dt,
                );
                animal.position = moved.position;
                animal.velocity = moved.velocity;
                animal.on_floor = moved.on_floor;
                continue;
            }

            // Zombie combat for boars
            if animal.species_data.danger
                && let Some(c) = &mut animal.ai.combat
            {
                let z_hits = c.fight_zombies(animal.id, animal.position, ctx.zombies, &animal.species_data);
                if !z_hits.is_empty() {
                    attacks.extend(z_hits);
                    animal.play_once("attack", 0.8);
                }
            }

            if !animal.ai_paused {
                let player_info = ctx.player_position.map(|pos| {
                    (
                        pos,
                        ctx.player_is_crouching,
                        ctx.player_is_sprinting,
                        ctx.player_is_dead,
                        ctx.player_hunting_stealth_bonus,
                    )
                });
                let out = animal.ai.think(
                    dt,
                    animal.position,
                    animal.yaw,
                    player_info,
                    &animal.species_data,
                    &b,
                );

                if let Some(threat) = out.herd_alarm
                    && let Some(hid) = animal.herd_id {
                        alarms.push((hid, threat));
                    }
                if let Some(snd) = out.sound_id {
                    sounds.push((snd, animal.position));
                }
                if let Some((clip, dur)) = out.one_shot {
                    animal.play_once(clip, dur);
                } else {
                    animal.set_locomotion(out.locomotion);
                }
                if out.attack_strike
                    && let (Some(p_pos), Some(c)) = (ctx.player_position, &mut animal.ai.combat)
                {
                    let hit = c.strike(animal.id, animal.position, animal.yaw, p_pos, &animal.species_data);
                    attacks.push(hit);
                }

                // Movement
                let walk_vel = if let Some((target, speed)) = out.wanted_target {
                    let (vel, yaw, still_moving) = animal.motion.step(animal.position, animal.yaw, target, speed);
                    if still_moving {
                        animal.yaw = yaw;
                    }
                    vel
                } else {
                    Vec3::ZERO
                };
                if let Some(wanted_yaw) = out.wanted_yaw {
                    animal.yaw = wanted_yaw;
                }
                animal.knock *= (1.0 - 6.0 * dt).max(0.0);
                let moved = AnimalMotion::move_and_slide(
                    &self.mover,
                    physics,
                    animal.position,
                    walk_vel,
                    animal.knock,
                    animal.velocity.y,
                    animal.on_floor,
                    dt,
                );
                animal.position = moved.position;
                animal.velocity = moved.velocity;
                animal.on_floor = moved.on_floor;
            }

            animal.tick_visual(dt);
        }

        // 4. Herd alarms broadcast
        for (hid, threat) in alarms {
            for animal in &mut self.list {
                if animal.herd_id == Some(hid) && animal.is_valid_and_alive() {
                    let _ = animal.ai.react(threat, true, &animal.species_data, &b);
                }
            }
        }

        AnimalFrame {
            draws: self.draw_states(),
            attacks,
            sounds,
        }
    }
}
