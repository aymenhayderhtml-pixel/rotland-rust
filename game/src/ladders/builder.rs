use glam::{Mat4, Vec3};
use pack::Marker;

use super::rail::LadderRail;

/// Prefix for ladder bottom markers.
const BOTTOM_PREFIX: &str = "ladder_bottom";
/// Prefix for ladder top markers.
const TOP_PREFIX: &str = "ladder_top";

/// Pairs each `ladder_bottom*` marker with the `ladder_top*` marker sharing the same suffix.
/// Transforms their positions by `model_matrix`.
/// Unpaired markers are skipped.
/// Port of `scripts/World/LadderBuilder.cs`.
pub fn ladders_from_markers(markers: &[Marker], model_matrix: Mat4) -> Vec<LadderRail> {
    let mut rails = Vec::new();
    for marker in markers {
        let Some(suffix) = marker.name.strip_prefix(BOTTOM_PREFIX) else {
            continue;
        };
        let expected_top = format!("{TOP_PREFIX}{suffix}");
        let Some(top_marker) = markers.iter().find(|m| m.name == expected_top) else {
            continue;
        };
        let foot = model_matrix.transform_point3(Vec3::from_array(marker.translation));
        let deck = model_matrix.transform_point3(Vec3::from_array(top_marker.translation));
        rails.push(LadderRail::new(foot, deck, suffix));
    }
    rails
}
