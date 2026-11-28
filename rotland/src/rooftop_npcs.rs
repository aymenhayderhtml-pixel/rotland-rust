//! Rooftop camp people (R44-05). The people are drawn as props by the chunk contents (`world`);
//! this finds where the rooftop trader stands, for the [E] prompt and the shop.

use glam::Vec3;
use world::WorldGenerator;
use world::metro::safehouse::{rooftop_camp_in_chunk, rooftop_camp_tower, safe_house_layout};

/// Clip for a rooftop person who sits (the camp's sitting clip) and for one who stands.
const SIT_CLIP: &str = "sit_ground"; // new: DECISIONS 2026-11-28 (camp clip; falls back to idle)
const STAND_CLIP: &str = "idle";

/// A rooftop camp person drawn as an animated figure (`camp_npcs`), not as a prop.
#[derive(Debug, Clone, PartialEq)]
pub struct RooftopFigure {
    pub id: String,
    pub model: String,
    pub position: Vec3,
    pub yaw: f32,
    pub clip: String,
}

/// Every rooftop camp person, with the clip they play (sit or stand). Empty without a camp.
pub fn rooftop_people(world_gen: &WorldGenerator) -> Vec<RooftopFigure> {
    let (Some(metro), Some(layout)) = (world_gen.metro_world(), safe_house_layout()) else {
        return Vec::new();
    };
    let Some(tower) = rooftop_camp_tower(metro) else {
        return Vec::new();
    };
    let t = &metro.towers[tower];
    let size = WorldGenerator::CHUNK_SIZE;
    let chunk = ((t.pos.x / size).floor() as i32, (t.pos.y / size).floor() as i32);
    let (_, npcs) = rooftop_camp_in_chunk(metro, layout, chunk);
    npcs.into_iter()
        .map(|n| RooftopFigure {
            clip: if n.sitting { SIT_CLIP } else { STAND_CLIP }.to_string(),
            id: n.id,
            model: n.model,
            position: n.position,
            yaw: n.yaw,
        })
        .collect()
}

/// Where the rooftop trader stands, when the player is near the rooftop camp tower.
pub fn trader_position(world_gen: &WorldGenerator, player: Vec3) -> Option<Vec3> {
    let metro = world_gen.metro_world()?;
    let layout = safe_house_layout()?;
    let tower = rooftop_camp_tower(metro)?;
    let t = metro.towers.get(tower)?;
    // Only looked up while the player is in the tower's chunk (no radius number needed)
    let size = WorldGenerator::CHUNK_SIZE;
    let chunk = ((t.pos.x / size).floor() as i32, (t.pos.y / size).floor() as i32);
    if ((player.x / size).floor() as i32, (player.z / size).floor() as i32) != chunk {
        return None;
    }
    let (_, npcs) = rooftop_camp_in_chunk(metro, layout, chunk);
    npcs.into_iter().find(|n| n.role == "trader").map(|n| n.position)
}
