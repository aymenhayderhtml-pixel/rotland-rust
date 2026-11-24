//! Car driver seating, steering and exit animation state (A11).
//!
//! Drives the `Air` layer with `car_sit_drive` (fading in over `enterFade`),
//! borrows `Crouch` on the upper body for `car_steer_left`/`car_steer_right`
//! (positive `steer_angle` = left, matching Godot +Y yaw), suppresses weapon
//! layers while seated, and emits one-shot exit/thrown-out events on exit.
//! Leaves the animator untouched until `set_input` is called.

use std::sync::OnceLock;

use glam::{Mat4, Vec3};
use serde::Deserialize;

use crate::layers::{LayerBlend, LayerId, LayerStack};
use crate::mask::MaskId;

/// Looping seated driver pose with hands on the steering wheel.
pub const CAR_SIT_DRIVE_CLIP: &str = "car_sit_drive";
/// Upper-body looping clip turning the wheel to the left (positive steer angle).
pub const CAR_STEER_LEFT_CLIP: &str = "car_steer_left";
/// Upper-body looping clip turning the wheel to the right (negative steer angle).
pub const CAR_STEER_RIGHT_CLIP: &str = "car_steer_right";
/// One-shot clip played at the exit spot when getting out of an upright car.
pub const CAR_GET_OUT_CLIP: &str = "car_get_out";
/// One-shot clip played when exiting a flipped car.
pub const CAR_THROWN_OUT_CLIP: &str = "car_thrown_out";

const JUMP_FALLBACK_CLIP: &str = "jump";
const DEFAULT_SPEED: f32 = 1.0; // new: DECISIONS 2026-11-24
const FULL_WEIGHT: f32 = 1.0; // new: DECISIONS 2026-11-24
const ZERO_WEIGHT: f32 = 0.0; // new: DECISIONS 2026-11-24
const ZERO_TIME: f32 = 0.0; // new: DECISIONS 2026-11-24

/// Tuning values for car driver animations from `data/car_anim.json`.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CarAnimConfig {
    pub version: u32,
    pub seat_offset: [f32; 3],
    pub seat_yaw_deg: f32,
    pub enter_fade: f32,
    pub steer_full_rad: f32,
    pub steer_rate: f32,
    pub steer_max_weight: f32,
    pub exit_fade: f32,
    pub thrown_fade: f32,
}

impl Default for CarAnimConfig {
    fn default() -> Self {
        Self {
            version: 1,                        // data: car_anim.json version
            seat_offset: [-0.36, 0.05, -0.12], // data: car_anim.json seatOffset
            seat_yaw_deg: 0.0,                 // data: car_anim.json seatYawDeg
            enter_fade: 0.3,                   // data: car_anim.json enterFade
            steer_full_rad: 0.45,              // data: car_anim.json steerFullRad
            steer_rate: 8.0,                   // data: car_anim.json steerRate
            steer_max_weight: 1.0,             // data: car_anim.json steerMaxWeight
            exit_fade: 0.15,                   // data: car_anim.json exitFade
            thrown_fade: 0.1,                  // data: car_anim.json thrownFade
        }
    }
}

impl CarAnimConfig {
    /// Parses a `CarAnimConfig` from JSON text.
    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }

    /// Loads and caches `data/car_anim.json`, falling back to defaults.
    pub fn load_shared() -> &'static CarAnimConfig {
        static CONFIG: OnceLock<CarAnimConfig> = OnceLock::new();
        CONFIG.get_or_init(|| {
            let path = paths::repo_path("data/car_anim.json");
            match std::fs::read_to_string(&path) {
                Ok(text) => match Self::from_json(&text) {
                    Ok(cfg) => cfg,
                    Err(err) => {
                        log::warn!("failed to parse {}: {err}; using defaults", path.display());
                        Self::default()
                    }
                },
                Err(err) => {
                    log::warn!("failed to read {}: {err}; using defaults", path.display());
                    Self::default()
                }
            }
        })
    }
}

/// One-shot transition clip emitted when the player leaves the driver's seat.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CarEvent {
    GotOut,
    ThrownOut,
}

/// Returns the clip name for a car exit event.
pub fn car_event_clip(event: CarEvent) -> &'static str {
    match event {
        CarEvent::GotOut => CAR_GET_OUT_CLIP,
        CarEvent::ThrownOut => CAR_THROWN_OUT_CLIP,
    }
}

/// Character world matrix for the seat from the car body's world matrix.
pub fn seat_world(car_world: Mat4, cfg: &CarAnimConfig) -> Mat4 {
    let offset = Vec3::from_array(cfg.seat_offset);
    let yaw = if cfg.seat_yaw_deg.is_finite() {
        cfg.seat_yaw_deg.to_radians()
    } else {
        ZERO_TIME
    };
    let local = Mat4::from_translation(offset) * Mat4::from_rotation_y(yaw);
    car_world * local
}

#[derive(Clone, Debug)]
struct SavedLayer {
    clip: Option<String>,
    mask: MaskId,
    blend: LayerBlend,
    speed: f32,
    looping: bool,
}

/// Per-character car driver animation state machine.
#[derive(Clone, Debug)]
pub struct CarAnimState {
    config: CarAnimConfig,
    driven: bool,
    seated: bool,
    was_seated: bool,
    steer_rad: f32,
    seat_weight: f32,
    steer_weight: f32,
    last_flipped: bool,
    owns_layers: bool,
    saved_steer_layer: Option<SavedLayer>,
}

impl Default for CarAnimState {
    fn default() -> Self {
        Self::new()
    }
}

impl CarAnimState {
    /// Creates an inert state using the shared `data/car_anim.json` config.
    pub fn new() -> Self {
        Self::with_config(CarAnimConfig::load_shared().clone())
    }

    /// Creates an inert state with an explicit config (preferred for tests).
    pub fn with_config(config: CarAnimConfig) -> Self {
        Self {
            config,
            driven: false,
            seated: false,
            was_seated: false,
            steer_rad: ZERO_WEIGHT,
            seat_weight: ZERO_WEIGHT,
            steer_weight: ZERO_WEIGHT,
            last_flipped: false,
            owns_layers: false,
            saved_steer_layer: None,
        }
    }

    /// seated = the player is driving; steer = car steer angle (rad, positive = left);
    /// flipped = car upside down/on its side. Inert until first call.
    pub fn set_input(&mut self, seated: bool, steer: f32, flipped: bool) {
        self.seated = seated;
        self.steer_rad = if steer.is_finite() { steer } else { ZERO_WEIGHT };
        if seated {
            self.last_flipped = flipped;
        } else if flipped {
            self.last_flipped = true;
        }
        self.driven = true;
    }

    /// Returns an event once, on the frame the player leaves the seat.
    pub fn tick(&mut self, dt: f32) -> Option<CarEvent> {
        if !self.driven {
            return None;
        }
        let step = if dt.is_finite() && dt > ZERO_TIME { dt } else { ZERO_TIME };
        let event = if !self.seated && self.was_seated {
            Some(if self.last_flipped {
                CarEvent::ThrownOut
            } else {
                CarEvent::GotOut
            })
        } else {
            None
        };

        if self.seated {
            let enter_fade = self.config.enter_fade;
            if !self.was_seated {
                self.seat_weight = if enter_fade.is_finite() && enter_fade > ZERO_TIME {
                    ZERO_WEIGHT
                } else {
                    FULL_WEIGHT
                };
                self.steer_weight = ZERO_WEIGHT;
            }
            if enter_fade.is_finite() && enter_fade > ZERO_TIME {
                self.seat_weight = (self.seat_weight + step / enter_fade)
                    .clamp(ZERO_WEIGHT, FULL_WEIGHT);
            } else {
                self.seat_weight = FULL_WEIGHT;
            }

            let full_rad = self.config.steer_full_rad;
            let max_w = if self.config.steer_max_weight.is_finite() {
                self.config.steer_max_weight.clamp(ZERO_WEIGHT, FULL_WEIGHT)
            } else {
                FULL_WEIGHT
            };
            let target = if full_rad.is_finite() && full_rad > ZERO_WEIGHT {
                (self.steer_rad / full_rad).clamp(-FULL_WEIGHT, FULL_WEIGHT) * max_w
            } else {
                ZERO_WEIGHT
            };
            if step > ZERO_TIME {
                let rate = if self.config.steer_rate.is_finite()
                    && self.config.steer_rate > ZERO_WEIGHT
                {
                    self.config.steer_rate
                } else {
                    ZERO_WEIGHT
                };
                let k = FULL_WEIGHT - (-rate * step).exp();
                self.steer_weight += (target - self.steer_weight) * k;
            }
            self.steer_weight = if self.steer_weight.is_finite() {
                self.steer_weight.clamp(-FULL_WEIGHT, FULL_WEIGHT)
            } else {
                ZERO_WEIGHT
            };
        } else {
            self.last_flipped = false;
            self.seat_weight = ZERO_WEIGHT;
            self.steer_weight = ZERO_WEIGHT;
        }

        self.was_seated = self.seated;
        event
    }

    /// True while driven and seated in the car.
    pub fn is_seated(&self) -> bool {
        self.driven && self.seated
    }

    /// Signed steering blend weight in `-1..1` (positive = left, negative = right).
    pub fn steer_weight(&self) -> f32 {
        self.steer_weight
    }

    /// Current fade-in weight in `0..1` for the seated pose.
    pub fn seat_weight(&self) -> f32 {
        self.seat_weight
    }

    /// Configured fade duration in seconds for a car exit event.
    pub fn event_fade(&self, event: CarEvent) -> f32 {
        let fade = match event {
            CarEvent::GotOut => self.config.exit_fade,
            CarEvent::ThrownOut => self.config.thrown_fade,
        };
        if fade.is_finite() && fade >= ZERO_TIME { fade } else { ZERO_TIME }
    }

    /// Drives `Air` with `car_sit_drive`, borrows `Crouch` for upper-body steering,
    /// and fades weapon/additive layers to zero.
    pub fn apply_car(&mut self, layers: &mut LayerStack) {
        if !self.owns_layers {
            let crouch = layers.get(LayerId::Crouch);
            self.saved_steer_layer = Some(SavedLayer {
                clip: crouch.clip.clone(),
                mask: crouch.mask,
                blend: crouch.blend,
                speed: crouch.speed,
                looping: crouch.looping,
            });
            self.owns_layers = true;
        }

        let sit_changed = layers.get(LayerId::Air).clip.as_deref() != Some(CAR_SIT_DRIVE_CLIP);
        layers.set_clip(LayerId::Air, Some(CAR_SIT_DRIVE_CLIP));
        let air = layers.get_mut(LayerId::Air);
        air.mask = MaskId::Full;
        air.blend = LayerBlend::Override;
        air.weight = self.seat_weight;
        air.target = self.seat_weight;
        air.speed = DEFAULT_SPEED;
        air.looping = true;
        if sit_changed {
            air.time = ZERO_TIME;
        }

        let steer_clip = if self.steer_weight >= ZERO_WEIGHT {
            CAR_STEER_LEFT_CLIP
        } else {
            CAR_STEER_RIGHT_CLIP
        };
        let steer_w = (self.steer_weight.abs() * self.seat_weight)
            .clamp(ZERO_WEIGHT, FULL_WEIGHT);
        let steer_changed = layers.get(LayerId::Crouch).clip.as_deref() != Some(steer_clip);
        layers.set_clip(LayerId::Crouch, Some(steer_clip));
        let steer = layers.get_mut(LayerId::Crouch);
        steer.mask = MaskId::Upper;
        steer.blend = LayerBlend::Override;
        steer.weight = steer_w;
        steer.target = steer_w;
        steer.speed = DEFAULT_SPEED;
        steer.looping = true;
        if steer_changed {
            steer.time = ZERO_TIME;
        }

        layers.get_mut(LayerId::Stance).target = ZERO_WEIGHT;
        layers.get_mut(LayerId::Aim).target = ZERO_WEIGHT;
        layers.get_mut(LayerId::Additive).target = ZERO_WEIGHT;
    }

    /// Restores `Air` and the borrowed steering layer when leaving the seat.
    pub fn release_layers(&mut self, layers: &mut LayerStack) {
        if !self.owns_layers {
            return;
        }
        layers.set_clip(LayerId::Air, Some(JUMP_FALLBACK_CLIP));
        let air = layers.get_mut(LayerId::Air);
        air.mask = MaskId::Full;
        air.blend = LayerBlend::Override;
        air.time = ZERO_TIME;
        air.weight = ZERO_WEIGHT;
        air.target = ZERO_WEIGHT;
        air.speed = DEFAULT_SPEED;
        air.looping = true;

        if let Some(saved) = self.saved_steer_layer.take() {
            let crouch = layers.get_mut(LayerId::Crouch);
            crouch.clip = saved.clip;
            crouch.mask = saved.mask;
            crouch.blend = saved.blend;
            crouch.time = ZERO_TIME;
            crouch.weight = ZERO_WEIGHT;
            crouch.target = ZERO_WEIGHT;
            crouch.speed = saved.speed;
            crouch.looping = saved.looping;
        }
        self.owns_layers = false;
    }
}
