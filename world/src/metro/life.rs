//! New Halden silent life placement: power lines, billboards, flags, plastic sheets,
//! and crow perches. Pure logic: no GPU.

use std::f32::consts::{FRAC_1_SQRT_2, TAU};
use std::sync::OnceLock;

use glam::{Vec2, Vec3};
use serde::Deserialize;

use crate::metro::layout::{MetroBlock, MetroDistrict};
use crate::metro::life_wires::{WIRE_ATTACH_HEIGHT_M, WIRE_SAG_M, WireSpan, collect_wires_in_chunk};
use crate::metro::place::MetroSite;
use crate::metro::state::MetroWorld;
use crate::metro::styles::{StyleDef, TowerStyle, default_styles, style_for};
use crate::metro::tower_parts::{PartKind, tower_parts};
use crate::metro::towers::TowerSpot;
use crate::metro::traces::{car_wall_sites, traces_props_in_chunk};
use crate::random::DotnetRandom;
use crate::scatter::PropInstance;

pub(super) const CHUNK_SIZE_M: f32 = 64.0; // data: metro.json blockSize
const PROP_SCALE: f32 = 1.0; // new: DECISIONS 2026-11-27
const HALF: f32 = 0.5; // new: DECISIONS 2026-11-27
const FNV_OFFSET: u32 = 2_166_136_261; // new: DECISIONS 2026-11-27
const FNV_PRIME: u32 = 16_777_619; // new: DECISIONS 2026-11-27
const ROOF_BILLBOARD_FLOORS: [u32; 2] = [4, 10]; // new: DECISIONS 2026-11-27 (R41-03 low roofs)
const STREET_BILLBOARD_FACE_SCALE: f32 = 0.66; // new: DECISIONS 2026-11-27 (R41-04 street face scale)
const STREET_BILLBOARD_CLEAR_M: f32 = 4.0; // new: DECISIONS 2026-11-27
const FLAG_HEIGHT_RANGE_M: [f32; 2] = [4.0, 6.0]; // new: DECISIONS 2026-11-27 (R41-03 4-6 m)
const FLAG_WALL_EMBED_M: f32 = 0.03; // new: DECISIONS 2026-11-27 (2-5 cm into wall)
const FLAG_DOOR_OFFSET_SHARE: f32 = 0.28; // new: DECISIONS 2026-11-27 (clears lobby door)
const FLAG_POLE_LENGTH_M: f32 = 2.5; // new: DECISIONS 2026-11-27 (R41-04 2.5 m pole at 45 deg)
const CAR_WALL_TOP_Y_M: f32 = 2.4; // new: DECISIONS 2026-11-27
const LOT_FENCE_Y_M: f32 = 1.6; // new: DECISIONS 2026-11-27
const CAR_ROOF_Y_M: f32 = 1.5; // new: DECISIONS 2026-11-27
const ROOF_EDGE_INSET_M: f32 = 1.5; // new: DECISIONS 2026-11-27
const LOT_EDGE_INSET_M: f32 = 2.0; // new: DECISIONS 2026-11-27

const BILLBOARD_FACES: [&str; 4] = [
    "billboard_face_a",
    "billboard_face_b",
    "billboard_face_c",
    "billboard_face_d",
];

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

/// Per-district crow perch probability per 64 m block (`data/metro.json` `life.crowPerchChance`).
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CrowPerchChance {
    pub edge: f32,
    pub midtown: f32,
    pub core: f32,
}

impl Default for CrowPerchChance {
    fn default() -> Self {
        Self {
            edge: 0.55,    // data: metro.json crowPerchChance
            midtown: 0.45, // data: metro.json crowPerchChance
            core: 0.35,    // data: metro.json crowPerchChance
        }
    }
}

impl CrowPerchChance {
    pub fn for_district(&self, d: MetroDistrict) -> f32 {
        match d {
            MetroDistrict::Edge => self.edge,
            MetroDistrict::Midtown => self.midtown,
            MetroDistrict::Core => self.core,
        }
    }
}

/// Silent life settings loaded from `data/metro.json` `life`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct LifeConfig {
    pub pole_spacing_m: f32,
    pub pole_avenue_share: f32,
    pub pole_fallen_chance: f32,
    pub pole_leaning_chance: f32,
    pub billboard_roof_chance: f32,
    pub billboard_street_chance: f32,
    pub flag_chance: f32,
    pub plastic_chance: f32,
    pub crow_perch_chance: CrowPerchChance,
    pub crows_per_perch: [u8; 2],
}

impl Default for LifeConfig {
    fn default() -> Self {
        Self {
            pole_spacing_m: 20.0,         // data: metro.json life
            pole_avenue_share: 0.35,      // data: metro.json life
            pole_fallen_chance: 0.15,     // data: metro.json life
            pole_leaning_chance: 0.2,     // data: metro.json life
            billboard_roof_chance: 0.18,  // data: metro.json life
            billboard_street_chance: 0.12,// data: metro.json life
            flag_chance: 0.35,            // data: metro.json life
            plastic_chance: 0.5,          // data: metro.json life
            crow_perch_chance: CrowPerchChance::default(),
            crows_per_perch: [3, 9],      // data: metro.json life
        }
    }
}

impl LifeConfig {
    /// Validates probabilities and ranges.
    pub fn validate(&self) -> Result<(), String> {
        if self.pole_spacing_m <= 0.0 {
            return Err("metro.json life.poleSpacingM must be positive".into());
        }
        let chances = [
            ("poleAvenueShare", self.pole_avenue_share),
            ("poleFallenChance", self.pole_fallen_chance),
            ("poleLeaningChance", self.pole_leaning_chance),
            ("billboardRoofChance", self.billboard_roof_chance),
            ("billboardStreetChance", self.billboard_street_chance),
            ("flagChance", self.flag_chance),
            ("plasticChance", self.plastic_chance),
            ("crowPerchChance.edge", self.crow_perch_chance.edge),
            ("crowPerchChance.midtown", self.crow_perch_chance.midtown),
            ("crowPerchChance.core", self.crow_perch_chance.core),
        ];
        for (name, v) in chances {
            if !(0.0..=1.0).contains(&v) {
                return Err(format!("metro.json life.{name} ({v}) must be within 0..=1"));
            }
        }
        if self.pole_fallen_chance + self.pole_leaning_chance > 1.0 {
            return Err("metro.json life: poleFallenChance + poleLeaningChance must be <= 1".into());
        }
        if self.crows_per_perch[0] > self.crows_per_perch[1] {
            return Err("metro.json life.crowsPerPerch min must be <= max".into());
        }
        Ok(())
    }
}

#[derive(Deserialize)]
struct LifeWrapper {
    #[serde(default)]
    life: LifeConfig,
}

/// Parses and validates the top-level `life` object from a `metro.json` string.
pub fn load_life(json: &str) -> Result<LifeConfig, String> {
    let wrapper: LifeWrapper =
        serde_json::from_str(json).map_err(|e| format!("metro.json life: {e}"))?;
    wrapper.life.validate()?;
    Ok(wrapper.life)
}

/// Cached `LifeConfig` loaded from `data/metro.json` (falls back to `LifeConfig::default()`).
pub fn default_life() -> &'static LifeConfig {
    static CACHE: OnceLock<LifeConfig> = OnceLock::new();
    CACHE.get_or_init(|| {
        let path = paths::repo_path("data/metro.json");
        let json = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
        let root: serde_json::Value = serde_json::from_str(&json)
            .unwrap_or_else(|e| panic!("failed to parse {}: {e}", path.display()));
        if root.get("life").is_none() {
            return LifeConfig::default();
        }
        load_life(&json).expect("data/metro.json life must be valid")
    })
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

fn place_block_billboards_and_flags(
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

fn flag_for_tower(
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

fn lot_plastic_for_block(
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

#[allow(clippy::too_many_arguments)]
fn pick_perch_anchor(
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
        if p.model.starts_with("body_sheet") && in_chunk(p.position.x, p.position.z, chunk) {
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

trait BlockKey {
    fn block_key(&self) -> (i32, i32);
}

impl BlockKey for MetroBlock {
    fn block_key(&self) -> (i32, i32) {
        (self.ix, self.iz)
    }
}

fn block_sidewalk_point(metro: &MetroWorld, block: &MetroBlock) -> Vec2 {
    let every = metro.config.avenue_every;
    let sw_w = if block.ix % every == 0 { metro.config.avenue_width } else { metro.config.street_width };
    let local = Vec2::new(block.rect.0.x + sw_w * HALF + 0.5, (block.rect.0.y + block.rect.1.y) * HALF);
    metro.layout.site.to_world(local)
}

fn is_low_billboard_tower(tower: &TowerSpot) -> bool {
    !tower.landmark
        && matches!(tower.style, TowerStyle::Brick | TowerStyle::Slab)
        && (ROOF_BILLBOARD_FLOORS[0]..=ROOF_BILLBOARD_FLOORS[1]).contains(&tower.floors)
}

fn roof_top_y(spot: &TowerSpot, style: &StyleDef, ground_y: f32, fallback_fh: f32) -> Option<f32> {
    let parts = tower_parts(spot, style, ground_y, 0);
    let last_floor = parts.iter().rfind(|p| p.kind == PartKind::Floor)?;
    let fh = if style.floor_height > 0.0 { style.floor_height } else { fallback_fh };
    Some(last_floor.pos.y + fh + style.roof_deck_height)
}

fn nearest_avenue_yaw(metro: &MetroWorld, wp: Vec2, fallback: f32) -> f32 {
    let local = metro.layout.site.to_local(wp);
    let best = metro
        .layout
        .streets
        .iter()
        .filter(|s| s.avenue)
        .map(|s| {
            let cp = s.closest_point(local);
            (cp.distance_squared(local), cp)
        })
        .min_by(|a, b| a.0.total_cmp(&b.0));
    let Some((_, cp)) = best else { return fallback };
    let target_w = metro.layout.site.to_world(cp);
    let dir = (target_w - wp).normalize_or_zero();
    if dir.length_squared() > 0.0 { dir_yaw(dir) } else { fallback }
}

fn pick_lot_spot(
    metro: &MetroWorld,
    block: &MetroBlock,
    clearance: f32,
    rng: &mut DotnetRandom,
) -> Option<(Vec2, f32)> {
    let every = metro.config.avenue_every;
    let w = |line: i32| if line % every == 0 { metro.config.avenue_width } else { metro.config.street_width };
    let lo = block.rect.0 + Vec2::new(w(block.ix), w(block.iz)) * HALF + Vec2::splat(clearance);
    let hi = block.rect.1 - Vec2::new(w(block.ix + 1), w(block.iz + 1)) * HALF - Vec2::splat(clearance);
    if hi.x <= lo.x || hi.y <= lo.y {
        return None;
    }
    for _ in 0..8 {
        let lp = Vec2::new(
            lo.x + (hi.x - lo.x) * rng.next_double() as f32,
            lo.y + (hi.y - lo.y) * rng.next_double() as f32,
        );
        let wp = metro.layout.site.to_world(lp);
        if !metro.in_tower_footprint(wp) {
            let yaw = nearest_avenue_yaw(metro, wp, 0.0);
            return Some((wp, yaw));
        }
    }
    None
}

fn overlapping_blocks(metro: &MetroWorld, chunk: (i32, i32)) -> Vec<&MetroBlock> {
    let min = Vec2::new(chunk.0 as f32, chunk.1 as f32) * CHUNK_SIZE_M;
    let max = min + Vec2::splat(CHUNK_SIZE_M);
    let pad = metro.config.avenue_width.max(metro.config.street_width) * HALF + 2.0;
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

pub(super) fn in_chunk(x: f32, z: f32, chunk: (i32, i32)) -> bool {
    (x / CHUNK_SIZE_M).floor() as i32 == chunk.0 && (z / CHUNK_SIZE_M).floor() as i32 == chunk.1
}

pub(super) fn dir_yaw(d: Vec2) -> f32 {
    (-d.x).atan2(-d.y)
}

pub(super) fn mix_seed(site: &MetroSite, block: Option<(i32, i32)>, salt: &[u8]) -> i32 {
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
    if let Some((ix, iz)) = block {
        feed(&ix.to_le_bytes());
        feed(&iz.to_le_bytes());
    }
    feed(salt);
    hash as i32
}
