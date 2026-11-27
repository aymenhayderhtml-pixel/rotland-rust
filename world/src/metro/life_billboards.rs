//! Silent life billboards and the roof flags placed on low towers and lots.
//! Pure logic: no GPU.

use glam::Vec3;

use crate::metro::layout::MetroBlock;
use crate::metro::life::PROP_SCALE;
use crate::metro::life_config::LifeConfig;
use crate::metro::life_flags::flag_for_tower;
use crate::metro::life_helpers::{in_chunk, mix_seed, nearest_avenue_yaw, pick_lot_spot};
use crate::metro::state::MetroWorld;
use crate::metro::styles::{StyleDef, TowerStyle, style_for};
use crate::metro::tower_parts::{PartKind, tower_parts};
use crate::metro::towers::TowerSpot;
use crate::random::DotnetRandom;
use crate::scatter::PropInstance;

const ROOF_BILLBOARD_FLOORS: [u32; 2] = [4, 10]; // new: DECISIONS 2026-11-27 (R41-03 low roofs)
const STREET_BILLBOARD_FACE_SCALE: f32 = 0.66; // new: DECISIONS 2026-11-27 (R41-04 street face scale)
const STREET_BILLBOARD_CLEAR_M: f32 = 4.0; // new: DECISIONS 2026-11-27

const BILLBOARD_FACES: [&str; 4] = [
    "billboard_face_a",
    "billboard_face_b",
    "billboard_face_c",
    "billboard_face_d",
];

pub(super) fn place_block_billboards_and_flags(
    metro: &MetroWorld,
    block: &MetroBlock,
    cfg: &LifeConfig,
    styles: &[StyleDef],
    chunk: (i32, i32),
    height: &dyn Fn(f32, f32) -> f32,
    out: &mut Vec<PropInstance>,
) {
    if let Some(indices) = metro.towers_by_block.get(&(block.ix, block.iz)) {
        for &idx in indices {
            if metro.damage.tower_is_replaced(idx) {
                continue;
            }
            let tower = &metro.towers[idx];
            if let Some((pole, _flapper)) = flag_for_tower(metro, idx, tower, cfg, height)
                && in_chunk(pole.position.x, pole.position.z, chunk)
            {
                out.push(pole);
            }
            if is_low_billboard_tower(tower) {
                let mut rng = DotnetRandom::new(mix_seed(&metro.layout.site, Some((idx as i32, block.ix)), b"life_bb_roof"));
                if rng.next_double() < f64::from(cfg.billboard_roof_chance)
                    && let Some(style) = style_for(styles, tower.style)
                    && let Some(roof_y) = roof_top_y(tower, style, height(tower.pos.x, tower.pos.y), metro.config.towers.floor_height)
                    && in_chunk(tower.pos.x, tower.pos.y, chunk)
                {
                    let frame = if rng.next(2) == 0 { "billboard_roof_a" } else { "billboard_roof_b" };
                    let face = BILLBOARD_FACES[rng.next(BILLBOARD_FACES.len() as i32) as usize];
                    let yaw = nearest_avenue_yaw(metro, tower.pos, tower.yaw);
                    let pos = Vec3::new(tower.pos.x, roof_y, tower.pos.y);
                    out.push(PropInstance::new(frame, pos, yaw, PROP_SCALE));
                    out.push(PropInstance::new(face, pos, yaw, PROP_SCALE));
                }
            }
        }
    }

    let mut rng = DotnetRandom::new(mix_seed(&metro.layout.site, Some((block.ix, block.iz)), b"life_bb_street"));
    if rng.next_double() < f64::from(cfg.billboard_street_chance)
        && let Some((wp, yaw)) = pick_lot_spot(metro, block, STREET_BILLBOARD_CLEAR_M, &mut rng)
        && in_chunk(wp.x, wp.y, chunk)
    {
        let face = BILLBOARD_FACES[rng.next(BILLBOARD_FACES.len() as i32) as usize];
        let pos = Vec3::new(wp.x, height(wp.x, wp.y), wp.y);
        out.push(PropInstance::new("billboard_street", pos, yaw, PROP_SCALE));
        out.push(PropInstance::new(face, pos, yaw, STREET_BILLBOARD_FACE_SCALE));
    }
}

pub(super) fn is_low_billboard_tower(tower: &TowerSpot) -> bool {
    !tower.landmark
        && matches!(tower.style, TowerStyle::Brick | TowerStyle::Slab)
        && (ROOF_BILLBOARD_FLOORS[0]..=ROOF_BILLBOARD_FLOORS[1]).contains(&tower.floors)
}

pub(super) fn roof_top_y(spot: &TowerSpot, style: &StyleDef, ground_y: f32, fallback_fh: f32) -> Option<f32> {
    let parts = tower_parts(spot, style, ground_y, 0);
    let last_floor = parts.iter().rfind(|p| p.kind == PartKind::Floor)?;
    let fh = if style.floor_height > 0.0 { style.floor_height } else { fallback_fh };
    Some(last_floor.pos.y + fh + style.roof_deck_height)
}
