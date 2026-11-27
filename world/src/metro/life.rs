//! New Halden silent life placement: power lines, billboards, flags, plastic sheets,
//! and crow perches. Pure logic: no GPU.

use std::f32::consts::TAU;

use glam::{Vec2, Vec3};

use crate::metro::life_billboards::place_block_billboards_and_flags;
use crate::metro::life_flags::{flag_for_tower, lot_plastic_for_block};
use crate::metro::life_perches::pick_perch_anchor;
use crate::metro::life_wires::collect_wires_in_chunk;
use crate::metro::state::MetroWorld;
use crate::metro::styles::default_styles;
use crate::metro::traces::{car_wall_sites, traces_props_in_chunk};
use crate::random::DotnetRandom;
use crate::scatter::PropInstance;

pub use crate::metro::life_config::{CrowPerchChance, LifeConfig, default_life, load_life};
pub(super) use crate::metro::life_helpers::{dir_yaw, in_chunk, mix_seed};
use crate::metro::life_helpers::overlapping_blocks;

pub(super) const CHUNK_SIZE_M: f32 = 64.0; // data: metro.json blockSize
pub(super) const PROP_SCALE: f32 = 1.0; // new: DECISIONS 2026-11-27
const CAR_WALL_TOP_Y_M: f32 = 2.4; // new: DECISIONS 2026-11-27

/// Kind of animated wind flapper (`Flapper`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FlapKind {
    Flag,
    Plastic,
}

/// One animated flag or plastic tarp placed in New Halden.
#[derive(Debug, Clone, PartialEq)]
pub struct Flapper {
    pub kind: FlapKind,
    pub pos: Vec3,
    pub yaw: f32,
    pub scale: f32,
    pub phase: f32,
}

/// Anchor kind for a crow flock perch (`CrowPerch`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PerchKind {
    Roof,
    Wire,
    Car,
    Body,
    Street,
}

/// One crow flock perch in New Halden.
#[derive(Debug, Clone, PartialEq)]
pub struct CrowPerch {
    pub pos: Vec3,
    pub yaw: f32,
    pub kind: PerchKind,
    pub count: u8,
}

/// Static silent-life props (`power_pole*`, `power_wire_*`, `billboard_*`, `flag_pole_wall`)
/// whose XZ position falls inside `chunk`.
pub fn life_props_in_chunk(
    metro: &MetroWorld,
    chunk: (i32, i32),
    height: &dyn Fn(f32, f32) -> f32,
) -> Vec<PropInstance> {
    let cfg = default_life();
    let styles = default_styles();
    let mut out = Vec::new();
    let _spans = collect_wires_in_chunk(metro, cfg, chunk, height, &mut out);
    let blocks = overlapping_blocks(metro, chunk);
    for block in blocks {
        if block.reserved.is_some() || metro.damage.tower_is_burnt((block.ix, block.iz)) {
            continue;
        }
        place_block_billboards_and_flags(metro, block, cfg, styles, chunk, height, &mut out);
    }
    out
}

/// Animated flags and plastic sheets whose XZ position falls inside `chunk`.
pub fn flappers_in_chunk(
    metro: &MetroWorld,
    chunk: (i32, i32),
    height: &dyn Fn(f32, f32) -> f32,
) -> Vec<Flapper> {
    let cfg = default_life();
    let mut out = Vec::new();

    for (idx, (cw_pos, cw_yaw)) in car_wall_sites(metro).into_iter().enumerate() {
        let mut rng = DotnetRandom::new(mix_seed(&metro.layout.site, Some((idx as i32, 0)), b"life_cw_plastic"));
        if rng.next_double() < f64::from(cfg.plastic_chance) {
            let right = Vec2::new(cw_yaw.cos(), -cw_yaw.sin());
            let off = (rng.next_double() as f32 * 2.0 - 1.0) * 3.0;
            let p = cw_pos + right * off;
            if in_chunk(p.x, p.y, chunk) {
                let y = height(p.x, p.y) + CAR_WALL_TOP_Y_M;
                out.push(Flapper {
                    kind: FlapKind::Plastic,
                    pos: Vec3::new(p.x, y, p.y),
                    yaw: cw_yaw,
                    scale: PROP_SCALE,
                    phase: rng.next_double() as f32 * TAU,
                });
            }
        }
    }

    for block in overlapping_blocks(metro, chunk) {
        if block.reserved.is_some() || metro.damage.tower_is_burnt((block.ix, block.iz)) {
            continue;
        }
        if let Some(indices) = metro.towers_by_block.get(&(block.ix, block.iz)) {
            for &idx in indices {
                if metro.damage.tower_is_replaced(idx) {
                    continue;
                }
                if let Some((_pole, flapper)) = flag_for_tower(metro, idx, &metro.towers[idx], cfg, height)
                    && in_chunk(flapper.pos.x, flapper.pos.z, chunk)
                {
                    out.push(flapper);
                }
            }
        }
        if let Some(plastic) = lot_plastic_for_block(metro, block, cfg, height)
            && in_chunk(plastic.pos.x, plastic.pos.z, chunk)
        {
            out.push(plastic);
        }
    }
    out
}

/// Crow flock perches whose XZ position falls inside `chunk`.
pub fn crow_perches_in_chunk(
    metro: &MetroWorld,
    chunk: (i32, i32),
    height: &dyn Fn(f32, f32) -> f32,
) -> Vec<CrowPerch> {
    let cfg = default_life();
    let styles = default_styles();
    let blocks = overlapping_blocks(metro, chunk);
    if blocks.is_empty() {
        return Vec::new();
    }
    let mut dummy_props = Vec::new();
    let spans = collect_wires_in_chunk(metro, cfg, chunk, height, &mut dummy_props);
    let trace_props = traces_props_in_chunk(metro, chunk, height);
    let mut out = Vec::new();

    for block in blocks {
        if block.reserved.is_some() {
            continue;
        }
        let mut rng = DotnetRandom::new(mix_seed(&metro.layout.site, Some((block.ix, block.iz)), b"life_crows"));
        let chance = cfg.crow_perch_chance.for_district(block.district);
        if rng.next_double() >= f64::from(chance) {
            continue;
        }
        let count = rng.next_range(cfg.crows_per_perch[0] as i32, cfg.crows_per_perch[1] as i32 + 1) as u8;
        let Some((kind, pos, yaw)) = pick_perch_anchor(metro, block, &spans, &trace_props, styles, chunk, height, &mut rng) else {
            continue;
        };
        if in_chunk(pos.x, pos.z, chunk) {
            out.push(CrowPerch { pos, yaw, kind, count });
        }
    }
    out
}
