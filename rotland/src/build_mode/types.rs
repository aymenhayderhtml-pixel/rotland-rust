//! Building part definitions and category ordering for the part picker.

/// Ordered list of placeable part IDs matching `data/build_parts.json`.
pub const PARTS_ORDER: &[&str] = &[
    "floor",
    "wall",
    "wall_window",
    "wall_door",
    "half_wall",
    "roof",
    "stairs",
    "bed",
    "storage_box",
    "workbench",
    "light",
    "spikes",
    "barricade",
];

/// Returns the pack model name for a building part (e.g. "floor" -> "base_floor").
pub fn part_model_name(part_id: &str) -> String {
    format!("base_{part_id}")
}

/// Human-readable display name for the part.
pub fn part_display_name(part_id: &str) -> &'static str {
    match part_id {
        "floor" => "Floor (2x2)",
        "wall" => "Wall",
        "wall_window" => "Wall with Window",
        "wall_door" => "Wall with Door",
        "half_wall" => "Half Wall",
        "roof" => "Flat Roof (2x2)",
        "stairs" => "Stairs",
        "bed" => "Bed",
        "storage_box" => "Storage Box",
        "workbench" => "Workbench",
        "light" => "Lantern",
        "spikes" => "Spikes",
        "barricade" => "Barricade",
        _ => "Unknown Part",
    }
}
