//! Public data types shared by companion AI, the game loop and rendering.

use glam::Vec3;

/// Animation clip state for rendering a companion body.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompanionAnim {
    Idle,
    Walk,
    Run,
    Attack,
    Downed,
}

/// Minimal zombie snapshot passed to [`CompanionSquad::tick`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CompanionTarget {
    pub id: u32,
    pub position: Vec3,
}

/// World snapshot needed for one companion update step.
pub struct CompanionWorld<'a> {
    pub player: Vec3,
    pub player_forward: Vec3,
    pub camp: Vec3,
    pub zombies: &'a [CompanionTarget],
}

/// Gameplay events produced by companions during [`CompanionSquad::tick`], `damage`, or `on_morning`.
#[derive(Clone, Debug, PartialEq)]
pub enum CompanionEvent {
    Hit {
        companion: String,
        zombie_id: u32,
        damage: f32,
    },
    Downed {
        companion: String,
    },
    Recovered {
        companion: String,
    },
}

/// Read-only render snapshot for one hired teammate.
#[derive(Clone, Debug, PartialEq)]
pub struct CompanionDraw {
    pub id: String,
    pub name: String,
    pub position: Vec3,
    pub yaw: f32,
    pub anim: CompanionAnim,
    pub health_fraction: f32,
}
