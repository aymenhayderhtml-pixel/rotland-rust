//! Group behaviours: herd sighting alerts and horde shared wandering (port of ZombieHerd.cs and ZombieHorde.cs).

use glam::Vec3;

use super::state::ZombieState;

/// Info about another active zombie nearby for herd decisions.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HerdZombie {
    pub position: Vec3,
    pub state: ZombieState,
    pub last_seen: Vec3,
}

impl HerdZombie {
    pub fn new(position: Vec3, state: ZombieState, last_seen: Vec3) -> Self {
        Self {
            position,
            state,
            last_seen,
        }
    }

    /// Whether this zombie is currently chasing or attacking (matches Zombie.IsChasing).
    pub fn is_chasing(&self) -> bool {
        matches!(self.state, ZombieState::Chase | ZombieState::Attack)
    }
}

/// The nearest other zombie within radius (flat distance) that is chasing or attacking, or None.
/// Matches `ZombieHerd.FindChasingNear`.
pub fn find_chasing_near(
    self_pos: Vec3,
    zombies: &[HerdZombie],
    radius: f32,
) -> Option<HerdZombie> {
    let mut best: Option<HerdZombie> = None;
    let mut best_distance = radius;
    for other in zombies {
        if !other.is_chasing() {
            continue;
        }
        let dx = other.position.x - self_pos.x;
        let dz = other.position.z - self_pos.z;
        let distance = (dx * dx + dz * dz).sqrt();
        // Ignore self and anything beyond radius
        if distance > 1e-4 && distance <= best_distance {
            best_distance = distance;
            best = Some(*other);
        }
    }
    best
}

/// A night group sharing a wander target (port of ZombieHorde.cs).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ZombieHorde {
    pub wander_target: Option<Vec3>,
}

impl ZombieHorde {
    pub fn new() -> Self {
        Self {
            wander_target: None,
        }
    }
}
