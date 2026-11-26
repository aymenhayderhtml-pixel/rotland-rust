//! Near tower meshes: one merged world-space vertex buffer per loaded chunk, built once when the
//! chunk loads. A tower belongs to the chunk its centre stands in.

use engine::render::{ColorVertex, GpuMesh};
use glam::{IVec2, Vec2, Vec3};
use world::WorldGenerator;
use world::metro::{MetroLook, TowerSpot};

use super::boxes::{Detail, to_world, tower_mesh};

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

/// World-space near vertices of the towers in chunk `coord`, merged. None when the chunk has none.
pub fn near_tower_vertices(
    towers: &[TowerSpot],
    coord: IVec2,
    ground_y: f32,
    look: &MetroLook,
) -> Option<Vec<ColorVertex>> {
    let mut out = Vec::new();
    for t in towers_in_chunk(towers, coord) {
        let local = tower_mesh(t, look, Detail::Near);
        out.extend(to_world(&local, base_point(t, ground_y), t.yaw));
    }
    (!out.is_empty()).then_some(out)
}

/// Uploads a chunk's near towers as one GPU mesh. None when the chunk has no towers.
pub fn near_tower_mesh(
    device: &wgpu::Device,
    towers: &[TowerSpot],
    coord: IVec2,
    ground_y: f32,
    look: &MetroLook,
) -> Option<GpuMesh> {
    let verts = near_tower_vertices(towers, coord, ground_y, look)?;
    Some(GpuMesh::new(device, "metro towers", &verts))
}

/// World position of a tower's base centre.
pub fn base_point(t: &TowerSpot, ground_y: f32) -> Vec3 {
    Vec3::new(t.pos.x, ground_y, t.pos.y)
}
