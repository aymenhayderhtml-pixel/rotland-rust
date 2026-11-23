//! Ordered animation blend layers evaluated without per-frame allocations.

use crate::anim_config::AnimConfig;
use crate::mask::MaskId;

const DEFAULT_SPEED: f32 = 1.0; // new: DECISIONS 2026-11-23
const FULL_WEIGHT: f32 = 1.0; // new: DECISIONS 2026-11-23
const ZERO_WEIGHT: f32 = 0.0; // new: DECISIONS 2026-11-23
const MANUAL_RATE: f32 = 0.0; // new: DECISIONS 2026-11-23

/// Fixed evaluation order of layers in the humanoid animation pipeline.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayerId {
    Locomotion,
    Air,
    Crouch,
    Stance,
    Aim,
    Action,
    Additive,
    FullBody,
}

impl LayerId {
    const COUNT: usize = 8; // new: DECISIONS 2026-11-23

    const fn index(self) -> usize {
        match self {
            LayerId::Locomotion => 0,
            LayerId::Air => 1,
            LayerId::Crouch => 2,
            LayerId::Stance => 3,
            LayerId::Aim => 4,
            LayerId::Action => 5,
            LayerId::Additive => 6,
            LayerId::FullBody => 7,
        }
    }
}

/// How a layer combines its sampled pose with the accumulated pose below it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayerBlend {
    Override,
    Additive,
}

/// State of a single animation layer.
#[derive(Clone, Debug)]
pub struct AnimLayer {
    pub id: LayerId,
    pub mask: MaskId,
    pub blend: LayerBlend,
    pub weight: f32,
    pub target: f32,
    pub rate: f32,
    pub clip: Option<String>,
    pub time: f32,
    pub speed: f32,
    pub looping: bool,
}

/// Fixed-order stack of `AnimLayer`s updated in place without frame allocations.
#[derive(Clone, Debug)]
pub struct LayerStack {
    layers: [AnimLayer; LayerId::COUNT],
}

impl LayerStack {
    /// Constructs the default humanoid layer stack from `cfg`.
    /// `Stance` and `Additive` start empty for later jobs.
    pub fn new(cfg: &AnimConfig) -> Self {
        let rate = cfg.blend_rate; // data: animation.json blendRate
        let make = |id, mask, blend, weight, target, r, clip: Option<&str>, looping| AnimLayer {
            id,
            mask,
            blend,
            weight,
            target,
            rate: r,
            clip: clip.map(str::to_owned),
            time: 0.0,
            speed: DEFAULT_SPEED,
            looping,
        };
        Self {
            layers: [
                make(
                    LayerId::Locomotion,
                    MaskId::Full,
                    LayerBlend::Override,
                    FULL_WEIGHT,
                    FULL_WEIGHT,
                    rate,
                    None,
                    true,
                ),
                make(
                    LayerId::Air,
                    MaskId::Full,
                    LayerBlend::Override,
                    ZERO_WEIGHT,
                    ZERO_WEIGHT,
                    rate,
                    Some("jump"),
                    true,
                ),
                make(
                    LayerId::Crouch,
                    MaskId::Full,
                    LayerBlend::Override,
                    ZERO_WEIGHT,
                    ZERO_WEIGHT,
                    rate,
                    Some("crouch"),
                    true,
                ),
                make(
                    LayerId::Stance,
                    MaskId::Upper,
                    LayerBlend::Override,
                    ZERO_WEIGHT,
                    ZERO_WEIGHT,
                    rate,
                    None,
                    true,
                ),
                make(
                    LayerId::Aim,
                    MaskId::Upper,
                    LayerBlend::Override,
                    ZERO_WEIGHT,
                    ZERO_WEIGHT,
                    rate,
                    None,
                    true,
                ),
                make(
                    LayerId::Action,
                    MaskId::Upper,
                    LayerBlend::Override,
                    ZERO_WEIGHT,
                    ZERO_WEIGHT,
                    MANUAL_RATE,
                    None,
                    false,
                ),
                make(
                    LayerId::Additive,
                    MaskId::Full,
                    LayerBlend::Additive,
                    ZERO_WEIGHT,
                    ZERO_WEIGHT,
                    rate,
                    None,
                    true,
                ),
                make(
                    LayerId::FullBody,
                    MaskId::Full,
                    LayerBlend::Override,
                    ZERO_WEIGHT,
                    ZERO_WEIGHT,
                    MANUAL_RATE,
                    None,
                    false,
                ),
            ],
        }
    }

    pub fn get(&self, id: LayerId) -> &AnimLayer {
        &self.layers[id.index()]
    }

    pub fn get_mut(&mut self, id: LayerId) -> &mut AnimLayer {
        &mut self.layers[id.index()]
    }

    /// Sets the clip name for `id`, only allocating when the name actually changes.
    pub fn set_clip(&mut self, id: LayerId, clip: Option<&str>) {
        let layer = self.get_mut(id);
        if layer.clip.as_deref() == clip {
            return;
        }
        layer.clip = clip.map(str::to_owned);
    }

    /// Advances all layer clocks and exponentially smooths weights for layers with `rate > 0`.
    pub fn advance(&mut self, dt: f32) {
        if !dt.is_finite() || dt <= 0.0 {
            return;
        }
        for layer in &mut self.layers {
            if layer.rate > 0.0 {
                let k = 1.0 - (-layer.rate * dt).exp();
                layer.weight += (layer.target - layer.weight) * k;
            }
            layer.time += dt * layer.speed;
        }
    }
}
