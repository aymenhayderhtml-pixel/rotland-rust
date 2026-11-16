//! Growth calculation for breeder puppies over time.
// Godot: DogGrowth.cs

use super::config::DogConfigData;

/// How big a dog is. A breeder puppy starts at puppyScale and grows to full size
/// over growDays of in-game time. Adult dogs pass start hours of -1.0.
// Godot: DogGrowth.cs
pub struct DogGrowth;

impl DogGrowth {
    pub const NO_START: f64 = -1.0;
    pub const HOURS_PER_DAY: f64 = 24.0;

    /// Visual scale for a dog that was born (bought) at start_hours, read at now_hours.
    // Godot: DogGrowth.cs Scale
    pub fn scale(now_hours: f64, start_hours: f64, cfg: &DogConfigData) -> f32 {
        if start_hours < 0.0 || cfg.grow_days <= 0.0 {
            return 1.0;
        }
        let age = (now_hours - start_hours) / (Self::HOURS_PER_DAY * cfg.grow_days as f64);
        let t = age.clamp(0.0, 1.0) as f32;
        cfg.puppy_scale + (1.0 - cfg.puppy_scale) * t
    }
}
