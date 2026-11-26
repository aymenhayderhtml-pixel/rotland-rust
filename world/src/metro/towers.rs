//! New Halden tower spots: packed styled lots, 1 to `maxLotsPerBlock` per block.
//!
//! Every lot is a snapped square footprint (`16`, `24`, or `32` m) inside its block's usable
//! rect (the block minus half a street or avenue on each edge, plus `lotGap`), so no tower stands
//! on a street. Pure logic.

use std::f32::consts::FRAC_PI_2;

use glam::Vec2;

use crate::metro::config::{DistrictTowers, MetroConfig};
use crate::metro::layout::{MetroBlock, MetroDistrict, MetroLayout, Reserved};
use crate::metro::place::MetroSite;
use crate::metro::styles::{StyleDef, TowerStyle, default_styles, style_for};
use crate::random::DotnetRandom;

/// Kind of a tower spot: its district decides the kind, landmarks are their own kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TowerKind {
    Lowrise,
    Midrise,
    Highrise,
    Landmark,
}

/// One tower lot: a snapped square footprint with a style, floor count and height.
#[derive(Debug, Clone, PartialEq)]
pub struct TowerSpot {
    /// World-space centre on the ground.
    pub pos: Vec2,
    /// Local half extent of the square footprint (`size / 2` on both axes).
    pub half: Vec2,
    /// World yaw in radians (site yaw + quarter turn pointing the lobby front at the nearest street).
    pub yaw: f32,
    /// Height in metres (`base_height + floors * floor_height + crown_height` from the style;
    /// `towers.floorHeight` is used only when a style does not give one).
    pub height: f32,
    pub floors: u32,
    pub kind: TowerKind,
    pub landmark: bool,
    /// Grid cell `(ix, iz)` of the block the tower stands in.
    pub block: (i32, i32),
    /// Architectural style of the tower.
    pub style: TowerStyle,
    /// Snapped square module width in metres (`16`, `24`, or `32`).
    pub size: u32,
}

/// Share of the spare room in a cell that a lot may slide sideways (keeps lots off the cell edges).
const JITTER_SHARE: f32 = 0.5; // new: DECISIONS 2026-11-26 (lot offset share of spare room)
/// FNV-1a 32-bit offset basis and prime (same scheme as the layout and town seeds).
const FNV_OFFSET: u32 = 2_166_136_261; // new: DECISIONS 2026-11-26
const FNV_PRIME: u32 = 16_777_619; // new: DECISIONS 2026-11-26
/// Minimum block distance between two landmarks (keeps them spread over the Core).
const LANDMARK_SPACING_BLOCKS: i32 = 4; // new: DECISIONS 2026-11-26 (landmark spread)

/// Plans every tower of the city using `data/metro.json` styles. Same site, same vector.
pub fn plan_towers(layout: &MetroLayout, cfg: &MetroConfig) -> Vec<TowerSpot> {
    plan_towers_with_styles(layout, cfg, default_styles())
}

/// Plans every tower of the city using an explicit slice of style definitions.
pub fn plan_towers_with_styles(
    layout: &MetroLayout,
    cfg: &MetroConfig,
    styles: &[StyleDef],
) -> Vec<TowerSpot> {
    let landmarks = pick_landmarks(layout, cfg);
    let mut out = Vec::new();
    for block in &layout.blocks {
        let key = (block.ix, block.iz);
        match block.reserved {
            Some(Reserved::SafeHouseTower) => {
                let mut rng = DotnetRandom::new(mix_seed(&layout.site, Some(key)));
                out.extend(single_tower(layout, block, cfg, styles, &mut rng, TowerSet::SafeHouse));
            }
            Some(_) => {}
            None if landmarks.contains(&key) => {
                let mut rng = DotnetRandom::new(mix_seed(&layout.site, Some(key)));
                out.extend(single_tower(layout, block, cfg, styles, &mut rng, TowerSet::Landmark));
            }
            None => plan_block(layout, block, cfg, styles, &mut out),
        }
    }
    out
}

#[derive(Clone, Copy)]
enum TowerSet {
    SafeHouse,
    Landmark,
}

struct Lot<'a> {
    center: Vec2,
    size: u32,
    floors: u32,
    kind: TowerKind,
    yaw: f32,
    style: &'a StyleDef,
}

/// Packs 1..=`maxLotsPerBlock` lots into one ordinary block (nothing if it cannot fit a lot).
fn plan_block(
    layout: &MetroLayout,
    block: &MetroBlock,
    cfg: &MetroConfig,
    styles: &[StyleDef],
    out: &mut Vec<TowerSpot>,
) {
    let Some((lo, hi)) = usable_rect(block, cfg) else { return };
    let candidates: Vec<&StyleDef> =
        styles.iter().filter(|s| s.districts.contains(&block.district)).collect();
    if candidates.is_empty() {
        return;
    }
    let fp = district_towers(cfg, block.district).footprint;
    let d_floors = district_towers(cfg, block.district).floors;
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
        let style = candidates[rng.next(candidates.len() as i32) as usize];
        let raw_side = (fp[0] + (fp[1] - fp[0]) * rng.next_double() as f32).min(cell.min_element());
        let jitter = Vec2::new(rng.next_double() as f32, rng.next_double() as f32) - Vec2::splat(0.5);
        let floors = pick_floors(clamp_floors(d_floors, style.floors), &mut rng);
        let Some(snapped) = snap_size(raw_side, &style.sizes) else { continue };
        let (c, r) = ((lot % cols) as f32, (lot / cols) as f32);
        let origin = lo + Vec2::new(c * (cell.x + gap), r * (cell.y + gap));
        let center = origin + cell * 0.5 + jitter * (cell - Vec2::splat(snapped as f32)) * JITTER_SHARE;
        out.push(to_spot(layout, block, cfg, Lot {
            center,
            size: snapped,
            floors,
            kind: kind_for(block.district),
            yaw: front_yaw(layout, center),
            style,
        }));
    }
}

/// One tower per block: the safe-house tower (district footprint and floors) or a landmark.
fn single_tower(
    layout: &MetroLayout,
    block: &MetroBlock,
    cfg: &MetroConfig,
    styles: &[StyleDef],
    rng: &mut DotnetRandom,
    set: TowerSet,
) -> Option<TowerSpot> {
    let (lo, hi) = usable_rect(block, cfg)?;
    let (fp, floors, kind, style) = match set {
        TowerSet::SafeHouse => {
            let candidates: Vec<&StyleDef> =
                styles.iter().filter(|s| s.districts.contains(&block.district)).collect();
            let style = *candidates.get(rng.next(candidates.len().max(1) as i32) as usize)?;
            let d = district_towers(cfg, block.district);
            (d.footprint, pick_floors(clamp_floors(d.floors, style.floors), rng), kind_for(block.district), style)
        }
        TowerSet::Landmark => {
            let style = style_for(styles, TowerStyle::Deco)?;
            (
                cfg.towers.core.footprint,
                pick_floors(cfg.towers.landmark_floors, rng),
                TowerKind::Landmark,
                style,
            )
        }
    };
    let size = hi - lo;
    let side = fp[1].min(size.min_element());
    if side < fp[0] {
        return None;
    }
    let snapped = snap_size(side, &style.sizes)?;
    let center = (lo + hi) * 0.5;
    Some(to_spot(layout, block, cfg, Lot {
        center,
        size: snapped,
        floors,
        kind,
        yaw: front_yaw(layout, center),
        style,
    }))
}

/// Snaps `side` down to the largest allowed module size `<= side`.
fn snap_size(side: f32, allowed: &[u32]) -> Option<u32> {
    allowed.iter().copied().filter(|&s| (s as f32) <= side).max()
}

/// Clamps a district floor range to a style's floor range.
fn clamp_floors(district: [u32; 2], style: [u32; 2]) -> [u32; 2] {
    let lo = district[0].max(style[0]);
    let hi = district[1].min(style[1]);
    if lo <= hi { [lo, hi] } else { style }
}

/// Quarter-turn yaw (`site.yaw + k * PI/2`) that points Blender +Y (glTF -Z) at the nearest street.
fn front_yaw(layout: &MetroLayout, center: Vec2) -> f32 {
    let mut best_dist_sq = f32::INFINITY;
    let mut to_street = Vec2::new(0.0, -1.0);
    for s in &layout.streets {
        let p = s.closest_point(center);
        let d_sq = center.distance_squared(p);
        if d_sq < best_dist_sq {
            best_dist_sq = d_sq;
            to_street = p - center;
        }
    }
    const LOCAL_FRONTS: [Vec2; 4] = [
        Vec2::new(0.0, -1.0),
        Vec2::new(-1.0, 0.0),
        Vec2::new(0.0, 1.0),
        Vec2::new(1.0, 0.0),
    ];
    let best_k = (0..4)
        .max_by(|&a, &b| LOCAL_FRONTS[a].dot(to_street).total_cmp(&LOCAL_FRONTS[b].dot(to_street)))
        .unwrap_or(0);
    layout.site.yaw + best_k as f32 * FRAC_PI_2
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

fn cell_grid(count: usize, size: Vec2) -> (usize, usize) {
    match count {
        1 => (1, 1),
        2 if size.x >= size.y => (2, 1),
        2 => (1, 2),
        _ => (2, 2),
    }
}

fn grid_fits(count: usize, size: Vec2, gap: f32, min_side: f32) -> bool {
    let (cols, rows) = cell_grid(count, size);
    let cell_x = (size.x - (cols as f32 - 1.0) * gap) / cols as f32;
    let cell_y = (size.y - (rows as f32 - 1.0) * gap) / rows as f32;
    cell_x.min(cell_y) >= min_side
}

fn to_spot(layout: &MetroLayout, block: &MetroBlock, cfg: &MetroConfig, lot: Lot<'_>) -> TowerSpot {
    let fh = if lot.style.floor_height > 0.0 { lot.style.floor_height } else { cfg.towers.floor_height };
    let height = lot.style.base_height + lot.floors as f32 * fh + lot.style.crown_height;
    TowerSpot {
        pos: layout.site.to_world(lot.center),
        half: Vec2::splat(lot.size as f32 * 0.5),
        yaw: lot.yaw,
        height,
        floors: lot.floors,
        kind: lot.kind,
        landmark: lot.kind == TowerKind::Landmark,
        block: (block.ix, block.iz),
        style: lot.style.key,
        size: lot.size,
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
