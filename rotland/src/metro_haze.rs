//! Applies New Halden's grey-green haze to the frame's lighting, after the weather and biome fog.
//! Pure maths lives in `rotland_ui::metro_ui::haze`; this only reads the player and writes fog.

use engine::render::Lighting;
use glam::{Vec2, Vec3};
use rotland_ui::metro_ui::{MetroHazeConfig, blend_fog, city_signed_distance, haze_weight};
use world::metro::MetroSite;

/// Blends the fog toward the New Halden haze by how close the player is to the city.
/// Returns early (no change) when the player is outside the haze.
pub fn apply(
    lighting: &mut Lighting,
    player_pos: Vec3,
    site: Option<&MetroSite>,
    cfg: &MetroHazeConfig,
) {
    let Some(site) = site else {
        return;
    };
    let player_xz = Vec2::new(player_pos.x, player_pos.z);
    let weight = haze_weight(city_signed_distance(player_xz, site), cfg.blend_m);
    if weight <= 0.0 {
        return;
    }
    let (color, begin, end) = blend_fog(
        lighting.fog_color,
        lighting.fog_begin,
        lighting.fog_end,
        cfg,
        weight,
    );
    lighting.fog_color = color;
    lighting.fog_begin = begin;
    lighting.fog_end = end;
}
