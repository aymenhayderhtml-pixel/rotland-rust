//! The campfire's look, built in code like Godot's CampfireVisuals: a ring of grey stones, two crossed
//! logs, and an orange/yellow flame cone that shows while the fire burns. Origins sit on the ground.

use engine::render::GpuMesh;
use glam::{Mat4, Vec3};
use pack::PackVertex;

const STONES: usize = 8;
const RING_RADIUS: f32 = 0.42;

pub struct CampfireMeshes {
    /// Stones and logs (always drawn for a placed fire).
    pub base: GpuMesh,
    /// The flame (drawn while lit). Alpha 0 = fully self-lit (see lit.wgsl).
    pub flame: GpuMesh,
}

impl CampfireMeshes {
    pub fn new(device: &wgpu::Device) -> Self {
        let mut base = Vec::new();
        for i in 0..STONES {
            let a = i as f32 / STONES as f32 * std::f32::consts::TAU;
            let at = Mat4::from_translation(Vec3::new(a.cos() * RING_RADIUS, 0.06, a.sin() * RING_RADIUS)) * Mat4::from_rotation_y(a);
            add_box(&mut base, at, Vec3::new(0.12, 0.07, 0.09), [120, 118, 112, 255]);
        }
        for (yaw, lift) in [(0.3_f32, 0.06_f32), (1.87, 0.12)] {
            let at = Mat4::from_translation(Vec3::new(0.0, lift, 0.0)) * Mat4::from_rotation_y(yaw);
            add_box(&mut base, at, Vec3::new(0.42, 0.045, 0.045), [96, 62, 34, 255]);
        }
        let mut flame = Vec::new();
        add_cone(&mut flame, 0.25, 0.65, 0.1, [255, 120, 30, 0]);
        add_cone(&mut flame, 0.13, 0.42, 0.12, [255, 220, 90, 0]);
        Self { base: GpuMesh::new(device, "campfire_base", &base), flame: GpuMesh::new(device, "campfire_flame", &flame) }
    }
}

fn vertex(p: Vec3, n: Vec3, color: [u8; 4]) -> PackVertex {
    PackVertex { position: p.to_array(), normal: n.to_array(), color, ..PackVertex::ZERO }
}

/// A box of half-size `h` placed by `at` (flat-shaded: 6 faces, 2 triangles each).
fn add_box(out: &mut Vec<PackVertex>, at: Mat4, h: Vec3, color: [u8; 4]) {
    let faces = [Vec3::X, Vec3::NEG_X, Vec3::Y, Vec3::NEG_Y, Vec3::Z, Vec3::NEG_Z];
    for n in faces {
        // Two axes across the face, ordered so the triangles face outwards
        let (u, v) = if n.x != 0.0 { (Vec3::Y, Vec3::Z) } else if n.y != 0.0 { (Vec3::Z, Vec3::X) } else { (Vec3::X, Vec3::Y) };
        let (u, v) = if n.x + n.y + n.z < 0.0 { (v, u) } else { (u, v) };
        let c = n * h;
        let corner = |su: f32, sv: f32| c + u * h * su + v * h * sv;
        let quad = [corner(-1.0, -1.0), corner(1.0, -1.0), corner(1.0, 1.0), corner(-1.0, 1.0)];
        let world_n = at.transform_vector3(n).normalize();
        for i in [0, 1, 2, 0, 2, 3] {
            out.push(vertex(at.transform_point3(quad[i]), world_n, color));
        }
    }
}

/// A six-sided cone standing on the ground at height `lift`.
fn add_cone(out: &mut Vec<PackVertex>, radius: f32, height: f32, lift: f32, color: [u8; 4]) {
    let tip = Vec3::new(0.0, lift + height, 0.0);
    for i in 0..6 {
        let a0 = i as f32 / 6.0 * std::f32::consts::TAU;
        let a1 = (i + 1) as f32 / 6.0 * std::f32::consts::TAU;
        let p0 = Vec3::new(a0.cos() * radius, lift, a0.sin() * radius);
        let p1 = Vec3::new(a1.cos() * radius, lift, a1.sin() * radius);
        let n = (p1 - tip).cross(p0 - tip).normalize();
        for p in [p0, tip, p1] {
            out.push(vertex(p, n, color));
        }
    }
}
