use glam::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BountyReward {
    pub dollars: i32,
    pub xp: i32,
    pub skill: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BountyAcceptError {
    Gone,
    LimitReached(usize),
}

impl std::fmt::Display for BountyAcceptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Gone => write!(f, "That bounty is gone."),
            Self::LimitReached(max) => {
                write!(f, "You can take {max} at a time. Finish or drop one first.")
            }
        }
    }
}

impl std::error::Error for BountyAcceptError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BountyClaimError {
    NotFound,
    NotFinished,
}

impl std::fmt::Display for BountyClaimError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound => write!(f, "Nothing to claim."),
            Self::NotFinished => write!(f, "Not finished yet."),
        }
    }
}

impl std::error::Error for BountyClaimError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BountySaveError {
    MissingField(String),
    InvalidValue(String),
}

impl std::fmt::Display for BountySaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingField(k) => write!(f, "save data missing field '{k}'"),
            Self::InvalidValue(msg) => write!(f, "invalid save value: {msg}"),
        }
    }
}

impl std::error::Error for BountySaveError {}

/// One bounty on the board (offered) or in hand (active).
// Godot: BountyDef.cs
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BountyDef {
    pub id: String,
    pub bounty_type: String,
    pub text: String,
    pub target: i32,
    pub progress: i32,
    pub dollars: i32,
    pub xp: i32,
    pub skill: String,
    pub zombie: Option<String>,
    pub animal: Option<String>,
    pub item: Option<String>,
    pub building: Option<String>,
    pub poi_id: Option<String>,
    pub target_x: f32,
    pub target_z: f32,
    pub expires_on: i32,
    pub weekly: bool,
}

impl Default for BountyDef {
    fn default() -> Self {
        Self {
            id: String::new(),
            bounty_type: String::new(),
            text: String::new(),
            target: 1,
            progress: 0,
            dollars: 0,
            xp: 0,
            skill: String::new(),
            zombie: None,
            animal: None,
            item: None,
            building: None,
            poi_id: None,
            target_x: 0.0,
            target_z: 0.0,
            expires_on: 0,
            weekly: false,
        }
    }
}

impl BountyDef {
    // Godot: BountyDef.cs Done
    pub fn is_done(&self) -> bool {
        self.progress >= self.target
    }

    // Godot: BountyDef.cs TargetXZ
    pub fn target_xz(&self) -> Vec2 {
        Vec2::new(self.target_x, self.target_z)
    }

    // Godot: BountyDef.cs ToDictionary
    pub fn to_save(&self) -> serde_json::Value {
        crate::bounties::save::to_save_bounty(self)
    }

    // Godot: BountyDef.cs FromDictionary
    pub fn from_save(data: &serde_json::Value) -> Result<Self, BountySaveError> {
        crate::bounties::save::from_save_bounty(data)
    }
}
