//! Tree spatial query and identification.
//! Port of `scripts/World/TreeHit.cs` and `scripts/World/TreeStrike.cs`.

use glam::{IVec2, Vec3};

/// Information about a tree hit by a swing.
/// Port of `scripts/World/TreeHit.cs`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TreeHit {
    pub chunk: IVec2,
    pub index: usize,
    pub point: Vec3,
}

impl TreeHit {
    pub fn new(chunk: IVec2, index: usize, point: Vec3) -> Self {
        Self { chunk, index, point }
    }

    /// Stable tree identifier matching `TreeFelling.IdOf` in Godot.
    pub fn tree_id(&self) -> String {
        tree_id(self.chunk, self.index)
    }
}

/// A candidate prop in the world for tree strike queries.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TreeCandidate<'a> {
    pub chunk: IVec2,
    pub index: usize,
    pub position: Vec3,
    pub model: &'a str,
}

impl<'a> TreeCandidate<'a> {
    pub fn new(chunk: IVec2, index: usize, position: Vec3, model: &'a str) -> Self {
        Self {
            chunk,
            index,
            position,
            model,
        }
    }

    /// Stable tree identifier matching `TreeFelling.IdOf` in Godot.
    pub fn tree_id(&self) -> String {
        tree_id(self.chunk, self.index)
    }
}

/// True for tree props (model names start with "tree" or "pine_").
/// Matches `scripts/World/ScatterConfig.cs` and `world/src/scatter.rs:PropInstance::is_tree`.
pub fn is_tree_kind(kind: &str) -> bool {
    kind.starts_with("tree") || kind.starts_with("pine_")
}

/// Generates a stable tree identifier from chunk coordinate and prop index.
/// Format: `"tree_{chunk_x}_{chunk_y}_{prop_index}"`.
/// Port of `scripts/World/TreeFelling.cs:IdOf(Vector2I coord, int index)`.
pub fn tree_id(chunk: IVec2, index: usize) -> String {
    format!("tree_{}_{}_{}", chunk.x, chunk.y, index)
}

/// Parses a stable tree identifier back into chunk coordinates and prop index.
pub fn parse_tree_id(id: &str) -> Option<(IVec2, usize)> {
    let parts: Vec<&str> = id.split('_').collect();
    if parts.len() == 4 && parts[0] == "tree" {
        let x = parts[1].parse::<i32>().ok()?;
        let y = parts[2].parse::<i32>().ok()?;
        let idx = parts[3].parse::<usize>().ok()?;
        Some((IVec2::new(x, y), idx))
    } else {
        None
    }
}

/// Finds the nearest tree within the swing arc and range.
/// Port of `scripts/World/TreeStrike.cs:Find`.
///
/// - `origin`: Centre of the swing (e.g. player's chest).
/// - `forward`: Horizontal swing facing direction.
/// - `range`: Maximum reach in metres.
/// - `arc_deg`: Swing arc in degrees.
/// - `candidates`: Scatter props to test against.
pub fn find_tree_strike<'a>(
    origin: Vec3,
    forward: Vec3,
    range: f32,
    arc_deg: f32,
    candidates: impl IntoIterator<Item = TreeCandidate<'a>>,
) -> Option<TreeHit> {
    let half_arc = (arc_deg * 0.5).to_radians();
    let fwd_flat = Vec3::new(forward.x, 0.0, forward.z);
    let fwd_norm = if fwd_flat.length_squared() > 0.0001 {
        fwd_flat.normalize()
    } else {
        Vec3::Z
    };

    let mut best_distance = f32::MAX;
    let mut best_hit = None;

    for candidate in candidates {
        if !is_tree_kind(candidate.model) {
            continue;
        }

        let mut to_tree = candidate.position - origin;
        to_tree.y = 0.0;
        let distance = to_tree.length();

        if distance > range {
            continue;
        }

        if distance > 0.001 {
            let to_tree_norm = to_tree / distance;
            let dot = fwd_norm.dot(to_tree_norm).clamp(-1.0, 1.0);
            let angle = dot.acos();
            if angle > half_arc {
                continue;
            }
        }

        if distance < best_distance {
            best_distance = distance;
            best_hit = Some(TreeHit::new(
                candidate.chunk,
                candidate.index,
                candidate.position,
            ));
        }
    }

    best_hit
}
