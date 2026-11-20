//! Pure logic for the F1 developer overlay and --at=X,Z CLI start flag.

use glam::Vec3;

use crate::dir::yaw_forward;

/// Number of recent frame durations retained by [FpsHistory].
pub const FPS_HISTORY_CAPACITY: usize = 300; // new: DECISIONS 2026-11-20

/// Frame-time budget in milliseconds for 60 FPS (1000 / 60).
pub const FRAME_MS_60FPS: f32 = 1000.0 / 60.0; // new: DECISIONS 2026-11-20

/// Frame-time budget in milliseconds for 30 FPS (1000 / 30).
pub const FRAME_MS_30FPS: f32 = 1000.0 / 30.0; // new: DECISIONS 2026-11-20

/// Vertical clearance added above ground height when teleporting the player (metres).
pub const DEV_TELEPORT_GROUND_OFFSET: f32 = 0.1; // new: DECISIONS 2026-11-20

/// Forward distance in front of the player where dev-spawned zombies appear (metres).
pub const DEV_ZOMBIE_SPAWN_DISTANCE: f32 = 8.0; // new: DECISIONS 2026-11-20

/// Radius around the player cleared by the "Kill all zombies" button (metres).
pub const DEV_KILL_ZOMBIE_RADIUS: f32 = 50.0; // new: DECISIONS 2026-11-20

/// Lethal damage dealt per zombie by the "Kill all zombies" dev action.
pub const DEV_KILL_ZOMBIE_DAMAGE: f32 = 9999.0; // new: DECISIONS 2026-11-20

/// Fallback radial offset from a POI centre when building dimensions are unknown (metres).
pub const DEV_DEFAULT_POI_OFFSET: f32 = 12.0; // new: DECISIONS 2026-11-20

/// Extra clearance outside a POI lot reach + margin when teleporting adjacent to it (metres).
pub const DEV_POI_APPROACH_BUFFER: f32 = 2.0; // new: DECISIONS 2026-11-20

/// Minimum squared horizontal length before falling back to +X when offsetting from a POI.
const MIN_POI_DIR_LEN_SQ: f32 = 1e-4; // new: DECISIONS 2026-11-20

/// Spawnable zombie kinds defined in data/zombies.json.
// data: zombies.json types
pub const DEV_ZOMBIE_TYPES: &[&str] = &["walker", "runner"];

/// Sorted catalogue of all item ids defined in data/items.json.
// data: items.json
pub const DEV_ALL_ITEM_IDS: &[&str] = &[
    "ak47",
    "ammo_762",
    "antler",
    "apple",
    "axe",
    "baked_potato",
    "bandage",
    "bat",
    "bread",
    "campfire_kit",
    "canned_beans",
    "carrot",
    "cloth",
    "cooked_meat",
    "crowbar",
    "dirty_water",
    "flashlight",
    "fuel_can",
    "hide",
    "hoe",
    "katana",
    "knife",
    "machete",
    "medkit",
    "pistol",
    "pistol_ammo",
    "potato",
    "raw_meat",
    "rifle",
    "rifle_rounds",
    "scrap_metal",
    "seeds_carrot",
    "seeds_potato",
    "seeds_wheat",
    "shells_12g",
    "shotgun",
    "smg",
    "soda",
    "vegetable_stew",
    "water_bottle",
    "watering_can",
    "wheat",
    "wood",
];

/// Fixed-capacity ring buffer storing up to 300 chronological frame durations in milliseconds.
///
/// Samples stay contiguous from oldest (0) to newest (len - 1) so [Self::as_slice]
/// can be plotted left-to-right directly without wrapping seams.
#[derive(Clone, Debug)]
pub struct FpsHistory {
    samples: [f32; FPS_HISTORY_CAPACITY],
    len: usize,
}

impl Default for FpsHistory {
    fn default() -> Self {
        Self::new()
    }
}

impl FpsHistory {
    /// Creates an empty frame-time history buffer.
    pub fn new() -> Self {
        Self {
            samples: [0.0; FPS_HISTORY_CAPACITY],
            len: 0,
        }
    }

    /// Appends a frame duration in milliseconds, dropping the oldest sample once full.
    pub fn push(&mut self, ms: f32) {
        let clamped = if ms.is_finite() { ms.max(0.0) } else { 0.0 };
        if self.len < FPS_HISTORY_CAPACITY {
            self.samples[self.len] = clamped;
            self.len += 1;
        } else {
            self.samples.copy_within(1.., 0);
            self.samples[FPS_HISTORY_CAPACITY - 1] = clamped;
        }
    }

    /// Number of recorded frame samples (`0..=300`).
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if no frame durations have been recorded yet.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Chronological slice of recorded frame durations in milliseconds (oldest to newest).
    pub fn as_slice(&self) -> &[f32] {
        &self.samples[..self.len]
    }

    /// Mean frame duration in milliseconds, or `0.0` when empty.
    pub fn avg_ms(&self) -> f32 {
        if self.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.as_slice().iter().sum();
        sum / self.len as f32
    }

    /// Maximum frame duration in milliseconds, or `0.0` when empty.
    pub fn max_ms(&self) -> f32 {
        self.as_slice().iter().copied().fold(0.0_f32, f32::max)
    }
}

/// Parses comma-separated world coordinates "X,Z" for the --at=X,Z CLI flag and custom teleport.
pub fn parse_at_flag(s: &str) -> Option<(f32, f32)> {
    let mut parts = s.split(',');
    let x: f32 = parts.next()?.trim().parse().ok()?;
    let z: f32 = parts.next()?.trim().parse().ok()?;
    if parts.next().is_some() || !x.is_finite() || !z.is_finite() {
        return None;
    }
    Some((x, z))
}

/// Filters items by case-insensitive substring match against query.
pub fn filter_items<'a>(items: &'a [&'a str], query: &str) -> Vec<&'a str> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return items.to_vec();
    }
    items
        .iter()
        .copied()
        .filter(|id| id.to_lowercase().contains(&q))
        .collect()
}

/// Computes the world target point `distance` metres in front of the player using Godot yaw.
pub fn dev_zombie_spawn_spot(player_pos: Vec3, player_yaw: f32, distance: f32) -> Vec3 {
    player_pos + yaw_forward(player_yaw) * distance
}

/// Offsets a POI centre horizontally toward the player so teleporting lands adjacent to the lot.
pub fn dev_adjacent_poi_spot(player_pos: Vec3, poi_pos: Vec3, offset: f32) -> Vec3 {
    let delta = Vec3::new(player_pos.x - poi_pos.x, 0.0, player_pos.z - poi_pos.z);
    let dir = if delta.length_squared() > MIN_POI_DIR_LEN_SQ {
        delta.normalize()
    } else {
        Vec3::X
    };
    Vec3::new(
        poi_pos.x + dir.x * offset,
        poi_pos.y,
        poi_pos.z + dir.z * offset,
    )
}

#[cfg(test)]
mod tests {
    use std::f32::consts::FRAC_PI_2;

    use super::*;

    #[test]
    fn dev_panel_fps_history_empty_and_push() {
        let mut h = FpsHistory::new();
        assert!(h.is_empty());
        assert_eq!(h.len(), 0);
        assert!(h.as_slice().is_empty());
        assert!(h.avg_ms().abs() < 1e-6);
        assert!(h.max_ms().abs() < 1e-6);

        h.push(10.0);
        h.push(20.0);
        h.push(30.0);
        assert!(!h.is_empty());
        assert_eq!(h.len(), 3);
        assert_eq!(h.as_slice().len(), 3);
        assert!((h.avg_ms() - 20.0).abs() < 1e-4);
        assert!((h.max_ms() - 30.0).abs() < 1e-4);
    }

    #[test]
    fn dev_panel_fps_history_ring_wrap_preserves_order_and_stats() {
        let mut h = FpsHistory::new();
        for i in 0..(FPS_HISTORY_CAPACITY + 50) {
            h.push(i as f32);
        }
        assert_eq!(h.len(), FPS_HISTORY_CAPACITY);
        let slice = h.as_slice();
        assert_eq!(slice.len(), FPS_HISTORY_CAPACITY);
        assert!((slice[0] - 50.0).abs() < 1e-4);
        assert!((slice[FPS_HISTORY_CAPACITY - 1] - 349.0).abs() < 1e-4);
        assert!((h.avg_ms() - 199.5).abs() < 1e-3);
        assert!((h.max_ms() - 349.0).abs() < 1e-4);
    }

    #[test]
    fn dev_panel_fps_history_clamps_invalid_values() {
        let mut h = FpsHistory::new();
        h.push(-5.0);
        h.push(f32::NAN);
        h.push(f32::INFINITY);
        h.push(16.0);
        assert_eq!(h.len(), 4);
        assert!(h.as_slice()[0].abs() < 1e-6);
        assert!(h.as_slice()[1].abs() < 1e-6);
        assert!(h.as_slice()[2].abs() < 1e-6);
        assert!((h.avg_ms() - 4.0).abs() < 1e-4);
        assert!((h.max_ms() - 16.0).abs() < 1e-4);
    }

    #[test]
    fn dev_panel_parse_at_flag_valid_inputs() {
        let (x, z) = parse_at_flag("150.0,-220.5").expect("valid coords");
        assert!((x - 150.0).abs() < 1e-4);
        assert!((z - (-220.5)).abs() < 1e-4);

        let (x, z) = parse_at_flag("  -12.25 , 48  ").expect("spaces allowed");
        assert!((x - (-12.25)).abs() < 1e-4);
        assert!((z - 48.0).abs() < 1e-4);

        let (x, z) = parse_at_flag("0,0").expect("origin");
        assert!(x.abs() < 1e-6);
        assert!(z.abs() < 1e-6);
    }

    #[test]
    fn dev_panel_parse_at_flag_invalid_inputs() {
        assert!(parse_at_flag("").is_none());
        assert!(parse_at_flag("150.0").is_none());
        assert!(parse_at_flag("1,2,3").is_none());
        assert!(parse_at_flag("abc,10").is_none());
        assert!(parse_at_flag("10,").is_none());
        assert!(parse_at_flag(",10").is_none());
        assert!(parse_at_flag("NaN,0").is_none());
        assert!(parse_at_flag("inf,-5").is_none());
    }

    #[test]
    fn dev_panel_filter_items_case_insensitive_and_empty() {
        let items = ["canned_beans", "water_bottle", "Watering_Can", "ak47"];
        assert_eq!(filter_items(&items, ""), items.to_vec());
        assert_eq!(filter_items(&items, "   "), items.to_vec());
        assert_eq!(
            filter_items(&items, "WATER"),
            vec!["water_bottle", "Watering_Can"]
        );
        assert_eq!(filter_items(&items, "  beans "), vec!["canned_beans"]);
        assert!(filter_items(&items, "shotgun").is_empty());
    }

    #[test]
    fn dev_panel_zombie_spawn_spot_follows_godot_yaw() {
        let origin = Vec3::new(10.0, 2.0, 20.0);
        let east = dev_zombie_spawn_spot(origin, -FRAC_PI_2, DEV_ZOMBIE_SPAWN_DISTANCE);
        assert!((east.x - 18.0).abs() < 1e-4);
        assert!((east.y - 2.0).abs() < 1e-4);
        assert!((east.z - 20.0).abs() < 1e-4);

        let north = dev_zombie_spawn_spot(origin, 0.0, DEV_ZOMBIE_SPAWN_DISTANCE);
        assert!((north.x - 10.0).abs() < 1e-4);
        assert!((north.z - 12.0).abs() < 1e-4);
    }

    #[test]
    fn dev_panel_adjacent_poi_spot_offsets_toward_player_and_fallback() {
        let poi = Vec3::new(100.0, 5.0, 50.0);
        let player = Vec3::new(100.0, 5.0, 80.0);
        let spot = dev_adjacent_poi_spot(player, poi, 10.0);
        assert!((spot.x - 100.0).abs() < 1e-4);
        assert!((spot.z - 60.0).abs() < 1e-4);

        let fallback = dev_adjacent_poi_spot(poi, poi, 10.0);
        assert!((fallback.x - 110.0).abs() < 1e-4);
        assert!((fallback.z - 50.0).abs() < 1e-4);
    }

    #[test]
    fn dev_panel_catalog_contains_expected_data_entries() {
        assert_eq!(DEV_ZOMBIE_TYPES, &["walker", "runner"]);
        assert_eq!(DEV_ALL_ITEM_IDS.len(), 43);
        assert!(DEV_ALL_ITEM_IDS.contains(&"canned_beans"));
        assert!(DEV_ALL_ITEM_IDS.contains(&"ak47"));
        assert!(DEV_ALL_ITEM_IDS.contains(&"vegetable_stew"));
    }
}
