use serde::{Deserialize, Serialize};

/// The two biomes of the open world. The terrain blends between them with no seams.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BiomeId {
    Plains,
    Forest,
}
