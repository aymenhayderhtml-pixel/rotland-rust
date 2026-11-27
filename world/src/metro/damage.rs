//! New Halden major damage set pieces: one fallen tower across a street, one leaning tower
//! resting toward its neighbour, and one burnt-out block. Pure logic, no GPU.

use std::collections::HashMap;
use std::sync::OnceLock;

use glam::Vec2;
use serde::{Deserialize, Deserializer};

use crate::metro::layout::{MetroBlock, MetroDistrict, MetroLayout, MetroStreet};
use crate::metro::place::MetroSite;
use crate::metro::state::index_towers_by_block;
use crate::metro::towers::TowerSpot;
use crate::random::DotnetRandom;

#[path = "damage_props.rs"]
mod props;
#[cfg(test)]
#[path = "damage_tests.rs"]
mod tests;

pub use props::damage_props_in_chunk;
#[cfg(test)]
use props::damage_props_in_chunk_with_config;
#[cfg(test)]
use crate::metro::damage_burnt::soot_sheets;
#[cfg(test)]
use props::plan_damage_props;

pub(super) const CHUNK_SIZE_M: f32 = 64.0; // new: DECISIONS 2026-11-26 (one chunk = 64 m)
pub(super) const FALLEN_MODEL_LENGTH_M: f32 = 72.0; // new: DECISIONS 2026-11-26 (R40-04 fallen model length)
const FALLEN_STREET_MARGIN: f32 = 0.42; // new: DECISIONS 2026-11-26 (27 m clear of corner avenues)
pub(super) const CAR_MIN_CENTER_DIST_M: f32 = 2.0; // new: DECISIONS 2026-11-26 (off street centreline)
const FNV_OFFSET: u32 = 2_166_136_261; // new: DECISIONS 2026-11-26
const FNV_PRIME: u32 = 16_777_619; // new: DECISIONS 2026-11-26
const EPS: f32 = 1e-3; // new: DECISIONS 2026-11-26
pub(super) const NEIGHBOURS: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

/// Settings for the three damage set pieces (`damage` in `data/metro.json`).
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DamageConfig {
    pub fallen_length_m: [f32; 2],
    pub fallen_width_m: f32,
    #[serde(deserialize_with = "deserialize_district")]
    pub fallen_district: MetroDistrict,
    #[serde(deserialize_with = "deserialize_district")]
    pub leaning_district: MetroDistrict,
    pub leaning_min_size: u32,
    #[serde(deserialize_with = "deserialize_district")]
    pub burnt_district: MetroDistrict,
    pub burnt_min_towers: usize,
    pub burnt_car_count: [usize; 2],
    pub ash_patches_per_tower: usize,
    pub rubble_fields_per_site: usize,
}

impl Default for DamageConfig {
    fn default() -> Self {
        Self {
            fallen_length_m: [60.0, 80.0],           // data: metro.json damage
            fallen_width_m: 24.0,                    // data: metro.json damage
            fallen_district: MetroDistrict::Midtown, // data: metro.json damage
            leaning_district: MetroDistrict::Core,   // data: metro.json damage
            leaning_min_size: 24,                    // data: metro.json damage
            burnt_district: MetroDistrict::Midtown,  // data: metro.json damage
            burnt_min_towers: 2,                     // data: metro.json damage
            burnt_car_count: [3, 5],                 // data: metro.json damage
            ash_patches_per_tower: 1,                // data: metro.json damage
            rubble_fields_per_site: 3,               // data: metro.json damage
        }
    }
}

fn deserialize_district<'de, D: Deserializer<'de>>(d: D) -> Result<MetroDistrict, D::Error> {
    match String::deserialize(d)?.as_str() {
        "edge" => Ok(MetroDistrict::Edge),
        "midtown" => Ok(MetroDistrict::Midtown),
        "core" => Ok(MetroDistrict::Core),
        other => Err(serde::de::Error::custom(format!("unknown district '{other}'"))),
    }
}

#[derive(Deserialize)]
struct DamageWrapper {
    #[serde(default)]
    damage: DamageConfig,
}

/// Parses and validates the top-level `damage` object from a `metro.json` string.
pub fn load_damage(json: &str) -> Result<DamageConfig, String> {
    let wrapper: DamageWrapper =
        serde_json::from_str(json).map_err(|e| format!("metro.json damage: {e}"))?;
    validate_damage(&wrapper.damage)?;
    Ok(wrapper.damage)
}

/// Validates ranges and minimums in a `DamageConfig`.
pub fn validate_damage(cfg: &DamageConfig) -> Result<(), String> {
    if cfg.fallen_length_m[0] <= 0.0 || cfg.fallen_length_m[0] > cfg.fallen_length_m[1] {
        return Err("metro.json damage: fallenLengthM must have 0 < min <= max".into());
    }
    if cfg.fallen_width_m <= 0.0 || cfg.burnt_car_count[0] > cfg.burnt_car_count[1] {
        return Err("metro.json damage: invalid fallenWidthM or burntCarCount".into());
    }
    if cfg.leaning_min_size == 0 || cfg.burnt_min_towers == 0 {
        return Err("metro.json damage: leaningMinSize and burntMinTowers must be >= 1".into());
    }
    Ok(())
}

/// Cached `DamageConfig` loaded from `data/metro.json` (with `#[serde(default)]` fallback).
pub fn default_damage() -> &'static DamageConfig {
    static CACHE: OnceLock<DamageConfig> = OnceLock::new();
    CACHE.get_or_init(|| {
        let path = paths::repo_path("data/metro.json");
        let json = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
        load_damage(&json).expect("data/metro.json damage must be valid")
    })
}

/// Placement of a single damaged-tower site in world XZ.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Site {
    pub block: (i32, i32),
    pub pos: Vec2,
    pub yaw: f32,
}

/// Selected damage locations and replaced tower indices for one city.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DamageSites {
    pub fallen: Site,
    pub leaning: Site,
    pub burnt_block: (i32, i32),
    pub replaced_towers: Vec<usize>,
}

impl DamageSites {
    /// True when `tower_index` is replaced by the fallen or leaning set piece (or crushed by it).
    pub fn tower_is_replaced(&self, tower_index: usize) -> bool {
        self.replaced_towers.contains(&tower_index)
    }

    /// True when `block` `(ix, iz)` is the burnt-out block.
    pub fn tower_is_burnt(&self, block: (i32, i32)) -> bool {
        self.burnt_block == block
    }
}

/// Deterministically picks the fallen tower, leaning tower, burnt block, and replaced towers.
pub fn damage_sites(layout: &MetroLayout, towers: &[TowerSpot], cfg: &DamageConfig) -> DamageSites {
    let by_block = index_towers_by_block(towers);
    let mut rng = DotnetRandom::new(mix_seed(&layout.site, b"damage"));
    let mut replaced_towers = Vec::new();
    let (fallen, fallen_opp) = pick_fallen(layout, towers, &by_block, cfg, &mut rng, &mut replaced_towers);
    let leaning = pick_leaning(layout, towers, &by_block, cfg, &mut rng, [fallen.block, fallen_opp], &mut replaced_towers);
    let burnt_block = pick_burnt(layout, &by_block, cfg, &mut rng, [fallen.block, fallen_opp, leaning.block]);
    DamageSites { fallen, leaning, burnt_block, replaced_towers }
}

fn pick_fallen(
    layout: &MetroLayout, towers: &[TowerSpot], by_block: &HashMap<(i32, i32), Vec<usize>>,
    cfg: &DamageConfig, rng: &mut DotnetRandom, replaced: &mut Vec<usize>,
) -> (Site, (i32, i32)) {
    struct Cand { block_a: (i32, i32), block_b: (i32, i32), a: Vec2, b: Vec2, tower: usize, single: bool }
    let mut cands = Vec::new();
    for ba in layout.blocks.iter().filter(|b| b.district == cfg.fallen_district && b.reserved.is_none()) {
        let Some(list_a) = by_block.get(&(ba.ix, ba.iz)) else { continue };
        for (dx, dz) in NEIGHBOURS {
            let Some(bb) = layout.block_at(ba.ix + dx, ba.iz + dz) else { continue };
            if bb.reserved.is_some() || by_block.get(&(bb.ix, bb.iz)).is_some_and(|lb| lb.iter().any(|&i| towers[i].landmark)) { continue; }
            let Some(s) = street_between(layout, ba, bb).filter(|s| !s.avenue) else { continue };
            let smid = layout.site.to_world((s.a + s.b) * 0.5);
            let Some(&tower) = list_a.iter().filter(|&&i| !towers[i].landmark)
                .min_by(|&&x, &&y| towers[x].pos.distance_squared(smid).total_cmp(&towers[y].pos.distance_squared(smid))) else { continue };
            cands.push(Cand { block_a: (ba.ix, ba.iz), block_b: (bb.ix, bb.iz), a: s.a, b: s.b, tower, single: list_a.len() == 1 });
        }
    }
    let pool: Vec<&Cand> = if cands.iter().any(|c| c.single) { cands.iter().filter(|c| c.single).collect() } else { cands.iter().collect() };
    if pool.is_empty() { return (Site { block: (-1, -1), ..Site::default() }, (-1, -1)); }
    let pick = pool[rng.next(pool.len() as i32) as usize];
    let s_dir = pick.b - pick.a;
    let proj = ((layout.site.to_local(towers[pick.tower].pos) - pick.a).dot(s_dir) / s_dir.length_squared().max(EPS))
        .clamp(FALLEN_STREET_MARGIN, 1.0 - FALLEN_STREET_MARGIN);
    let pos = layout.site.to_world(pick.a + s_dir * proj);
    let across_l = Vec2::new((pick.block_b.0 - pick.block_a.0) as f32, (pick.block_b.1 - pick.block_a.1) as f32);
    let yaw = yaw_for_x((layout.site.to_world(across_l) - layout.site.to_world(Vec2::ZERO)).normalize_or(Vec2::X));
    let half_ext = Vec2::new(FALLEN_MODEL_LENGTH_M.clamp(cfg.fallen_length_m[0], cfg.fallen_length_m[1]) * 0.5, cfg.fallen_width_m * 0.5);
    replaced.push(pick.tower);
    for (i, t) in towers.iter().enumerate() {
        if i != pick.tower && obb_overlaps(pos, yaw, half_ext, t.pos, t.yaw, t.half) { replaced.push(i); }
    }
    (Site { block: pick.block_a, pos, yaw }, pick.block_b)
}

fn pick_leaning(
    layout: &MetroLayout, towers: &[TowerSpot], by_block: &HashMap<(i32, i32), Vec<usize>>,
    cfg: &DamageConfig, rng: &mut DotnetRandom, exclude: [(i32, i32); 2], replaced: &mut Vec<usize>,
) -> Site {
    struct Cand { block: (i32, i32), tower: usize, neighbour: usize, size: u32 }
    let mut cands = Vec::new();
    for bl in layout.blocks.iter().filter(|b| b.district == cfg.leaning_district && b.reserved.is_none() && !exclude.contains(&(b.ix, b.iz))) {
        let Some(list_l) = by_block.get(&(bl.ix, bl.iz)) else { continue };
        for (dx, dz) in NEIGHBOURS {
            let Some(bn) = layout.block_at(bl.ix + dx, bl.iz + dz) else { continue };
            if bn.reserved.is_some() || exclude.contains(&(bn.ix, bn.iz)) || street_between(layout, bl, bn).is_none_or(|s| s.avenue) { continue; }
            let Some(list_n) = by_block.get(&(bn.ix, bn.iz)) else { continue };
            let n_mid = layout.site.to_world((bn.rect.0 + bn.rect.1) * 0.5);
            let Some(&tower) = list_l.iter().filter(|&&i| !towers[i].landmark && towers[i].size >= cfg.leaning_min_size && !replaced.contains(&i))
                .min_by(|&&x, &&y| towers[x].pos.distance_squared(n_mid).total_cmp(&towers[y].pos.distance_squared(n_mid))) else { continue };
            let t_pos = towers[tower].pos;
            let Some(&neighbour) = list_n.iter().min_by(|&&x, &&y| towers[x].pos.distance_squared(t_pos).total_cmp(&towers[y].pos.distance_squared(t_pos))) else { continue };
            cands.push(Cand { block: (bl.ix, bl.iz), tower, neighbour, size: towers[tower].size });
        }
    }
    let max_sz = cands.iter().map(|c| c.size).max().unwrap_or(0);
    let pool: Vec<&Cand> = cands.iter().filter(|c| c.size == max_sz).collect();
    if pool.is_empty() { return Site { block: (-1, -1), ..Site::default() }; }
    let pick = pool[rng.next(pool.len() as i32) as usize];
    replaced.push(pick.tower);
    let t_pos = towers[pick.tower].pos;
    Site { block: pick.block, pos: t_pos, yaw: yaw_for_x((towers[pick.neighbour].pos - t_pos).normalize_or(Vec2::X)) }
}

fn pick_burnt(
    layout: &MetroLayout, by_block: &HashMap<(i32, i32), Vec<usize>>,
    cfg: &DamageConfig, rng: &mut DotnetRandom, exclude: [(i32, i32); 3],
) -> (i32, i32) {
    let (mut all_plain, mut some_plain) = (Vec::new(), Vec::new());
    for b in layout.blocks.iter().filter(|b| b.district == cfg.burnt_district && b.reserved.is_none() && !exclude.contains(&(b.ix, b.iz))) {
        if by_block.get(&(b.ix, b.iz)).map_or(0, Vec::len) < cfg.burnt_min_towers { continue; }
        let edges = bordering_streets(layout, b);
        if edges.iter().all(|s| !s.avenue) { all_plain.push((b.ix, b.iz)); }
        if edges.iter().any(|s| !s.avenue) { some_plain.push((b.ix, b.iz)); }
    }
    let pool = if all_plain.is_empty() { &some_plain } else { &all_plain };
    if pool.is_empty() { (-1, -1) } else { pool[rng.next(pool.len() as i32) as usize] }
}

fn street_between<'a>(layout: &'a MetroLayout, a: &MetroBlock, b: &MetroBlock) -> Option<&'a MetroStreet> {
    if (a.ix - b.ix).abs() + (a.iz - b.iz).abs() != 1 { return None; }
    let mid = ((a.rect.0 + a.rect.1) * 0.5 + (b.rect.0 + b.rect.1) * 0.5) * 0.5;
    layout.streets.iter().find(|s| ((s.a + s.b) * 0.5).distance_squared(mid) < 1.0)
}

pub(super) fn bordering_streets<'a>(layout: &'a MetroLayout, b: &MetroBlock) -> Vec<&'a MetroStreet> {
    NEIGHBOURS.iter().filter_map(|&(dx, dz)| layout.block_at(b.ix + dx, b.iz + dz).and_then(|n| street_between(layout, b, n))).collect()
}

fn obb_overlaps(pos_a: Vec2, yaw_a: f32, half_a: Vec2, pos_b: Vec2, yaw_b: f32, half_b: Vec2) -> bool {
    let (axes_a, axes_b) = ([x_axis(yaw_a), y_axis(yaw_a)], [x_axis(yaw_b), y_axis(yaw_b)]);
    let delta = pos_b - pos_a;
    for u in [axes_a[0], axes_a[1], axes_b[0], axes_b[1]] {
        let ra = half_a.x * axes_a[0].dot(u).abs() + half_a.y * axes_a[1].dot(u).abs();
        let rb = half_b.x * axes_b[0].dot(u).abs() + half_b.y * axes_b[1].dot(u).abs();
        if delta.dot(u).abs() > ra + rb + EPS { return false; }
    }
    true
}

pub(crate) fn x_axis(yaw: f32) -> Vec2 { Vec2::new(yaw.cos(), -yaw.sin()) }
pub(crate) fn y_axis(yaw: f32) -> Vec2 { Vec2::new(-yaw.sin(), -yaw.cos()) }
pub(super) fn yaw_for_x(dir: Vec2) -> f32 { (-dir.y).atan2(dir.x) }

pub(super) fn mix_seed(site: &MetroSite, salt: &[u8]) -> i32 {
    let mut hash = FNV_OFFSET;
    for b in site.key.bytes().chain(site.center.x.to_bits().to_le_bytes()).chain(site.center.y.to_bits().to_le_bytes()).chain(salt.iter().copied()) {
        hash ^= u32::from(b);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash as i32
}
