//! Silent life crow perches: picks the anchor a crow flock lands on in a block.
//! Pure logic: no GPU.

use std::f32::consts::TAU;

use glam::{Vec2, Vec3};

use crate::metro::layout::MetroBlock;
use crate::metro::life::PerchKind;
use crate::metro::life_billboards::{is_low_billboard_tower, roof_top_y};
use crate::metro::life_helpers::{BlockKey, HALF, dir_yaw, in_chunk};
use crate::metro::life_wires::{WIRE_ATTACH_HEIGHT_M, WIRE_SAG_M, WireSpan};
use crate::metro::state::MetroWorld;
use crate::metro::styles::{StyleDef, style_for};
use crate::random::DotnetRandom;
use crate::scatter::PropInstance;

const CAR_ROOF_Y_M: f32 = 1.5; // new: DECISIONS 2026-11-27
const ROOF_EDGE_INSET_M: f32 = 1.5; // new: DECISIONS 2026-11-27

#[allow(clippy::too_many_arguments)]
pub(super) fn pick_perch_anchor(
    metro: &MetroWorld,
    block: &MetroBlock,
    spans: &[WireSpan],
    trace_props: &[PropInstance],
    styles: &[StyleDef],
    chunk: (i32, i32),
    height: &dyn Fn(f32, f32) -> f32,
    rng: &mut DotnetRandom,
) -> Option<(PerchKind, Vec3, f32)> {
    let mut candidates: Vec<(PerchKind, Vec3, f32)> = Vec::new();
    let b_min = metro.layout.site.to_world(block.rect.0);
    let b_max = metro.layout.site.to_world(block.rect.1);
    let b_center = (b_min + b_max) * HALF;

    if let Some(indices) = metro.towers_by_block.get(&(block.ix, block.iz)) {
        for &idx in indices {
            if metro.damage.tower_is_replaced(idx) || metro.damage.tower_is_burnt(block.block_key()) {
                continue;
            }
            let t = &metro.towers[idx];
            if is_low_billboard_tower(t)
                && let Some(style) = style_for(styles, t.style)
                && let Some(ry) = roof_top_y(t, style, height(t.pos.x, t.pos.y), metro.config.towers.floor_height)
            {
                let edge = t.pos + Vec2::new(-t.yaw.sin(), -t.yaw.cos()) * (t.half.x - ROOF_EDGE_INSET_M).max(0.0);
                candidates.push((PerchKind::Roof, Vec3::new(edge.x, ry, edge.y), t.yaw));
            }
        }
    }
    for sp in spans {
        let mid = (sp.a + sp.b) * HALF;
        if in_chunk(mid.x, mid.y, chunk) {
            let y = sp.y_base + WIRE_ATTACH_HEIGHT_M - WIRE_SAG_M;
            candidates.push((PerchKind::Wire, Vec3::new(mid.x, y, mid.y), dir_yaw((sp.b - sp.a).normalize_or_zero())));
        }
    }
    for j in &metro.jam {
        if (j.model.starts_with("car_") || j.model.starts_with("bus_") || j.model.starts_with("truck_"))
            && in_chunk(j.pos.x, j.pos.z, chunk)
            && Vec2::new(j.pos.x, j.pos.z).distance(b_center) < metro.config.block_size
        {
            let y = height(j.pos.x, j.pos.z) + CAR_ROOF_Y_M;
            candidates.push((PerchKind::Car, Vec3::new(j.pos.x, y, j.pos.z), j.yaw));
            break;
        }
    }
    for p in trace_props {
        // Only bodies out on the street: crows don't go into lobbies.
        if p.model.starts_with("body_sheet")
            && in_chunk(p.position.x, p.position.z, chunk)
            && !metro.in_tower_footprint(Vec2::new(p.position.x, p.position.z))
        {
            // new: DECISIONS 2026-11-27 (small perch offset beside body)
            candidates.push((PerchKind::Body, p.position + Vec3::new(0.4, 0.0, 0.3), p.yaw));
            break;
        }
    }
    let street_p = block_sidewalk_point(metro, block);
    candidates.push((
        PerchKind::Street,
        Vec3::new(street_p.x, height(street_p.x, street_p.y) + metro.look.curb_height, street_p.y),
        rng.next_double() as f32 * TAU,
    ));

    let pick = rng.next(candidates.len() as i32) as usize;
    Some(candidates.swap_remove(pick))
}

fn block_sidewalk_point(metro: &MetroWorld, block: &MetroBlock) -> Vec2 {
    let every = metro.config.avenue_every;
    let sw_w = if block.ix % every == 0 { metro.config.avenue_width } else { metro.config.street_width };
    let local = Vec2::new(block.rect.0.x + sw_w * HALF + 0.5, (block.rect.0.y + block.rect.1.y) * HALF);
    metro.layout.site.to_world(local)
}
