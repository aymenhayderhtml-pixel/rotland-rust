//! Raised curb and sidewalk chunk mesh: converts `world::metro::curbs::build_curbs_in_chunk`
//! into one `GpuMesh` per loaded chunk. Drawn with the ground in the main pass.

use engine::render::{ColorVertex, GpuMesh};
use glam::IVec2;
use world::WorldGenerator;
use world::metro::curbs::build_curbs_in_chunk;
use world::metro::{MetroLayout, MetroLook};

/// World-space curb vertices for chunk `coord`, or None if the chunk has no sidewalks.
pub fn curb_chunk_vertices(
    layout: &MetroLayout,
    look: &MetroLook,
    coord: IVec2,
) -> Option<Vec<ColorVertex>> {
    let tris = build_curbs_in_chunk(layout, look, coord, WorldGenerator::CHUNK_SIZE);
    if tris.is_empty() {
        return None;
    }
    let mut out = Vec::with_capacity(tris.len() * 3);
    for t in &tris {
        let color = [t.color[0], t.color[1], t.color[2], 255];
        let normal = t.normal.to_array();
        for p in t.positions {
            out.push(ColorVertex {
                position: p.to_array(),
                normal,
                color,
                bone: 0,
                _pad: [0; 3],
            });
        }
    }
    Some(out)
}

/// Uploads a chunk's raised curbs as one `GpuMesh`. None when the chunk has no sidewalks.
pub fn curb_chunk_mesh(
    device: &wgpu::Device,
    layout: &MetroLayout,
    look: &MetroLook,
    coord: IVec2,
) -> Option<GpuMesh> {
    let verts = curb_chunk_vertices(layout, look, coord)?;
    Some(GpuMesh::new(device, "metro curbs", &verts))
}
