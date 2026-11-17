use glam::{Mat4, Quat, Vec3};
use physics::PhysicsWorld;

use crate::items::Inventory;
use crate::vehicles::body::{CarBody, HUB_REST};
use crate::vehicles::config::VehicleConfigData;
use crate::vehicles::controls::{CarControls, CarInput};
use crate::vehicles::fuel::CarFuel;
use crate::vehicles::health::CarHealth;

pub const WHEEL_RADIUS: f32 = 0.30;
pub const NOISE_INTERVAL: f32 = 0.5;

/// Visual draw state of a car for the engine renderer.
#[derive(Clone, Debug, PartialEq)]
pub struct CarDrawState {
    pub id: String,
    pub model_name: String,
    pub world_matrix: Mat4,
    pub wheel_matrices: [Mat4; 4],
    pub lights_on: bool,
}

/// Events produced by a car step.
#[derive(Clone, Debug, Default)]
pub struct CarStepEvents {
    pub noise_made: Option<(Vec3, f32)>,
    pub honk: bool,
    pub engine_rpm: f32,
}

/// A drivable car entity (port of scripts/Vehicles/Car.cs).
#[derive(Clone, Debug)]
pub struct Car {
    pub id: String,
    pub colour: String,
    pub body: CarBody,
    pub tank: CarFuel,
    pub condition: CarHealth,
    pub trunk: Inventory,
    pub controls: CarControls,
    pub lights_on: bool,
    pub spin: [f32; 4],
    pub driven: bool,
    pub driver_id: Option<u64>,
    noise_timer: f32,
    manual_lights: Option<bool>,
}

impl Car {
    pub fn new(id: impl Into<String>, colour: impl Into<String>, cfg: &VehicleConfigData) -> Self {
        let trunk = Inventory::with_capacity(cfg.trunk_slots, cfg.trunk_max_weight);
        let id = id.into();
        let body = CarBody { self_tag: car_tag(&id), ..CarBody::new(Vec3::ZERO, Quat::IDENTITY) };
        Self {
            id,
            colour: colour.into(),
            body,
            tank: CarFuel::new(cfg.tank_litres),
            condition: CarHealth::new(cfg.health),
            trunk,
            controls: CarControls::default(),
            lights_on: false,
            spin: [0.0; 4],
            driven: false,
            driver_id: None,
            noise_timer: 0.0,
            manual_lights: None,
        }
    }

    pub fn is_driven(&self) -> bool {
        self.driven
    }

    pub fn set_driven(&mut self, driven: bool) {
        self.driven = driven;
        if !driven {
            self.driver_id = None;
            self.controls.clear();
            self.controls.handbrake = true;
        }
    }

    pub fn is_engine_on(&self) -> bool {
        self.driven && !self.tank.is_empty() && !self.condition.is_wrecked()
    }

    pub fn speed_kmh(&self) -> f32 {
        self.body.speed_kmh()
    }

    pub fn forward_speed(&self) -> f32 {
        self.body.forward_speed()
    }

    pub fn compression(&self, wheel: usize) -> f32 {
        self.body.compression[wheel.min(3)]
    }

    pub fn is_grounded(&self) -> bool {
        self.body.grounded
    }

    /// Driver seat and left exit in car local space.
    pub fn seat_local(&self) -> Vec3 {
        Vec3::new(-0.32, 0.50, -0.20)
    }

    pub fn exit_left_local(&self) -> Vec3 {
        Vec3::new(-2.00, 0.00, -0.50)
    }

    /// Advance car logic and physics for one step.
    pub fn step(
        &mut self,
        dt: f32,
        input: Option<&CarInput>,
        is_night: bool,
        cfg: &VehicleConfigData,
        physics: &PhysicsWorld,
    ) -> CarStepEvents {
        let mut events = CarStepEvents::default();

        if !self.driven {
            self.controls.clear();
            self.controls.handbrake = true;
        } else if let Some(inp) = input {
            self.controls = inp.to_controls();
            if inp.lights_toggle {
                let current = self.manual_lights.unwrap_or(self.lights_on);
                self.manual_lights = Some(!current);
            }
            if inp.horn {
                events.honk = true;
                events.noise_made = Some((self.body.position, cfg.horn_radius));
            }
        }

        let engine_on = self.is_engine_on();
        if engine_on {
            self.burn_fuel(dt, cfg);
        }

        self.body.update(dt, &self.controls, engine_on, cfg, physics);

        let speed = self.forward_speed();
        for i in 0..4 {
            self.spin[i] += speed * dt / WHEEL_RADIUS;
        }

        if engine_on {
            self.noise_timer -= dt;
            if self.noise_timer <= 0.0 {
                self.noise_timer = NOISE_INTERVAL;
                let load = self.controls.throttle.abs();
                let radius = cfg.noise_radius_idle + (cfg.noise_radius_full - cfg.noise_radius_idle) * load;
                if events.noise_made.is_none() {
                    events.noise_made = Some((self.body.position, radius));
                }
            }
            let load = self.controls.throttle.abs();
            events.engine_rpm = (0.5 * (self.speed_kmh() / cfg.top_speed_kmh).clamp(0.0, 1.0) + 0.5 * load).clamp(0.0, 1.0);
        }

        self.lights_on = match self.manual_lights {
            Some(manual) => manual,
            None => engine_on && is_night,
        };

        events
    }

    fn burn_fuel(&mut self, dt: f32, cfg: &VehicleConfigData) {
        let metres = self.forward_speed().abs() * dt;
        let load = self.controls.throttle.abs();
        let per_metre = cfg.litres_per_metre_cruise
            + (cfg.litres_per_metre_full_throttle - cfg.litres_per_metre_cruise) * load;
        self.tank.burn(metres * per_metre);
    }

    pub fn to_record(&self) -> serde_json::Value {
        let p = self.body.position;
        let q = self.body.rotation;
        let trunk_save = serde_json::to_value(self.trunk.save()).unwrap_or(serde_json::json!({"stacks": []}));
        serde_json::json!({
            "id": self.id,
            "colour": self.colour,
            "x": p.x,
            "y": p.y,
            "z": p.z,
            "qx": q.x,
            "qy": q.y,
            "qz": q.z,
            "qw": q.w,
            "fuel": self.tank.litres(),
            "health": self.condition.current(),
            "trunk": trunk_save,
        })
    }

    pub fn apply_record(&mut self, record: &serde_json::Value) {
        if let Some(x) = record.get("x").and_then(|v| v.as_f64())
            && let Some(y) = record.get("y").and_then(|v| v.as_f64())
            && let Some(z) = record.get("z").and_then(|v| v.as_f64())
        {
            self.body.position = Vec3::new(x as f32, y as f32, z as f32);
        }
        let qx = record.get("qx").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
        let qy = record.get("qy").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
        let qz = record.get("qz").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
        let qw = record.get("qw").and_then(|v| v.as_f64()).unwrap_or(1.0) as f32;
        self.body.rotation = Quat::from_xyzw(qx, qy, qz, qw).normalize();
        self.body.linear_velocity = Vec3::ZERO;
        self.body.angular_velocity = Vec3::ZERO;

        if let Some(fuel) = record.get("fuel").and_then(|v| v.as_f64()) {
            self.tank.set(fuel as f32);
        }
        if let Some(health) = record.get("health").and_then(|v| v.as_f64()) {
            self.condition.set(health as f32);
        }
        if let Some(trunk_val) = record.get("trunk")
            && let Ok(save_data) = serde_json::from_value(trunk_val.clone())
        {
            self.trunk.load_save(&save_data);
        }
    }

    pub fn draw_state(&self) -> CarDrawState {
        let world_matrix = Mat4::from_rotation_translation(self.body.rotation, self.body.position);
        let mut wheel_matrices = [Mat4::IDENTITY; 4];

        for i in 0..4 {
            let hub = HUB_REST[i];
            let steer = if i <= 1 { self.body.steer_angle } else { 0.0 };
            let local_trans = Mat4::from_translation(Vec3::new(hub.x, hub.y + self.body.compression[i], hub.z));
            let local_rot = Mat4::from_rotation_y(steer) * Mat4::from_rotation_x(self.spin[i]);
            wheel_matrices[i] = world_matrix * (local_trans * local_rot);
        }

        CarDrawState {
            id: self.id.clone(),
            model_name: format!("car_hatchback_{}", self.colour),
            world_matrix,
            wheel_matrices,
            lights_on: self.lights_on,
        }
    }
}

/// The physics tag of a car's parked box: a stable non-zero hash of its id (FNV-1a).
pub fn car_tag(id: &str) -> u128 {
    let mut h: u128 = 0x6c62272e07bb014262b821756295c58d;
    for b in id.bytes() {
        h ^= b as u128;
        h = h.wrapping_mul(0x0000000001000000000000000000013B);
    }
    h | 1
}
