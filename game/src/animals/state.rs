//! Animal state enum.
// Godot: AnimalState.cs

use serde::{Deserialize, Serialize};

/// What a wild animal is doing. Prey: Calm, Alert, Flee. Boars: Calm, Charge, Attack. Any: Dead.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(Default)]
pub enum AnimalState {
    #[default]
    Calm,
    Alert,
    Flee,
    Charge,
    Attack,
    Dead,
}

