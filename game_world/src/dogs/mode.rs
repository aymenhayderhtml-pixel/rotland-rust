//! Orders and modes for the dog.
// Godot: DogMode.cs

use std::fmt;
use serde::{Deserialize, Serialize};

/// What the dog is doing on the player's orders.
/// Follow is the default; G toggles Stay; Shift+G sends it Home.
// Godot: DogMode.cs
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DogMode {
    /// Walks behind the player and fights.
    #[default]
    Follow,
    /// Sits where it is and waits (still fights zombies that come close).
    Stay,
    /// Walks to the camp and waits there. Becomes Stay on arrival.
    Home,
}

impl DogMode {
    /// Parses mode string case-insensitively, falling back to Follow.
    pub fn from_str_loose(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "stay" => Self::Stay,
            "home" => Self::Home,
            _ => Self::Follow,
        }
    }
}

impl fmt::Display for DogMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Follow => write!(f, "Follow"),
            Self::Stay => write!(f, "Stay"),
            Self::Home => write!(f, "Home"),
        }
    }
}
