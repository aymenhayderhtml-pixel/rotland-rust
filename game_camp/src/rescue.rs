//! Trapped world survivors: config, area-clear checks, interaction prompt, and rescue state.

use std::collections::HashSet;
use std::path::Path;

use glam::Vec3;
use serde::{Deserialize, Serialize};

use crate::guild::{GuildRecruit, GuildRecruits, RecruitHireError};
use crate::rescue_place::RescueSpot;

/// Tuning loaded from `data/rescue.json`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RescueConfig {
    pub chance_per_poi: f32,
    pub max_survivors: usize,
    pub min_dist_from_camp: f32,
    pub clear_radius: f32,
    pub talk_range: f32,
    pub min_zombies_near: usize,
    pub per_town: [usize; 2],
    /// Survivors placed on climb-tower stop floors (R44). Missing key = 0.
    #[serde(default)]
    pub floor_survivors: usize,
}

impl Default for RescueConfig {
    fn default() -> Self {
        Self {
            chance_per_poi: 0.15,    // data: rescue.json chancePerPoi
            max_survivors: 6,        // data: rescue.json maxSurvivors
            min_dist_from_camp: 300.0, // data: rescue.json minDistFromCamp
            clear_radius: 12.0,      // data: rescue.json clearRadius
            talk_range: 2.5,         // data: rescue.json talkRange
            min_zombies_near: 3,     // data: rescue.json minZombiesNear
            per_town: [1, 2],        // data: rescue.json perTown
            floor_survivors: 3,      // data: rescue.json floorSurvivors
        }
    }
}

/// Errors when loading `data/rescue.json`.
#[derive(Debug)]
pub enum RescueLoadError {
    Io(std::io::Error),
    Json(serde_json::Error),
    NotFound(String),
}

impl std::fmt::Display for RescueLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "rescue IO error: {err}"),
            Self::Json(err) => write!(f, "rescue JSON parse error: {err}"),
            Self::NotFound(path) => write!(f, "rescue config file not found: {path}"),
        }
    }
}

impl std::error::Error for RescueLoadError {}

impl From<std::io::Error> for RescueLoadError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<serde_json::Error> for RescueLoadError {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err)
    }
}

impl RescueConfig {
    /// Loads rescue configuration from `data/rescue.json` using repo path discovery.
    pub fn from_repo() -> Result<Self, RescueLoadError> {
        let path = paths::repo_path("data/rescue.json");
        Self::from_file(&path)
    }

    /// Loads rescue configuration from an explicit file path.
    pub fn from_file(path: &Path) -> Result<Self, RescueLoadError> {
        if !path.exists() {
            return Err(RescueLoadError::NotFound(path.display().to_string()));
        }
        let content = std::fs::read_to_string(path)?;
        Self::from_json(&content)
    }

    /// Parses rescue configuration from JSON text.
    pub fn from_json(json: &str) -> Result<Self, RescueLoadError> {
        let cfg: Self = serde_json::from_str(json)?;
        Ok(cfg)
    }
}

/// Reason a survivor cannot be rescued right now.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RescueBlocker {
    NotFound,
    AlreadyRescued,
    OutOfRange,
    ZombiesNearby,
    TeamFull,
}

fn flat_dist_sq(a: Vec3, b: Vec3) -> f32 {
    let dx = a.x - b.x;
    let dz = a.z - b.z;
    dx * dx + dz * dz
}

/// Tracks unrescued world spots and the set of already-rescued survivor IDs.
#[derive(Clone, Debug)]
pub struct RescueState {
    pub config: RescueConfig,
    pub spots: Vec<RescueSpot>,
    pub rescued_ids: HashSet<String>,
}

impl RescueState {
    /// Creates a new rescue state from config and deterministic world spots.
    pub fn new(config: RescueConfig, spots: Vec<RescueSpot>) -> Self {
        Self {
            config,
            spots,
            rescued_ids: HashSet::new(),
        }
    }

    /// Applies saved rescued IDs so those survivors never reappear.
    pub fn restore_rescued(&mut self, ids: &[String]) {
        for id in ids {
            self.rescued_ids.insert(id.clone());
        }
        self.spots.retain(|s| !self.rescued_ids.contains(&s.id));
    }

    /// Sorted list of rescued survivor IDs for deterministic saving.
    pub fn rescued_ids_sorted(&self) -> Vec<String> {
        let mut ids: Vec<String> = self.rescued_ids.iter().cloned().collect();
        ids.sort();
        ids
    }

    /// Active (unrescued) survivor spots remaining in the world.
    pub fn active_spots(&self) -> &[RescueSpot] {
        &self.spots
    }

    /// True if `id` has already been rescued.
    pub fn is_rescued(&self, id: &str) -> bool {
        self.rescued_ids.contains(id)
    }

    /// True if an unrescued survivor is currently placed at `poi_id`.
    pub fn has_survivor_at_poi(&self, poi_id: &str) -> bool {
        self.spots.iter().any(|s| s.poi_id == poi_id)
    }

    /// True when no living zombie is within `config.clear_radius` (flat XZ) of `spot`.
    pub fn is_area_clear(&self, spot: &RescueSpot, living_zombies: &[Vec3]) -> bool {
        let r_sq = self.config.clear_radius * self.config.clear_radius;
        !living_zombies
            .iter()
            .any(|&z_pos| flat_dist_sq(spot.position, z_pos) <= r_sq)
    }

    /// Closest active survivor within `config.talk_range` (flat XZ) of `player_pos`.
    pub fn nearest_in_range(&self, player_pos: Vec3) -> Option<&RescueSpot> {
        let r_sq = self.config.talk_range * self.config.talk_range;
        self.spots
            .iter()
            .filter_map(|spot| {
                let d_sq = flat_dist_sq(player_pos, spot.position);
                (d_sq <= r_sq).then_some((d_sq, spot))
            })
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .map(|(_, spot)| spot)
    }

    /// Validates whether `id` can currently be rescued by the player.
    pub fn can_rescue(
        &self,
        id: &str,
        player_pos: Vec3,
        living_zombies: &[Vec3],
        hired_count: usize,
        max_recruits: usize,
    ) -> Result<(), RescueBlocker> {
        let Some(spot) = self.spots.iter().find(|s| s.id == id) else {
            return if self.rescued_ids.contains(id) {
                Err(RescueBlocker::AlreadyRescued)
            } else {
                Err(RescueBlocker::NotFound)
            };
        };
        let talk_sq = self.config.talk_range * self.config.talk_range;
        if flat_dist_sq(player_pos, spot.position) > talk_sq {
            return Err(RescueBlocker::OutOfRange);
        }
        if !self.is_area_clear(spot, living_zombies) {
            return Err(RescueBlocker::ZombiesNearby);
        }
        if hired_count >= max_recruits {
            return Err(RescueBlocker::TeamFull);
        }
        Ok(())
    }

    /// Interaction prompt when player is within talk range and area is clear of living zombies.
    pub fn prompt_for(
        &self,
        player_pos: Vec3,
        living_zombies: &[Vec3],
        hired_count: usize,
        max_recruits: usize,
    ) -> Option<(&RescueSpot, String)> {
        let spot = self.nearest_in_range(player_pos)?;
        if !self.is_area_clear(spot, living_zombies) {
            return None;
        }
        let text = if hired_count >= max_recruits {
            "Team full".to_string()
        } else {
            format!("[E] Rescue {}", spot.name)
        };
        Some((spot, text))
    }

    /// Rescues `id`, adds them to `recruits` for free with Follow order, and removes the spot.
    pub fn try_rescue(
        &mut self,
        id: &str,
        player_pos: Vec3,
        living_zombies: &[Vec3],
        recruits: &mut GuildRecruits,
        max_recruits: usize,
    ) -> Result<GuildRecruit, RescueBlocker> {
        self.can_rescue(id, player_pos, living_zombies, recruits.hired().len(), max_recruits)?;
        let idx = self
            .spots
            .iter()
            .position(|s| s.id == id)
            .ok_or(RescueBlocker::NotFound)?;
        let recruit = self.spots[idx].to_recruit();
        match recruits.add_rescued(recruit.clone(), max_recruits) {
            Ok(()) => {}
            Err(RecruitHireError::SlotsFull(_)) => return Err(RescueBlocker::TeamFull),
            Err(RecruitHireError::AlreadyHired) => return Err(RescueBlocker::AlreadyRescued),
            Err(_) => return Err(RescueBlocker::NotFound),
        }
        self.spots.remove(idx);
        self.rescued_ids.insert(id.to_string());
        Ok(recruit)
    }

    /// Rescues the nearest survivor within talk range, if any.
    pub fn try_rescue_nearest(
        &mut self,
        player_pos: Vec3,
        living_zombies: &[Vec3],
        recruits: &mut GuildRecruits,
        max_recruits: usize,
    ) -> Result<GuildRecruit, RescueBlocker> {
        let id = self
            .nearest_in_range(player_pos)
            .map(|s| s.id.clone())
            .ok_or(RescueBlocker::OutOfRange)?;
        self.try_rescue(&id, player_pos, living_zombies, recruits, max_recruits)
    }

    /// Toast message shown when a survivor joins the player's team.
    pub fn joined_message(name: &str) -> String {
        format!("{name} joined your team")
    }
}

/// Save slice for rescued survivor IDs stored with the `"guild"` save entry.
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RescueSave {
    #[serde(default)]
    pub rescued_ids: Vec<String>,
}
