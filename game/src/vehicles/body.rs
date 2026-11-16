use glam::{Quat, Vec3};
use physics::PhysicsWorld;

use crate::vehicles::config::VehicleConfigData;
use crate::vehicles::controls::CarControls;

/// Hub rest positions in car local space (FL, FR, RL, RR), matching VEHICLE_CONTRACT.md and CarModel.cs.
pub const HUB_REST: [Vec3; 4] = [
    Vec3::new(-0.72, 0.30, -1.22), // FL
    Vec3::new(0.72, 0.30, -1.22),  // FR
    Vec3::new(-0.72, 0.30, 1.22),  // RL
    Vec3::new(0.72, 0.30, 1.22),   // RR
];

/// Rigid body physics simulation for the car (port of CarDrive.cs and RigidBody3D setup).
#[derive(Clone, Debug, PartialEq)]
pub struct CarBody {
    pub position: Vec3,
    pub rotation: Quat,
    pub linear_velocity: Vec3,
    pub angular_velocity: Vec3,
    pub compression: [f32; 4],
    pub steer_angle: f32,
    pub grounded: bool,
}

impl Default for CarBody {
    fn default() -> Self {
        Self::new(Vec3::ZERO, Quat::IDENTITY)
    }
}

impl CarBody {
    pub fn new(position: Vec3, rotation: Quat) -> Self {
        Self {
            position,
            rotation,
            linear_velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            compression: [0.0; 4],
            steer_angle: 0.0,
            grounded: false,
        }
    }

    pub fn forward_speed(&self) -> f32 {
        let fwd = self.rotation * -Vec3::Z;
        self.linear_velocity.dot(fwd)
    }

    pub fn speed_kmh(&self) -> f32 {
        self.forward_speed().abs() * 3.6
    }

    /// Advances the body physics by dt, breaking into 1/60s sub-steps.
    pub fn update(
        &mut self,
        dt: f32,
        controls: &CarControls,
        engine_on: bool,
        cfg: &VehicleConfigData,
        physics: &PhysicsWorld,
    ) {
        let sub_dt = 1.0 / 60.0;
        let mut remaining = dt;
        while remaining > 0.0001 {
            let step = remaining.min(sub_dt);
            self.substep(step, controls, engine_on, cfg, physics);
            remaining -= step;
        }
    }

    /// Single sub-step of physics integration matching CarDrive.cs.
    fn substep(
        &mut self,
        dt: f32,
        controls: &CarControls,
        engine_on: bool,
        cfg: &VehicleConfigData,
        physics: &PhysicsWorld,
    ) {
        let fwd = self.rotation * -Vec3::Z;
        let up = self.rotation * Vec3::Y;
        let forward_speed = self.linear_velocity.dot(fwd);
        let speed_kmh = forward_speed.abs() * 3.6;

        let steer_target = if engine_on {
            -controls.steer
                * cfg.max_steer_rad
                * (1.0 - 0.4 * (speed_kmh / cfg.top_speed_kmh).clamp(0.0, 1.0))
        } else {
            0.0
        };
        let max_steer_step = cfg.steer_speed_rad * dt;
        let diff = steer_target - self.steer_angle;
        if diff.abs() <= max_steer_step {
            self.steer_angle = steer_target;
        } else {
            self.steer_angle += diff.signum() * max_steer_step;
        }

        let throttle = if engine_on {
            controls.throttle.clamp(-1.0, 1.0)
        } else {
            0.0
        };
        let mut drive_total = 0.0f32;
        let mut brake_total = cfg.brake_force * controls.brake.clamp(0.0, 1.0);

        if throttle > 0.0 && forward_speed < -0.5 {
            brake_total = brake_total.max(cfg.brake_force * throttle);
        } else if throttle > 0.0 {
            drive_total =
                cfg.engine_force * throttle * (1.0 - speed_kmh / cfg.top_speed_kmh).clamp(0.0, 1.0);
        } else if throttle < 0.0 && forward_speed > 0.5 {
            brake_total = brake_total.max(cfg.brake_force * -throttle);
        } else if throttle < 0.0 {
            drive_total = -cfg.reverse_force
                * -throttle
                * (1.0 - speed_kmh / cfg.reverse_top_kmh).clamp(0.0, 1.0);
        }

        if controls.handbrake {
            brake_total = brake_total.max(cfg.brake_force * 0.5);
        }

        let com_local = Vec3::new(0.0, 0.3, 0.0);
        let com_world = self.position + self.rotation * com_local;

        let mut net_force = Vec3::new(0.0, -cfg.mass * 9.8, 0.0);
        let mut net_torque = Vec3::ZERO;

        self.grounded = false;
        let wheel_mass = cfg.mass / 4.0;

        for i in 0..4 {
            let hub = self.position + self.rotation * HUB_REST[i];
            let max_dist = cfg.suspension_rest + cfg.suspension_travel + 0.1;
            let hit = physics.raycast(hub, -up, max_dist);

            let Some(hit) = hit else {
                self.compression[i] = 0.0;
                continue;
            };

            let dist = hit.distance;
            if dist > cfg.suspension_rest + cfg.suspension_travel {
                self.compression[i] = 0.0;
                continue;
            }

            self.grounded = true;
            let compression = (cfg.suspension_rest - dist).clamp(-cfg.suspension_travel, cfg.suspension_travel);
            self.compression[i] = compression;

            let point = hit.point;
            let offset = point - com_world;
            let point_velocity = self.linear_velocity + self.angular_velocity.cross(offset);

            let load = (cfg.spring_stiffness * compression - cfg.spring_damping * point_velocity.dot(up)).max(0.0);
            let mut wheel_force = up * load;

            let front = i <= 1;
            let steer_rot = if front {
                Quat::from_axis_angle(up, self.steer_angle)
            } else {
                Quat::IDENTITY
            };
            let wheel_forward = (steer_rot * fwd).normalize();
            let wheel_right = wheel_forward.cross(up).normalize();
            let v_long = point_velocity.dot(wheel_forward);
            let v_lat = point_velocity.dot(wheel_right);

            let friction_max = cfg.grip_mu * load;
            let grip = if !front && controls.handbrake {
                cfg.handbrake_grip
            } else {
                1.0
            };
            let lateral = (-v_lat * wheel_mass / dt * 0.5).clamp(-friction_max * grip, friction_max * grip);
            wheel_force += wheel_right * lateral;

            if front {
                wheel_force += wheel_forward * (drive_total * 0.5);
            }

            let wheel_brake = brake_total / 4.0;
            if wheel_brake > 0.0 && v_long.abs() > 0.01 {
                let cap = v_long.abs() * wheel_mass / dt * 0.5;
                wheel_force += -wheel_forward * v_long.signum() * wheel_brake.min(cap);
            }

            net_force += wheel_force;
            net_torque += offset.cross(wheel_force);
        }

        let lin_acc = net_force / cfg.mass;
        self.linear_velocity += lin_acc * dt;
        self.linear_velocity *= (1.0 - 0.05 * dt).max(0.0);
        self.position += self.linear_velocity * dt;

        // Box inertia: w=1.56, h=1.16, d=3.76
        let ix = (cfg.mass / 12.0) * (1.16 * 1.16 + 3.76 * 3.76);
        let iy = (cfg.mass / 12.0) * (1.56 * 1.56 + 3.76 * 3.76);
        let iz = (cfg.mass / 12.0) * (1.56 * 1.56 + 1.16 * 1.16);
        let inv_inertia = Vec3::new(1.0 / ix, 1.0 / iy, 1.0 / iz);

        let local_torque = self.rotation.inverse() * net_torque;
        let local_ang_acc = local_torque * inv_inertia;
        let ang_acc = self.rotation * local_ang_acc;
        self.angular_velocity += ang_acc * dt;
        self.angular_velocity *= (1.0 - 1.5 * dt).max(0.0);

        let ang_speed = self.angular_velocity.length();
        if ang_speed > 1e-6 {
            let axis = self.angular_velocity / ang_speed;
            let angle = ang_speed * dt;
            self.rotation = (Quat::from_axis_angle(axis, angle) * self.rotation).normalize();
        }

        self.pushout_obstacles(physics);
    }

    /// Pushes the body out of static obstacles (walls, buildings, trees) with horizontal corner raycasts.
    fn pushout_obstacles(&mut self, physics: &PhysicsWorld) {
        let center = self.position + self.rotation * Vec3::new(0.0, 0.4, 0.0);
        let corners = [
            Vec3::new(0.0, 0.0, -1.9),     // Front
            Vec3::new(0.0, 0.0, 1.9),      // Rear
            Vec3::new(-0.85, 0.0, 0.0),    // Left
            Vec3::new(0.85, 0.0, 0.0),     // Right
            Vec3::new(-0.8, 0.0, -1.85),   // Front-left
            Vec3::new(0.8, 0.0, -1.85),    // Front-right
            Vec3::new(-0.8, 0.0, 1.85),    // Rear-left
            Vec3::new(0.8, 0.0, 1.85),     // Rear-right
        ];

        for local_pt in corners {
            let world_target = center + self.rotation * local_pt;
            let delta = world_target - center;
            let dist = delta.length();
            if dist < 1e-4 {
                continue;
            }
            let dir = delta / dist;
            if let Some(hit) = physics.raycast(center, dir, dist) {
                let penetration = dist - hit.distance;
                if penetration > 0.001 {
                    self.position += hit.normal * penetration;
                    let v_into = self.linear_velocity.dot(hit.normal);
                    if v_into < 0.0 {
                        self.linear_velocity -= hit.normal * v_into;
                    }
                }
            }
        }
    }
}
