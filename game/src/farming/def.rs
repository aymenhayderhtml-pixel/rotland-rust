use serde::{Deserialize, Serialize};

/// How far a crop has grown. Picks which of the three models is shown.
// Godot: CropStage.cs
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CropStage {
    Sprout,
    Young,
    Ripe,
}

impl CropStage {
    /// Zero-based stage index: 0 for Sprout, 1 for Young, 2 for Ripe.
    pub fn index(&self) -> usize {
        match self {
            Self::Sprout => 0,
            Self::Young => 1,
            Self::Ripe => 2,
        }
    }

    /// Lowercase stage name matching model suffix ("sprout", "young", "ripe").
    pub fn name(&self) -> &'static str {
        match self {
            Self::Sprout => "sprout",
            Self::Young => "young",
            Self::Ripe => "ripe",
        }
    }
}

/// One crop line from data/farming.json: its seed, what it gives and how long it takes.
// Godot: CropDef.cs
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CropDef {
    pub id: String,
    pub name: String,
    /// Item id of the seeds that plant it.
    pub seed: String,
    /// Item id given at harvest.
    pub product: String,
    /// Products at harvest before the farming level bonus.
    #[serde(rename = "yield")]
    pub yield_count: u32,
    /// Watered growing days from planting to ripe.
    pub grow_days: f32,
    /// Model name prefix: assets/models/farming/<model>_sprout.glb (and _young, _ripe).
    pub model: String,
}

impl CropDef {
    /// Watered growing hours needed to ripen.
    // Godot: CropDef.cs TotalHours
    pub fn total_hours(&self) -> f64 {
        (self.grow_days as f64) * 24.0
    }

    /// Model name for a given crop stage: <model>_<stage>.
    pub fn stage_model(&self, stage: CropStage) -> String {
        format!("{}_{}", self.model, stage.name())
    }
}

/// What E does at a bed. Chosen from the bed's state and the bag; only one per press.
// Godot: PlotAction.cs
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PlotAction {
    None,
    Plant,
    Water,
    Harvest,
    Clear,
}

/// Reason why tilling a spot failed.
// Godot: FarmManager.Actions.cs GroundTill
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TillError {
    TooCloseToAnotherBed,
    GroundTooSteep,
    NoHoe,
    NoGround,
    FacingInvalid,
}

impl TillError {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::TooCloseToAnotherBed => "Too close to another bed",
            Self::GroundTooSteep => "The ground there is too steep",
            Self::NoHoe => "No hoe in inventory",
            Self::NoGround => "There is no ground there",
            Self::FacingInvalid => "Face the ground to till it",
        }
    }
}

impl std::fmt::Display for TillError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::error::Error for TillError {}

/// Output per plot for rendering: position, crop id, stage index, stage model name, watered, dead.
#[derive(Clone, Debug, PartialEq)]
pub struct PlotView {
    pub id: String,
    pub position: glam::Vec3,
    pub camp: bool,
    pub crop_id: Option<String>,
    pub stage_index: Option<usize>,
    pub stage_model: Option<String>,
    pub watered: bool,
    pub dead: bool,
}
