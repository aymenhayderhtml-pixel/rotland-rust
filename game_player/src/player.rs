use glam::{Vec2, Vec3};
use physics::{CharacterMover, PhysicsWorld};

use game_core::actions::Actions;

// Movement speeds in m/s (from PlayerController.cs)
pub const WALK_SPEED: f32 = 4.0;
pub const SPRINT_SPEED: f32 = 7.0;
pub const CROUCH_SPEED: f32 = 2.0;

// Jump and acceleration parameters (from PlayerController.cs)
pub const JUMP_VELOCITY: f32 = 4.8;
pub const ACCELERATION: f32 = 12.0;
pub const DECELERATION: f32 = 14.0;
pub const AIR_CONTROL: f32 = 0.3;
pub const TURN_SPEED: f32 = 10.0;
pub const COYOTE_TIME: f32 = 0.1;
pub const JUMP_BUFFER_TIME: f32 = 0.1;
pub const SPRINT_FORWARD_THRESHOLD: f32 = 0.5;

// Capsule dimensions (from Player.tscn and PlayerController.cs)
pub const STAND_HEIGHT: f32 = 1.8;
pub const CROUCH_HEIGHT: f32 = 1.2;
pub const MAX_STEP_HEIGHT: f32 = 0.35;
pub const DEFAULT_GRAVITY: f32 = 9.8;

/// Stamina pool for sprinting and jumping (from Stamina.cs and PlayerStaminaUse.cs).
#[derive(Clone, Copy, Debug)]
pub struct Stamina {
    pub current: f32,
    pub max: f32,
    pub regen_per_second: f32,
    pub regen_delay: f32,
    pub regen_multiplier: f32,
    pub warmth_regen_multiplier: f32,
    pub time_since_use: f32,
    pub sprint_cost_per_sec: f32,
    pub min_to_start_sprint: f32,
    pub jump_cost: f32,
}

impl Default for Stamina {
    fn default() -> Self {
        Self {
            current: 100.0,
            max: 100.0,
            regen_per_second: 18.0,
            regen_delay: 0.8,
            regen_multiplier: 1.0,
            // new: DECISIONS 2026-11-25
            warmth_regen_multiplier: 1.0,
            time_since_use: 0.8,
            sprint_cost_per_sec: 15.0,
            min_to_start_sprint: 10.0,
            jump_cost: 8.0,
        }
    }
}

impl Stamina {
    /// Advances stamina regeneration when outside the regen delay pause.
    pub fn update(&mut self, dt: f32) {
        self.time_since_use += dt;
        if self.time_since_use >= self.regen_delay && self.current < self.max {
            self.current = (self.current
                + self.regen_per_second
                    * self.regen_multiplier
                    * self.warmth_regen_multiplier
                    * dt)
                .min(self.max);
        }
    }

    /// Checks if sprint is allowed and drains sprint stamina for this frame.
    pub fn try_sprint(&mut self, was_sprinting: bool, dt: f32) -> bool {
        let allowed = if was_sprinting {
            self.current > 0.0
        } else {
            self.current >= self.min_to_start_sprint
        };
        if !allowed {
            return false;
        }
        self.current = (self.current - self.sprint_cost_per_sec * dt).max(0.0);
        self.time_since_use = 0.0;
        true
    }

    /// Spends jump cost if enough stamina exists.
    pub fn try_jump(&mut self) -> bool {
        if self.current < self.jump_cost {
            return false;
        }
        self.current -= self.jump_cost;
        self.time_since_use = 0.0;
        true
    }
}

/// Player character state and kinematics, ported from Godot's PlayerController.cs.
#[derive(Clone, Debug)]
pub struct Player {
    pub position: Vec3,
    pub velocity: Vec3,
    /// Horizontal push from special zombie hits (m/s), fading out each frame.
    pub knock: Vec3,
    pub yaw: f32,
    pub on_floor: bool,
    pub crouching: bool,
    pub is_sprinting: bool,
    pub first_person: bool,
    pub stamina: Stamina,
    pub coyote_timer: f32,
    pub jump_buffer_timer: f32,
    pub speed_multiplier: f32,
    pub can_sprint: bool,
    pub can_jump: bool,
    pub water_float_y: Option<f32>,
    pub in_deep_water: bool,
    /// Metres of lake water above the feet (0 on land); set each frame by the game loop.
    pub water_depth: f32,
    pub dirty_water_cooldown: f32,
    pub gravity: f32,
}

/// How fast a special zombie's push fades (per second, linear factor). // new: DECISIONS 2026-11-23
const KNOCK_DECAY: f32 = 6.0;

impl Player {
    /// Adds a horizontal push (the y part is ignored), e.g. from a brute's hit.
    pub fn push(&mut self, horizontal: Vec3) {
        self.knock += Vec3::new(horizontal.x, 0.0, horizontal.z);
    }

    /// Spawns a player at the given world position.
    pub fn new(position: Vec3) -> Self {
        Self {
            position,
            velocity: Vec3::ZERO,
            knock: Vec3::ZERO,
            yaw: 0.0,
            on_floor: true,
            crouching: false,
            is_sprinting: false,
            first_person: false,
            stamina: Stamina::default(),
            coyote_timer: 0.0,
            jump_buffer_timer: 0.0,
            speed_multiplier: 1.0,
            can_sprint: true,
            can_jump: true,
            water_float_y: None,
            in_deep_water: false,
            water_depth: 0.0,
            dirty_water_cooldown: 0.0,
            gravity: DEFAULT_GRAVITY,
        }
    }

    /// Applies water movement effects (speed multiplier, jump/sprint restrictions, floating, stamina drain).
    pub fn apply_water_movement(&mut self, res: &crate::swimming::WaterMovementResult) {
        self.speed_multiplier = res.speed_multiplier;
        self.can_sprint = res.can_sprint;
        if !self.can_sprint {
            self.is_sprinting = false;
        }
        self.can_jump = res.can_jump;
        self.water_float_y = res.float_y;
        self.in_deep_water = res.zone == world::LakeZone::Deep;
        if res.stamina_drain > 0.0 {
            self.stamina.current = (self.stamina.current - res.stamina_drain).max(0.0);
            self.stamina.time_since_use = 0.0;
        }
    }

    /// Horizontal speed in m/s, matching Godot's PlayerController.HorizontalSpeed.
    pub fn horizontal_speed(&self) -> f32 {
        Vec2::new(self.velocity.x, self.velocity.z).length()
    }

    /// Updates kinematic movement, jumping, sprinting, crouching and body rotation.
    pub fn update(
        &mut self,
        actions: &Actions,
        camera_yaw: f32,
        physics: &PhysicsWorld,
        mover: &CharacterMover,
        dt: f32,
    ) {
        if dt <= 0.0 {
            return;
        }

        self.stamina.update(dt);

        // Crouch request: check ceiling before uncrouching
        let want_crouch = actions.crouch;
        if want_crouch != self.crouching {
            if !want_crouch && !self.can_stand_up(physics) {
                // Ceiling blocked standing up, stay crouched
            } else {
                self.crouching = want_crouch;
            }
        }

        // Camera-relative wish direction
        let input = actions.move_vec;
        let (sy, cy) = camera_yaw.sin_cos();
        let forward = Vec3::new(-sy, 0.0, -cy);
        let right = Vec3::new(cy, 0.0, -sy);
        let mut wish = right * input.x + forward * (-input.y);
        wish.y = 0.0;
        if wish.length_squared() > 1.0 {
            wish = wish.normalize();
        }

        let has_input = input.length_squared() > 0.01;

        // Sprint gate: must be on floor, not crouching, pressing forward, and have stamina
        let sprint_held = actions.sprint;
        let sprinting = sprint_held
            && self.can_sprint
            && self.on_floor
            && !self.crouching
            && has_input
            && input.y <= -SPRINT_FORWARD_THRESHOLD
            && self.stamina.try_sprint(self.is_sprinting, dt);
        self.is_sprinting = sprinting;

        // Target horizontal speed with exponential smoothing (PlayerController.cs line 144)
        let base_speed = if self.crouching {
            CROUCH_SPEED
        } else if self.is_sprinting {
            SPRINT_SPEED
        } else {
            WALK_SPEED
        };
        let speed = base_speed * self.speed_multiplier;
        let target_vel = wish * speed;

        let rate = if !self.on_floor {
            ACCELERATION * AIR_CONTROL
        } else if has_input {
            ACCELERATION
        } else {
            DECELERATION
        };
        let weight = 1.0 - (-rate * dt).exp();
        let current_h = Vec2::new(self.velocity.x, self.velocity.z);
        let target_h = Vec2::new(target_vel.x, target_vel.z);
        let smoothed_h = current_h.lerp(target_h, weight);

        let mut vel = Vec3::new(
            smoothed_h.x + self.knock.x,
            self.velocity.y,
            smoothed_h.y + self.knock.z,
        );
        self.knock *= (1.0 - KNOCK_DECAY * dt).max(0.0);

        // Ground snap and gravity
        if self.on_floor {
            self.coyote_timer = COYOTE_TIME;
            if vel.y < 0.0 {
                vel.y = 0.0;
            }
        } else {
            self.coyote_timer -= dt;
            vel.y -= self.gravity * dt;
        }

        // Jump buffer: a press just before landing still triggers jump
        if actions.jump {
            self.jump_buffer_timer = JUMP_BUFFER_TIME;
        } else {
            self.jump_buffer_timer -= dt;
        }

        if self.can_jump && self.jump_buffer_timer > 0.0 && self.coyote_timer > 0.0 {
            self.jump_buffer_timer = 0.0;
            if self.stamina.try_jump() {
                vel.y = JUMP_VELOCITY;
                self.coyote_timer = 0.0;
            }
        }

        // Kinematic slide with crouch-height adjustment and autostep
        let mut active_mover = *mover;
        active_mover.height = if self.crouching {
            CROUCH_HEIGHT
        } else {
            STAND_HEIGHT
        };
        active_mover.max_step_height = MAX_STEP_HEIGHT;
        active_mover.floor_snap_length = 0.3;
        active_mover.max_floor_angle = 46.0f32.to_radians();

        let res = active_mover.move_and_slide(physics, self.position, vel, dt);
        self.position = res.position;
        self.velocity = res.velocity;
        self.on_floor = res.on_floor;
        if let Some(float_y) = self.water_float_y
            && self.position.y < float_y
        {
            self.position.y = float_y;
            if self.velocity.y < 0.0 {
                self.velocity.y = 0.0;
            }
        }
        if self.dirty_water_cooldown > 0.0 {
            self.dirty_water_cooldown = (self.dirty_water_cooldown - dt).max(0.0);
        }

        // Body facing rotation: turns toward move direction in 3rd person, faces camera in 1st person or aiming
        self.update_facing(wish, has_input, camera_yaw, actions.aim, dt);
    }

    /// Upward raycast checking for low ceilings when trying to uncrouch.
    fn can_stand_up(&self, physics: &PhysicsWorld) -> bool {
        let origin = self.position + Vec3::Y * (CROUCH_HEIGHT - 0.05);
        let max_dist = STAND_HEIGHT - CROUCH_HEIGHT + 0.1;
        physics.raycast(origin, Vec3::Y, max_dist).is_none()
    }

    /// Turns body yaw smoothly toward movement direction or snaps to camera when aiming/1st person.
    fn update_facing(
        &mut self,
        wish: Vec3,
        has_input: bool,
        camera_yaw: f32,
        is_aiming: bool,
        dt: f32,
    ) {
        if is_aiming || self.first_person {
            self.yaw = camera_yaw;
            return;
        }
        if !has_input {
            return;
        }
        let target_yaw = (-wish.x).atan2(-wish.z);
        use std::f32::consts::PI;
        let diff = (target_yaw - self.yaw + PI).rem_euclid(2.0 * PI) - PI;
        let max_step = TURN_SPEED * dt;
        let clamped_diff = diff.clamp(-max_step, max_step);
        self.yaw += clamped_diff;
        self.yaw = (self.yaw + PI).rem_euclid(2.0 * PI) - PI;
    }
}
