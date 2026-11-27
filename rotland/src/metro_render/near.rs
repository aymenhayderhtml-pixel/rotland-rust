//! Near tower placement: instanced modular parts per loaded chunk.
//! A tower belongs to the chunk its centre stands in.

use glam::{IVec2, Vec2, Vec3};
use world::WorldGenerator;
use world::metro::{MetroWorld, StyleDef, TowerSpot};

use crate::props::{Placed, PropLibrary};

use super::parts::{chunk_tower_parts_with_climb, tower_load_ring_range};

/// The chunk a world XZ point stands in (same rule as `terrain::chunk_of`).
pub fn tower_chunk(pos: Vec2) -> IVec2 {
    IVec2::new(
        (pos.x / WorldGenerator::CHUNK_SIZE).floor() as i32,
        (pos.y / WorldGenerator::CHUNK_SIZE).floor() as i32,
    )
}

/// Towers whose centre is in chunk `coord`.
pub fn towers_in_chunk(towers: &[TowerSpot], coord: IVec2) -> impl Iterator<Item = &TowerSpot> {
    towers.iter().filter(move |t| tower_chunk(t.pos) == coord)
}

/// Places all tower kit parts in chunk `coord` as `Placed` instances in `library`.
pub fn place_chunk_towers(
    device: &wgpu::Device,
    pack: &pack::ModelPack,
    library: &mut PropLibrary,
    metro: &MetroWorld,
    coord: IVec2,
    styles: &[StyleDef],
) -> Vec<Placed> {
    let climb_indices: Vec<usize> = world::metro::climb_towers(metro)
        .into_iter()
        .map(|c| c.tower)
        .collect();
    let parts = chunk_tower_parts_with_climb(
        &metro.towers,
        coord,
        metro.layout.site.ground_y,
        styles,
        &metro.config,
        &metro.damage,
        &climb_indices,
    );
    let lod0 = metro.config.towers.lod0_distance;
    let range = tower_load_ring_range();
    let mut out = Vec::with_capacity(parts.len());
    for part in parts {
        if let Some(slot) = library.get_tower(device, pack, &part.model, lod0, range) {
            out.push(library.place(slot, part.pos, part.yaw, 1.0));
        }
    }
    out
}

/// World position of a tower's base centre.
pub fn base_point(t: &TowerSpot, ground_y: f32) -> Vec3 {
    Vec3::new(t.pos.x, ground_y, t.pos.y)
}
