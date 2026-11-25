//! Types for zombie vocalizations (ports of VoiceZombie, VoiceState, VoiceEvent).

use glam::Vec3;

/// Zombie behavior state from the perspective of the voice system.
///
/// Direct port of the Godot `ZombieState` states relevant to vocalizations:
/// Idle, Wander, Chase, Attack, and Dead.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VoiceState {
    Idle,
    Wander,
    Chase,
    Attack,
    Dead,
}

impl From<crate::zombie::ZombieState> for VoiceState {
    fn from(state: crate::zombie::ZombieState) -> Self {
        match state {
            crate::zombie::ZombieState::Idle => VoiceState::Idle,
            crate::zombie::ZombieState::Wander | crate::zombie::ZombieState::Investigate => {
                VoiceState::Wander
            }
            crate::zombie::ZombieState::Chase => VoiceState::Chase,
            crate::zombie::ZombieState::Attack => VoiceState::Attack,
            crate::zombie::ZombieState::Dead => VoiceState::Dead,
            crate::zombie::ZombieState::Scream => VoiceState::Attack,
            crate::zombie::ZombieState::Eat => VoiceState::Idle,
            crate::zombie::ZombieState::Bang => VoiceState::Attack,
        }
    }
}

/// A snapshot of a zombie used for audio voice evaluation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VoiceZombie {
    pub id: u64,
    pub position: Vec3,
    pub state: VoiceState,
}

impl VoiceZombie {
    pub const fn new(id: u64, position: Vec3, state: VoiceState) -> Self {
        Self { id, position, state }
    }
}

/// An audio event emitted by the zombie voice system to be played by the audio engine.
#[derive(Clone, Debug, PartialEq)]
pub struct VoiceEvent {
    pub sound_id: String,
    pub position: Vec3,
}

impl VoiceEvent {
    pub fn new(sound_id: impl Into<String>, position: Vec3) -> Self {
        Self {
            sound_id: sound_id.into(),
            position,
        }
    }
}
