//! States and events for the pressure director.

use serde::{Deserialize, Serialize};

/// State of the AI director pacing cycle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum DirectorState {
    #[default]
    Calm,
    Building,
    Wave,
    Relax,
}

/// Category of wave triggered by the director.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WaveKind {
    Night,
    City,
    Noise,
}

/// Events emitted by the director to coordinate game pressure and horde spawns.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DirectorEvent {
    WaveStart { size: usize, kind: WaveKind },
    CampAttack { size: usize },
}

/// Events exposed to the HUD and camp systems.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HordeEvent {
    HordeIncoming { size: usize },
    CampAttack { size: usize },
    PhaseChange { from: DirectorState, to: DirectorState },
}

/// Save state struct that preserves director pacing across game saves.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct DirectorSaveState {
    #[serde(default)]
    pub state: DirectorState,
    #[serde(default)]
    pub timer: f32,
    #[serde(default)]
    pub night_timer: f32,
    #[serde(default)]
    pub night_horde_triggered: bool,
    #[serde(default)]
    pub was_night: bool,
}
