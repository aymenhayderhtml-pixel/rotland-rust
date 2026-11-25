use glam::Vec2;

use crate::biome::layout::WorldLayout;
use crate::math::lerp;
use crate::noise::GodotNoise;
use crate::rivers::{River, RiverSeedSpec, RiversData};

// new: DECISIONS 2026-11-25
const WIGGLE_SEED_OFFSET: i32 = 409;
// new: DECISIONS 2026-11-25
const HALF: f32 = 0.5;
// new: DECISIONS 2026-11-25
const TANGENT_TIE_EPS: f32 = 1e-3;
// new: DECISIONS 2026-11-25
const STRAIGHT_BIAS: f32 = 1e-4;
// new: DECISIONS 2026-11-25
const STOP_TARGET_STEPS: f32 = 2.0;

/// Traces a single downhill river polyline outside `core_radius + blend_band + core_margin_m`.
pub(crate) fn trace_river(
    spec: &RiverSeedSpec,
    data: &RiversData,
    layout: &WorldLayout,
    seed: i32,
    origin: Vec2,
    base_height: &dyn Fn(f32, f32) -> f32,
) -> River {
    let min_radius = layout.data().core_radius + layout.data().blend_band + data.core_margin_m;
    // Ensure even chord midpoints stay outside `min_radius` when gliding around the core edge.
    let safe_radius = (min_radius * min_radius + (HALF * data.step_m).powi(2)).sqrt();

    let start_dir = layout.direction_at_deg(spec.start_angle_deg);
    let start_radius = spec.start_radius_m.max(safe_radius);
    let start_pos = origin + start_dir * start_radius;

    let target_dir = layout.direction_at_deg(spec.target_angle_deg);
    let target_pos = origin + target_dir * spec.target_radius_m.max(safe_radius);

    let init_vec = (target_pos - start_pos).try_normalize().unwrap_or(start_dir);
    let mut heading = init_vec.y.atan2(init_vec.x);

    let wiggle_noise = GodotNoise::simplex(
        seed.wrapping_add(WIGGLE_SEED_OFFSET),
        data.wiggle_frequency,
    );
    let max_steps = (data.max_length_m / data.step_m).floor().max(1.0) as usize;
    let spread_rad = data.sample_spread_deg.to_radians();
    let max_turn_rad = data.max_turn_deg.to_radians();
    let wiggle_rad = data.wiggle_deg.to_radians();

    let mut points = Vec::with_capacity(max_steps + 1);
    points.push(start_pos);

    let mut pos = start_pos;
    for _ in 0..max_steps {
        let to_target_vec = target_pos - pos;
        if to_target_vec.length() <= data.step_m * STOP_TARGET_STEPS {
            break;
        }
        let to_target = to_target_vec.try_normalize().unwrap_or(Vec2::ZERO);
        let downhill_angle = pick_downhill_heading(
            pos,
            heading,
            to_target,
            data,
            spread_rad,
            base_height,
        );
        let downhill_dir = Vec2::new(downhill_angle.cos(), downhill_angle.sin());
        let pull = data.target_pull.clamp(0.0, 1.0);
        let blended_dir = (downhill_dir * (1.0 - pull) + to_target * pull)
            .try_normalize()
            .unwrap_or(to_target);
        let blended_angle = blended_dir.y.atan2(blended_dir.x);
        let wiggle = wiggle_noise.get_noise_2d(pos.x, pos.y) * wiggle_rad;
        let desired_angle = blended_angle + wiggle;
        let turn = wrap_angle_rad(desired_angle - heading).clamp(-max_turn_rad, max_turn_rad);
        let stepped_angle = heading + turn;
        let step_dir = Vec2::new(stepped_angle.cos(), stepped_angle.sin());
        let candidate = pos + step_dir * data.step_m;

        let next_pos = if (candidate - origin).length() < safe_radius {
            turn_along_core_edge(pos, origin, safe_radius, data.step_m, step_dir, to_target)
        } else {
            candidate
        };

        let delta = next_pos - pos;
        if delta.length_squared() <= f32::EPSILON {
            break;
        }
        heading = delta.y.atan2(delta.x);
        points.push(next_pos);
        pos = next_pos;
    }

    let widths = build_widths(points.len(), data.width_start_m, data.width_end_m);
    let water_levels = build_water_levels(&points, &widths, data.depth_m, base_height);

    River {
        points,
        widths,
        water_levels,
    }
}

/// Returns the smooth unit XZ forward tangent at vertex `idx` of a river polyline.
pub(crate) fn vertex_tangent(points: &[Vec2], idx: usize) -> Vec2 {
    if points.len() < 2 {
        return Vec2::X;
    }
    if idx == 0 {
        return (points[1] - points[0]).try_normalize().unwrap_or(Vec2::X);
    }
    if idx + 1 >= points.len() {
        return (points[points.len() - 1] - points[points.len() - 2])
            .try_normalize()
            .unwrap_or(Vec2::X);
    }
    let prev = (points[idx] - points[idx - 1])
        .try_normalize()
        .unwrap_or(Vec2::X);
    let next = (points[idx + 1] - points[idx])
        .try_normalize()
        .unwrap_or(prev);
    (prev + next).try_normalize().unwrap_or(next)
}

fn pick_downhill_heading(
    pos: Vec2,
    heading: f32,
    to_target: Vec2,
    data: &RiversData,
    spread_rad: f32,
    base_height: &dyn Fn(f32, f32) -> f32,
) -> f32 {
    let count = data.sample_count;
    let mut best_angle = heading;
    let mut best_score = f32::INFINITY;

    for i in 0..count {
        let frac = if count <= 1 {
            0.0
        } else {
            (i as f32 / (count - 1) as f32) * 2.0 - 1.0
        };
        let angle = heading + frac * spread_rad;
        let dir = Vec2::new(angle.cos(), angle.sin());
        let sample_pos = pos + dir * data.step_m;
        let height = base_height(sample_pos.x, sample_pos.y);

        // Gentle pull toward the target sector + tiny straight tie-break on flat terrain.
        let score = height - data.target_pull * dir.dot(to_target) + frac.abs() * STRAIGHT_BIAS;
        if score < best_score {
            best_score = score;
            best_angle = angle;
        }
    }

    best_angle
}

fn wrap_angle_rad(angle: f32) -> f32 {
    (angle + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI
}

fn turn_along_core_edge(
    pos: Vec2,
    origin: Vec2,
    safe_radius: f32,
    step_m: f32,
    desired_dir: Vec2,
    to_target: Vec2,
) -> Vec2 {
    let rel = pos - origin;
    let keep_radius = rel.length().max(safe_radius);
    let radial = rel.try_normalize().unwrap_or(Vec2::X);
    let t_ccw = Vec2::new(-radial.y, radial.x);
    let t_cw = -t_ccw;

    let ccw_dot = t_ccw.dot(desired_dir);
    let cw_dot = t_cw.dot(desired_dir);
    let tangent = if (ccw_dot - cw_dot).abs() > TANGENT_TIE_EPS {
        if ccw_dot >= cw_dot { t_ccw } else { t_cw }
    } else if t_ccw.dot(to_target) >= t_cw.dot(to_target) {
        t_ccw
    } else {
        t_cw
    };

    let raw_next = pos + tangent * step_m;
    let next_radial = (raw_next - origin).try_normalize().unwrap_or(radial);
    origin + next_radial * keep_radius
}

fn build_widths(count: usize, start_m: f32, end_m: f32) -> Vec<f32> {
    let mut widths = Vec::with_capacity(count);
    let denom = count.saturating_sub(1).max(1) as f32;
    for i in 0..count {
        let t = if count <= 1 { 0.0 } else { i as f32 / denom };
        widths.push(lerp(start_m, end_m, t));
    }
    widths
}

fn build_water_levels(
    points: &[Vec2],
    widths: &[f32],
    depth_m: f32,
    base_height: &dyn Fn(f32, f32) -> f32,
) -> Vec<f32> {
    let mut levels = Vec::with_capacity(points.len());
    let mut prev_level = f32::INFINITY;

    for (idx, (&p, &w)) in points.iter().zip(widths.iter()).enumerate() {
        let fwd = vertex_tangent(points, idx);
        let right = Vec2::new(-fwd.y, fwd.x);
        let half_w = w * HALF;
        let left_bank = p - right * half_w;
        let right_bank = p + right * half_w;

        let bank_h = base_height(p.x, p.y)
            .min(base_height(left_bank.x, left_bank.y))
            .min(base_height(right_bank.x, right_bank.y));
        let candidate = bank_h - depth_m;
        let level = prev_level.min(candidate);
        levels.push(level);
        prev_level = level;
    }

    levels
}
