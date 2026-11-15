//! Ground chunks from the world generator, turned into vertices for the GPU.

use engine::render::ColorVertex;
use glam::{IVec2, Vec3};
use world::WorldGenerator;

/// Vertex colours from the generator are sRGB floats (Godot's vertex colours are sRGB); store them as bytes.
fn srgb_byte(c: f32) -> u8 {
    (c.clamp(0.0, 1.0) * 255.0).round() as u8
}

/// The chunk's drawable vertices and its 33x33 heights (for collision).
pub fn chunk_vertices(world_gen: &WorldGenerator, coord: IVec2) -> (Vec<ColorVertex>, Vec<f32>) {
    let ground = world_gen.build_chunk_ground(coord);
    let mut out = Vec::with_capacity(ground.triangles.len() * 3);
    for t in &ground.triangles {
        let c = t.color.to_array();
        let color = [srgb_byte(c[0]), srgb_byte(c[1]), srgb_byte(c[2]), 255];
        for p in t.positions {
            out.push(ColorVertex { position: p.to_array(), normal: t.normal.to_array(), color, bone: 0, _pad: [0; 3] });
        }
    }
    (out, ground.heights)
}

pub fn chunk_of(p: Vec3) -> IVec2 {
    IVec2::new((p.x / WorldGenerator::CHUNK_SIZE).floor() as i32, (p.z / WorldGenerator::CHUNK_SIZE).floor() as i32)
}
