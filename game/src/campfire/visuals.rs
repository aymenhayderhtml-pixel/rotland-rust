use glam::Vec3;

/// Base light energy in Godot units.
// Godot: CampfireVisuals.cs BaseLightEnergy
pub const BASE_LIGHT_ENERGY: f32 = 2.2;
/// Ring radius of stones around fire.
// Godot: CampfireVisuals.cs RingRadius
pub const RING_RADIUS: f32 = 0.45;
/// Warm light color [R, G, B].
// Godot: CampfireVisuals.cs Warm
pub const WARM_LIGHT_COLOR: [f32; 3] = [1.0, 0.62, 0.3];
/// Omni light range in metres.
// Godot: CampfireVisuals.cs OmniRange
pub const OMNI_RANGE: f32 = 8.0;

/// Flame on/off state, flicker light energy, and flame scale numbers.
/// Port of `scripts/World/CampfireVisuals.cs`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CampfireVisuals {
    pub is_lit: bool,
    pub time: f32,
    pub light_energy: f32,
    pub flame_scale: Vec3,
}

impl Default for CampfireVisuals {
    fn default() -> Self {
        Self::new(true)
    }
}

impl CampfireVisuals {
    pub fn new(is_lit: bool) -> Self {
        let mut v = Self {
            is_lit,
            time: 0.0,
            light_energy: if is_lit { BASE_LIGHT_ENERGY } else { 0.0 },
            flame_scale: if is_lit { Vec3::ONE } else { Vec3::ZERO },
        };
        if is_lit {
            v.update(0.0);
        }
        v
    }

    pub fn set_lit(&mut self, lit: bool) {
        self.is_lit = lit;
        if !lit {
            self.light_energy = 0.0;
            self.flame_scale = Vec3::ZERO;
        } else {
            self.update(0.0);
        }
    }

    /// Advances the flicker timer and calculates current light energy and flame scale.
    // Godot: CampfireVisuals.cs _Process
    pub fn update(&mut self, dt: f32) {
        if !self.is_lit {
            self.light_energy = 0.0;
            self.flame_scale = Vec3::ZERO;
            return;
        }
        self.time += dt;
        let flicker = 0.85 + 0.15 * (self.time * 13.0).sin() + 0.08 * (self.time * 29.0 + 1.0).sin();
        self.light_energy = BASE_LIGHT_ENERGY * flicker;
        self.flame_scale = Vec3::new(
            1.0 + 0.08 * (self.time * 17.0).sin(),
            1.0 + 0.12 * (self.time * 11.0 + 2.0).sin(),
            1.0 + 0.08 * (self.time * 15.0).sin(),
        );
    }
}
