//! Simulation tick, locomotion, and player following for Dog.
// Godot: Dog.cs

use glam::Vec3;
use physics::{CharacterMover, PhysicsWorld};

use super::combat::DogBite;
use super::config::DogConfigData;
use super::dog::Dog;
use super::mode::DogMode;
use super::motion::DogMotion;

impl Dog {
    /// Runs one simulation tick of the dog.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn tick(
        &mut self,
        dt: f32,
        player_pos: Option<Vec3>,
        player_forward: Vec3,
        camp_pos: Option<Vec3>,
        cfg: &DogConfigData,
        mover: &CharacterMover,
        physics: &PhysicsWorld,
        zombies: &[(u64, Vec3, bool)],
        animals: &[(u64, Vec3, bool)],
        bites: &mut Vec<DogBite>,
        sounds: &mut Vec<(String, Vec3)>,
    ) {
        if self.busy > 0.0 {
            self.busy = (self.busy - dt).max(0.0);
        }

        if self.is_dead() {
            self.body_left -= dt;
            let moved = DogMotion::move_and_slide(
                mover,
                physics,
                self.position,
                Vec3::ZERO,
                Vec3::ZERO,
                self.velocity.y,
                self.on_floor,
                dt,
            );
            self.position = moved.position;
            self.velocity = moved.velocity;
            self.on_floor = moved.on_floor;
            return;
        }

        self.regenerate(dt, cfg);

        let mut one_shot = None;
        self.combat.step(
            self.id,
            self.position,
            &mut self.health,
            dt,
            cfg,
            zombies,
            animals,
            bites,
            sounds,
            &mut one_shot,
        );
        if let Some((clip, dur)) = one_shot {
            self.play_once(clip, dur);
        }

        // Movement towards combat target or following orders
        let walk_vel = if let Some(fight) = self.combat.fight_point(self.position, cfg) {
            self.walk_toward(fight, cfg.run_speed, dt, cfg, mover, physics)
        } else if !self.combat.is_engaged() {
            self.mode_step(dt, player_pos, player_forward, camp_pos, cfg, mover, physics)
        } else {
            self.motion.idle(dt);
            self.speed = 0.0;
            Vec3::ZERO
        };

        self.knock *= (1.0 - 6.0 * dt).max(0.0);
        let moved = DogMotion::move_and_slide(
            mover,
            physics,
            self.position,
            walk_vel,
            self.knock,
            self.velocity.y,
            self.on_floor,
            dt,
        );
        self.position = moved.position;
        self.velocity = moved.velocity;
        self.on_floor = moved.on_floor;

        self.update_visual(cfg);
    }

    fn walk_toward(
        &mut self,
        target: Vec3,
        speed: f32,
        dt: f32,
        cfg: &DogConfigData,
        _mover: &CharacterMover,
        physics: &PhysicsWorld,
    ) -> Vec3 {
        let (vel, yaw, still_moving) = self.motion.step(
            self.position,
            self.yaw,
            target,
            speed,
            dt,
            cfg,
            &mut self.steering,
            physics,
        );
        if still_moving {
            self.yaw = yaw;
            self.speed = speed;
        } else {
            self.speed = 0.0;
        }
        vel
    }

    fn mode_step(
        &mut self,
        dt: f32,
        player_pos: Option<Vec3>,
        player_forward: Vec3,
        camp_pos: Option<Vec3>,
        cfg: &DogConfigData,
        mover: &CharacterMover,
        physics: &PhysicsWorld,
    ) -> Vec3 {
        match self.mode {
            DogMode::Follow => {
                if let Some(p_pos) = player_pos {
                    self.follow_player(p_pos, player_forward, dt, cfg, mover, physics)
                } else {
                    self.motion.idle(dt);
                    self.speed = 0.0;
                    Vec3::ZERO
                }
            }
            DogMode::Home => self.go_home(camp_pos, dt, cfg, mover, physics),
            DogMode::Stay => {
                self.motion.idle(dt);
                self.speed = 0.0;
                Vec3::ZERO
            }
        }
    }

    fn follow_player(
        &mut self,
        player_pos: Vec3,
        player_forward: Vec3,
        dt: f32,
        cfg: &DogConfigData,
        mover: &CharacterMover,
        physics: &PhysicsWorld,
    ) -> Vec3 {
        let behind = Self::behind_player(player_pos, player_forward, cfg.follow_distance);
        let from_player = DogMotion::flat(self.position - player_pos).length();

        if from_player > cfg.teleport_distance {
            self.place_at(behind);
            return Vec3::ZERO;
        }

        let speed = if from_player > cfg.catch_up_distance {
            cfg.run_speed
        } else {
            cfg.walk_speed
        };
        self.walk_toward(behind, speed, dt, cfg, mover, physics)
    }

    fn go_home(
        &mut self,
        camp_pos: Option<Vec3>,
        dt: f32,
        cfg: &DogConfigData,
        mover: &CharacterMover,
        physics: &PhysicsWorld,
    ) -> Vec3 {
        let Some(camp) = camp_pos else {
            self.mode = DogMode::Stay;
            self.motion.idle(dt);
            self.speed = 0.0;
            return Vec3::ZERO;
        };

        let dist = DogMotion::flat(camp - self.position).length();
        if dist <= cfg.home_arrive_distance {
            self.mode = DogMode::Stay;
            self.motion.idle(dt);
            self.speed = 0.0;
            return Vec3::ZERO;
        }

        let speed = if dist > cfg.catch_up_distance {
            cfg.run_speed
        } else {
            cfg.walk_speed
        };
        self.walk_toward(camp, speed, dt, cfg, mover, physics)
    }

    pub fn behind_player(player_pos: Vec3, player_forward: Vec3, follow_dist: f32) -> Vec3 {
        let flat_fwd = DogMotion::flat(player_forward);
        let fwd = if flat_fwd.length_squared() < 1e-4 {
            Vec3::NEG_Z
        } else {
            flat_fwd.normalize()
        };
        player_pos - fwd * follow_dist
    }
}
