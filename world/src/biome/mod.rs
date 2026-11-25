use serde::{Deserialize, Serialize};

pub mod layout;
pub mod registry;

pub use layout::{BiomeSample, SectorConfig, TierConfig, WorldLayout, WorldLayoutData};
pub use registry::{BiomeDef, BiomeGround, BiomeHeight, BiomeRegistry};

/// The six biomes of the open world. Plains and Forest keep discriminants 0 and 1 for golden save/test compatibility.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BiomeId {
    // new: DECISIONS 2026-11-25
    Plains = 0,
    // new: DECISIONS 2026-11-25
    Forest = 1,
    // new: DECISIONS 2026-11-25
    Desert = 2,
    // new: DECISIONS 2026-11-25
    Snow = 3,
    // new: DECISIONS 2026-11-25
    Swamp = 4,
    // new: DECISIONS 2026-11-25
    Mountains = 5,
}

impl BiomeId {
    // new: DECISIONS 2026-11-25
    pub const COUNT: usize = 6;

    // new: DECISIONS 2026-11-25
    pub const ALL: [BiomeId; Self::COUNT] = [
        BiomeId::Plains,
        BiomeId::Forest,
        BiomeId::Desert,
        BiomeId::Snow,
        BiomeId::Swamp,
        BiomeId::Mountains,
    ];

    pub const fn key(self) -> &'static str {
        match self {
            BiomeId::Plains => "plains",
            BiomeId::Forest => "forest",
            BiomeId::Desert => "desert",
            BiomeId::Snow => "snow",
            BiomeId::Swamp => "swamp",
            BiomeId::Mountains => "mountains",
        }
    }

    pub fn from_key(key: &str) -> Option<BiomeId> {
        match key {
            "plains" => Some(BiomeId::Plains),
            "forest" => Some(BiomeId::Forest),
            "desert" => Some(BiomeId::Desert),
            "snow" => Some(BiomeId::Snow),
            "swamp" => Some(BiomeId::Swamp),
            "mountains" => Some(BiomeId::Mountains),
            _ => None,
        }
    }

    pub const fn index(self) -> usize {
        self as usize
    }
}
