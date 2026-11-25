use glam::{IVec2, Vec3};

use crate::farming::config::FarmConfigData;
use crate::farming::def::CropStage;
use crate::farming::rules::FarmRules;

/// The state of one bed: position, crop planted, growth hours, and watering status.
/// Hours are absolute in-game hours (day 1, 00:00 = 0). WateredAt is -1 when never watered.
// Godot: PlotRecord.cs
#[derive(Clone, Debug, PartialEq)]
pub struct PlotRecord {
    pub id: String,
    pub position: Vec3,
    /// Camp beds are safe from zombies and created with the camp.
    pub camp: bool,
    /// Crop id, or None for a tilled bed with nothing planted.
    pub crop: Option<String>,
    /// Watered growing hours accumulated so far.
    pub growth: f64,
    /// Consecutive hours with no water. Growth pauses while this is above zero.
    pub dry_hours: f64,
    /// Absolute in-game hour when last watered, or -1.0 if never watered.
    pub watered_at: f64,
    /// True if dried out past dryHoursToDie.
    pub dead: bool,
}

impl PlotRecord {
    pub fn new(id: impl Into<String>, position: Vec3) -> Self {
        Self {
            id: id.into(),
            position,
            camp: false,
            crop: None,
            growth: 0.0,
            dry_hours: 0.0,
            watered_at: -1.0,
            dead: false,
        }
    }

    /// The chunk this bed lies in (same 64 m grid as world chunks).
    // Godot: PlotRecord.cs Chunk
    pub fn chunk(&self) -> IVec2 {
        IVec2::new(
            (self.position.x / world::WorldGenerator::CHUNK_SIZE).floor() as i32,
            (self.position.z / world::WorldGenerator::CHUNK_SIZE).floor() as i32,
        )
    }

    /// Whether this plot currently counts as watered at in-game hour `now`.
    pub fn is_watered(&self, now: f64, window_hours: f64) -> bool {
        FarmRules::is_watered(now, self.watered_at, window_hours)
    }

    /// Crop stage if a valid crop is planted.
    pub fn stage(&self, cfg: &FarmConfigData) -> Option<CropStage> {
        let crop_id = self.crop.as_deref()?;
        let def = cfg.try_crop(crop_id)?;
        Some(FarmRules::stage_of(self.growth, def.total_hours()))
    }

    /// Zero-based stage index (0: Sprout, 1: Young, 2: Ripe) if a valid crop is planted.
    pub fn stage_index(&self, cfg: &FarmConfigData) -> Option<usize> {
        self.stage(cfg).map(|s| s.index())
    }

    /// Stage model name if a valid crop is planted (e.g. "wheat_sprout").
    pub fn stage_model(&self, cfg: &FarmConfigData) -> Option<String> {
        let crop_id = self.crop.as_deref()?;
        let def = cfg.try_crop(crop_id)?;
        let st = FarmRules::stage_of(self.growth, def.total_hours());
        Some(def.stage_model(st))
    }

    /// Resets the plot back to empty tilled soil.
    pub fn clear(&mut self) {
        self.crop = None;
        self.growth = 0.0;
        self.dry_hours = 0.0;
        self.watered_at = -1.0;
        self.dead = false;
    }
}
