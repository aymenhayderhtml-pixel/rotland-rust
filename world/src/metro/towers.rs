//! New Halden tower spots: packed placeholder lots, 1 to `maxLotsPerBlock` per block.
//!
//! Every lot is a square footprint inside its block's usable rect (the block minus half a
//! street or avenue on each edge, plus `lotGap`), so no tower stands on a street. Pure logic.

use std::f32::consts::FRAC_PI_2;

use glam::Vec2;

use crate::metro::config::{DistrictTowers, MetroConfig};
use crate::metro::layout::{MetroBlock, MetroDistrict, MetroLayout, Reserved};
use crate::metro::place::MetroSite;
use crate::random::DotnetRandom;

/// Kind of a tower spot: its district decides the kind, landmarks are their own kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TowerKind {
    Lowrise,
    Midrise,
    Highrise,
    Landmark,
}

/// One placeholder tower: a square footprint with a height in floors.
#[derive(Debug, Clone, PartialEq)]
pub struct TowerSpot {
    /// World-space centre on the ground.
    pub pos: Vec2,
    /// Local half extent of the square footprint (both axes).
    pub half: Vec2,
    /// World yaw in radians (the site yaw, plus a quarter turn on alternate lots).
    pub yaw: f32,
    /// Height in metres (`floors * floorHeight`).
    pub height: f32,
    pub floors: u32,
    pub kind: TowerKind,
    pub landmark: bool,
    /// Grid cell `(ix, iz)` of the block the tower stands in.
    pub block: (i32, i32),
}

/// Share of the spare room in a cell that a lot may slide sideways (keeps lots off the cell edges).
const JITTER_SHARE: f32 = 0.5; // new: DECISIONS 2026-11-26 (lot offset share of spare room)
/// FNV-1a 32-bit offset basis and prime (same scheme as the layout and town seeds).
const FNV_OFFSET: u32 = 2_166_136_261; // new: DECISIONS 2026-11-26
const FNV_PRIME: u32 = 16_777_619; // new: DECISIONS 2026-11-26

/// Plans every tower of the city. Same site, same vector (no clock, no global random).
pub fn plan_towers(layout: &MetroLayout, cfg: &MetroConfig) -> Vec<TowerSpot> {
    let landmarks = pick_landmarks(layout, cfg);
    let mut out = Vec::new();
    for block in &layout.blocks {
        let key = (block.ix, block.iz);
        match block.reserved {
            Some(Reserved::SafeHouseTower) => {
                let mut rng = DotnetRandom::new(mix_seed(&layout.site, Some(key)));
                out.extend(single_tower(layout, block, cfg, &mut rng, TowerSet::SafeHouse));
            }
            Some(_) => {}
            None if landmarks.contains(&key) => {
                let mut rng = DotnetRandom::new(mix_seed(&layout.site, Some(key)));
                out.extend(single_tower(layout, block, cfg, &mut rng, TowerSet::Landmark));
            }
            None => plan_block(layout, block, cfg, &mut out),
        }
    }
    out
}

/// Which single-tower rule a block uses.
#[derive(Clone, Copy)]
enum TowerSet {
    SafeHouse,
    Landmark,
}

/// One lot to turn into a tower: centre (site-local), square side, floors, kind and yaw.
struct Lot {
    center: Vec2,
    side: f32,
    floors: u32,
    kind: TowerKind,
    yaw: f32,
}

/// Packs 1..=`maxLotsPerBlock` lots into one ordinary block (nothing if it cannot fit a lot).
fn plan_block(layout: &MetroLayout, block: &MetroBlock, cfg: &MetroConfig, out: &mut Vec<TowerSpot>) {
    let Some((lo, hi)) = usable_rect(block, cfg) else { return };
    let fp = district_towers(cfg, block.district).footprint;
    let floors = district_towers(cfg, block.district).floors;
    let size = hi - lo;
    if size.min_element() < fp[0] {
        return;
    }
    let mut rng = DotnetRandom::new(mix_seed(&layout.site, Some((block.ix, block.iz))));
    let mut count = 1 + rng.next(cfg.towers.max_lots_per_block as i32) as usize;
    if !grid_fits(count, size, cfg.towers.lot_gap, fp[0]) {
        count = 1;
    }
    let (cols, rows) = cell_grid(count, size);
    let gap = cfg.towers.lot_gap;
    let cell = Vec2::new(
        (size.x - (cols as f32 - 1.0) * gap) / cols as f32,
        (size.y - (rows as f32 - 1.0) * gap) / rows as f32,
    );
    for lot in 0..count {
        let (c, r) = ((lot % cols) as f32, (lot / cols) as f32);
        let side = (fp[0] + (fp[1] - fp[0]) * rng.next_double() as f32).min(cell.min_element());
        let jitter = Vec2::new(rng.next_double() as f32, rng.next_double() as f32) - Vec2::splat(0.5);
        let origin = lo + Vec2::new(c * (cell.x + gap), r * (cell.y + gap));
        let center = origin + cell * 0.5 + jitter * (cell - Vec2::splat(side)) * JITTER_SHARE;
        out.push(to_spot(layout, block, cfg, Lot {
            center,
            side,
            floors: pick_floors(floors, &mut rng),
            kind: kind_for(block.district),
            yaw: layout.site.yaw + if lot % 2 == 1 { FRAC_PI_2 } else { 0.0 },
        }));
    }
}

/// One tower per block: the safe-house tower (district footprint and floors) or a landmark.
fn single_tower(
    layout: &MetroLayout,
    block: &MetroBlock,
    cfg: &MetroConfig,
    rng: &mut DotnetRandom,
    set: TowerSet,
) -> Option<TowerSpot> {
    let (lo, hi) = usable_rect(block, cfg)?;
    let (fp, floors, kind) = match set {
        TowerSet::SafeHouse => {
            let d = district_towers(cfg, block.district);
            (d.footprint, pick_floors(d.floors, rng), kind_for(block.district))
        }
        TowerSet::Landmark => {
            let fp = cfg.towers.core.footprint;
            (fp, pick_floors(cfg.towers.landmark_floors, rng), TowerKind::Landmark)
        }
    };
    let size = hi - lo;
    let side = fp[1].min(size.min_element());
    if side < fp[0] {
        return None;
    }
    Some(to_spot(layout, block, cfg, Lot {
        center: (lo + hi) * 0.5,
        side,
        floors,
        kind,
        yaw: layout.site.yaw,
    }))
}

/// Picks the landmark blocks: 3..=5 distinct, non-reserved Core blocks, spread apart.
fn pick_landmarks(layout: &MetroLayout, cfg: &MetroConfig) -> Vec<(i32, i32)> {
    let min_fit = cfg.towers.core.footprint[0];
    let mut candidates: Vec<(i32, i32)> = layout
        .blocks
        .iter()
        .filter(|b| b.district == MetroDistrict::Core && b.reserved.is_none())
        .filter(|b| usable_rect(b, cfg).is_some_and(|(lo, hi)| (hi - lo).min_element() >= min_fit))
        .map(|b| (b.ix, b.iz))
        .collect();
    let [lo_n, hi_n] = cfg.towers.landmark_count;
    let mut rng = DotnetRandom::new(mix_seed(&layout.site, None));
    let want = rng.next_range(lo_n as i32, hi_n as i32 + 1) as usize;
    let mut picked: Vec<(i32, i32)> = Vec::new();
    while picked.len() < want && !candidates.is_empty() {
        let i = rng.next(candidates.len() as i32) as usize;
        let chosen = candidates.swap_remove(i);
        candidates.retain(|c| chebyshev(*c, chosen) >= LANDMARK_SPACING_BLOCKS);
        picked.push(chosen);
    }
    picked
}

/// Minimum block distance between two landmarks (keeps them spread over the Core).
const LANDMARK_SPACING_BLOCKS: i32 = 4; // new: DECISIONS 2026-11-26 (landmark spread)

fn chebyshev(a: (i32, i32), b: (i32, i32)) -> i32 {
    (a.0 - b.0).abs().max((a.1 - b.1).abs())
}

/// Block rect shrunk by half a street or avenue on each edge plus `lotGap`; None if empty.
pub(crate) fn usable_rect(block: &MetroBlock, cfg: &MetroConfig) -> Option<(Vec2, Vec2)> {
    let every = cfg.avenue_every;
    let width = |line: i32| if line % every == 0 { cfg.avenue_width } else { cfg.street_width };
    let gap = Vec2::splat(cfg.towers.lot_gap);
    let lo_inset = Vec2::new(width(block.ix), width(block.iz)) * 0.5 + gap;
    let hi_inset = Vec2::new(width(block.ix + 1), width(block.iz + 1)) * 0.5 + gap;
    let lo = block.rect.0 + lo_inset;
    let hi = block.rect.1 - hi_inset;
    (hi.x > lo.x && hi.y > lo.y).then_some((lo, hi))
}

/// Cells for `count` lots: one cell, two along the longer side, or a 2x2 grid.
fn cell_grid(count: usize, size: Vec2) -> (usize, usize) {
    match count {
        1 => (1, 1),
        2 if size.x >= size.y => (2, 1),
        2 => (1, 2),
        _ => (2, 2),
    }
}

/// True if the cells for `count` lots are still at least `min_side` wide.
fn grid_fits(count: usize, size: Vec2, gap: f32, min_side: f32) -> bool {
    let (cols, rows) = cell_grid(count, size);
    let cell_x = (size.x - (cols as f32 - 1.0) * gap) / cols as f32;
    let cell_y = (size.y - (rows as f32 - 1.0) * gap) / rows as f32;
    cell_x.min(cell_y) >= min_side
}

fn to_spot(layout: &MetroLayout, block: &MetroBlock, cfg: &MetroConfig, lot: Lot) -> TowerSpot {
    TowerSpot {
        pos: layout.site.to_world(lot.center),
        half: Vec2::splat(lot.side * 0.5),
        yaw: lot.yaw,
        height: lot.floors as f32 * cfg.towers.floor_height,
        floors: lot.floors,
        kind: lot.kind,
        landmark: lot.kind == TowerKind::Landmark,
        block: (block.ix, block.iz),
    }
}

fn pick_floors(range: [u32; 2], rng: &mut DotnetRandom) -> u32 {
    rng.next_range(range[0] as i32, range[1] as i32 + 1) as u32
}

fn district_towers(cfg: &MetroConfig, d: MetroDistrict) -> &DistrictTowers {
    match d {
        MetroDistrict::Edge => &cfg.towers.edge,
        MetroDistrict::Midtown => &cfg.towers.midtown,
        MetroDistrict::Core => &cfg.towers.core,
    }
}

fn kind_for(d: MetroDistrict) -> TowerKind {
    match d {
        MetroDistrict::Edge => TowerKind::Lowrise,
        MetroDistrict::Midtown => TowerKind::Midrise,
        MetroDistrict::Core => TowerKind::Highrise,
    }
}

/// FNV-1a over the site key, centre bits and (for a block) its grid cell.
fn mix_seed(site: &MetroSite, block: Option<(i32, i32)>) -> i32 {
    let mut bytes: Vec<u8> = site.key.bytes().collect();
    bytes.extend(site.center.x.to_bits().to_le_bytes());
    bytes.extend(site.center.y.to_bits().to_le_bytes());
    if let Some((ix, iz)) = block {
        bytes.extend(ix.to_le_bytes());
        bytes.extend(iz.to_le_bytes());
    }
    let mut hash = FNV_OFFSET;
    for b in bytes {
        hash ^= u32::from(b);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash as i32
}
