use glam::{Quat, Vec3};
use physics::PhysicsWorld;

use crate::vehicles::config::VehicleConfigData;
use crate::vehicles::controls::CarControls;

/// Hub rest positions in car local space (FL, FR, RL, RR), matching VEHICLE_CONTRACT.md and CarModel.cs.
pub const HUB_REST: [Vec3; 4] = [
    Vec3::new(-0.78, 0.30, -1.26), // FL
    Vec3::new(0.78, 0.30, -1.26),  // FR
    Vec3::new(-0.78, 0.30, 1.26),  // RL
    Vec3::new(0.78, 0.30, 1.26),   // RR
];

/// Anti-roll bar stiffness linking the left and right wheel of an axle.
const ANTI_ROLL_STIFFNESS: f32 = 9500.0;

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
    /// The physics tag of this car's own parked box (see `car_tag`), skipped by its rays.
    pub self_tag: u128,
    /// Fastest impact speed (m/s) this frame's obstacle sweep detected (reset every update).
    pub last_impact_speed: f32,
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
            self_tag: 0,
            last_impact_speed: 0.0,
        }
    }

    pub fn forward_speed(&self) -> f32 {
        let fwd = self.rotation * -Vec3::Z;
        self.linear_velocity.dot(fwd)
    }

    pub fn speed_kmh(&self) -> f32 {
        self.forward_speed().abs() * 3.6
    }

    /// True when the vehicle is rolled past ~75 degrees or upside-down.
    pub fn is_flipped(&self) -> bool {
        (self.rotation * Vec3::Y).y < 0.25
    }

    /// Rights an overturned vehicle in place while preserving its forward heading.
    pub fn recover_flip(&mut self) {
        let mut fwd = self.rotation * -Vec3::Z;
        fwd.y = 0.0;
        let yaw = if fwd.length_squared() > 1e-4 {
            let f = fwd.normalize();
            (-f.x).atan2(-f.z)
        } else {
            0.0
        };
        self.position.y += 0.65;
        self.rotation = Quat::from_rotation_y(yaw);
        self.linear_velocity = Vec3::ZERO;
        self.angular_velocity = Vec3::ZERO;
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
        self.last_impact_speed = 0.0;
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

        // 4WD low range: extra crawl torque when pulling uphill below 40 km/h
        if drive_total > 0.0 && fwd.y > 0.04 && speed_kmh < 40.0 {
            drive_total *= 1.0 + (fwd.y * 1.6).clamp(0.0, 0.35);
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

        // Pass 1: cast all four wheel rays (shared vertical lift so a hub pushed under
        // the ground on a hard landing still finds it)
        let mut raw_comp = [0.0f32; 4];
        let mut grounded_w = [false; 4];
        let mut hit_point = [Vec3::ZERO; 4];
        for (i, rest) in HUB_REST.iter().enumerate() {
            let hub = self.position + self.rotation * *rest;
            let max_dist = cfg.suspension_rest + cfg.suspension_travel + 0.1;
            // Cast from 1 m above the hub: a hub pushed under the ground on a hard landing or a slope
            // change must still find the ground (a ray starting inside it misses, and the car fell through)
            const LIFT: f32 = 1.0;
            let hit = physics.raycast_except(hub + up * LIFT, -up, max_dist + LIFT, self.self_tag).map(|mut h| {
                h.distance -= LIFT;
                h
            });

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
            grounded_w[i] = true;
            hit_point[i] = hit.point;
            // Godot CarDrive: the spring uses the full compression; only the reported value is clamped
            let compression = cfg.suspension_rest - dist;
            raw_comp[i] = compression;
            self.compression[i] = compression.clamp(-cfg.suspension_travel, cfg.suspension_travel);
        }

        // Anti-roll bars couple the left and right wheels of an axle; equal
        // compression (flat ground) cancels out and keeps equilibrium exact
        let front_arb = if grounded_w[0] && grounded_w[1] {
            (self.compression[0] - self.compression[1]) * ANTI_ROLL_STIFFNESS
        } else {
            0.0
        };
        let rear_arb = if grounded_w[2] && grounded_w[3] {
            (self.compression[2] - self.compression[3]) * ANTI_ROLL_STIFFNESS
        } else {
            0.0
        };
        let arb = [front_arb, -front_arb, rear_arb, -rear_arb];

        // Pass 2: suspension load, tires, drive and brakes, friction circle
        for i in 0..4 {
            if !grounded_w[i] {
                continue;
            }
            let front = i <= 1;

            let steer_rot = if front {
                Quat::from_axis_angle(up, self.steer_angle)
            } else {
                Quat::IDENTITY
            };
            let wheel_forward = (steer_rot * fwd).normalize();
            let wheel_right = wheel_forward.cross(up).normalize();

            let offset = hit_point[i] - com_world;
            let point_velocity = self.linear_velocity + self.angular_velocity.cross(offset);
            let v_long = point_velocity.dot(wheel_forward);
            let v_lat = point_velocity.dot(wheel_right);

            // Progressive bump-stop: stiffer the deeper the suspension packs down (above 85% travel)
            let bump_thresh = cfg.suspension_travel * 0.85;
            let bump_extra = (raw_comp[i] - bump_thresh).max(0.0) * cfg.spring_stiffness * 2.5;

            let load = (cfg.spring_stiffness * raw_comp[i]
                + bump_extra
                + arb[i]
                - cfg.spring_damping * point_velocity.dot(up))
            .max(0.0);

            // Hill-hold: parked (or braking) on a slope, holds fore-aft and cross-slope without creeping
            let parked_hold = (!engine_on || controls.handbrake || controls.brake > 0.5)
                && v_long.abs() < 0.8
                && throttle.abs() < 0.05;

            // Lateral: cross-slope hold, slip-angle curve, or linear damping
            let grip = if !front && controls.handbrake {
                cfg.handbrake_grip
            } else if !front {
                1.05
            } else {
                1.0
            };
            let lat_max = cfg.grip_mu * load * grip;
            let lateral = if parked_hold && v_lat.abs() < 0.8 {
                let cross_slope_hold = wheel_mass * 9.8 * wheel_right.y;
                (-v_lat * wheel_mass * 14.0 + cross_slope_hold).clamp(-lat_max, lat_max)
            } else if v_long.abs() > 0.6 {
                let slip = (v_lat / v_long.abs().max(2.5)) * 4.5;
                let raw_lat = -lat_max * (slip / (1.0 + slip.abs()));
                let no_flip = v_lat.abs() * wheel_mass / dt * 0.85;
                raw_lat.clamp(-no_flip, no_flip)
            } else {
                (-v_lat * wheel_mass * 12.0).clamp(-lat_max, lat_max)
            };

            // Longitudinal: drive (4WD split), hill-hold slope force, brake, rolling drag
            let drive_share = if front { 0.225 } else { 0.275 };
            let mut f_long = drive_total * drive_share;
            let wheel_brake = brake_total * 0.25;
            if parked_hold {
                let slope_hold = wheel_mass * 9.8 * wheel_forward.y;
                let creep_damp = -v_long * wheel_mass * 14.0;
                let hold_cap = cfg.grip_mu * load;
                f_long = (slope_hold + creep_damp).clamp(-hold_cap, hold_cap);
            } else if wheel_brake > 0.0 && v_long.abs() > 0.01 {
                let cap = v_long.abs() * wheel_mass / dt * 0.65;
                f_long += -v_long.signum() * wheel_brake.min(cap);
            } else if throttle.abs() < 0.05 && v_long.abs() > 0.05 {
                f_long += -v_long.signum() * 55.0;
            }

            // Combined friction circle: longitudinal and lateral share one grip budget
            let max_planar = cfg.grip_mu * load * 1.15;
            let planar_mag = f_long.hypot(lateral);
            let mut lateral = lateral;
            if planar_mag > max_planar && planar_mag > 1e-4 {
                let scale = max_planar / planar_mag;
                f_long *= scale;
                lateral *= scale;
            }

            let wheel_force = up * load + wheel_right * lateral + wheel_forward * f_long;
            net_force += wheel_force;
            net_torque += offset.cross(wheel_force);
        }

        // Aerodynamic drag (quadratic in speed)
        let speed = self.linear_velocity.length();
        net_force += -self.linear_velocity * speed * 0.35;

        let lin_acc = net_force / cfg.mass;
        self.linear_velocity += lin_acc * dt;
        self.linear_velocity *= (1.0 - 0.02 * dt).max(0.0);
        self.position += self.linear_velocity * dt;

        // Box inertia (w = 1.72, h = 1.24, d = 3.92)
        let ix = (cfg.mass / 12.0) * (1.24 * 1.24 + 3.92 * 3.92);
        let iy = (cfg.mass / 12.0) * (1.72 * 1.72 + 3.92 * 3.92);
        let iz = (cfg.mass / 12.0) * (1.72 * 1.72 + 1.24 * 1.24);
        let inv_inertia = Vec3::new(1.0 / ix, 1.0 / iy, 1.0 / iz);

        let local_torque = self.rotation.inverse() * net_torque;
        let local_ang_acc = local_torque * inv_inertia;
        let ang_acc = self.rotation * local_ang_acc;
        self.angular_velocity += ang_acc * dt;
        let ang_damp = if self.grounded { 1.6 } else { 2.5 };
        self.angular_velocity *= (1.0 - ang_damp * dt).max(0.0);

        let ang_speed = self.angular_velocity.length();
        if ang_speed > 1e-6 {
            let axis = self.angular_velocity / ang_speed;
            let angle = ang_speed * dt;
            self.rotation = (Quat::from_axis_angle(axis, angle) * self.rotation).normalize();
        }

        self.pushout_obstacles(com_world, physics);
    }

    /// Pushes the body out of static obstacles at bumper and cabin height with ring sweeps.
    /// Records impact speed in `last_impact_speed`, reflects the into-wall velocity,
    /// scrapes tangential motion, and nudges the car off-center in a crash.
    fn pushout_obstacles(&mut self, com_world: Vec3, physics: &PhysicsWorld) {
        let bumper_origin = self.position + self.rotation * Vec3::new(0.0, 0.62, 0.0);
        let roof_origin = self.position + self.rotation * Vec3::new(0.0, 1.38, 0.0);

        // Bumper level: nose, tail, flanks, corners, front-mid flanks
        let bumper_points = [
            Vec3::new(0.0, 0.0, -2.02),
            Vec3::new(0.0, 0.0, 2.02),
            Vec3::new(-0.88, 0.0, 0.0),
            Vec3::new(0.88, 0.0, 0.0),
            Vec3::new(-0.84, 0.0, -1.96),
            Vec3::new(0.84, 0.0, -1.96),
            Vec3::new(-0.84, 0.0, 1.96),
            Vec3::new(0.84, 0.0, 1.96),
            Vec3::new(-0.86, 0.0, -0.95),
            Vec3::new(0.86, 0.0, -0.95),
        ];
        // Cabin/roof level: four corners of the greenhouse
        let roof_points = [
            Vec3::new(-0.70, 0.0, -1.55),
            Vec3::new(0.70, 0.0, -1.55),
            Vec3::new(-0.70, 0.0, 1.65),
            Vec3::new(0.70, 0.0, 1.65),
        ];

        for (origin, points) in [(bumper_origin, &bumper_points[..]), (roof_origin, &roof_points[..])] {
            for &local_pt in points {
                let world_target = origin + self.rotation * local_pt;
                let delta = world_target - origin;
                let dist = delta.length();
                if dist < 1e-4 {
                    continue;
                }
                let dir = delta / dist;
                // Ground and slopes (normal pointing up) are the wheels' job, not walls: driving up a hill
                // must not count as hitting one
                if let Some(hit) = physics.raycast_except(origin, dir, dist, self.self_tag) {
                    if hit.normal.y >= 0.6 {
                        continue;
                    }
                    let penetration = dist - hit.distance;
                    if penetration <= 0.001 {
                        continue;
                    }
                    self.position += hit.normal * penetration;
                    // Into-wall velocity (positive when heading into the obstacle)
                    let v_into = -self.linear_velocity.dot(hit.normal);
                    if v_into > 0.0 {
                        self.last_impact_speed = self.last_impact_speed.max(v_into);
                        self.linear_velocity += hit.normal * (v_into * 1.18);
                        // Scrape friction along the wall's tangent
                        let tangent = self.linear_velocity - hit.normal * self.linear_velocity.dot(hit.normal);
                        self.linear_velocity -= tangent * 0.10;
                        // Subtle off-center yaw deflection in a crash
                        let yaw_kick = (world_target - com_world).cross(hit.normal * v_into).y * 0.14;
                        self.angular_velocity.y += yaw_kick.clamp(-2.5, 2.5);
                    }
                }
            }
        }
    }
}
