//! Chunk prop planning and placement for New Halden's major damage set pieces. Pure logic, no GPU.

use std::f32::consts::{FRAC_PI_2, PI};

use glam::{Vec2, Vec3};

use crate::metro::damage_burnt::soot_sheets;
use crate::metro::layout::MetroStreet;
use crate::metro::state::MetroWorld;
use crate::metro::towers::TowerSpot;
use crate::random::DotnetRandom;
use crate::scatter::PropInstance;

use super::{
    CAR_MIN_CENTER_DIST_M, CHUNK_SIZE_M, DamageConfig, FALLEN_MODEL_LENGTH_M, bordering_streets,
    default_damage, mix_seed, x_axis, y_axis, yaw_for_x,
};

const STREET_EDGE_MARGIN: f32 = 0.25; // new: DECISIONS 2026-11-26 (16 m clear of corners)
const ASH_WALL_OFFSET_M: f32 = 1.5; // new: DECISIONS 2026-11-26 (half of lotGap = 3.0 m)
const CURB_CAR_INSET_M: f32 = 0.5; // new: DECISIONS 2026-11-26 (straddles street edge off centre)
const CAR_PILE_YAW_JITTER: f32 = 0.25; // new: DECISIONS 2026-11-26 (slight wreck skew)
const RUBBLE_SPAN_SHARE: f32 = 0.35; // new: DECISIONS 2026-11-26 (keeps mounds on street/curbs)

/// Returns the damage props whose centre falls in `chunk`, placed at `height(x, z)`.
pub fn damage_props_in_chunk(
    metro: &MetroWorld,
    chunk: (i32, i32),
    height: &dyn Fn(f32, f32) -> f32,
) -> Vec<PropInstance> {
    damage_props_in_chunk_with_config(metro, chunk, height, default_damage())
}

pub(crate) fn damage_props_in_chunk_with_config(
    metro: &MetroWorld,
    chunk: (i32, i32),
    height: &dyn Fn(f32, f32) -> f32,
    cfg: &DamageConfig,
) -> Vec<PropInstance> {
    plan_damage_props(metro, cfg)
        .into_iter()
        .filter(|p| in_chunk(p.pos.x, p.pos.y, chunk))
        .map(|p| {
            PropInstance::new(
                p.model,
                Vec3::new(p.pos.x, height(p.pos.x, p.pos.y) + p.y, p.pos.y),
                p.yaw,
                p.scale,
            )
        })
        .collect()
}

/// `y`: height above the terrain at `pos` (0 for ground props).
pub(super) struct DamagePropSpec {
    pub(super) model: &'static str,
    pub(super) pos: Vec2,
    pub(super) yaw: f32,
    pub(super) scale: f32,
    pub(super) y: f32,
}

pub(super) fn plan_damage_props(metro: &MetroWorld, cfg: &DamageConfig) -> Vec<DamagePropSpec> {
    let mut out = Vec::new();
    let fallen = &metro.damage.fallen;
    if metro.layout.block_at(fallen.block.0, fallen.block.1).is_some() {
        out.push(DamagePropSpec { model: "tw_fallen_section", pos: fallen.pos, yaw: fallen.yaw, scale: 1.0, y: 0.0 });
        out.push(DamagePropSpec { model: "rubble_ramp", pos: fallen.pos, yaw: fallen.yaw, scale: 1.0, y: 0.0 });
        let (across, along) = (x_axis(fallen.yaw), y_axis(fallen.yaw));
        let half_len = FALLEN_MODEL_LENGTH_M.clamp(cfg.fallen_length_m[0], cfg.fallen_length_m[1]) * 0.5;
        let half_w = cfg.fallen_width_m * 0.5;
        out.push(DamagePropSpec { model: "rubble_field_l", pos: fallen.pos - across * half_len, yaw: fallen.yaw, scale: 1.0, y: 0.0 });
        out.push(DamagePropSpec { model: "rubble_field_l", pos: fallen.pos + across * half_len, yaw: fallen.yaw + PI, scale: 1.0, y: 0.0 });
        let n = cfg.rubble_fields_per_site;
        for k in 0..n {
            let t = if n <= 1 { 0.0 } else { (k as f32 / (n - 1) as f32) * 2.0 - 1.0 };
            let side = if k % 2 == 0 { 1.0 } else { -1.0 };
            let pos = fallen.pos + across * (t * half_len * RUBBLE_SPAN_SHARE) + along * (side * half_w);
            out.push(DamagePropSpec {
                model: "rubble_field_m",
                pos,
                yaw: fallen.yaw + if side < 0.0 { PI } else { 0.0 },
                scale: 1.0,
                y: 0.0,
            });
        }
    }
    let leaning = &metro.damage.leaning;
    if metro.layout.block_at(leaning.block.0, leaning.block.1).is_some() {
        out.push(DamagePropSpec { model: "tw_leaning_tower", pos: leaning.pos, yaw: leaning.yaw, scale: 1.0, y: 0.0 });
    }
    push_burnt_props(metro, cfg, &mut out);
    out
}

fn push_burnt_props(metro: &MetroWorld, cfg: &DamageConfig, out: &mut Vec<DamagePropSpec>) {
    let bb = metro.damage.burnt_block;
    let Some(block) = metro.layout.block_at(bb.0, bb.1) else { return };
    let mut rng = DotnetRandom::new(mix_seed(&metro.layout.site, b"damage_burnt"));
    if let Some(indices) = metro.towers_by_block.get(&bb) {
        for &idx in indices.iter().filter(|&&i| !metro.damage.tower_is_replaced(i)) {
            let spot = &metro.towers[idx];
            for sheet in soot_sheets(spot) {
                out.push(DamagePropSpec { model: "burnt_floor_overlay", pos: sheet.pos, yaw: sheet.yaw, scale: sheet.scale, y: sheet.y });
            }
            for a in 0..cfg.ash_patches_per_tower {
                out.push(DamagePropSpec { model: "ash_patch", pos: pick_ash_pos(metro, spot, bb, a), yaw: a as f32 * FRAC_PI_2, scale: 1.0, y: 0.0 });
            }
        }
    }
    let count = rng.next_range(cfg.burnt_car_count[0] as i32, cfg.burnt_car_count[1] as i32 + 1) as usize;
    let plain_edges: Vec<&MetroStreet> = bordering_streets(&metro.layout, block).into_iter().filter(|s| !s.avenue).collect();
    if plain_edges.is_empty() || count == 0 {
        return;
    }
    let block_centre = (block.rect.0 + block.rect.1) * 0.5;
    for k in 0..count {
        let s = plain_edges[k % plain_edges.len()];
        let slot = k / plain_edges.len();
        let slots = (count - (k % plain_edges.len())).div_ceil(plain_edges.len());
        let t = STREET_EDGE_MARGIN + (1.0 - 2.0 * STREET_EDGE_MARGIN) * ((slot as f32 + 0.5) / slots as f32);
        let to_block = (block_centre - (s.a + s.b) * 0.5).normalize_or(Vec2::X);
        let offset = (s.width * 0.5 - metro.look.sidewalk_width - CURB_CAR_INSET_M).max(CAR_MIN_CENTER_DIST_M);
        let pos = metro.layout.site.to_world(s.a.lerp(s.b, t) + to_block * offset);
        let dir_w = (metro.layout.site.to_world(s.b) - metro.layout.site.to_world(s.a)).normalize_or(Vec2::X);
        let jitter = (rng.next_double() as f32 * 2.0 - 1.0) * CAR_PILE_YAW_JITTER;
        out.push(DamagePropSpec { model: "car_burnt_pile", pos, yaw: yaw_for_x(dir_w) + jitter, scale: 1.0, y: 0.0 });
    }
}

fn pick_ash_pos(metro: &MetroWorld, spot: &TowerSpot, bb: (i32, i32), index: usize) -> Vec2 {
    let front = y_axis(spot.yaw);
    let dirs = [front, -front, front.perp(), -front.perp()];
    let reach = spot.size as f32 * 0.5 + ASH_WALL_OFFSET_M + (index / 4) as f32;
    for step in 0..4 {
        let p = spot.pos + dirs[(index + step) % 4] * reach;
        if metro.layout.block_at_point(p).is_some_and(|b| (b.ix, b.iz) == bb)
            && metro.layout.street_at(p).is_none()
            && !metro.in_tower_footprint(p)
        {
            return p;
        }
    }
    spot.pos + front * reach
}

fn in_chunk(x: f32, z: f32, chunk: (i32, i32)) -> bool {
    (x / CHUNK_SIZE_M).floor() as i32 == chunk.0 && (z / CHUNK_SIZE_M).floor() as i32 == chunk.1
}
