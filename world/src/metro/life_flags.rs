//! Silent life wall flags and lot plastic sheets (wind flappers). Pure logic: no GPU.

use std::f32::consts::{FRAC_1_SQRT_2, TAU};

use glam::{Vec2, Vec3};

use crate::metro::layout::MetroBlock;
use crate::metro::life::{FlapKind, Flapper, PROP_SCALE};
use crate::metro::life_config::LifeConfig;
use crate::metro::life_helpers::{mix_seed, pick_lot_spot};
use crate::metro::state::MetroWorld;
use crate::metro::towers::TowerSpot;
use crate::random::DotnetRandom;
use crate::scatter::PropInstance;

const FLAG_HEIGHT_RANGE_M: [f32; 2] = [4.0, 6.0]; // new: DECISIONS 2026-11-27 (R41-03 4-6 m)
const FLAG_WALL_EMBED_M: f32 = 0.03; // new: DECISIONS 2026-11-27 (2-5 cm into wall)
const FLAG_DOOR_OFFSET_SHARE: f32 = 0.28; // new: DECISIONS 2026-11-27 (clears lobby door)
const FLAG_POLE_LENGTH_M: f32 = 2.5; // new: DECISIONS 2026-11-27 (R41-04 2.5 m pole at 45 deg)
const LOT_FENCE_Y_M: f32 = 1.6; // new: DECISIONS 2026-11-27
const LOT_EDGE_INSET_M: f32 = 2.0; // new: DECISIONS 2026-11-27

pub(super) fn flag_for_tower(
    metro: &MetroWorld,
    idx: usize,
    tower: &TowerSpot,
    cfg: &LifeConfig,
    height: &dyn Fn(f32, f32) -> f32,
) -> Option<(PropInstance, Flapper)> {
    let mut rng = DotnetRandom::new(mix_seed(&metro.layout.site, Some((idx as i32, tower.block.1)), b"life_flag"));
    if rng.next_double() >= f64::from(cfg.flag_chance) {
        return None;
    }
    let f = Vec2::new(-tower.yaw.sin(), -tower.yaw.cos());
    let r = Vec2::new(-f.y, f.x);
    let side = if rng.next(2) == 0 { -1.0_f32 } else { 1.0_f32 };
    let xz = tower.pos + f * (tower.half.x - FLAG_WALL_EMBED_M) + r * (side * FLAG_DOOR_OFFSET_SHARE * tower.size as f32);
    let y_off = FLAG_HEIGHT_RANGE_M[0] + (FLAG_HEIGHT_RANGE_M[1] - FLAG_HEIGHT_RANGE_M[0]) * rng.next_double() as f32;
    let pole_pos = Vec3::new(xz.x, height(xz.x, xz.y) + y_off, xz.y);
    let pole = PropInstance::new("flag_pole_wall", pole_pos, tower.yaw, PROP_SCALE);
    let reach = FLAG_POLE_LENGTH_M * FRAC_1_SQRT_2;
    let tip_pos = pole_pos + Vec3::new(f.x * reach, reach, f.y * reach);
    let flapper = Flapper {
        kind: FlapKind::Flag,
        pos: tip_pos,
        yaw: tower.yaw,
        scale: PROP_SCALE,
        phase: rng.next_double() as f32 * TAU,
    };
    Some((pole, flapper))
}

pub(super) fn lot_plastic_for_block(
    metro: &MetroWorld,
    block: &MetroBlock,
    cfg: &LifeConfig,
    height: &dyn Fn(f32, f32) -> f32,
) -> Option<Flapper> {
    let mut rng = DotnetRandom::new(mix_seed(&metro.layout.site, Some((block.ix, block.iz)), b"life_plastic"));
    if rng.next_double() >= f64::from(cfg.plastic_chance) {
        return None;
    }
    let (wp, yaw) = pick_lot_spot(metro, block, LOT_EDGE_INSET_M, &mut rng)?;
    Some(Flapper {
        kind: FlapKind::Plastic,
        pos: Vec3::new(wp.x, height(wp.x, wp.y) + LOT_FENCE_Y_M, wp.y),
        yaw,
        scale: PROP_SCALE,
        phase: rng.next_double() as f32 * TAU,
    })
}
