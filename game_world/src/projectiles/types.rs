//! Projectile definitions and hit event types.

use glam::Vec3;
use serde::{Deserialize, Serialize};

/// Projectile type: arrows from bows or thrown spears.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectileKind {
    Arrow,
    CrossbowBolt,
    ThrownSpear,
}

/// Result of a collision raycast along a projectile's flight path.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Hit {
    Zombie {
        point: Vec3,
        id: u64,
        headshot: bool,
    },
    World {
        point: Vec3,
    },
}

impl Hit {
    pub fn zombie(point: Vec3, id: u64, headshot: bool) -> Self {
        Self::Zombie { point, id, headshot }
    }

    pub fn world(point: Vec3) -> Self {
        Self::World { point }
    }

    pub fn point(&self) -> Vec3 {
        match *self {
            Self::Zombie { point, .. } | Self::World { point } => point,
        }
    }
}

/// Events produced during projectile simulation steps.
#[derive(Clone, Debug, PartialEq)]
pub enum ProjectileEvent {
    HitZombie {
        id: u64,
        damage: f32,
        headshot: bool,
    },
    HitWorld {
        point: Vec3,
        pickup: Option<String>,
    },
    Expired,
}

impl ProjectileEvent {
    pub fn pickup(&self) -> Option<&str> {
        match self {
            Self::HitWorld { pickup, .. } => pickup.as_deref(),
            _ => None,
        }
    }
}

/// One physical projectile flying through the world.
#[derive(Clone, Debug, PartialEq)]
pub struct Projectile {
    pub pos: Vec3,
    pub vel: Vec3,
    pub kind: ProjectileKind,
    pub damage: f32,
    pub owner: Option<u64>,
    pub headshot_multiplier: f32,
    pub gravity: f32,
    pub max_flight_seconds: f32,
    pub flight_time: f32,
    pub pickup_chance: f32,
}

impl Projectile {
    pub fn new(
        pos: Vec3,
        vel: Vec3,
        kind: ProjectileKind,
        damage: f32,
        owner: Option<u64>,
    ) -> Self {
        // data: weapons.json gravity
        let gravity = 9.8;
        // data: weapons.json maxFlightSeconds
        let max_flight_seconds = 10.0;
        let (pickup_chance, headshot_multiplier) = match kind {
            ProjectileKind::Arrow => (
                // data: weapons.json pickupChance
                0.6,
                // data: weapons.json headshotMultiplier
                2.5,
            ),
            ProjectileKind::CrossbowBolt => (
                // data: weapons.json crossbow.pickupChance
                0.75,
                // data: weapons.json crossbow.headshotMultiplier
                2.5,
            ),
            ProjectileKind::ThrownSpear => (
                1.0,
                2.0,
            ),
        };
        Self {
            pos,
            vel,
            kind,
            damage,
            owner,
            headshot_multiplier,
            gravity,
            max_flight_seconds,
            flight_time: 0.0,
            pickup_chance,
        }
    }
}

impl Default for Projectile {
    fn default() -> Self {
        Self::new(Vec3::ZERO, Vec3::ZERO, ProjectileKind::Arrow, 0.0, None)
    }
}
