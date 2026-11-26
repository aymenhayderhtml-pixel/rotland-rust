//! Street, lot, tower facade vine, and roof bush/tree placement for `overgrowth.rs`.

use std::f32::consts::{FRAC_PI_2, PI, TAU};

use glam::{Vec2, Vec3};

use crate::metro::layout::{MetroBlock, MetroStreet};
use crate::metro::overgrowth::{DistrictOvergrowth, HALF, OvergrowthConfig, PROP_SCALE, lerp_f32};
use crate::metro::place::MetroSite;
use crate::metro::state::MetroWorld;
use crate::metro::styles::{StyleDef, style_for};
use crate::metro::tower_parts::{PartKind, tower_parts};
use crate::metro::towers::TowerSpot;
use crate::random::DotnetRandom;
use crate::scatter::PropInstance;

const EDGE_MATCH_EPS: f32 = 0.05; // new: DECISIONS 2026-11-26
const ENDPOINT_EPS: f32 = 0.01; // new: DECISIONS 2026-11-26
const TREE_END_MARGIN_M: f32 = 1.0; // new: DECISIONS 2026-11-26
const MOSS_END_MARGIN_M: f32 = 0.5; // new: DECISIONS 2026-11-26
const LOT_RETRY_FACTOR: usize = 6; // new: DECISIONS 2026-11-26 (bounded rejection tries per lot prop)
/// Baked vine sheet heights in metres (R40-02).
const VINE_VARIANTS: [(&str, f32); 3] = [ // new: DECISIONS 2026-11-26
    ("vine_wall_s", 4.0),  // new: DECISIONS 2026-11-26
    ("vine_wall_m", 8.0),  // new: DECISIONS 2026-11-26
    ("vine_wall_l", 14.0), // new: DECISIONS 2026-11-26
];
/// Non-front face offsets relative to `TowerSpot.yaw` (back, right, left; front at 0 is lobby).
const NON_FRONT_FACES: [f32; 3] = [PI, -FRAC_PI_2, FRAC_PI_2]; // new: DECISIONS 2026-11-26

struct BlockEdge<'a> {
    street: &'a MetroStreet,
    dir: Vec2,
    into_block: Vec2,
    t0: f32,
    t1: f32,
}

pub(super) fn place_street_overgrowth(
    metro: &MetroWorld,
    block: &MetroBlock,
    cfg: &OvergrowthConfig,
    entrance_avenues: &[usize],
    height: &dyn Fn(f32, f32) -> f32,
    rng: &mut DotnetRandom,
    out: &mut Vec<PropInstance>,
) {
    let edges = block_edges(metro, block, entrance_avenues);
    if edges.is_empty() {
        return;
    }
    let district = cfg.for_district(block.district);
    let sw = metro.look.sidewalk_width;
    let weed_margin = cfg.crack_weed_length_m * HALF;
    for _ in 0..district.crack_weeds_per_block {
        let e = &edges[rng.next(edges.len() as i32) as usize];
        if e.t1 - e.t0 <= weed_margin * 2.0 {
            continue;
        }
        let t = lerp_f32(e.t0 + weed_margin, e.t1 - weed_margin, rng.next_double() as f32);
        let d = (e.street.width * HALF - sw - cfg.crack_weed_curb_offset_m).max(0.0);
        let local = e.street.a + e.dir * t + e.into_block * d;
        if on_excluded_street(metro, local, entrance_avenues) {
            continue;
        }
        let wp = metro.layout.site.to_world(local);
        let yaw = local_dir_yaw(&metro.layout.site, e.dir) + FRAC_PI_2;
        out.push(PropInstance::new("weeds_crack", Vec3::new(wp.x, height(wp.x, wp.y), wp.y), yaw, PROP_SCALE));
    }
    for _ in 0..district.moss_per_block {
        let e = &edges[rng.next(edges.len() as i32) as usize];
        if e.t1 - e.t0 <= MOSS_END_MARGIN_M * 2.0 {
            continue;
        }
        let t = lerp_f32(e.t0 + MOSS_END_MARGIN_M, e.t1 - MOSS_END_MARGIN_M, rng.next_double() as f32);
        let d = (e.street.width * HALF - sw).max(0.0);
        let local = e.street.a + e.dir * t + e.into_block * d;
        if on_excluded_street(metro, local, entrance_avenues) {
            continue;
        }
        let wp = metro.layout.site.to_world(local);
        out.push(PropInstance::new("moss_patch", Vec3::new(wp.x, height(wp.x, wp.y), wp.y), rng.next_double() as f32 * TAU, PROP_SCALE));
    }
    for e in &edges {
        let (lo, hi) = (e.t0 + TREE_END_MARGIN_M, e.t1 - TREE_END_MARGIN_M);
        if hi <= lo {
            continue;
        }
        let d_in = e.street.width * HALF - sw;
        let slots = ((hi - lo) / cfg.street_tree_spacing_m).floor().max(1.0) as usize;
        let step = (hi - lo) / slots as f32;
        let mut last_t = f32::NEG_INFINITY;
        for s in 0..slots {
            if rng.next_double() >= f64::from(district.street_tree_chance) {
                continue;
            }
            let t = lerp_f32(lo + s as f32 * step, lo + (s + 1) as f32 * step, rng.next_double() as f32);
            if t - last_t < cfg.street_tree_spacing_m {
                continue;
            }
            let off = lerp_f32(cfg.street_tree_curb_offset_m[0], cfg.street_tree_curb_offset_m[1], rng.next_double() as f32);
            let local = e.street.a + e.dir * t + e.into_block * (d_in - off).max(0.0);
            if on_excluded_street(metro, local, entrance_avenues) {
                continue;
            }
            let wp = metro.layout.site.to_world(local);
            out.push(PropInstance::new("tree_asphalt", Vec3::new(wp.x, height(wp.x, wp.y), wp.y), rng.next_double() as f32 * TAU, PROP_SCALE));
            last_t = t;
        }
    }
}

fn block_edges<'a>(metro: &'a MetroWorld, block: &MetroBlock, excluded: &[usize]) -> Vec<BlockEdge<'a>> {
    let (bmin, bmax) = block.rect;
    let mut edges = Vec::new();
    for (idx, s) in metro.layout.streets.iter().enumerate() {
        let len = s.b.distance(s.a);
        if excluded.contains(&idx) || len <= ENDPOINT_EPS {
            continue;
        }
        let dir = (s.b - s.a) / len;
        let into = if dir.x.abs() > dir.y.abs()
            && (s.a.x - bmin.x).abs() < EDGE_MATCH_EPS && (s.b.x - bmax.x).abs() < EDGE_MATCH_EPS
        {
            if (s.a.y - bmin.y).abs() < EDGE_MATCH_EPS { Some(Vec2::Y) }
            else if (s.a.y - bmax.y).abs() < EDGE_MATCH_EPS { Some(-Vec2::Y) }
            else { None }
        } else if dir.y.abs() >= dir.x.abs()
            && (s.a.y - bmin.y).abs() < EDGE_MATCH_EPS && (s.b.y - bmax.y).abs() < EDGE_MATCH_EPS
        {
            if (s.a.x - bmin.x).abs() < EDGE_MATCH_EPS { Some(Vec2::X) }
            else if (s.a.x - bmax.x).abs() < EDGE_MATCH_EPS { Some(-Vec2::X) }
            else { None }
        } else {
            None
        };
        let Some(into_block) = into else { continue };
        let t0 = endpoint_trim(s.a, dir, &metro.layout.streets);
        let t1 = len - endpoint_trim(s.b, dir, &metro.layout.streets);
        if t1 > t0 + ENDPOINT_EPS {
            edges.push(BlockEdge { street: s, dir, into_block, t0, t1 });
        }
    }
    edges
}

fn endpoint_trim(p: Vec2, dir: Vec2, streets: &[MetroStreet]) -> f32 {
    streets
        .iter()
        .filter(|o| {
            (o.b - o.a).normalize_or_zero().dot(dir).abs() < HALF
                && (o.a.distance(p) < ENDPOINT_EPS || o.b.distance(p) < ENDPOINT_EPS || o.contains_point(p))
        })
        .map(|o| o.width * HALF)
        .fold(0.0_f32, f32::max)
}

fn local_dir_yaw(site: &MetroSite, local_dir: Vec2) -> f32 {
    let w = site.to_world(local_dir) - site.to_world(Vec2::ZERO);
    (-w.x).atan2(-w.y)
}

pub(super) fn place_lot_overgrowth(
    metro: &MetroWorld,
    block: &MetroBlock,
    cfg: &OvergrowthConfig,
    height: &dyn Fn(f32, f32) -> f32,
    rng: &mut DotnetRandom,
    out: &mut Vec<PropInstance>,
) {
    let every = metro.config.avenue_every;
    let w = |line: i32| if line % every == 0 { metro.config.avenue_width } else { metro.config.street_width };
    let lo = block.rect.0 + Vec2::new(w(block.ix), w(block.iz)) * HALF;
    let hi = block.rect.1 - Vec2::new(w(block.ix + 1), w(block.iz + 1)) * HALF;
    if hi.x <= lo.x || hi.y <= lo.y {
        return;
    }
    let district = cfg.for_district(block.district);
    let towers = metro.towers_by_block.get(&(block.ix, block.iz)).map(Vec::as_slice).unwrap_or(&[]);
    let groups = [
        ("shrub_wild", district.lot_shrubs_per_block),
        ("sapling_city", district.lot_trees_per_block),
        ("grass_tall_patch", district.lot_grass_per_block),
    ];
    for (model, want) in groups {
        let mut placed = 0;
        for _ in 0..(want * LOT_RETRY_FACTOR) {
            if placed >= want {
                break;
            }
            let lp = Vec2::new(
                lerp_f32(lo.x, hi.x, rng.next_double() as f32),
                lerp_f32(lo.y, hi.y, rng.next_double() as f32),
            );
            let wp = metro.layout.site.to_world(lp);
            if in_any_tower_clearance(metro, towers, wp, cfg.lot_tower_clearance_m) {
                continue;
            }
            out.push(PropInstance::new(model, Vec3::new(wp.x, height(wp.x, wp.y), wp.y), rng.next_double() as f32 * TAU, PROP_SCALE));
            placed += 1;
        }
    }
}

fn in_any_tower_clearance(metro: &MetroWorld, indices: &[usize], p: Vec2, clearance: f32) -> bool {
    indices.iter().map(|&i| &metro.towers[i]).any(|t| {
        let d = p - t.pos;
        let (s, c) = t.yaw.sin_cos();
        let local = Vec2::new(d.x * c - d.y * s, d.x * s + d.y * c);
        local.x.abs() <= t.half.x + clearance && local.y.abs() <= t.half.y + clearance
    })
}

pub(super) fn place_tower_overgrowth(
    metro: &MetroWorld,
    block: &MetroBlock,
    cfg: &OvergrowthConfig,
    styles: &[StyleDef],
    height: &dyn Fn(f32, f32) -> f32,
    rng: &mut DotnetRandom,
    out: &mut Vec<PropInstance>,
) {
    let Some(indices) = metro.towers_by_block.get(&(block.ix, block.iz)) else {
        return;
    };
    let district = cfg.for_district(block.district);
    for &idx in indices {
        let spot = &metro.towers[idx];
        let Some(style) = style_for(styles, spot.style) else { continue };
        let ground_y = height(spot.pos.x, spot.pos.y);
        let fh = if style.floor_height > 0.0 { style.floor_height } else { metro.config.towers.floor_height };
        let parts = tower_parts(spot, style, ground_y, 0);
        let Some(last_floor_top) = parts.iter().rfind(|p| p.kind == PartKind::Floor).map(|p| p.pos.y + fh) else {
            continue;
        };
        place_wall_vines(spot, last_floor_top - ground_y, district, cfg, height, rng, out);
        place_roof_prop(spot, last_floor_top, district, cfg, rng, out);
    }
}

fn place_wall_vines(
    spot: &TowerSpot,
    facade_height: f32,
    district: &DistrictOvergrowth,
    cfg: &OvergrowthConfig,
    height: &dyn Fn(f32, f32) -> f32,
    rng: &mut DotnetRandom,
    out: &mut Vec<PropInstance>,
) {
    if rng.next_double() >= f64::from(district.vine_tower_chance) {
        return;
    }
    let fitting: Vec<(&str, f32)> = VINE_VARIANTS.iter().copied().filter(|&(_, h)| h <= facade_height).collect();
    if fitting.is_empty() {
        return;
    }
    let face_count = 1 + rng.next(2) as usize;
    let mut pool = NON_FRONT_FACES.to_vec();
    let half_span = (spot.half.x - cfg.vine_corner_margin_m).max(0.0);
    for _ in 0..face_count {
        if pool.is_empty() {
            break;
        }
        let face_yaw = spot.yaw + pool.swap_remove(rng.next(pool.len() as i32) as usize);
        let normal = Vec2::new(-face_yaw.sin(), -face_yaw.cos());
        let tangent = Vec2::new(-normal.y, normal.x);
        let slots = if half_span > 0.0 { ((2.0 * half_span) / cfg.vine_spacing_m).floor().max(1.0) as usize } else { 1 };
        let step = (2.0 * half_span) / slots as f32;
        for s in 0..slots {
            let u = lerp_f32(-half_span + s as f32 * step, -half_span + (s + 1) as f32 * step, rng.next_double() as f32);
            let (model, _) = fitting[rng.next(fitting.len() as i32) as usize];
            let standoff = lerp_f32(cfg.vine_wall_standoff_m[0], cfg.vine_wall_standoff_m[1], rng.next_double() as f32);
            let xz = spot.pos + tangent * u + normal * (spot.half.x + cfg.vine_half_depth_m + standoff);
            out.push(PropInstance::new(model, Vec3::new(xz.x, height(xz.x, xz.y), xz.y), face_yaw, PROP_SCALE));
        }
    }
}

fn place_roof_prop(
    spot: &TowerSpot,
    roof_y: f32,
    district: &DistrictOvergrowth,
    cfg: &OvergrowthConfig,
    rng: &mut DotnetRandom,
    out: &mut Vec<PropInstance>,
) {
    if spot.landmark || rng.next_double() >= f64::from(district.roof_bush_chance) {
        return;
    }
    let rx = (spot.half.x - cfg.roof_edge_inset_m).max(0.0);
    let rz = (spot.half.y - cfg.roof_edge_inset_m).max(0.0);
    let lx = (rng.next_double() as f32 * 2.0 - 1.0) * rx;
    let lz = (rng.next_double() as f32 * 2.0 - 1.0) * rz;
    let (s, c) = spot.yaw.sin_cos();
    let wx = spot.pos.x + lx * c + lz * s;
    let wz = spot.pos.y - lx * s + lz * c;
    let model = if rng.next_double() < f64::from(cfg.roof_tree_chance) { "roof_tree" } else { "roof_bush" };
    out.push(PropInstance::new(model, Vec3::new(wx, roof_y, wz), rng.next_double() as f32 * TAU, PROP_SCALE));
}

fn on_excluded_street(metro: &MetroWorld, local: Vec2, excluded: &[usize]) -> bool {
    excluded.iter().any(|&idx| metro.layout.streets[idx].contains_point(local))
}
