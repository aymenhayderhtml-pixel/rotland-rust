//! Walkable brick fire escapes: deterministic tower selection and drop/roof ladders. Pure logic.

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use glam::{Vec2, Vec3};

use crate::metro::climb::{ClimbLadder, climb_towers, roof_deck_y};
use crate::metro::layout::MetroDistrict;
use crate::metro::place::MetroSite;
use crate::metro::state::MetroWorld;
use crate::metro::styles::{StyleDef, TowerStyle, default_styles, style_for};
use crate::metro::towers::TowerSpot;
use crate::random::DotnetRandom;

const FNV_OFFSET: u32 = 2_166_136_261; // new: DECISIONS 2026-11-27
const FNV_PRIME: u32 = 16_777_619; // new: DECISIONS 2026-11-27
const DEFAULT_FLOOR_HEIGHT: f32 = 3.8; // data: metro.json floorHeight
const DEFAULT_BASE_HEIGHT: f32 = 7.6; // data: metro.json baseHeight

// Brick fire escape geometry from `tools/blender/towers/brick.py` & `brick_parts.py`:
// Front face is +Y in Blender = -Z in game (`lz = -half`).
const FE_U_16: f32 = -2.0; // new: DECISIONS 2026-11-27 (brick.py:92,135 fe_u for size 16)
const FE_U_24: f32 = -4.0; // new: DECISIONS 2026-11-27 (brick.py:92,135 fe_u for size 24)
const DROP_LADDER_X_OFF: f32 = -2.075; // new: DECISIONS 2026-11-27 (brick.py:137-138 mid of -2.3..-1.85)
const DROP_FOOT_OUT: f32 = 1.15; // new: DECISIONS 2026-11-27 (brick_parts.py:97 landing depth 1.15 m)
const DROP_DECK_OUT: f32 = 0.55; // new: DECISIONS 2026-11-27 (mid-depth of first landing)
const FIRST_LANDING_Y: f32 = 3.92; // new: DECISIONS 2026-11-27 (brick.py:136 z0=3.85 + 0.07 slab)
const ROOF_LADDER_OUT: f32 = 0.6; // new: DECISIONS 2026-11-27 (mid-depth of top landing)
const ROOF_DECK_INSET: f32 = 1.0; // new: DECISIONS 2026-11-27 (1.0 m inside parapet on deck)

/// Picks `fireEscapes.count` brick towers in the configured districts, spread by `minSpacingBlocks`.
/// Deterministic per site and tower list; memoised per thread.
pub fn fire_escape_towers(metro: &MetroWorld) -> Vec<usize> {
    fire_escape_towers_cached(metro).as_ref().clone()
}

fn fire_escape_towers_cached(metro: &MetroWorld) -> Rc<Vec<usize>> {
    struct Memo {
        center: Vec2,
        yaw: f32,
        tower_count: usize,
        picks: Rc<Vec<usize>>,
    }
    thread_local! {
        static CACHE: RefCell<Option<Memo>> = const { RefCell::new(None) };
    }
    let site = &metro.layout.site;
    let count = metro.towers.len();
    CACHE.with(|cell| {
        if let Some(m) = cell.borrow().as_ref()
            && m.center == site.center
            && m.yaw == site.yaw
            && m.tower_count == count
        {
            return Rc::clone(&m.picks);
        }
        let picks = Rc::new(compute_fire_escape_towers(metro));
        *cell.borrow_mut() = Some(Memo {
            center: site.center,
            yaw: site.yaw,
            tower_count: count,
            picks: Rc::clone(&picks),
        });
        picks
    })
}

fn compute_fire_escape_towers(metro: &MetroWorld) -> Vec<usize> {
    let cfg = &metro.config.fire_escapes;
    if cfg.count == 0 {
        return Vec::new();
    }
    let climb_set: HashSet<usize> = climb_towers(metro).into_iter().map(|c| c.tower).collect();
    let allowed_districts: Vec<MetroDistrict> = cfg
        .districts
        .iter()
        .filter_map(|d| match d.as_str() {
            "edge" => Some(MetroDistrict::Edge),
            "midtown" => Some(MetroDistrict::Midtown),
            "core" => Some(MetroDistrict::Core),
            _ => None,
        })
        .collect();

    let mut candidates: Vec<usize> = metro
        .towers
        .iter()
        .enumerate()
        .filter(|(idx, t)| {
            t.style == TowerStyle::Brick
                && !t.landmark
                && t.floors <= cfg.max_floors
                && !climb_set.contains(idx)
                && !metro.damage.tower_is_replaced(*idx)
                && !metro.damage.tower_is_burnt(t.block)
                && metro
                    .layout
                    .block_at(t.block.0, t.block.1)
                    .is_some_and(|b| b.reserved.is_none() && allowed_districts.contains(&b.district))
        })
        .map(|(idx, _)| idx)
        .collect();

    let mut rng = DotnetRandom::new(mix_seed(&metro.layout.site, b"fire_escapes"));
    for i in (1..candidates.len()).rev() {
        let j = rng.next((i + 1) as i32) as usize;
        candidates.swap(i, j);
    }

    let mut picked = Vec::with_capacity(cfg.count);
    for idx in candidates {
        let b = metro.towers[idx].block;
        let spaced = picked.iter().all(|&other: &usize| {
            let ob = metro.towers[other].block;
            (b.0 - ob.0).abs().max((b.1 - ob.1).abs()) >= cfg.min_spacing_blocks
        });
        if spaced {
            picked.push(idx);
            if picked.len() == cfg.count {
                break;
            }
        }
    }
    picked
}

/// Returns the drop ladder (street -> first landing) and roof ladder (top landing -> deck)
/// for brick tower `idx` in world space.
pub fn fire_escape_ladders(metro: &MetroWorld, idx: usize) -> Vec<ClimbLadder> {
    let Some(t) = metro.towers.get(idx) else {
        return Vec::new();
    };
    if t.style != TowerStyle::Brick || t.landmark {
        return Vec::new();
    }
    let styles = default_styles();
    let Some(style) = style_for(styles, t.style) else {
        return Vec::new();
    };
    let ground_y = metro.layout.site.ground_y;
    let h = t.half.x;
    let fe_u = if t.size == 16 { FE_U_16 } else { FE_U_24 }; // data: metro.json sizes

    let drop_x = fe_u + DROP_LADDER_X_OFF;
    let drop = ClimbLadder {
        foot: to_world(t, drop_x, ground_y, -(h + DROP_FOOT_OUT)),
        deck: to_world(t, drop_x, ground_y + FIRST_LANDING_Y, -(h + DROP_DECK_OUT)),
        id: format!("metro_fe_{idx}_drop"),
    };

    let crown_y = crown_base_y(t, style, ground_y);
    let deck_y = roof_deck_y(t, style, ground_y);
    let roof = ClimbLadder {
        foot: to_world(t, fe_u, crown_y, -(h + ROOF_LADDER_OUT)),
        deck: to_world(t, fe_u, deck_y, -(h - ROOF_DECK_INSET)),
        id: format!("metro_fe_{idx}_roof"),
    };

    vec![drop, roof]
}

fn crown_base_y(t: &TowerSpot, style: &StyleDef, ground_y: f32) -> f32 {
    let fh = if style.floor_height > 0.0 {
        style.floor_height
    } else {
        DEFAULT_FLOOR_HEIGHT
    };
    let bh = if style.base_height > 0.0 {
        style.base_height
    } else {
        DEFAULT_BASE_HEIGHT
    };
    ground_y + bh + t.floors as f32 * fh
}

fn to_world(t: &TowerSpot, lx: f32, y: f32, lz: f32) -> Vec3 {
    let (sin_y, cos_y) = t.yaw.sin_cos();
    Vec3::new(
        t.pos.x + lx * cos_y + lz * sin_y,
        y,
        t.pos.y - lx * sin_y + lz * cos_y,
    )
}

fn mix_seed(site: &MetroSite, salt: &[u8]) -> i32 {
    let mut hash = FNV_OFFSET;
    let mut feed = |bytes: &[u8]| {
        for &b in bytes {
            hash ^= u32::from(b);
            hash = hash.wrapping_mul(FNV_PRIME);
        }
    };
    feed(site.key.as_bytes());
    feed(&site.center.x.to_bits().to_le_bytes());
    feed(&site.center.y.to_bits().to_le_bytes());
    feed(salt);
    hash as i32
}
