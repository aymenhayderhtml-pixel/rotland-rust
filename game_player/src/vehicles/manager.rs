use std::collections::HashMap;
use glam::{IVec2, Vec3};
use physics::PhysicsWorld;

use game_core::damage::DamageInfo;
use game_core::items::Inventory;
use crate::vehicles::car::{Car, CarDrawState};
use crate::vehicles::config::VehicleConfigData;
use crate::vehicles::controls::CarInput;
use crate::vehicles::run_over::CarRunOver;
use crate::vehicles::zombie_damage::{calculate_zombie_damage, CarDamageBox};
use crate::vehicles::save;
use crate::vehicles::spawn_rules;

/// Output frame data from Cars::update for driving audio and game events.
#[derive(Clone, Debug, Default)]
pub struct CarFrame {
    pub driven_car_id: Option<String>,
    pub speed_kmh: f32,
    pub engine_rpm: f32,
    pub crash: bool,
    /// How hard a crash was (0 = none, up to 1.0); only set when `crash` is true this frame.
    pub crash_intensity: f32,
    /// True when the driven car righted itself via [R] this frame.
    pub flip_recovered: bool,
    pub horn: bool,
    pub noise_events: Vec<(Vec3, f32)>,
}

/// Owns and manages all vehicles in the world (port of scripts/Vehicles/CarManager.cs).
#[derive(Clone, Debug)]
pub struct Cars {
    pub config: VehicleConfigData,
    pub seed: i32,
    records: HashMap<String, serde_json::Value>,
    live: HashMap<String, Car>,
    run_over_tracker: CarRunOver,
    pub driven_car_id: Option<String>,
    pub is_night: bool,
    last_crash_time: f32,
}

impl Cars {
    pub fn new(config: VehicleConfigData) -> Self {
        Self {
            config,
            seed: 0,
            records: HashMap::new(),
            live: HashMap::new(),
            run_over_tracker: CarRunOver::new(),
            driven_car_id: None,
            is_night: false,
            last_crash_time: 0.0,
        }
    }

    pub fn from_repo() -> Result<Self, String> {
        let config = VehicleConfigData::from_repo()?;
        Ok(Self::new(config))
    }

    pub fn config(&self) -> &VehicleConfigData { &self.config }
    #[allow(dead_code)]
    fn record_count(&self) -> usize { self.records.len() }
    pub fn has_record(&self, id: &str) -> bool { self.records.contains_key(id) || self.live.contains_key(id) }
    pub fn live_count(&self) -> usize { self.live.len() }
    pub fn live_cars(&self) -> impl Iterator<Item = &Car> { self.live.values() }
    pub fn live_cars_mut(&mut self) -> impl Iterator<Item = &mut Car> { self.live.values_mut() }
    pub fn get(&self, id: &str) -> Option<&Car> { self.live.get(id) }
    pub fn get_mut(&mut self, id: &str) -> Option<&mut Car> { self.live.get_mut(id) }
    pub fn driven_car(&self) -> Option<&Car> { self.driven_car_id.as_deref().and_then(|id| self.live.get(id)) }
    #[allow(dead_code)]
    fn driven_car_mut(&mut self) -> Option<&mut Car> {
        let id = self.driven_car_id.clone()?;
        self.live.get_mut(&id)
    }

    #[allow(dead_code)]
    fn add_record_for_test(&mut self, record: serde_json::Value) {
        if let Some(id) = record.get("id").and_then(|v| v.as_str()) {
            self.records.insert(id.to_string(), record);
        }
    }

    pub fn spawn_car(
        &mut self,
        id: &str,
        colour: &str,
        at: Vec3,
        yaw: f32,
        fuel: f32,
        health: f32,
    ) -> &mut Car {
        let rec = spawn_rules::new_record(id, colour, at, yaw, fuel, health);
        self.records.insert(id.to_string(), rec.clone());
        let mut car = Car::new(id, colour, &self.config);
        car.apply_record(&rec);
        match self.live.entry(id.to_string()) {
            std::collections::hash_map::Entry::Occupied(mut occ) => {
                occ.insert(car);
                occ.into_mut()
            }
            std::collections::hash_map::Entry::Vacant(vac) => vac.insert(car),
        }
    }

    pub fn respawn_chunk(&mut self, coord: IVec2) {
        let chunk_size = 64.0;
        let to_spawn: Vec<serde_json::Value> = self
            .records
            .values()
            .filter(|r| {
                let x = r.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                let z = r.get("z").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                let c = spawn_rules::chunk_of(Vec3::new(x, 0.0, z), chunk_size);
                c == coord
            })
            .cloned()
            .collect();

        for rec in to_spawn {
            if let Some(id) = rec.get("id").and_then(|v| v.as_str())
                && !self.live.contains_key(id)
            {
                let colour = rec.get("colour").and_then(|v| v.as_str()).unwrap_or("red");
                let mut car = Car::new(id, colour, &self.config);
                car.apply_record(&rec);
                self.live.insert(id.to_string(), car);
            }
        }
    }

    pub fn store_chunk(&mut self, coord: IVec2) {
        let chunk_size = 64.0;
        let mut to_store = Vec::new();
        for (id, car) in &self.live {
            let c = spawn_rules::chunk_of(car.body.position, chunk_size);
            if c == coord && !car.is_driven() {
                to_store.push(id.clone());
            }
        }
        for id in to_store {
            if let Some(car) = self.live.remove(&id) {
                self.records.insert(id, car.to_record());
            }
        }
    }

    /// Advances simulation for all live cars and steps the driven car with player input.
    pub fn update(&mut self, dt: f32, input: &CarInput, physics: &PhysicsWorld) -> CarFrame {
        let mut frame = CarFrame { driven_car_id: self.driven_car_id.clone(), ..Default::default() };
        self.last_crash_time = (self.last_crash_time - dt).max(0.0);

        for (id, car) in &mut self.live {
            let is_driven = self.driven_car_id.as_deref() == Some(id.as_str());
            let inp = if is_driven { Some(input) } else { None };
            let step_events = car.step(dt, inp, self.is_night, &self.config, physics);

            if is_driven {
                frame.speed_kmh = car.speed_kmh();
                frame.engine_rpm = step_events.engine_rpm;
                if step_events.honk {
                    frame.horn = true;
                }
                if step_events.flip_recovered {
                    frame.flip_recovered = true;
                }
            }
            if step_events.crash_intensity > 0.0 && self.last_crash_time <= 0.0 {
                frame.crash = true;
                frame.crash_intensity = frame.crash_intensity.max(step_events.crash_intensity);
                self.last_crash_time = 0.35;
            }
            if let Some(noise) = step_events.noise_made {
                frame.noise_events.push(noise);
            }
        }

        frame
    }

    /// Attempts to enter the nearest available car within 3.5 m.
    pub fn try_enter(&mut self, player_pos: Vec3, facing: Vec3) -> Option<String> {
        if self.driven_car_id.is_some() {
            return None;
        }

        let mut best: Option<(String, f32)> = None;
        for (id, car) in &self.live {
            if car.is_driven() || car.condition.is_wrecked() {
                continue;
            }
            let dist = player_pos.distance(car.body.position);
            if dist > 3.5 {
                continue;
            }
            let to_car = (car.body.position - player_pos).normalize_or_zero();
            if facing.dot(to_car) < -0.2 {
                continue;
            }
            if best.as_ref().is_none_or(|(_, d)| dist < *d) {
                best = Some((id.clone(), dist));
            }
        }

        if let Some((best_id, _)) = best
            && let Some(car) = self.live.get_mut(&best_id) {
                car.set_driven(true);
                self.driven_car_id = Some(best_id.clone());
                return Some(best_id);
            }
        None
    }

    /// Exits the car and calculates the player exit ground spot.
    pub fn exit(&mut self, car_id: &str, physics: &PhysicsWorld) -> Vec3 {
        let Some(car) = self.live.get_mut(car_id) else {
            return Vec3::ZERO;
        };

        let left = car.exit_left_local();
        let candidates = [
            left,
            Vec3::new(-left.x, left.y, left.z),
            Vec3::new(0.0, 1.5, 0.0),
        ];

        let mut exit_spot = car.body.position + car.body.rotation * left;
        for local in candidates {
            let world = car.body.position + car.body.rotation * local;
            let ground = if let Some(hit) = physics.raycast(world + Vec3::Y * 2.0, Vec3::NEG_Y, 5.0) {
                hit.point
            } else {
                world
            };
            if physics.raycast(ground + Vec3::Y * 0.1, Vec3::Y, 1.7).is_none() {
                exit_spot = ground;
                break;
            }
        }

        car.set_driven(false);
        if self.driven_car_id.as_deref() == Some(car_id) {
            self.driven_car_id = None;
        }

        exit_spot
    }

    /// Checks moving cars against targets (zombies/animals) for collisions.
    pub fn run_over(&mut self, targets: &[(u64, Vec3)]) -> Vec<(u64, DamageInfo, Vec3)> {
        let mut all_hits = Vec::new();
        for car in self.live.values_mut() {
            let hits = self.run_over_tracker.step(
                car.body.position,
                car.body.rotation,
                car.speed_kmh(),
                car.is_engine_on(),
                car.condition.is_wrecked(),
                &self.config,
                targets,
            );
            if !hits.is_empty() {
                car.condition.damage(self.config.run_over_car_damage * hits.len() as f32);
                // Plowing through zombies drags the hull a little
                let slow = (1.0 - 0.035 * hits.len() as f32).max(0.75);
                car.body.linear_velocity *= slow;
                all_hits.extend(hits);
            }
        }
        all_hits
    }

    /// Repairs a car with one scrap_metal from the bag (+50 HP); revives wrecked cars.
    pub fn repair(&mut self, car_id: &str, inventory: &mut Inventory) -> bool {
        let Some(car) = self.live.get_mut(car_id) else {
            return false;
        };
        if car.condition.fraction() >= 1.0 {
            return false;
        }
        if inventory.remove("scrap_metal", 1) {
            car.condition.repair(50.0);
            true
        } else {
            false
        }
    }

    /// Returns damage bounding boxes for all live cars.
    pub fn damage_boxes(&self) -> Vec<CarDamageBox> {
        self.live
            .values()
            .map(|car| car.damage_box(&self.config))
            .collect()
    }

    /// Evaluates damage from attacking/chasing zombies to all live cars.
    pub fn apply_zombie_damage(&mut self, zombie_positions: &[Vec3], dt: f32) -> Vec<(String, f32)> {
        let boxes = self.damage_boxes();
        let damages = calculate_zombie_damage(
            &boxes,
            zombie_positions,
            self.config.zombie_hit_damage,
            self.config.zombie_hit_seconds,
            self.config.zombie_hit_reach,
            dt,
        );
        for (id, damage) in &damages {
            if *damage > 0.0 && let Some(car) = self.live.get_mut(id) {
                car.condition.damage(*damage);
            }
        }
        damages
    }

    /// Pours a fuel_can item into the car's tank.
    pub fn refuel(&mut self, car_id: &str, inventory: &mut Inventory) -> bool {
        let Some(car) = self.live.get_mut(car_id) else {
            return false;
        };
        if car.condition.is_wrecked() || car.tank.fraction() >= 1.0 {
            return false;
        }
        if inventory.remove("fuel_can", 1) {
            car.tank.add(self.config.fuel_can_litres);
            true
        } else {
            false
        }
    }

    pub fn draw_states(&self) -> Vec<CarDrawState> {
        self.live.values().map(Car::draw_state).collect()
    }

    /// Serializes cars data matching Godot CarManager.cs Save (system id "cars").
    pub fn to_save(&self) -> serde_json::Value {
        save::to_save(&self.records, &self.live)
    }

    /// Deserializes cars data matching Godot CarManager.cs Load.
    pub fn from_save(&mut self, data: &serde_json::Value) {
        self.driven_car_id = None;
        save::from_save(&mut self.records, &mut self.live, data);
    }
}
