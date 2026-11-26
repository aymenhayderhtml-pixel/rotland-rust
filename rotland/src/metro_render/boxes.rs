//! Placeholder tower meshes: one procedural box per `TowerSpot`, built in tower-local space
//! (origin at the base centre, +Y up) and then moved to the world by `to_world`. Pure: no GPU.
//!
//! Winding: every triangle is clockwise seen from outside, the same order as the terrain chunks
//! (the `mesh` pipeline culls back faces with `FrontFace::Cw`).

use engine::render::ColorVertex;
use glam::{Mat4, Vec2, Vec3};
use world::metro::{MetroLook, TowerSpot};

/// Near (street close-up: windows, base band, roof details) or far (skyline: walls and roof only).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Detail {
    Near,
    Far,
}

/// Tower-local mesh of one spot. Near: walls, base band, window panes, roof, parapet, rooftop box
/// (landmarks get a stepped top instead). Far: walls, roof and the landmark step, one colour.
pub fn tower_mesh(spot: &TowerSpot, look: &MetroLook, detail: Detail) -> Vec<ColorVertex> {
    let seed = spot_seed(spot);
    let body = look.wall_palette[mix(seed) as usize % look.wall_palette.len()];
    let faces = faces(spot.half);
    let mut out = Vec::new();
    match detail {
        Detail::Far => {
            let c = look.far_color;
            for f in &faces {
                f.band(&mut out, 0.0, spot.height, 0.0, c);
            }
            roof(&mut out, spot.half, spot.height, c);
            if spot.landmark {
                step_box(&mut out, spot, look, c);
            }
        }
        Detail::Near => {
            for f in &faces {
                f.band(&mut out, look.base_height, spot.height, 0.0, body);
                f.band(&mut out, 0.0, look.base_height, 0.0, look.base_color);
            }
            windows(&mut out, spot, look, &faces, seed);
            roof(&mut out, spot.half, spot.height, look.roof_color);
            if spot.landmark {
                step_box(&mut out, spot, look, body);
            } else {
                parapet(&mut out, spot, look, &faces);
                rooftop_box(&mut out, spot, look);
            }
        }
    }
    out
}

/// Moves a tower-local mesh to the world: yaw about +Y, then to `origin` (the base centre).
pub fn to_world(local: &[ColorVertex], origin: Vec3, yaw: f32) -> Vec<ColorVertex> {
    let rot = Mat4::from_rotation_y(yaw);
    let place = Mat4::from_translation(origin) * rot;
    local
        .iter()
        .map(|v| ColorVertex {
            position: place.transform_point3(Vec3::from(v.position)).to_array(),
            normal: rot.transform_vector3(Vec3::from(v.normal)).to_array(),
            ..*v
        })
        .collect()
}

/// Stable per-tower seed from its position (no clock, no global random).
pub fn spot_seed(spot: &TowerSpot) -> u32 {
    spot.pos.x.to_bits() ^ spot.pos.y.to_bits().rotate_left(16)
}

/// Integer hash for window and colour variety. // new: DECISIONS 2026-11-26 (window variety hash)
pub fn mix(mut h: u32) -> u32 {
    h ^= h >> 16;
    h = h.wrapping_mul(0x7feb_352d);
    h ^= h >> 15;
    h = h.wrapping_mul(0x846c_a68b);
    h ^ (h >> 16)
}

/// One outward vertical wall: `center` on its plane, `n` outward, `t` along the face, `half` its half length.
struct Face {
    center: Vec3,
    n: Vec3,
    t: Vec3,
    half: f32,
}

impl Face {
    /// Point at `u` along the face, height `y`, `d` metres out from the wall plane.
    fn at(&self, u: f32, y: f32, d: f32) -> Vec3 {
        self.center + self.t * u + self.n * d + Vec3::Y * y
    }

    /// The whole face between heights `y0` and `y1`, `d` metres out.
    fn band(&self, out: &mut Vec<ColorVertex>, y0: f32, y1: f32, d: f32, c: [u8; 3]) {
        self.rect(out, (-self.half, self.half), (y0, y1), d, c);
    }

    /// A sub-rectangle of the face: `u` (from, to) along it, `y` (from, to) high, `d` metres out.
    fn rect(&self, out: &mut Vec<ColorVertex>, u: (f32, f32), y: (f32, f32), d: f32, c: [u8; 3]) {
        let (u0, u1) = u;
        let (y0, y1) = y;
        quad(out, [self.at(u0, y0, d), self.at(u1, y0, d), self.at(u1, y1, d), self.at(u0, y1, d)], self.n, c);
    }
}

/// The four walls of a box with half extents `half` (x and z).
fn faces(half: Vec2) -> [Face; 4] {
    [
        Face { center: Vec3::new(half.x, 0.0, 0.0), n: Vec3::X, t: Vec3::Z, half: half.y },
        Face { center: Vec3::new(-half.x, 0.0, 0.0), n: Vec3::NEG_X, t: Vec3::Z, half: half.y },
        Face { center: Vec3::new(0.0, 0.0, half.y), n: Vec3::Z, t: Vec3::X, half: half.x },
        Face { center: Vec3::new(0.0, 0.0, -half.y), n: Vec3::NEG_Z, t: Vec3::X, half: half.x },
    ]
}

/// Emits a planar quad (corners in order around it) as two triangles, clockwise seen from `n`.
fn quad(out: &mut Vec<ColorVertex>, q: [Vec3; 4], n: Vec3, c: [u8; 3]) {
    let flipped = (q[1] - q[0]).cross(q[2] - q[0]).dot(n) > 0.0;
    let order = if flipped { [0, 2, 1, 0, 3, 2] } else { [0, 1, 2, 0, 2, 3] };
    for i in order {
        out.push(vertex(q[i], n, c));
    }
}

fn vertex(p: Vec3, n: Vec3, c: [u8; 3]) -> ColorVertex {
    ColorVertex {
        position: p.to_array(),
        normal: n.to_array(),
        color: [c[0], c[1], c[2], 255],
        bone: 0,
        _pad: [0; 3],
    }
}

/// Window grid: per floor, per face, up to `max_bays_per_face` bays; one pane per bay.
fn windows(out: &mut Vec<ColorVertex>, spot: &TowerSpot, look: &MetroLook, faces: &[Face; 4], seed: u32) {
    let floor_h = spot.height / spot.floors.max(1) as f32;
    let pane_y0_share = look.pane_bottom_share;
    for floor in 0..spot.floors {
        let y0 = floor as f32 * floor_h + floor_h * pane_y0_share;
        let y1 = y0 + floor_h * look.pane_height_share;
        if y0 < look.base_height || y1 > spot.height {
            continue;
        }
        for (fi, face) in faces.iter().enumerate() {
            let len = face.half * 2.0;
            let bays = ((len / look.bay_width).round() as u32).clamp(1, look.max_bays_per_face);
            let bay_len = len / bays as f32;
            let half_pane = bay_len * 0.5 * look.pane_width_share;
            for bay in 0..bays {
                let u = -face.half + (bay as f32 + 0.5) * bay_len;
                let h = mix(seed ^ mix(floor.wrapping_mul(97) ^ ((fi as u32) << 8) ^ bay));
                let c = if h.is_multiple_of(look.broken_one_in) {
                    look.broken_color
                } else if (h / 7).is_multiple_of(look.dark_one_in) {
                    look.glass_dark
                } else {
                    look.glass_color
                };
                face.rect(out, (u - half_pane, u + half_pane), (y0, y1), look.pane_offset, c);
            }
        }
    }
}

/// Flat top of the tower at `height`.
fn roof(out: &mut Vec<ColorVertex>, half: Vec2, height: f32, c: [u8; 3]) {
    let q = [
        Vec3::new(-half.x, height, -half.y),
        Vec3::new(half.x, height, -half.y),
        Vec3::new(half.x, height, half.y),
        Vec3::new(-half.x, height, half.y),
    ];
    quad(out, q, Vec3::Y, c);
}

/// Four low strips on the roofline (outer faces only).
fn parapet(out: &mut Vec<ColorVertex>, spot: &TowerSpot, look: &MetroLook, faces: &[Face; 4]) {
    for f in faces {
        f.band(out, spot.height, spot.height + look.parapet_height, 0.0, look.roof_color);
    }
}

/// Box on the roof, `roof_box_share` of the footprint and `roof_box_height` high.
fn rooftop_box(out: &mut Vec<ColorVertex>, spot: &TowerSpot, look: &MetroLook) {
    let half = spot.half * look.roof_box_share;
    let top = spot.height + look.roof_box_height;
    closed_box(out, half, spot.height, top, look.roof_color);
}

/// Landmark stepped top: a smaller box on the roof.
fn step_box(out: &mut Vec<ColorVertex>, spot: &TowerSpot, look: &MetroLook, c: [u8; 3]) {
    let half = spot.half * look.step_share;
    closed_box(out, half, spot.height, spot.height + look.step_height, c);
}

/// Four walls and a top of a box centred on the spot's axis, from `y0` to `y1`.
fn closed_box(out: &mut Vec<ColorVertex>, half: Vec2, y0: f32, y1: f32, c: [u8; 3]) {
    for f in &faces(half) {
        f.band(out, y0, y1, 0.0, c);
    }
    roof(out, half, y1, c);
}
