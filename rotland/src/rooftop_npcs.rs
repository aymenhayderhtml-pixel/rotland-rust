//! Rooftop camp people (R44-05). The people are drawn as props by the chunk contents (`world`);
//! this finds where the rooftop trader stands, for the [E] prompt and the shop.

use glam::Vec3;
use world::WorldGenerator;
use world::metro::safehouse::{rooftop_camp_in_chunk, rooftop_camp_tower, safe_house_layout};

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
