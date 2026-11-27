//! R43-01: New Halden climbable towers — picking, stop-off floors, stairwells, roof hatches and mast.
//! Pure logic, no GPU.

use std::cell::RefCell;
use std::rc::Rc;

use glam::{Vec2, Vec3};
use serde::Deserialize;

use crate::biome::BiomeId;
use crate::loot::LootSpot;
use crate::metro::config::SetbackSpec;
use crate::metro::layout::{MetroLayout, Reserved};
use crate::metro::loot_helpers::in_chunk;
use crate::metro::place::MetroSite;
use crate::metro::state::MetroWorld;
use crate::metro::styles::{StyleDef, default_styles, style_for};
use crate::metro::towers::{TowerKind, TowerSpot};
use crate::poi::PoiInstance;
use crate::random::DotnetRandom;
use crate::scatter::PropInstance;

pub use crate::metro::climb_hulls::climb_hull_boxes;

const DEFAULT_FLOOR_HEIGHT: f32 = 3.8; // data: metro.json floorHeight
const LANDMARK_MAST_DECK_HEIGHT: f32 = 0.2; // new: DECISIONS 2026-11-27 (landmarks.py roof_deck top = 0.20)
const HATCH_CLEAR_STEP_M: f32 = 1.0; // new: DECISIONS 2026-11-27 (step off toward shaft-free +X side)
const MAST_PLATFORM_SLAB_T: f32 = 0.22; // new: DECISIONS 2026-11-27 (landmarks.py pz..pz+0.22)
const MAST_RAIL_TOP: f32 = 1.45; // new: DECISIONS 2026-11-27 (landmarks.py pz+1.25 + 0.20)
const MAST_RAIL_HALF_T: f32 = 0.05; // new: DECISIONS 2026-11-27
const FNV_OFFSET: u32 = 2_166_136_261; // new: DECISIONS 2026-11-26
const FNV_PRIME: u32 = 16_777_619; // new: DECISIONS 2026-11-26

/// District / role of a climbable tower.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClimbKind {
    Midtown,
    Core,
    Landmark,
}

/// Interior theme of a stop-off floor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StopKind {
    Office,
    Flat,
    Camp,
}

impl StopKind {
    pub fn key(self) -> &'static str {
        match self {
            StopKind::Office => "office",
            StopKind::Flat => "flat",
            StopKind::Camp => "camp",
        }
    }
}

/// One climbable tower in the city: its index, kind, stop-off floors and local deck height.
#[derive(Debug, Clone, PartialEq)]
pub struct ClimbTower {
    pub tower: usize,
    pub kind: ClimbKind,
    pub stop_floors: Vec<u32>,
    pub stop_kinds: Vec<StopKind>,
    pub deck_y: f32,
}

/// One vertical ladder rail emitted by a climbable tower or rooftop link.
#[derive(Debug, Clone, PartialEq)]
pub struct ClimbLadder {
    pub foot: Vec3,
    pub deck: Vec3,
    pub id: String,
}

/// Extra world-space box collider (e.g. the landmark mast platform and rails).
#[derive(Debug, Clone, PartialEq)]
pub struct ClimbBox {
    pub centre: Vec3,
    pub yaw: f32,
    pub half: Vec3,
}

/// Chunk contents emitted by climbable towers, rooftops or fire escapes.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ClimbChunk {
    pub pois: Vec<PoiInstance>,
    pub props: Vec<PropInstance>,
    pub crates: Vec<LootSpot>,
    pub ladders: Vec<ClimbLadder>,
    pub boxes: Vec<ClimbBox>,
}

/// One crate entry for a stop-off floor interior (`data/climb_floors.json`, R43-04).
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct FloorCrateEntry {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub yaw: f32,
    pub model: String,
    #[serde(default)]
    pub table: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ClimbTowersCount {
    pub midtown: usize,
    pub core: usize,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct StopFloorsConfig {
    pub midtown: usize,
    pub core: usize,
    pub landmark: usize,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct StopKindWeight {
    pub kind: StopKind,
    pub weight: f32,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MastConfig {
    pub platform_y: f32,
    pub platform_half: f32,
    pub ladder_foot: [f32; 2],
    pub ladder_top: [f32; 2],
}

/// Climb configuration (`climb` in `data/metro.json`).
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ClimbConfig {
    pub towers: ClimbTowersCount,
    pub min_size: u32,
    pub min_spacing_blocks: i32,
    pub stop_floors: StopFloorsConfig,
    pub stop_kinds: Vec<StopKindWeight>,
    pub stair_b_chance: f32,
    pub stair_zombie_chance: f32,
    pub shaft_centre_z_share: f32,
    pub shaft_size: [f32; 2],
    pub hatch: [f32; 2],
    /// Lobby desk spot on climb towers (x, z shares of the tower size), clear of the stair door.
    pub lobby_desk: [f32; 2],
    pub mast: MastConfig,
}

impl Default for ClimbConfig {
    fn default() -> Self {
        Self {
            towers: ClimbTowersCount { midtown: 2, core: 2 }, // data: metro.json climb
            min_size: 24,                                     // data: metro.json minSize
            min_spacing_blocks: 3,                            // data: metro.json minSpacingBlocks
            stop_floors: StopFloorsConfig { midtown: 2, core: 3, landmark: 4 }, // data: metro.json stopFloors
            stop_kinds: vec![
                StopKindWeight { kind: StopKind::Office, weight: 0.45 }, // data: metro.json stopKinds
                StopKindWeight { kind: StopKind::Flat, weight: 0.35 },   // data: metro.json stopKinds
                StopKindWeight { kind: StopKind::Camp, weight: 0.2 },    // data: metro.json stopKinds
            ],
            stair_b_chance: 0.3,                              // data: metro.json stairBChance
            stair_zombie_chance: 0.12,                        // data: metro.json stairZombieChance
            shaft_centre_z_share: 0.25,                       // data: metro.json shaftCentreZShare
            shaft_size: [4.4, 6.4],                           // data: metro.json shaftSize
            hatch: [3.4, 0.0],                                // data: metro.json hatch
            lobby_desk: [0.18, -0.30],                        // data: metro.json lobbyDesk
            mast: MastConfig {
                platform_y: 18.0,                             // data: metro.json mast
                platform_half: 3.0,                           // data: metro.json platformHalf
                ladder_foot: [0.0, 3.2],                      // data: metro.json ladderFoot
                ladder_top: [0.0, 2.6],                       // data: metro.json ladderTop
            },
        }
    }
}

/// Walkable roof deck height in world space (`ground_y` + local deck height).
pub fn roof_deck_y(t: &TowerSpot, style: &StyleDef, ground_y: f32) -> f32 {
    let fh = if style.floor_height > 0.0 { style.floor_height } else { DEFAULT_FLOOR_HEIGHT };
    let crown_y = ground_y + style.base_height + t.floors as f32 * fh;
    if t.landmark { crown_y + LANDMARK_MAST_DECK_HEIGHT } else { crown_y + style.roof_deck_height }
}

/// Picks the 5 climbable towers (2 Midtown, 2 Core + tallest landmark), memoised per layout.
pub fn climb_towers(metro: &MetroWorld) -> Vec<ClimbTower> {
    struct Memo {
        key: (Vec2, f32, f32, usize, usize, usize),
        towers: Rc<Vec<ClimbTower>>,
    }
    thread_local! {
        static MEMO: RefCell<Option<Memo>> = const { RefCell::new(None) };
    }
    let s = &metro.layout.site;
    let cfg = &metro.config.climb;
    let key = (s.center, s.yaw, s.half_size, metro.towers.len(), cfg.towers.midtown, cfg.towers.core);
    if let Some(hit) = MEMO.with(|m| m.borrow().as_ref().filter(|x| x.key == key).map(|x| Rc::clone(&x.towers))) {
        return (*hit).clone();
    }
    let computed = Rc::new(compute_climb_towers(metro));
    MEMO.with(|m| *m.borrow_mut() = Some(Memo { key, towers: Rc::clone(&computed) }));
    (*computed).clone()
}

fn compute_climb_towers(metro: &MetroWorld) -> Vec<ClimbTower> {
    let cfg = &metro.config.climb;
    let styles = default_styles();
    let mut rng = DotnetRandom::new(mix_seed(&metro.layout.site, 0, b"climb"));
    let safe_block = metro.layout.blocks.iter().find(|b| b.reserved == Some(Reserved::SafeHouseTower)).map(|b| (b.ix, b.iz));

    let mut midtown_cands = Vec::new();
    let mut core_cands = Vec::new();
    for (i, t) in metro.towers.iter().enumerate() {
        if t.landmark || t.size < cfg.min_size || metro.damage.tower_is_replaced(i)
            || metro.damage.tower_is_burnt(t.block) || safe_block == Some(t.block)
        {
            continue;
        }
        let ave = faces_avenue(&metro.layout, t, metro.config.towers.lot_gap + metro.config.avenue_width * 0.5);
        match t.kind {
            TowerKind::Midrise => midtown_cands.push((i, ave)),
            TowerKind::Highrise => core_cands.push((i, ave)),
            _ => {}
        }
    }

    let mut picked_blocks: Vec<(i32, i32)> = Vec::new();
    let mut out = Vec::new();
    for (cands, want, kind, stop_count) in [
        (&mut midtown_cands, cfg.towers.midtown, ClimbKind::Midtown, cfg.stop_floors.midtown),
        (&mut core_cands, cfg.towers.core, ClimbKind::Core, cfg.stop_floors.core),
    ] {
        for idx in pick_district_towers(cands, &metro.towers, want, cfg.min_spacing_blocks, &mut picked_blocks, &mut rng) {
            out.push(build_climb_tower(idx, &metro.towers[idx], kind, stop_count, styles, metro, &mut rng));
        }
    }

    if let Some(tallest) = metro.towers.iter().enumerate()
        .filter(|(i, t)| t.landmark && !metro.damage.tower_is_replaced(*i))
        .max_by(|(_, a), (_, b)| a.height.total_cmp(&b.height))
        .map(|(i, _)| i)
    {
        out.push(build_climb_tower(
            tallest, &metro.towers[tallest], ClimbKind::Landmark, cfg.stop_floors.landmark, styles, metro, &mut rng,
        ));
    }
    out
}

fn pick_district_towers(
    cands: &mut Vec<(usize, bool)>,
    towers: &[TowerSpot],
    want: usize,
    spacing: i32,
    picked_blocks: &mut Vec<(i32, i32)>,
    rng: &mut DotnetRandom,
) -> Vec<usize> {
    let mut chosen = Vec::new();
    for prefer_ave in [true, false] {
        while chosen.len() < want {
            let pool: Vec<usize> = cands.iter().enumerate()
                .filter(|(_, (idx, ave))| (!prefer_ave || *ave) && far_enough(towers[*idx].block, picked_blocks, spacing))
                .map(|(pos, _)| pos)
                .collect();
            if pool.is_empty() {
                break;
            }
            let pos = pool[rng.next(pool.len() as i32) as usize];
            let (idx, _) = cands.swap_remove(pos);
            picked_blocks.push(towers[idx].block);
            chosen.push(idx);
        }
    }
    chosen
}

fn far_enough(block: (i32, i32), picked: &[(i32, i32)], spacing: i32) -> bool {
    picked.iter().all(|b| (block.0 - b.0).abs().max((block.1 - b.1).abs()) >= spacing)
}

fn faces_avenue(layout: &MetroLayout, t: &TowerSpot, probe_dist: f32) -> bool {
    let front = Vec2::new(-t.yaw.sin(), -t.yaw.cos());
    layout.street_at(t.pos + front * (t.half.y + probe_dist)).is_some_and(|s| s.avenue)
}

fn build_climb_tower(
    idx: usize,
    t: &TowerSpot,
    kind: ClimbKind,
    stop_count: usize,
    styles: &[StyleDef],
    metro: &MetroWorld,
    rng: &mut DotnetRandom,
) -> ClimbTower {
    let style = style_for(styles, t.style).unwrap_or(&styles[0]);
    let stop_floors = pick_stop_floors(t.floors, stop_count);
    let mut stop_kinds: Vec<StopKind> = (0..stop_floors.len())
        .map(|_| pick_weighted_stop(&metro.config.climb.stop_kinds, rng))
        .collect();
    if kind == ClimbKind::Landmark && let Some(last) = stop_kinds.last_mut() {
        *last = StopKind::Camp;
    }
    ClimbTower { tower: idx, kind, stop_floors, stop_kinds, deck_y: roof_deck_y(t, style, 0.0) }
}

fn pick_stop_floors(floors: u32, count: usize) -> Vec<u32> {
    let top = floors.saturating_sub(1);
    if top <= 2 || count == 0 {
        return vec![top];
    }
    let lo = ((floors as f32 * 0.25).ceil() as u32).max(1); // new: DECISIONS 2026-11-27 (25%..85% stop floors)
    let hi = ((floors as f32 * 0.85).floor() as u32).min(top.saturating_sub(2)).max(lo); // new: DECISIONS 2026-11-27
    let mut out = Vec::with_capacity(count + 1);
    if count == 1 {
        out.push((lo + hi) / 2);
    } else {
        for k in 0..count {
            let target = lo + ((k as u32) * (hi - lo) + (count as u32 - 1) / 2) / (count as u32 - 1);
            let min_allowed = out.last().map_or(1, |&p| p + 2);
            let f = target.max(min_allowed).min(top.saturating_sub(2));
            if out.last().is_none_or(|&p| f >= p + 2) {
                out.push(f);
            }
        }
    }
    out.push(top);
    out
}

fn pick_weighted_stop(weights: &[StopKindWeight], rng: &mut DotnetRandom) -> StopKind {
    let total: f32 = weights.iter().map(|w| w.weight.max(0.0)).sum();
    if total <= 0.0 {
        return StopKind::Office;
    }
    let mut roll = rng.next_double() as f32 * total;
    for w in weights {
        roll -= w.weight.max(0.0);
        if roll <= 0.0 {
            return w.kind;
        }
    }
    weights.last().map_or(StopKind::Office, |w| w.kind)
}

/// Emits stairwell modules, floor numbers, lobby sign, stop-floor POIs, hatch and mast for `chunk`.
pub fn climb_in_chunk(
    metro: &MetroWorld,
    chunk: (i32, i32),
    height: &dyn Fn(f32, f32) -> f32,
) -> ClimbChunk {
    climb_in_chunk_with(metro, chunk, height, &|_| &[])
}

/// Same as `climb_in_chunk`, with an explicit lookup for stop-floor interior crates (R43-04).
pub fn climb_in_chunk_with<'a>(
    metro: &MetroWorld,
    chunk: (i32, i32),
    height: &dyn Fn(f32, f32) -> f32,
    floor_crates: &dyn Fn(&str) -> &'a [FloorCrateEntry],
) -> ClimbChunk {
    let mut out = ClimbChunk::default();
    let cfg = &metro.config.climb;
    let styles = default_styles();
    for plan in climb_towers(metro) {
        let t = &metro.towers[plan.tower];
        if !in_chunk(t.pos.x, t.pos.y, chunk) {
            continue;
        }
        let style = style_for(styles, t.style).unwrap_or(&styles[0]);
        let fh = if style.floor_height > 0.0 { style.floor_height } else { DEFAULT_FLOOR_HEIGHT };
        let gy = height(t.pos.x, t.pos.y);
        let shaft_z = cfg.shaft_centre_z_share * t.size as f32;

        out.props.push(PropInstance::new("tw_stair_open", local_to_world(t, 0.0, 0.0, shaft_z, gy), t.yaw, 1.0));
        out.props.push(PropInstance::new("tw_stair_flight", local_to_world(t, 0.0, fh, shaft_z, gy), t.yaw, 1.0));
        // Lobby "ROOF" sign on the front exterior wall beside the 2.8 m door gap.
        out.props.push(PropInstance::new("stair_sign_roof", local_to_world(t, 2.0, 2.0, -t.half.y - 0.05, gy), t.yaw, 1.0)); // new: DECISIONS 2026-11-27

        let mut rng = DotnetRandom::new(mix_seed(&metro.layout.site, plan.tower as u64, b"stair"));
        for i in 0..t.floors {
            let fy = style.base_height + i as f32 * fh;
            let pos = local_to_world(t, 0.0, fy, shaft_z, gy);
            if i + 1 == t.floors {
                out.props.push(PropInstance::new("tw_stair_top", pos, t.yaw, 1.0));
            } else if plan.stop_floors.contains(&i) {
                out.props.push(PropInstance::new("tw_stair_open", pos, t.yaw, 1.0));
            } else if rng.next_double() < f64::from(cfg.stair_zombie_chance) {
                out.pois.push(PoiInstance::new(format!("metro_stair_{}_{i}", plan.tower), "tw_stair_flight_z", pos, t.yaw));
            } else {
                let model = if rng.next_double() < f64::from(cfg.stair_b_chance) { "tw_stair_flight_b" } else { "tw_stair_flight" };
                out.props.push(PropInstance::new(model, pos, t.yaw, 1.0));
            }
            push_floor_digits(&mut out.props, t, i + 2, fy, shaft_z, gy);
        }

        for (&fi, &sk) in plan.stop_floors.iter().zip(&plan.stop_kinds) {
            let fy = style.base_height + fi as f32 * fh;
            let sz = module_size_at(t, fi, style, &metro.config.towers.setbacks);
            let model = format!("tw_interior_{}_{sz}", sk.key());
            let poi = PoiInstance::new(format!("metro_floor_{}_{fi}", plan.tower), &model, local_to_world(t, 0.0, fy, 0.0, gy), t.yaw)
                .with_loot_table(format!("metro_tower_{}", sk.key()));
            out.pois.push(poi);
            for (n, c) in floor_crates(&model).iter().enumerate() {
                let mut spot = LootSpot::new(format!("metro_floor_{}_{fi}_c{n}", plan.tower), local_to_world(t, c.x, fy + c.y, c.z, gy), t.yaw + c.yaw, BiomeId::Plains);
                spot.table = Some(c.table.clone().unwrap_or_else(|| format!("metro_tower_{}", sk.key())));
                spot.model = Some(c.model.clone());
                out.crates.push(spot);
            }
        }

        let top_y = style.base_height + t.floors.saturating_sub(1) as f32 * fh;
        let (hx, hz) = (cfg.hatch[0], shaft_z + cfg.hatch[1]);
        out.ladders.push(ClimbLadder {
            foot: local_to_world(t, hx, top_y, hz, gy),
            deck: local_to_world(t, hx + HATCH_CLEAR_STEP_M, plan.deck_y, hz, gy),
            id: format!("metro_hatch_{}", plan.tower),
        });
        push_ladder_segs(&mut out.props, t, hx, hz, top_y, plan.deck_y, gy);
        out.props.push(PropInstance::new("roof_hatch", local_to_world(t, hx, plan.deck_y, hz, gy), t.yaw, 1.0));

        if plan.kind == ClimbKind::Landmark {
            push_mast(&mut out, t, style, plan.tower, plan.deck_y, gy, cfg);
        }
    }
    out
}

fn push_floor_digits(props: &mut Vec<PropInstance>, t: &TowerSpot, num: u32, fy: f32, shaft_z: f32, gy: f32) {
    let (ly, lz) = (fy + 1.6, shaft_z - 2.98); // new: DECISIONS 2026-11-27 (inside landing -Z wall)
    if num >= 10 {
        props.push(PropInstance::new(format!("stair_digit_{}", (num / 10) % 10), local_to_world(t, 1.0, ly, lz, gy), t.yaw, 1.0));
        props.push(PropInstance::new(format!("stair_digit_{}", num % 10), local_to_world(t, 1.25, ly, lz, gy), t.yaw, 1.0)); // new: DECISIONS 2026-11-27 (digit spacing)
    } else {
        props.push(PropInstance::new(format!("stair_digit_{num}"), local_to_world(t, 1.0, ly, lz, gy), t.yaw, 1.0));
    }
}

fn push_ladder_segs(props: &mut Vec<PropInstance>, t: &TowerSpot, lx: f32, lz: f32, y0: f32, y1: f32, gy: f32) {
    let mut y = y0;
    while y < y1 {
        props.push(PropInstance::new("roof_ladder_seg", local_to_world(t, lx, y, lz, gy), t.yaw, 1.0));
        y += 1.0;
    }
}

fn push_mast(out: &mut ClimbChunk, t: &TowerSpot, style: &StyleDef, tower: usize, deck_y: f32, gy: f32, cfg: &ClimbConfig) {
    let fh = if style.floor_height > 0.0 { style.floor_height } else { DEFAULT_FLOOR_HEIGHT };
    let crown_y = style.base_height + t.floors as f32 * fh;
    let plat_top = crown_y + cfg.mast.platform_y + MAST_PLATFORM_SLAB_T;
    let [fx, fz] = cfg.mast.ladder_foot;
    let [tx, tz] = cfg.mast.ladder_top;
    out.ladders.push(ClimbLadder {
        foot: local_to_world(t, fx, deck_y, fz, gy),
        deck: local_to_world(t, tx, plat_top, tz, gy),
        id: format!("metro_mast_{tower}"),
    });
    push_ladder_segs(&mut out.props, t, fx, fz, deck_y, plat_top, gy);

    let ph = cfg.mast.platform_half;
    let slab_y = crown_y + cfg.mast.platform_y + MAST_PLATFORM_SLAB_T * 0.5;
    out.boxes.push(ClimbBox { centre: local_to_world(t, 0.0, slab_y, 0.0, gy), yaw: t.yaw, half: Vec3::new(ph, MAST_PLATFORM_SLAB_T * 0.5, ph) });
    let rail_y = (plat_top + crown_y + cfg.mast.platform_y + MAST_RAIL_TOP) * 0.5;
    let rail_hy = (crown_y + cfg.mast.platform_y + MAST_RAIL_TOP - plat_top) * 0.5;
    let r_pos = ph - 0.1; // new: DECISIONS 2026-11-27 (landmarks.py rails at +-2.9)
    for (cx, cz, hx, hz) in [
        (0.0, -r_pos, ph, MAST_RAIL_HALF_T),
        (-r_pos, 0.0, MAST_RAIL_HALF_T, ph),
        (r_pos, 0.0, MAST_RAIL_HALF_T, ph),
        (-1.8, r_pos, 1.2, MAST_RAIL_HALF_T), // new: DECISIONS 2026-11-27 (+Z rail split for ladder gap)
        (1.8, r_pos, 1.2, MAST_RAIL_HALF_T),  // new: DECISIONS 2026-11-27
    ] {
        out.boxes.push(ClimbBox { centre: local_to_world(t, cx, rail_y, cz, gy), yaw: t.yaw, half: Vec3::new(hx, rail_hy, hz) });
    }

    out.props.push(PropInstance::new("radio_console", local_to_world(t, -0.2, plat_top, -2.1, gy), t.yaw, 1.0)); // new: DECISIONS 2026-11-27
    let mut crate_spot = LootSpot::new("metro_radio_crate", local_to_world(t, -2.1, plat_top, -2.1, gy), t.yaw, BiomeId::Plains);
    crate_spot.model = Some("army_crate".to_string());
    crate_spot.table = Some("metro_radio".to_string());
    out.crates.push(crate_spot);
}

pub(crate) fn module_size_at(t: &TowerSpot, floor: u32, style: &StyleDef, setbacks: &[SetbackSpec]) -> u32 {
    if !t.landmark {
        return t.size;
    }
    let mut active: Option<(u32, u32)> = None;
    for sb in setbacks {
        let sb_floor = (t.floors as f32 * sb.at_floor).round() as u32;
        if floor >= sb_floor && active.is_none_or(|(f, _)| sb_floor >= f) {
            let target = t.size as f32 * sb.footprint_share;
            let sz = style.sizes.iter().copied().filter(|&s| (s as f32) <= target).max()
                .unwrap_or_else(|| *style.sizes.iter().min().unwrap_or(&24)); // data: metro.json minSize
            active = Some((sb_floor, sz));
        }
    }
    active.map_or(t.size, |(_, sz)| sz)
}

fn local_to_world(t: &TowerSpot, lx: f32, ly: f32, lz: f32, ground_y: f32) -> Vec3 {
    let (sin_y, cos_y) = t.yaw.sin_cos();
    Vec3::new(t.pos.x + lx * cos_y + lz * sin_y, ground_y + ly, t.pos.y - lx * sin_y + lz * cos_y)
}

fn mix_seed(site: &MetroSite, index: u64, salt: &[u8]) -> i32 {
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
    feed(&index.to_le_bytes());
    feed(salt);
    hash as i32
}
