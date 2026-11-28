//! R44-03: New Halden safe house and rooftop camp placement. Pure logic, no GPU.
//! The safe house sits on one fire-escape brick tower; the rooftop camp on a Core climb tower;
//! three survivors stand on stop floors of climb towers. Layout numbers come from
//! `data/safe_house.json` (read through `safe_house_layout`).

use std::sync::OnceLock;

use glam::Vec3;
use serde::Deserialize;

use crate::metro::climb::{ClimbChunk, ClimbKind, ClimbTower, StopKind, climb_towers, module_size_at, roof_deck_y};
use crate::metro::fire_escapes::fire_escape_towers;
use crate::metro::loot_helpers::in_chunk;
use crate::metro::place::MetroSite;
use crate::metro::state::MetroWorld;
use crate::metro::styles::{default_styles, style_for};
use crate::metro::towers::TowerSpot;
use crate::random::DotnetRandom;
use crate::scatter::PropInstance;

const DEFAULT_FLOOR_HEIGHT: f32 = 3.8; // data: metro.json floorHeight
const FNV_OFFSET: u32 = 2_166_136_261; // new: DECISIONS 2026-11-27
const FNV_PRIME: u32 = 16_777_619; // new: DECISIONS 2026-11-27
const RESCUE_FLOOR_SEEDS: usize = 3; // data: rescue.json floorSurvivors
const FLOOR_IN_DIST_M: f32 = 3.0; // new: DECISIONS 2026-11-27 (3 m in from shaft door)
const STOP_FLOOR_SLAB_T_M: f32 = 0.3; // new: DECISIONS 2026-11-27 (stop-floor slab thickness)
const STAND_CLEAR_M: f32 = 0.05; // new: DECISIONS 2026-11-27 (clearance above slab)

/// The geometry blocks of `data/safe_house.json` (tower-local metres, R43 frame).
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SafeHouseLayout {
    pub shack: ShackLayout,
    pub ladder: LadderLayout,
    pub rooftop_camp: RooftopCampLayout,
}

impl SafeHouseLayout {
    pub fn from_json(raw: &str) -> Result<Self, String> {
        serde_json::from_str(raw).map_err(|e| e.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShackLayout {
    pub centre_local: [f32; 2],
    pub bed_local: [f32; 2],
    pub chest_local: [f32; 2],
    pub sign_local: [f32; 2],
    pub travel_point_local: [f32; 2],
}

/// Rope ladder on the BACK face: foot on the street `foot_out_m` beyond it, deck inset on the roof.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LadderLayout {
    pub foot_out_m: f32,
    pub deck_inset_m: f32,
    pub centre_x: f32,
    pub segment_m: f32,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RooftopCampLayout {
    pub map_label: String,
    pub props: Vec<PropSpec>,
    pub npcs: Vec<NpcSpec>,
}

/// A rooftop camp prop. `yaw` is relative to the tower.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropSpec {
    pub model: String,
    pub local: [f32; 2],
    pub yaw: f32,
}

/// A rooftop camp NPC. `yaw` is relative to the tower.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NpcSpec {
    pub id: String,
    pub model: String,
    pub local: [f32; 2],
    pub yaw: f32,
    pub sitting: bool,
    #[serde(default)]
    pub role: Option<String>,
}

/// The one loader for `data/safe_house.json`. `None` (with a warning) when the file is bad.
pub fn safe_house_layout() -> Option<&'static SafeHouseLayout> {
    static CACHE: OnceLock<Option<SafeHouseLayout>> = OnceLock::new();
    CACHE
        .get_or_init(|| {
            let path = paths::repo_path("data/safe_house.json");
            let raw = match std::fs::read_to_string(&path) {
                Ok(raw) => raw,
                Err(err) => {
                    eprintln!("[safehouse] cannot read {}: {err}; safe house off", path.display());
                    return None;
                }
            };
            match SafeHouseLayout::from_json(&raw) {
                Ok(layout) => Some(layout),
                Err(err) => {
                    eprintln!("[safehouse] bad data/safe_house.json: {err}; safe house off");
                    None
                }
            }
        })
        .as_ref()
}

/// Tower-local `(lx, lz)` to world XYZ (same frame as `fire_escapes.rs`).
fn to_world(t: &TowerSpot, lx: f32, y: f32, lz: f32) -> Vec3 {
    let (sin_y, cos_y) = t.yaw.sin_cos();
    Vec3::new(t.pos.x + lx * cos_y + lz * sin_y, y, t.pos.y - lx * sin_y + lz * cos_y)
}

/// The fire-escape tower nearest the city centre (ties: lowest index).
pub fn safe_house_tower(metro: &MetroWorld) -> Option<usize> {
    let centre = metro.layout.site.center;
    fire_escape_towers(metro).into_iter().min_by(|&a, &b| {
        let da = metro.towers[a].pos.distance_squared(centre);
        let db = metro.towers[b].pos.distance_squared(centre);
        da.total_cmp(&db).then(a.cmp(&b))
    })
}

/// The Core climb tower farthest from the safe house (no Core: any climb tower). Ties: lowest index.
pub fn rooftop_camp_tower(metro: &MetroWorld) -> Option<usize> {
    let safe = metro.towers[safe_house_tower(metro)?].pos;
    let plans = climb_towers(metro);
    let cores: Vec<&ClimbTower> = plans.iter().filter(|p| p.kind == ClimbKind::Core).collect();
    let pool: Vec<&ClimbTower> = if cores.is_empty() { plans.iter().collect() } else { cores };
    pool.into_iter().map(|p| p.tower).max_by(|&a, &b| {
        let da = metro.towers[a].pos.distance_squared(safe);
        let db = metro.towers[b].pos.distance_squared(safe);
        da.total_cmp(&db).then(b.cmp(&a))
    })
}

/// World positions of everything on the safe house tower.
#[derive(Debug, Clone, PartialEq)]
pub struct SafeHouseSpots {
    pub tower: usize,
    pub deck_y: f32,
    pub shack: Vec3,
    pub bed: Vec3,
    pub chest: Vec3,
    pub sign: Vec3,
    pub travel_point: Vec3,
    pub ladder_foot: Vec3,
    pub ladder_deck: Vec3,
}

pub fn safe_house_spots(
    metro: &MetroWorld,
    layout: &SafeHouseLayout,
    height: &dyn Fn(f32, f32) -> f32,
) -> Option<SafeHouseSpots> {
    let tower = safe_house_tower(metro)?;
    let t = &metro.towers[tower];
    let style = style_for(default_styles(), t.style)?;
    let deck_y = roof_deck_y(t, style, metro.layout.site.ground_y);
    let half = t.half.x;
    let on_roof = |p: [f32; 2]| to_world(t, p[0], deck_y, p[1]);
    let ladder = &layout.ladder;
    let foot = to_world(t, ladder.centre_x, 0.0, half + ladder.foot_out_m);
    Some(SafeHouseSpots {
        tower,
        deck_y,
        shack: on_roof(layout.shack.centre_local),
        bed: on_roof(layout.shack.bed_local),
        chest: on_roof(layout.shack.chest_local),
        sign: on_roof(layout.shack.sign_local),
        travel_point: on_roof(layout.shack.travel_point_local),
        ladder_foot: Vec3::new(foot.x, height(foot.x, foot.z), foot.z),
        ladder_deck: to_world(t, ladder.centre_x, deck_y, half - ladder.deck_inset_m),
    })
}

/// Props for the safe house chunk: shack, bed, chest, sign, plus the rope ladder
/// (`ladder_down`: segments from the street to the roof; up: one coil on the roof edge).
/// Ladder rails are added at runtime by the wiring task, not here.
pub fn safehouse_in_chunk(
    metro: &MetroWorld,
    layout: &SafeHouseLayout,
    chunk: (i32, i32),
    height: &dyn Fn(f32, f32) -> f32,
    ladder_down: bool,
) -> ClimbChunk {
    let mut out = ClimbChunk::default();
    let Some(spots) = safe_house_spots(metro, layout, height) else {
        return out;
    };
    let t = &metro.towers[spots.tower];
    if !in_chunk(t.pos.x, t.pos.y, chunk) {
        return out;
    }
    let yaw = t.yaw;
    out.props.push(PropInstance::new("safehouse_shack", spots.shack, yaw, 1.0));
    out.props.push(PropInstance::new("safehouse_bed", spots.bed, yaw, 1.0));
    out.props.push(PropInstance::new("safehouse_chest", spots.chest, yaw, 1.0));
    out.props.push(PropInstance::new("travel_sign", spots.sign, yaw, 1.0));
    if ladder_down {
        push_rope_segments(&mut out.props, &spots, &layout.ladder, yaw);
    } else {
        out.props.push(PropInstance::new("rope_ladder_coil", spots.ladder_deck, yaw, 1.0));
    }
    out
}

/// One rope segment per `segment_m` of the foot-to-deck line (the centre of each slot).
fn push_rope_segments(props: &mut Vec<PropInstance>, spots: &SafeHouseSpots, ladder: &LadderLayout, yaw: f32) {
    if ladder.segment_m <= 0.0 {
        return;
    }
    let rise = spots.ladder_deck.y - spots.ladder_foot.y;
    let count = (rise / ladder.segment_m).ceil().max(0.0) as usize;
    for i in 0..count {
        let f = (i as f32 + 0.5) / count as f32;
        let p = spots.ladder_foot.lerp(spots.ladder_deck, f);
        props.push(PropInstance::new("rope_ladder_seg", p, yaw, 1.0));
    }
}

/// A rooftop camp NPC. `role` is "" when the layout gives none.
#[derive(Debug, Clone, PartialEq)]
pub struct RooftopNpc {
    pub id: String,
    pub model: String,
    pub position: Vec3,
    pub yaw: f32,
    pub sitting: bool,
    pub role: String,
}

/// Rooftop camp props and NPCs on the camp tower's roof, only in its chunk.
/// (No `height` argument: everything sits on the roof deck.)
pub fn rooftop_camp_in_chunk(
    metro: &MetroWorld,
    layout: &SafeHouseLayout,
    chunk: (i32, i32),
) -> (ClimbChunk, Vec<RooftopNpc>) {
    let mut out = ClimbChunk::default();
    let mut npcs = Vec::new();
    let Some(tower) = rooftop_camp_tower(metro) else {
        return (out, npcs);
    };
    let t = &metro.towers[tower];
    let Some(style) = style_for(default_styles(), t.style) else {
        return (out, npcs);
    };
    if !in_chunk(t.pos.x, t.pos.y, chunk) {
        return (out, npcs);
    }
    let deck_y = roof_deck_y(t, style, metro.layout.site.ground_y);
    let camp = &layout.rooftop_camp;
    for p in &camp.props {
        let pos = to_world(t, p.local[0], deck_y, p.local[1]);
        out.props.push(PropInstance::new(p.model.clone(), pos, t.yaw + p.yaw, 1.0));
    }
    for n in &camp.npcs {
        npcs.push(RooftopNpc {
            id: n.id.clone(),
            model: n.model.clone(),
            position: to_world(t, n.local[0], deck_y, n.local[1]),
            yaw: t.yaw + n.yaw,
            sitting: n.sitting,
            role: n.role.clone().unwrap_or_default(),
        });
    }
    (out, npcs)
}

/// A stop-floor survivor seed. The R44-04 pass turns it into a `RescueSpot`.
#[derive(Debug, Clone, PartialEq)]
pub struct ClimbRescueSeed {
    pub poi_id: String,
    pub building_id: String,
    pub position: Vec3,
    pub yaw: f32,
}

/// Picks `RESCUE_FLOOR_SEEDS` stop floors (not the top floor, not Camp), on different towers
/// where possible. Deterministic per site; sorted by `poi_id`.
pub fn rescue_floor_seeds(metro: &MetroWorld) -> Vec<ClimbRescueSeed> {
    let mut cands: Vec<(usize, u32, StopKind)> = Vec::new();
    for plan in climb_towers(metro) {
        let top = metro.towers[plan.tower].floors.saturating_sub(1);
        for (&floor, &kind) in plan.stop_floors.iter().zip(&plan.stop_kinds) {
            if floor != top && kind != StopKind::Camp {
                cands.push((plan.tower, floor, kind));
            }
        }
    }
    let mut rng = DotnetRandom::new(mix_seed(&metro.layout.site, b"rescue_floors"));
    for i in (1..cands.len()).rev() {
        let j = rng.next((i + 1) as i32) as usize;
        cands.swap(i, j);
    }
    let mut picked: Vec<(usize, u32, StopKind)> = Vec::new();
    for c in &cands {
        if picked.len() == RESCUE_FLOOR_SEEDS {
            break;
        }
        if !picked.iter().any(|p| p.0 == c.0) {
            picked.push(*c);
        }
    }
    for c in &cands {
        if picked.len() == RESCUE_FLOOR_SEEDS {
            break;
        }
        if !picked.contains(c) {
            picked.push(*c);
        }
    }
    let mut out: Vec<ClimbRescueSeed> = picked
        .into_iter()
        .map(|(tower, floor, kind)| rescue_seed(metro, tower, floor, kind))
        .collect();
    out.sort_by(|a, b| a.poi_id.cmp(&b.poi_id));
    out
}

fn rescue_seed(metro: &MetroWorld, tower: usize, floor: u32, kind: StopKind) -> ClimbRescueSeed {
    let styles = default_styles();
    let cfg = &metro.config.climb;
    let t = &metro.towers[tower];
    let style = style_for(styles, t.style).unwrap_or(&styles[0]);
    let fh = if style.floor_height > 0.0 { style.floor_height } else { DEFAULT_FLOOR_HEIGHT };
    let fy = style.base_height + floor as f32 * fh;
    let door_lz = cfg.shaft_centre_z_share * t.size as f32 - cfg.shaft_size[1] * 0.5;
    let xz = to_world(t, 0.0, 0.0, door_lz - FLOOR_IN_DIST_M);
    let size = module_size_at(t, floor, style, &metro.config.towers.setbacks);
    ClimbRescueSeed {
        poi_id: format!("metro_floor_{tower}_{floor}"),
        building_id: format!("tw_interior_{}_{size}", kind.key()),
        position: Vec3::new(xz.x, metro.layout.site.ground_y + fy + STOP_FLOOR_SLAB_T_M + STAND_CLEAR_M, xz.z),
        yaw: t.yaw,
    }
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
