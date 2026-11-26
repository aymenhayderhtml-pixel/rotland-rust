//! New Halden's per-world game state: the far skyline batch and the haze settings, loaded once
//! when a world with a metro city starts. The city's data (towers, layout, name) lives in the world.

use std::collections::HashSet;

use glam::{IVec2, Vec3};
use rotland_ui::metro_ui::MetroHazeConfig;
use world::WorldGenerator;

use crate::metro_render::skyline::{SkylineBatch, SkylineInputs};
use crate::streaming::Streamer;

pub struct MetroPlay {
    pub skyline: SkylineBatch,
    pub haze: MetroHazeConfig,
    /// Loaded chunk coords, refilled each frame (no allocation after the first frame).
    loaded_scratch: HashSet<IVec2>,
}

impl MetroPlay {
    /// The metro state for `world_gen`, or None when the world has no New Halden.
    pub fn start(world_gen: &WorldGenerator) -> Option<Self> {
        world_gen.metro_world()?;
        Some(Self {
            skyline: SkylineBatch::default(),
            haze: MetroHazeConfig::load_default(),
            loaded_scratch: HashSet::new(),
        })
    }

    /// Rebuilds the far skyline when the loaded chunks or the player's cell changed (rare), not every frame.
    pub fn update_skyline(
        &mut self,
        device: &wgpu::Device,
        world_gen: &WorldGenerator,
        streamer: &Streamer,
        player: Vec3,
    ) {
        let Some(metro) = world_gen.metro_world() else {
            return;
        };
        self.loaded_scratch.clear();
        self.loaded_scratch.extend(streamer.chunks.keys().copied());
        let inputs = SkylineInputs {
            towers: &metro.towers,
            ground_y: metro.layout.site.ground_y,
            cfg: &metro.config,
            look: &metro.look,
        };
        let player_xz = glam::Vec2::new(player.x, player.z);
        self.skyline
            .rebuild_if_needed(device, &inputs, &self.loaded_scratch, player_xz);
    }
}

/// Fog distance divisor for the far skyline: the skyline fades out at `skyline_fog_end` while the
/// frame's fog ends at `frame_fog_end` (the shader divides the distance by this). Never below 1.
pub fn skyline_fog_scale(skyline_fog_end: f32, frame_fog_end: f32) -> f32 {
    (skyline_fog_end / frame_fog_end.max(1.0)).max(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skyline_scale_stretches_the_fog_to_the_skyline_distance() {
        // 6000 m skyline over a 190 m frame fog: a tower at 6000 m is at the fog's end
        let scale = skyline_fog_scale(6000.0, 190.0);
        assert!((6000.0 / scale - 190.0).abs() < 1e-3);
    }

    #[test]
    fn skyline_scale_never_drops_below_one() {
        assert_eq!(skyline_fog_scale(100.0, 190.0), 1.0);
        assert_eq!(skyline_fog_scale(6000.0, 0.0), 6000.0);
    }
}
