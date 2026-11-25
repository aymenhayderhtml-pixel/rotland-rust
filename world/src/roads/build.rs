use std::f32::consts::PI;

use glam::Vec2;

use crate::biome::BiomeId;
use crate::biome::layout::WorldLayout;
use crate::noise::GodotNoise;
use crate::roads::{Road, RoadKind, RoadsData};
use crate::towns::TownSite;

// new: DECISIONS 2026-11-25
const ROAD_WIGGLE_SEED_OFFSET: i32 = 307;
// new: DECISIONS 2026-11-25
const ROAD_NOISE_SALT_STEP: f32 = 137.0;
// new: DECISIONS 2026-11-25
const FULL_CIRCLE_DEG: f32 = 360.0;
// new: DECISIONS 2026-11-25
const HALF_FACTOR: f32 = 0.5;
// new: DECISIONS 2026-11-25
const QUAD_ARCH_SCALE: f32 = 4.0;
// new: DECISIONS 2026-11-25
const ARC_SUBDIVISIONS: usize = 128;
// new: DECISIONS 2026-11-25
const CORE_CLEARANCE_BUFFER_M: f32 = 1.0;

pub(crate) fn build_network(
    data: &RoadsData,
    towns: &[TownSite],
    layout: &WorldLayout,
    seed: i32,
    origin: Vec2,
) -> (Vec<Vec2>, Vec<Road>) {
    let wiggle_noise = GodotNoise::simplex(
        seed.wrapping_add(ROAD_WIGGLE_SEED_OFFSET),
        data.wiggle_frequency,
    );
    // data: world_layout.json core_radius, blend_band
    let core_outer_edge = layout.data().core_radius + layout.data().blend_band;
    let min_clearance_radius = core_outer_edge + CORE_CLEARANCE_BUFFER_M;

    let sector_trailheads = build_trailheads(data, layout, origin);
    let trailheads: Vec<Vec2> = sector_trailheads.iter().map(|(_, pos)| *pos).collect();

    let mut roads = Vec::new();
    let mut road_index = 0usize;

    for (biome, trailhead) in &sector_trailheads {
        let Some(town) = towns.iter().find(|t| t.biome == *biome) else {
            continue;
        };
        let base_points = [*trailhead, town.center];
        let points = resample_and_wiggle(
            &base_points,
            data,
            &wiggle_noise,
            road_index,
            origin,
            min_clearance_radius,
        );
        road_index += 1;
        roads.push(Road {
            kind: RoadKind::Highway,
            points,
            name: format!("Route to {}", town.name),
        });
    }

    let ring_roads = build_ring_trails(
        data,
        towns,
        layout,
        &wiggle_noise,
        road_index,
        origin,
        min_clearance_radius,
    );
    roads.extend(ring_roads);

    (trailheads, roads)
}

fn build_trailheads(
    data: &RoadsData,
    layout: &WorldLayout,
    origin: Vec2,
) -> Vec<(BiomeId, Vec2)> {
    let mut sectors: Vec<(BiomeId, f32)> = layout
        .data()
        .sectors
        .iter()
        .map(|s| (s.biome, s.center_deg.rem_euclid(FULL_CIRCLE_DEG)))
        .collect();
    sectors.sort_by(|a, b| a.1.total_cmp(&b.1));

    // data: world_layout.json core_radius, blend_band; data: roads.json trailhead_margin
    let radius = layout.data().core_radius + layout.data().blend_band + data.trailhead_margin;
    sectors
        .into_iter()
        .map(|(biome, deg)| {
            let dir = layout.direction_at_deg(deg);
            (biome, origin + dir * radius)
        })
        .collect()
}

fn build_ring_trails(
    data: &RoadsData,
    towns: &[TownSite],
    layout: &WorldLayout,
    wiggle_noise: &GodotNoise,
    start_road_index: usize,
    origin: Vec2,
    min_clearance_radius: f32,
) -> Vec<Road> {
    if towns.len() < 2 {
        return Vec::new();
    }

    let ref_dir = layout.reference_dir();
    let mut ordered: Vec<(&TownSite, f32)> = towns
        .iter()
        .map(|town| (town, angle_ccw_deg(town.center - origin, ref_dir)))
        .collect();
    ordered.sort_by(|a, b| a.1.total_cmp(&b.1));

    // data: roads.json wiggle_m, trail_width
    let safe_arc_radius = min_clearance_radius + data.wiggle_m + data.trail_width;
    let pair_count = if ordered.len() == 2 { 1 } else { ordered.len() };
    let mut ring_roads = Vec::with_capacity(pair_count);

    for i in 0..pair_count {
        let (town_a, deg_a) = ordered[i];
        let (town_b, deg_b) = ordered[(i + 1) % ordered.len()];
        let base_arc = sample_polar_arc(
            town_a.center,
            deg_a,
            town_b.center,
            deg_b,
            layout,
            origin,
            data.ring_radius_from_towns,
            safe_arc_radius,
        );
        let points = resample_and_wiggle(
            &base_arc,
            data,
            wiggle_noise,
            start_road_index + i,
            origin,
            min_clearance_radius,
        );
        ring_roads.push(Road {
            kind: RoadKind::Trail,
            points,
            name: format!("{} - {} Trail", town_a.name, town_b.name),
        });
    }

    ring_roads
}

#[allow(clippy::too_many_arguments)]
fn sample_polar_arc(
    start: Vec2,
    deg_a: f32,
    end: Vec2,
    deg_b: f32,
    layout: &WorldLayout,
    origin: Vec2,
    ring_radius_from_towns: f32,
    safe_arc_radius: f32,
) -> Vec<Vec2> {
    let r_a = start.distance(origin);
    let r_b = end.distance(origin);
    let delta_deg = (deg_b - deg_a).rem_euclid(FULL_CIRCLE_DEG);
    // data: roads.json ring_radius_from_towns
    let target_mid_radius = (r_a + r_b) * HALF_FACTOR * ring_radius_from_towns;

    let mut base_points = Vec::with_capacity(ARC_SUBDIVISIONS + 1);
    base_points.push(start);

    for step in 1..ARC_SUBDIVISIONS {
        let t = step as f32 / ARC_SUBDIVISIONS as f32;
        let deg = deg_a + delta_deg * t;
        let linear_r = r_a + (r_b - r_a) * t;
        let arch_weight = QUAD_ARCH_SCALE * t * (1.0 - t);
        let desired_r = linear_r + arch_weight * (target_mid_radius - linear_r);
        let r = desired_r.max(safe_arc_radius);
        let dir = layout.direction_at_deg(deg);
        base_points.push(origin + dir * r);
    }

    base_points.push(end);
    base_points
}

fn resample_and_wiggle(
    base_points: &[Vec2],
    data: &RoadsData,
    wiggle_noise: &GodotNoise,
    road_index: usize,
    origin: Vec2,
    min_clearance_radius: f32,
) -> Vec<Vec2> {
    let Some(&start) = base_points.first() else {
        return Vec::new();
    };
    let Some(&end) = base_points.last() else {
        return Vec::new();
    };
    if base_points.len() < 2 {
        return vec![start];
    }

    let mut cumulative = Vec::with_capacity(base_points.len());
    cumulative.push(0.0_f32);
    for pair in base_points.windows(2) {
        let prev = *cumulative.last().unwrap_or(&0.0);
        cumulative.push(prev + pair[0].distance(pair[1]));
    }

    let total_len = *cumulative.last().unwrap_or(&0.0);
    // data: roads.json point_spacing
    let segment_count = ((total_len / data.point_spacing).round() as usize).max(1);
    let road_salt = road_index as f32 * ROAD_NOISE_SALT_STEP;

    let mut points = Vec::with_capacity(segment_count + 1);
    points.push(start);

    for i in 1..segment_count {
        let t = i as f32 / segment_count as f32;
        let target_dist = total_len * t;
        let (base_pt, tangent) = sample_polyline_at_distance(base_points, &cumulative, target_dist);
        let normal = Vec2::new(-tangent.y, tangent.x);

        let envelope = (PI * t).sin();
        let noise_val = wiggle_noise.get_noise_2d(base_pt.x + road_salt, base_pt.y - road_salt);
        // data: roads.json wiggle_m
        let lateral_offset = noise_val * data.wiggle_m * envelope;
        let mut displaced = base_pt + normal * lateral_offset;

        let radial = displaced - origin;
        let radial_len = radial.length();
        if base_pt.distance(origin) >= min_clearance_radius
            && radial_len < min_clearance_radius
            && radial_len > f32::EPSILON
        {
            displaced = origin + (radial / radial_len) * min_clearance_radius;
        }

        points.push(displaced);
    }

    points.push(end);
    points
}

fn sample_polyline_at_distance(
    points: &[Vec2],
    cumulative: &[f32],
    target_dist: f32,
) -> (Vec2, Vec2) {
    let idx = match cumulative.binary_search_by(|d| d.total_cmp(&target_dist)) {
        Ok(i) => i.min(points.len().saturating_sub(2)),
        Err(i) => i.saturating_sub(1).min(points.len().saturating_sub(2)),
    };

    let a = points[idx];
    let b = points[idx + 1];
    let seg_start = cumulative[idx];
    let seg_len = cumulative[idx + 1] - seg_start;

    if seg_len <= f32::EPSILON {
        return (a, Vec2::X);
    }

    let local_t = ((target_dist - seg_start) / seg_len).clamp(0.0, 1.0);
    let pos = a.lerp(b, local_t);
    let tangent = (b - a) / seg_len;
    (pos, tangent)
}

fn angle_ccw_deg(delta: Vec2, reference_dir: Vec2) -> f32 {
    if delta.length_squared() <= f32::EPSILON {
        return 0.0;
    }
    let forward = delta.dot(reference_dir);
    let ccw_left = delta.x * reference_dir.y - delta.y * reference_dir.x;
    ccw_left
        .atan2(forward)
        .to_degrees()
        .rem_euclid(FULL_CIRCLE_DEG)
}
