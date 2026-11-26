//! New Halden overgrowth placement (pure, no GPU): crack weeds, street asphalt trees,
//! curb moss, lot scrub/saplings/tall grass, tower facade vines, and roof bushes/trees.

use std::sync::OnceLock;

use glam::Vec2;
use serde::Deserialize;

use crate::metro::layout::{MetroBlock, MetroDistrict, MetroLayout};
use crate::metro::place::MetroSite;
use crate::metro::state::MetroWorld;
use crate::metro::styles::{StyleDef, default_styles};
use crate::random::DotnetRandom;
use crate::scatter::PropInstance;

#[path = "overgrowth_roofs.rs"]
mod roofs;
#[cfg(test)]
#[path = "overgrowth_tests.rs"]
mod tests;

/// One chunk is 64 m square, matching `MetroConfig.block_size` and `jam_chunks::CHUNK_SIZE_M`.
pub(super) const CHUNK_SIZE_M: f32 = 64.0; // new: DECISIONS 2026-11-26
/// Overgrowth props are placed at authored scale.
pub(super) const PROP_SCALE: f32 = 1.0; // new: DECISIONS 2026-11-26
pub(super) const HALF: f32 = 0.5; // new: DECISIONS 2026-11-26
const AVENUE_COUNT: usize = 2; // new: DECISIONS 2026-11-26 (same as jam.rs entrance avenues)
const FNV_OFFSET: u32 = 2_166_136_261; // new: DECISIONS 2026-11-26
const FNV_PRIME: u32 = 16_777_619; // new: DECISIONS 2026-11-26

/// Per-district overgrowth counts and spawn probabilities per 64 m block (`data/metro.json` `overgrowth`).
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DistrictOvergrowth {
    pub crack_weeds_per_block: usize,
    pub street_tree_chance: f32,
    pub moss_per_block: usize,
    pub lot_shrubs_per_block: usize,
    pub lot_trees_per_block: usize,
    pub lot_grass_per_block: usize,
    pub vine_tower_chance: f32,
    pub roof_bush_chance: f32,
}

impl Default for DistrictOvergrowth {
    fn default() -> Self {
        Self {
            crack_weeds_per_block: 5, // data: metro.json overgrowth
            street_tree_chance: 0.45, // data: metro.json overgrowth
            moss_per_block: 3, // data: metro.json overgrowth
            lot_shrubs_per_block: 5, // data: metro.json overgrowth
            lot_trees_per_block: 2, // data: metro.json overgrowth
            lot_grass_per_block: 6, // data: metro.json overgrowth
            vine_tower_chance: 0.50, // data: metro.json overgrowth
            roof_bush_chance: 0.30, // data: metro.json overgrowth
        }
    }
}

/// City overgrowth configuration loaded from `data/metro.json` `overgrowth`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct OvergrowthConfig {
    pub edge: DistrictOvergrowth,
    pub midtown: DistrictOvergrowth,
    pub core: DistrictOvergrowth,
    pub vine_spacing_m: f32,
    pub crack_weed_length_m: f32,
    pub street_tree_spacing_m: f32,
    pub crack_weed_curb_offset_m: f32,
    pub street_tree_curb_offset_m: [f32; 2],
    pub lot_tower_clearance_m: f32,
    pub vine_half_depth_m: f32,
    pub vine_wall_standoff_m: [f32; 2],
    pub vine_corner_margin_m: f32,
    pub roof_edge_inset_m: f32,
    pub roof_tree_chance: f32,
}

impl Default for OvergrowthConfig {
    fn default() -> Self {
        Self {
            edge: DistrictOvergrowth {
                crack_weeds_per_block: 8, // data: metro.json overgrowth
                street_tree_chance: 0.70, // data: metro.json overgrowth
                moss_per_block: 6, // data: metro.json overgrowth
                lot_shrubs_per_block: 8, // data: metro.json overgrowth
                lot_trees_per_block: 4, // data: metro.json overgrowth
                lot_grass_per_block: 10, // data: metro.json overgrowth
                vine_tower_chance: 0.75, // data: metro.json overgrowth
                roof_bush_chance: 0.55, // data: metro.json overgrowth
            },
            midtown: DistrictOvergrowth::default(),
            core: DistrictOvergrowth {
                crack_weeds_per_block: 2, // data: metro.json overgrowth
                street_tree_chance: 0.20, // data: metro.json overgrowth
                moss_per_block: 1, // data: metro.json overgrowth
                lot_shrubs_per_block: 2, // data: metro.json overgrowth
                lot_trees_per_block: 1, // data: metro.json overgrowth
                lot_grass_per_block: 2, // data: metro.json overgrowth
                vine_tower_chance: 0.25, // data: metro.json overgrowth
                roof_bush_chance: 0.12, // data: metro.json overgrowth
            },
            vine_spacing_m: 6.0, // data: metro.json overgrowth
            crack_weed_length_m: 3.0, // data: metro.json overgrowth
            street_tree_spacing_m: 32.0, // data: metro.json overgrowth
            crack_weed_curb_offset_m: 0.3, // data: metro.json overgrowth
            street_tree_curb_offset_m: [1.5, 3.0], // data: metro.json overgrowth
            lot_tower_clearance_m: 1.5, // data: metro.json overgrowth
            vine_half_depth_m: 0.15, // data: metro.json overgrowth
            vine_wall_standoff_m: [0.02, 0.05], // data: metro.json overgrowth
            vine_corner_margin_m: 1.5, // data: metro.json overgrowth
            roof_edge_inset_m: 1.5, // data: metro.json overgrowth
            roof_tree_chance: 0.35, // data: metro.json overgrowth
        }
    }
}

impl OvergrowthConfig {
    pub(super) fn for_district(&self, d: MetroDistrict) -> &DistrictOvergrowth {
        match d {
            MetroDistrict::Edge => &self.edge,
            MetroDistrict::Midtown => &self.midtown,
            MetroDistrict::Core => &self.core,
        }
    }
}

#[derive(Deserialize)]
struct OvergrowthWrapper {
    #[serde(default)]
    overgrowth: OvergrowthConfig,
}

/// Parses and validates the top-level `overgrowth` object from a `metro.json` string.
pub fn load_overgrowth(json: &str) -> Result<OvergrowthConfig, String> {
    let wrapper: OvergrowthWrapper =
        serde_json::from_str(json).map_err(|e| format!("metro.json overgrowth: {e}"))?;
    validate_overgrowth(&wrapper.overgrowth)?;
    Ok(wrapper.overgrowth)
}

fn validate_overgrowth(cfg: &OvergrowthConfig) -> Result<(), String> {
    for (name, d) in [("edge", &cfg.edge), ("midtown", &cfg.midtown), ("core", &cfg.core)] {
        for (field, val) in [
            ("streetTreeChance", d.street_tree_chance),
            ("vineTowerChance", d.vine_tower_chance),
            ("roofBushChance", d.roof_bush_chance),
        ] {
            if !(0.0..=1.0).contains(&val) {
                return Err(format!("metro.json overgrowth.{name}.{field} must be in 0..=1"));
            }
        }
    }
    if !(0.0..=1.0).contains(&cfg.roof_tree_chance) {
        return Err("metro.json overgrowth.roofTreeChance must be in 0..=1".into());
    }
    if cfg.vine_spacing_m <= 0.0 || cfg.crack_weed_length_m <= 0.0 || cfg.street_tree_spacing_m <= 0.0 {
        return Err("metro.json overgrowth spacings and lengths must be positive".into());
    }
    let non_neg = [
        cfg.crack_weed_curb_offset_m,
        cfg.lot_tower_clearance_m,
        cfg.vine_half_depth_m,
        cfg.vine_corner_margin_m,
        cfg.roof_edge_inset_m,
        cfg.street_tree_curb_offset_m[0],
        cfg.vine_wall_standoff_m[0],
    ];
    if non_neg.iter().any(|&v| v < 0.0)
        || cfg.street_tree_curb_offset_m[0] > cfg.street_tree_curb_offset_m[1]
        || cfg.vine_wall_standoff_m[0] > cfg.vine_wall_standoff_m[1]
    {
        return Err("metro.json overgrowth offsets or ranges are invalid".into());
    }
    Ok(())
}

/// Cached overgrowth settings loaded from `data/metro.json`.
pub fn default_overgrowth() -> &'static OvergrowthConfig {
    static CACHE: OnceLock<OvergrowthConfig> = OnceLock::new();
    CACHE.get_or_init(|| {
        let path = paths::repo_path("data/metro.json");
        let json = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
        load_overgrowth(&json).expect("data/metro.json overgrowth must be valid")
    })
}

/// Pure, deterministic overgrowth props whose XZ falls inside `chunk`.
pub fn overgrowth_props_in_chunk(
    metro: &MetroWorld,
    chunk: (i32, i32),
    height: &dyn Fn(f32, f32) -> f32,
) -> Vec<PropInstance> {
    overgrowth_props_in_chunk_with_config(metro, chunk, height, default_overgrowth(), default_styles())
}

/// Same as `overgrowth_props_in_chunk`, with explicit `OvergrowthConfig` and `StyleDef` slice.
pub fn overgrowth_props_in_chunk_with_config(
    metro: &MetroWorld,
    chunk: (i32, i32),
    height: &dyn Fn(f32, f32) -> f32,
    cfg: &OvergrowthConfig,
    styles: &[StyleDef],
) -> Vec<PropInstance> {
    let blocks = overlapping_blocks(metro, chunk);
    if blocks.is_empty() {
        return Vec::new();
    }
    let entrance_avenues = pick_entrance_avenues(&metro.layout, metro.highway.points.last().copied());
    let mut out = Vec::new();
    for block in blocks {
        if block.reserved.is_some() {
            continue;
        }
        let mut rng = DotnetRandom::new(block_seed(&metro.layout.site, (block.ix, block.iz)));
        let mut block_props = Vec::new();
        roofs::place_street_overgrowth(metro, block, cfg, &entrance_avenues, height, &mut rng, &mut block_props);
        roofs::place_lot_overgrowth(metro, block, cfg, height, &mut rng, &mut block_props);
        roofs::place_tower_overgrowth(metro, block, cfg, styles, height, &mut rng, &mut block_props);
        out.extend(block_props.into_iter().filter(|p| in_chunk(p.position.x, p.position.z, chunk)));
    }
    out
}

fn in_chunk(x: f32, z: f32, chunk: (i32, i32)) -> bool {
    (x / CHUNK_SIZE_M).floor() as i32 == chunk.0 && (z / CHUNK_SIZE_M).floor() as i32 == chunk.1
}

/// Blocks whose site-local square overlaps the world-space chunk AABB (with street margin).
fn overlapping_blocks(metro: &MetroWorld, chunk: (i32, i32)) -> Vec<&MetroBlock> {
    let min = Vec2::new(chunk.0 as f32, chunk.1 as f32) * CHUNK_SIZE_M;
    let max = min + Vec2::splat(CHUNK_SIZE_M);
    let pad = metro.config.avenue_width.max(metro.config.street_width) * HALF + 2.0; // new: DECISIONS 2026-11-26
    let site = &metro.layout.site;
    let corners = [
        site.to_local(min),
        site.to_local(Vec2::new(max.x, min.y)),
        site.to_local(Vec2::new(min.x, max.y)),
        site.to_local(max),
    ];
    let local_min = corners.into_iter().reduce(Vec2::min).unwrap_or(Vec2::ZERO) - Vec2::splat(pad);
    let local_max = corners.into_iter().reduce(Vec2::max).unwrap_or(Vec2::ZERO) + Vec2::splat(pad);
    let half = site.half_size;
    if local_max.x < -half || local_min.x > half || local_max.y < -half || local_min.y > half {
        return Vec::new();
    }
    let (n, bs) = (metro.config.size_blocks, metro.config.block_size);
    let cell = |v: f32| (((v + half) / bs).floor() as i32).clamp(0, n - 1);
    let (ix0, ix1) = (cell(local_min.x), cell(local_max.x));
    let (iz0, iz1) = (cell(local_min.y), cell(local_max.y));
    (iz0..=iz1)
        .flat_map(|iz| (ix0..=ix1).filter_map(move |ix| metro.layout.block_at(ix, iz)))
        .collect()
}

/// Indices into `layout.streets` of the `AVENUE_COUNT` entrance avenue segments nearest `entrance`.
fn pick_entrance_avenues(layout: &MetroLayout, entrance: Option<Vec2>) -> Vec<usize> {
    let Some(entrance) = entrance else { return Vec::new() };
    let mut found: Vec<(f32, usize)> = layout
        .streets
        .iter()
        .enumerate()
        .filter(|(_, s)| s.avenue && layout.contains(layout.site.to_world((s.a + s.b) * HALF)))
        .map(|(idx, s)| {
            let dist = layout.site.to_world(s.a).distance(entrance).min(layout.site.to_world(s.b).distance(entrance));
            (dist, idx)
        })
        .collect();
    found.sort_by(|x, y| x.0.total_cmp(&y.0));
    found.into_iter().take(AVENUE_COUNT).map(|(_, idx)| idx).collect()
}

pub(super) fn lerp_f32(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn block_seed(site: &MetroSite, block: (i32, i32)) -> i32 {
    let mut hash = FNV_OFFSET;
    let bytes = site
        .key
        .bytes()
        .chain(site.center.x.to_bits().to_le_bytes())
        .chain(site.center.y.to_bits().to_le_bytes())
        .chain(block.0.to_le_bytes())
        .chain(block.1.to_le_bytes())
        .chain(*b"overgrowth");
    for b in bytes {
        hash ^= u32::from(b);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash as i32
}
