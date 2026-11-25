//! Simulation update and stray spawning step for Dogs manager.
// Godot: DogManager.cs _PhysicsProcess & StrayDogSpawner.cs Refresh

use glam::Vec3;
use physics::PhysicsWorld;

use super::manager::Dogs;
use super::mode::DogMode;
use super::motion::DogMotion;
use super::stray::{DogPoiSite, StrayDogSpawner};
use super::types::{DogDrawState, DogFrame, DogUpdateContext};

impl Dogs {
    /// Advances simulation by dt and returns draw states, bites, sounds, toasts.
    // Godot: DogManager.cs _PhysicsProcess
    pub fn update(
        &mut self,
        dt: f32,
        ctx: &DogUpdateContext,
        physics: &PhysicsWorld,
    ) -> DogFrame {
        let mut bites = Vec::new();
        let mut sounds = Vec::new();
        let mut toasts = Vec::new();

        // On player death: pet dog retreats to camp
        // Godot: DogManager.cs OnPlayerDied
        if ctx.player_dead && let Some(dog) = self.dog.as_mut() && !dog.is_dead() {
            dog.set_mode(DogMode::Home);
        }

        // 1. Tick pet dog
        let mut dog_died = false;
        let mut dog_name = String::new();
        if let Some(dog) = self.dog.as_mut() {
            let was_alive = !dog.is_dead();
            dog.tick(
                dt,
                ctx.player_position,
                ctx.player_forward,
                ctx.camp_position,
                &self.config,
                &self.mover,
                physics,
                ctx.zombies,
                ctx.animals,
                &mut bites,
                &mut sounds,
            );
            if was_alive && dog.is_dead() {
                dog_died = true;
                dog_name = dog.name.clone();
            }
        }
        if dog_died {
            self.dead_count += 1;
            toasts.push(format!("{dog_name} died. It is gone for good."));
        }

        // Clean up body after bodySeconds
        if let Some(dog) = &self.dog
            && dog.is_dead() && dog.body_left <= 0.0 {
                self.dog = None;
            }

        // 2. Stray dogs spawner refresh
        if self.spawner_enabled && let Some(p_pos) = ctx.player_position {
            self.stray_timer -= dt;
            if self.stray_timer <= 0.0 {
                self.stray_timer = 0.5;
                self.refresh_strays(p_pos, ctx.world_seed, ctx.poi_sites);
            }
        }

        // 3. Tick stray dogs
        for stray in &mut self.strays {
            stray.tick(
                dt,
                ctx.player_position,
                ctx.player_sprinting,
                ctx.player_velocity,
                &self.config,
                &self.mover,
                physics,
            );
        }

        // 4. Collect draw states
        let mut draws = Vec::new();
        if let Some(dog) = &self.dog {
            draws.push(DogDrawState {
                id: dog.id,
                model: dog.model.clone(),
                position: dog.position,
                yaw: dog.yaw,
                clip: dog.current_clip.clone(),
                alive: !dog.is_dead(),
                scale: dog.current_scale(ctx.now_hours, &self.config),
                is_stray: false,
            });
        }
        for stray in &self.strays {
            draws.push(DogDrawState {
                id: stray.id,
                model: stray.model.clone(),
                position: stray.position,
                yaw: stray.yaw,
                clip: if stray.fleeing { "run".to_string() } else { "sit".to_string() },
                alive: true,
                scale: 1.0,
                is_stray: true,
            });
        }

        DogFrame {
            draws,
            bites,
            sounds,
            toasts,
        }
    }

    // Godot: StrayDogSpawner.cs Refresh
    pub(super) fn refresh_strays(&mut self, player_pos: Vec3, world_seed: i32, poi_sites: &[DogPoiSite]) {
        // Despawn far strays
        let despawn_sq = self.config.stray_despawn_range * self.config.stray_despawn_range;
        self.strays.retain(|s| DogMotion::flat(s.position - player_pos).length_squared() <= despawn_sq);

        // Spawn for eligible buildings
        let spawn_sq = self.config.stray_spawn_range * self.config.stray_spawn_range;
        for site in poi_sites {
            if self.is_tamed(&site.poi_id) || self.strays.iter().any(|s| s.lot_id == site.poi_id) {
                continue;
            }
            if DogMotion::flat(site.position - player_pos).length_squared() > spawn_sq {
                continue;
            }
            if !StrayDogSpawner::rolls_stray(world_seed, &site.poi_id, &site.building_id, &self.config) {
                continue;
            }
            let sit = StrayDogSpawner::sit_point(site.position, site.yaw, site.half_depth);
            self.spawn_stray_at(&site.poi_id, sit);
        }
    }
}
