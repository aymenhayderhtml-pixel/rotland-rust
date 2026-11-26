//! Jam vehicle rules: one car (burnt or not, yaw jitter, seated body, luggage, clothes, shoulder
//! body sheet) and the ground props it places. All draws come from one seeded RNG, in order.

use std::f32::consts::TAU;

use glam::{Vec2, Vec3};

use crate::metro::config::MetroConfig;
use crate::metro::jam::{JamProp, Slot, dir_yaw};
use crate::random::DotnetRandom;

/// Suffix that turns a car id into its burnt variant (`car_jam_sedan_burnt`).
pub(super) const BURNT_SUFFIX: &str = "_burnt";
pub(super) const GANTRY_MODEL: &str = "highway_sign_gantry";
pub(super) const BUS_MODEL: &str = "bus_city";
pub(super) const BUS_SIDE_MODEL: &str = "bus_city_side";
pub(super) const TRUCK_MODEL: &str = "army_truck_jam";
pub(super) const BODY_MODEL: &str = "body_seated";
pub(super) const SHEET_MODEL: &str = "body_sheet";
pub(super) const CLOTHES_MODEL: &str = "clothes_spill";
/// Luggage spots pick one of these at random.
pub(super) const LUGGAGE_MODELS: [&str; 2] = ["luggage_pile", "suitcase_open"];

/// Maps a `[0, 1)` sample to `[-1, 1)`.
const SIGNED_SCALE: f32 = 2.0; // new: DECISIONS 2026-11-26 (maps [0, 1) to [-1, 1))

/// Seeded RNG plus the output list for one jam plan.
pub(super) struct Planner<'a> {
    rng: DotnetRandom,
    pub(super) cfg: &'a MetroConfig,
    pub(super) out: Vec<JamProp>,
}

impl<'a> Planner<'a> {
    pub(super) fn new(seed: i32, cfg: &'a MetroConfig) -> Self {
        Self { rng: DotnetRandom::new(seed), cfg, out: Vec::new() }
    }

    /// A sample in `[0, 1)`.
    fn unit(&mut self) -> f32 {
        self.rng.next_double() as f32
    }

    fn signed_unit(&mut self) -> f32 {
        self.unit() * SIGNED_SCALE - 1.0 // new: DECISIONS 2026-11-26 (see SIGNED_SCALE)
    }

    pub(super) fn chance(&mut self, p: f32) -> bool {
        self.unit() < p
    }

    pub(super) fn range_f(&mut self, r: [f32; 2]) -> f32 {
        r[0] + self.unit() * (r[1] - r[0])
    }

    /// Inclusive integer range.
    pub(super) fn range_usize(&mut self, r: [usize; 2]) -> usize {
        let span = (r[1].saturating_sub(r[0]) + 1) as i32;
        r[0] + self.rng.next(span) as usize
    }

    /// Uniform index below `len` (`len` must be at least 1).
    pub(super) fn index(&mut self, len: usize) -> usize {
        self.rng.next(len as i32) as usize
    }

    fn push_raw(&mut self, model: &str, pos: Vec3, yaw: f32) {
        self.out.push(JamProp { model: model.to_string(), pos, yaw, pitch: 0.0, roll: 0.0 });
    }

    /// A ground prop (y = 0) at `pos` facing `yaw`.
    pub(super) fn push_ground(&mut self, model: &str, pos: Vec2, yaw: f32) {
        self.push_raw(model, Vec3::new(pos.x, 0.0, pos.y), yaw);
    }

    /// One car in a lane slot. `wreck` forces the burnt variant. `shoulder` is the lateral
    /// offset of the shoulder edge (for the body sheet), or `None` off the highway.
    pub(super) fn push_car(&mut self, slot: &Slot, wreck: bool, shoulder: Option<f32>) {
        let cfg = self.cfg;
        let base = self.pick_model();
        let burnt = wreck || self.chance(cfg.jam.burnt_chance);
        let jitter = self.signed_unit() * cfg.jam.yaw_jitter_deg.to_radians();
        let yaw = dir_yaw(slot.heading_dir()) + jitter;
        let model = if burnt { format!("{base}{BURNT_SUFFIX}") } else { base.clone() };
        let pos = slot.pos();
        self.push_ground(&model, pos, yaw);
        if self.chance(cfg.jam.body_chance) {
            self.push_body(&base, pos, yaw);
        }
        if self.chance(cfg.jam.luggage_chance) {
            self.push_luggage(slot);
        }
        if let Some(edge) = shoulder
            && self.chance(cfg.jam.shoulder_body_chance)
        {
            let at = slot.road + slot.outward() * edge;
            self.push_ground(SHEET_MODEL, at, dir_yaw(slot.heading_dir()));
        }
    }

    fn pick_model(&mut self) -> String {
        let cfg = self.cfg;
        let models = &cfg.jam.car_models;
        let total: u32 = models.iter().map(|m| m.weight).sum();
        let mut r = self.rng.next(total as i32) as u32;
        for m in models {
            if r < m.weight {
                return m.id.clone();
            }
            r -= m.weight;
        }
        models[0].id.clone()
    }

    /// A slumped body in one of the car's seats. Cars with no seat points get no body.
    fn push_body(&mut self, base: &str, car: Vec2, yaw: f32) {
        let cfg = self.cfg;
        let Some(seats) = cfg.jam.seat_offsets.get(base).filter(|s| !s.is_empty()) else {
            return;
        };
        let seat = seats[self.index(seats.len())];
        let off = seat_offset(seat, yaw);
        self.push_raw(BODY_MODEL, Vec3::new(car.x + off.x, off.y, car.y + off.z), yaw);
    }

    /// Luggage on the lane edge beside the car, sometimes with clothes on the asphalt.
    fn push_luggage(&mut self, slot: &Slot) {
        let cfg = self.cfg;
        let offset = self.range_f(cfg.jam.luggage_offset_m);
        let spot = slot.pos() + slot.outward() * offset;
        let model = LUGGAGE_MODELS[self.index(LUGGAGE_MODELS.len())];
        let yaw = dir_yaw(slot.heading_dir());
        self.push_ground(model, spot, yaw);
        if self.chance(cfg.jam.clothes_spill_chance) {
            let spill_yaw = self.unit() * TAU;
            self.push_ground(CLOTHES_MODEL, spot, spill_yaw);
        }
    }
}

/// A seat point in model-local metres rotated by the car's yaw. Local forward is -Z, the same
/// convention as the car yaw (`(-sin yaw, -cos yaw)`).
fn seat_offset(seat: [f32; 3], yaw: f32) -> Vec3 {
    let (s, c) = yaw.sin_cos();
    Vec3::new(seat[0] * c + seat[2] * s, seat[1], -seat[0] * s + seat[2] * c)
}
