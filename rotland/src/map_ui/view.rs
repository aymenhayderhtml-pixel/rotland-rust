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

impl<'a> MapView<'a> {
    pub fn new(map_state: &'a game::MapState) -> Self {
        Self {
            player_pos: None,
            player_yaw: 0.0,
            map_state,
            generator: None,
            camps: &[],
            bounties: &[],
        }
    }

    pub fn with_player(mut self, pos: Option<Vec2>, yaw: f32) -> Self {
        self.player_pos = pos;
        self.player_yaw = yaw;
        self
    }

    pub fn with_generator(mut self, generator: Option<&'a world::WorldGenerator>) -> Self {
        self.generator = generator;
        self
    }

    pub fn with_camps(mut self, camps: &'a [Vec2]) -> Self {
        self.camps = camps;
        self
    }

    pub fn with_bounties(mut self, bounties: &'a [game::BountyDef]) -> Self {
        self.bounties = bounties;
        self
    }
}
