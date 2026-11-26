//! Raised curb and sidewalk mesh builder for New Halden. Pure: no GPU.
//!
//! Each `MetroStreet` gets two raised sidewalk strips (`curbHeight` tall) along its edges,
//! stopping at intersections with `curbWidth` ramps at open ends and `kerbStoneLength` seam
//! lines on the asphalt-facing curb. Park edges (`Reserved::Park`) stay grass with no sidewalk.

use glam::{IVec2, Vec2, Vec3};

use crate::metro::layout::{MetroLayout, MetroStreet, Reserved};
use crate::metro::paint::MetroLook;

// new: DECISIONS 2026-11-26 (curb face shading, kerb dimensions, and segmenting)
const CURB_FACE_SHADE: f32 = 0.72; // new: DECISIONS 2026-11-26
const KERB_SEAM_SHADE: f32 = 0.45; // new: DECISIONS 2026-11-26
const KERB_TOP_SHADE: f32 = 0.90; // new: DECISIONS 2026-11-26
const KERB_TOP_WIDTH: f32 = 0.16; // new: DECISIONS 2026-11-26
const KERB_SEAM_WIDTH: f32 = 0.04; // new: DECISIONS 2026-11-26
const TOP_SEGMENT_M: f32 = 8.0; // new: DECISIONS 2026-11-26
const ENDPOINT_EPS: f32 = 0.01; // new: DECISIONS 2026-11-26
const PARK_PROBE_M: f32 = 1.0; // new: DECISIONS 2026-11-26
const CURB_TOP_OFFSET_Y: f32 = 0.002; // new: DECISIONS 2026-11-26

/// One world-space triangle of the raised curb/sidewalk mesh (clockwise seen from `normal`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CurbTri {
    pub positions: [Vec3; 3],
    pub normal: Vec3,
    pub color: [u8; 3],
}

/// Builds the raised sidewalk and curb triangles for the whole city in world space.
pub fn build_curbs(layout: &MetroLayout, look: &MetroLook) -> Vec<CurbTri> {
    let mut out = Vec::new();
    for s in &layout.streets {
        append_street_curbs(layout, look, s, &mut out);
    }
    out
}

/// Builds the raised sidewalk and curb triangles belonging to chunk `coord` in world space.
pub fn build_curbs_in_chunk(
    layout: &MetroLayout,
    look: &MetroLook,
    coord: IVec2,
    chunk_size: f32,
) -> Vec<CurbTri> {
    let min_x = coord.x as f32 * chunk_size;
    let min_z = coord.y as f32 * chunk_size;
    let max_x = min_x + chunk_size;
    let max_z = min_z + chunk_size;
    let mut out = Vec::new();
    for s in &layout.streets {
        if !street_overlaps_box(layout, s, min_x, min_z, max_x, max_z) {
            continue;
        }
        append_street_curbs(layout, look, s, &mut out);
    }
    out.retain(|t| {
        let cx = (t.positions[0].x + t.positions[1].x + t.positions[2].x) / 3.0;
        let cz = (t.positions[0].z + t.positions[1].z + t.positions[2].z) / 3.0;
        cx >= min_x && cx < max_x && cz >= min_z && cz < max_z
    });
    out
}

fn street_overlaps_box(
    layout: &MetroLayout,
    s: &MetroStreet,
    min_x: f32,
    min_z: f32,
    max_x: f32,
    max_z: f32,
) -> bool {
    let wa = layout.site.to_world(s.a);
    let wb = layout.site.to_world(s.b);
    let pad = s.width * 0.5 + 1.0; // new: DECISIONS 2026-11-26 (half-width + margin for AABB reject)
    let slo_x = wa.x.min(wb.x) - pad;
    let shi_x = wa.x.max(wb.x) + pad;
    let slo_z = wa.y.min(wb.y) - pad;
    let shi_z = wa.y.max(wb.y) + pad;
    shi_x >= min_x && slo_x <= max_x && shi_z >= min_z && slo_z <= max_z
}

fn append_street_curbs(
    layout: &MetroLayout,
    look: &MetroLook,
    s: &MetroStreet,
    out: &mut Vec<CurbTri>,
) {
    let ab = s.b - s.a;
    let len = ab.length();
    if len <= ENDPOINT_EPS {
        return;
    }
    let dir = ab / len;
    let perp = Vec2::new(-dir.y, dir.x);
    let trim_a = endpoint_trim(s.a, dir, &layout.streets);
    let trim_b = endpoint_trim(s.b, dir, &layout.streets);
    let t0 = trim_a;
    let t1 = len - trim_b;
    if t1 <= t0 + ENDPOINT_EPS {
        return;
    }

    let d_in = (s.width * 0.5 - look.sidewalk_width).max(0.0);
    let d_out = s.width * 0.5;
    if d_out <= d_in + ENDPOINT_EPS {
        return;
    }

    let mid = (s.a + s.b) * 0.5;
    for sign in [-1.0_f32, 1.0] {
        let side = perp * sign;
        let probe = layout.site.to_world(mid + side * (d_out + PARK_PROBE_M));
        if matches!(layout.block_at_point(probe), Some(b) if b.reserved == Some(Reserved::Park)) {
            continue;
        }
        build_strip(layout, look, s.a, dir, side, t0, t1, d_in, d_out, out);
    }
}

fn endpoint_trim(p: Vec2, dir: Vec2, streets: &[MetroStreet]) -> f32 {
    streets
        .iter()
        .filter(|other| {
            let odir = (other.b - other.a).normalize_or_zero();
            odir.dot(dir).abs() < 0.5
                && (other.a.distance(p) < ENDPOINT_EPS
                    || other.b.distance(p) < ENDPOINT_EPS
                    || other.contains_point(p))
        })
        .map(|other| other.width * 0.5)
        .fold(0.0_f32, f32::max)
}

#[allow(clippy::too_many_arguments)]
fn build_strip(
    layout: &MetroLayout,
    look: &MetroLook,
    origin: Vec2,
    dir: Vec2,
    side: Vec2,
    t0: f32,
    t1: f32,
    d_in: f32,
    d_out: f32,
    out: &mut Vec<CurbTri>,
) {
    let ramp = look.curb_width.min((t1 - t0) * 0.5);
    let tf0 = t0 + ramp;
    let tf1 = t1 - ramp;
    let y0 = layout.site.ground_y;
    let y1 = y0 + look.curb_height;

    let top_color = look.sidewalk;
    let face_color = shade(look.sidewalk, CURB_FACE_SHADE);
    let seam_color = shade(look.sidewalk, KERB_SEAM_SHADE);
    let kerb_top_color = shade(look.sidewalk, KERB_TOP_SHADE);

    let pt = |t: f32, d: f32, y: f32| {
        let w = layout.site.to_world(origin + dir * t + side * d);
        Vec3::new(w.x, y, w.y)
    };
    let n_in = world_dir(layout, -side);
    let n_out = world_dir(layout, side);

    // Flat top surface, split into short segments so chunk filtering stays tight.
    let mut t = tf0;
    while t < tf1 - ENDPOINT_EPS {
        let tn = (t + TOP_SEGMENT_M).min(tf1);
        let q = [pt(t, d_in, y1), pt(tn, d_in, y1), pt(tn, d_out, y1), pt(t, d_out, y1)];
        emit_quad(out, q, Vec3::Y, top_color);
        t = tn;
    }

    // Thin kerb top stone along the asphalt edge.
    let kerb_d = (d_in + KERB_TOP_WIDTH).min(d_out);
    let mut tk = tf0;
    while tk < tf1 - ENDPOINT_EPS {
        let tn = (tk + TOP_SEGMENT_M).min(tf1);
        let yk = y1 + CURB_TOP_OFFSET_Y;
        let q = [pt(tk, d_in, yk), pt(tn, d_in, yk), pt(tn, kerb_d, yk), pt(tk, kerb_d, yk)];
        emit_quad(out, q, Vec3::Y, kerb_top_color);
        tk = tn;
    }

    // Vertical curb face on the asphalt side with kerb stone seam lines every `kerbStoneLength`.
    let step = look.kerb_stone_length.max(KERB_SEAM_WIDTH * 2.0);
    let seam_w = KERB_SEAM_WIDTH.min(step * 0.25);
    let mut ts = tf0;
    while ts < tf1 - ENDPOINT_EPS {
        let te = (ts + step).min(tf1);
        let stone_end = (te - seam_w).max(ts);
        if stone_end > ts + 1e-4 {
            let q = [
                pt(ts, d_in, y0),
                pt(stone_end, d_in, y0),
                pt(stone_end, d_in, y1),
                pt(ts, d_in, y1),
            ];
            emit_quad(out, q, n_in, face_color);
        }
        if te > stone_end + 1e-4 {
            let q = [
                pt(stone_end, d_in, y0),
                pt(te, d_in, y0),
                pt(te, d_in, y1),
                pt(stone_end, d_in, y1),
            ];
            emit_quad(out, q, n_in, seam_color);
        }
        ts = te;
    }

    // Start ramp (t0 -> tf0): slopes up from y0 to y1.
    if ramp > ENDPOINT_EPS {
        let q = [pt(t0, d_in, y0), pt(tf0, d_in, y1), pt(tf0, d_out, y1), pt(t0, d_out, y0)];
        let n_ramp = (q[1] - q[0]).cross(q[3] - q[0]).normalize_or(Vec3::Y);
        let n_ramp = if n_ramp.y < 0.0 { -n_ramp } else { n_ramp };
        emit_quad(out, q, n_ramp, top_color);
        emit_tri(out, pt(t0, d_in, y0), pt(tf0, d_in, y0), pt(tf0, d_in, y1), n_in, face_color);
        emit_tri(out, pt(t0, d_out, y0), pt(tf0, d_out, y1), pt(tf0, d_out, y0), n_out, face_color);
    }

    // End ramp (tf1 -> t1): slopes down from y1 to y0.
    if ramp > ENDPOINT_EPS {
        let q = [pt(tf1, d_in, y1), pt(t1, d_in, y0), pt(t1, d_out, y0), pt(tf1, d_out, y1)];
        let n_ramp = (q[1] - q[0]).cross(q[3] - q[0]).normalize_or(Vec3::Y);
        let n_ramp = if n_ramp.y < 0.0 { -n_ramp } else { n_ramp };
        emit_quad(out, q, n_ramp, top_color);
        emit_tri(out, pt(tf1, d_in, y0), pt(t1, d_in, y0), pt(tf1, d_in, y1), n_in, face_color);
        emit_tri(out, pt(tf1, d_out, y0), pt(tf1, d_out, y1), pt(t1, d_out, y0), n_out, face_color);
    }
}

fn world_dir(layout: &MetroLayout, local_dir: Vec2) -> Vec3 {
    let o = layout.site.to_world(Vec2::ZERO);
    let p = layout.site.to_world(local_dir);
    Vec3::new(p.x - o.x, 0.0, p.y - o.y).normalize_or_zero()
}

fn shade(c: [u8; 3], factor: f32) -> [u8; 3] {
    const MAX_CH: f32 = 255.0; // new: DECISIONS 2026-11-26
    [
        (f32::from(c[0]) * factor).round().clamp(0.0, MAX_CH) as u8,
        (f32::from(c[1]) * factor).round().clamp(0.0, MAX_CH) as u8,
        (f32::from(c[2]) * factor).round().clamp(0.0, MAX_CH) as u8,
    ]
}

fn emit_quad(out: &mut Vec<CurbTri>, q: [Vec3; 4], n: Vec3, color: [u8; 3]) {
    emit_tri(out, q[0], q[1], q[2], n, color);
    emit_tri(out, q[0], q[2], q[3], n, color);
}

fn emit_tri(out: &mut Vec<CurbTri>, a: Vec3, b: Vec3, c: Vec3, normal: Vec3, color: [u8; 3]) {
    let flipped = (b - a).cross(c - a).dot(normal) > 0.0;
    let positions = if flipped { [a, c, b] } else { [a, b, c] };
    out.push(CurbTri { positions, normal, color });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metro::config::MetroConfig;
    use crate::metro::place::MetroSite;

    fn load_cfg() -> MetroConfig {
        MetroConfig::load_from_file(paths::repo_path("data/metro.json")).expect("metro.json loads")
    }

    fn load_look() -> MetroLook {
        MetroLook::load_from_file(paths::repo_path("data/metro.json")).expect("metro.json look loads")
    }

    fn test_site(half_size: f32, ground_y: f32) -> MetroSite {
        MetroSite {
            key: "test".into(),
            name: "Test".into(),
            center: Vec2::ZERO,
            half_size,
            yaw: 0.0,
            ground_y,
        }
    }

    fn city_layout(cfg: &MetroConfig) -> MetroLayout {
        let half = cfg.size_blocks as f32 * cfg.block_size * 0.5;
        MetroLayout::generate(test_site(half, 20.0), cfg)
    }

    fn single_street_layout(a: Vec2, b: Vec2, width: f32) -> MetroLayout {
        let mut l = city_layout(&load_cfg());
        l.site = test_site(64.0, 10.0);
        for b in &mut l.blocks {
            b.reserved = None;
        }
        l.streets = vec![MetroStreet { a, b, width, avenue: false }];
        l
    }

    #[test]
    fn straight_street_gives_two_strips_with_matching_heights_and_edges() {
        let look = load_look();
        let l = single_street_layout(Vec2::new(-32.0, 0.0), Vec2::new(32.0, 0.0), 12.0);
        let tris = build_curbs(&l, &look);
        assert!(!tris.is_empty());

        let pos_side = tris.iter().any(|t| t.positions.iter().all(|p| p.z > 0.0));
        let neg_side = tris.iter().any(|t| t.positions.iter().all(|p| p.z < 0.0));
        assert!(pos_side && neg_side, "both sides of the street have strips");

        let y0 = l.site.ground_y;
        let y1 = y0 + look.curb_height;
        let max_y = tris.iter().flat_map(|t| t.positions).map(|p| p.y).fold(f32::MIN, f32::max);
        let min_y = tris.iter().flat_map(|t| t.positions).map(|p| p.y).fold(f32::MAX, f32::min);
        assert!((max_y - (y1 + CURB_TOP_OFFSET_Y)).abs() < 1e-4);
        assert!((min_y - y0).abs() < 1e-4);

        let d_in = 6.0 - look.sidewalk_width;
        let d_out = 6.0;
        for t in &tris {
            for p in t.positions {
                assert!(p.x >= -32.0 - 1e-3 && p.x <= 32.0 + 1e-3);
                assert!(p.z.abs() >= d_in - 1e-3 && p.z.abs() <= d_out + 1e-3);
            }
        }
    }

    #[test]
    fn corner_stops_at_intersection_with_ramps() {
        let look = load_look();
        let mut l = single_street_layout(Vec2::ZERO, Vec2::new(64.0, 0.0), 12.0);
        l.streets.push(MetroStreet { a: Vec2::ZERO, b: Vec2::new(0.0, 64.0), width: 12.0, avenue: false });
        let tris = build_curbs(&l, &look);
        assert!(!tris.is_empty());

        // Nothing enters the 12x12 intersection square around (0, 0).
        for t in &tris {
            let c = (t.positions[0] + t.positions[1] + t.positions[2]) / 3.0;
            assert!(!(c.x.abs() < 6.0 - 1e-3 && c.z.abs() < 6.0 - 1e-3), "inside intersection: {c:?}");
        }
        // Ramps exist with sloped normals (0 < normal.y < 1).
        assert!(tris.iter().any(|t| t.normal.y > 0.01 && t.normal.y < 0.99), "ramps present");
    }

    #[test]
    fn park_block_gets_no_sidewalk() {
        let cfg = load_cfg();
        let look = load_look();
        let l = city_layout(&cfg);
        let park = l.blocks.iter().find(|b| b.reserved == Some(Reserved::Park)).expect("park exists");
        let tris = build_curbs(&l, &look);

        // No triangle's centroid lies inside a park block's outer curb probe zone.
        let (pmin, pmax) = park.rect;
        for t in &tris {
            let c = (t.positions[0] + t.positions[1] + t.positions[2]) / 3.0;
            let on_park_edge = c.x > pmin.x - look.sidewalk_width
                && c.x < pmax.x + look.sidewalk_width
                && c.z > pmin.y - look.sidewalk_width
                && c.z < pmax.y + look.sidewalk_width
                && l.block_at_point(Vec2::new(c.x, c.z)).is_some_and(|b| b.reserved == Some(Reserved::Park));
            assert!(!on_park_edge, "curb triangle inside park: {c:?}");
        }
    }

    #[test]
    fn every_triangle_is_in_street_band_clockwise_and_deterministic() {
        let cfg = load_cfg();
        let look = load_look();
        let l = city_layout(&cfg);
        let a = build_curbs(&l, &look);
        let b = build_curbs(&l, &look);
        assert_eq!(a, b, "deterministic curb output");
        assert!(!a.is_empty());

        for t in &a {
            let [p0, p1, p2] = t.positions;
            assert!((p1 - p0).cross(p2 - p0).dot(t.normal) <= 1e-5, "clockwise winding");
            let c = (p0 + p1 + p2) / 3.0;
            let c2 = Vec2::new(c.x, c.z);
            let in_band = l.streets.iter().any(|s| {
                if !s.contains_point(c2) {
                    return false;
                }
                let dist = s.closest_point(c2).distance(c2);
                dist >= s.width * 0.5 - look.sidewalk_width - 1e-2 && dist <= s.width * 0.5 + 1e-2
            });
            assert!(in_band, "triangle centroid outside street sidewalk band: {c2:?}");
        }
    }
}
