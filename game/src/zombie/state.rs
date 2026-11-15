//! What a zombie is doing right now (port of ZombieState.cs).

/// Behaviour states. Every state the Godot ZombieState enum has, in the same order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZombieState {
    Idle,
    Wander,
    Investigate,
    Chase,
    Attack,
    Dead,
    /// Just spotted the player: a shout that alerts zombies nearby, then it chases.
    Scream,
    /// Standing still at a corpse, eating (idle variety).
    Eat,
    /// Banging on a door or ladder foot after losing a player who went in or up.
    Bang,
}

impl ZombieState {
    /// Chase, Attack, Scream and Dead ignore new noises; the others go and investigate.
    pub fn ignores_noise(self) -> bool {
        matches!(self, Self::Chase | Self::Attack | Self::Scream | Self::Dead)
    }
}
