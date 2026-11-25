//! Read-only view snapshot passed from the gameplay layer to the map UI renderer.

use glam::Vec2;

/// Read-only snapshot of the world and player state used by the map renderer.
pub struct MapView<'a> {
    pub player_pos: Option<Vec2>,
    pub player_yaw: f32,
    pub map_state: &'a game::MapState,
    pub generator: Option<&'a world::WorldGenerator>,
    pub camps: &'a [Vec2],
    pub bounties: &'a [game::BountyDef],
}

