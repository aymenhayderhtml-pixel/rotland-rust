//! Ground chunks from the world generator, turned into GPU meshes. Built on all CPU threads.

use engine::render::{ColorVertex, GpuMesh};
use glam::{IVec2, Vec3};
use rayon::prelude::*;
use world::WorldGenerator;

#[allow(dead_code)] // coord is used by streaming in R1
pub struct GroundChunk {
    pub coord: IVec2,
    pub mesh: GpuMesh,
    pub origin: Vec3,
}

/// Vertex colours from the generator are sRGB floats (Godot's vertex colours are sRGB); store them as bytes.
fn srgb_byte(c: f32) -> u8 {
    (c.clamp(0.0, 1.0) * 255.0).round() as u8
}

pub fn chunk_vertices(world_gen: &WorldGenerator, coord: IVec2) -> Vec<ColorVertex> {
    let ground = world_gen.build_chunk_ground(coord);
    let mut out = Vec::with_capacity(ground.triangles.len() * 3);
    for t in &ground.triangles {
        let c = t.color.to_array();
        let color = [srgb_byte(c[0]), srgb_byte(c[1]), srgb_byte(c[2]), 255];
        for p in t.positions {
            out.push(ColorVertex { position: p.to_array(), normal: t.normal.to_array(), color, bone: 0, _pad: [0; 3] });
        }
    }
    out
}

/// All chunks within `radius` (in chunks) of `center`, built in parallel, then uploaded.
pub fn build_ring(device: &wgpu::Device, world_gen: &WorldGenerator, center: IVec2, radius: i32) -> Vec<GroundChunk> {
    let coords: Vec<IVec2> = (-radius..=radius)
        .flat_map(|dz| (-radius..=radius).map(move |dx| center + IVec2::new(dx, dz)))
        .collect();
    let built: Vec<(IVec2, Vec<ColorVertex>)> = coords.par_iter().map(|&c| (c, chunk_vertices(world_gen, c))).collect();
    built
        .into_iter()
        .map(|(coord, verts)| GroundChunk {
            coord,
            mesh: GpuMesh::new(device, "ground", &verts),
            origin: Vec3::new(coord.x as f32 * WorldGenerator::CHUNK_SIZE, 0.0, coord.y as f32 * WorldGenerator::CHUNK_SIZE),
        })
        .collect()
}

pub fn chunk_of(p: Vec3) -> IVec2 {
    IVec2::new((p.x / WorldGenerator::CHUNK_SIZE).floor() as i32, (p.z / WorldGenerator::CHUNK_SIZE).floor() as i32)
}
