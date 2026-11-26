//! Graffiti/posters, rooftop camps, lobby dressing and sidewalk bodies for `traces.rs`. Pure logic: no GPU.

use std::collections::HashSet;
use std::f32::consts::{FRAC_PI_2, PI};

use glam::{Vec2, Vec3};

use crate::metro::config::LobbyProp;
use crate::metro::layout::{MetroBlock, MetroStreet};
use crate::metro::state::MetroWorld;
use crate::metro::styles::{StyleDef, TowerStyle, default_styles, style_for};
use crate::metro::tower_parts::{PartKind, tower_parts};
use crate::metro::towers::TowerSpot;
use crate::random::DotnetRandom;
use crate::scatter::PropInstance;

use super::{
    PROP_SCALE, TraceConfig, default_traces, dir_yaw, forward_of, in_chunk, is_damage_street,
    mix_seed, right_of, rng_range_f, tower_seed,
};

/// Fallback floor height when `StyleDef.floor_height` is unset (matches `tower_parts.rs`).
const DEFAULT_FLOOR_HEIGHT: f32 = 3.8; // data: metro.json floorHeight
const ENDPOINT_EPS: f32 = 0.01; // new: DECISIONS 2026-11-26 (matches curbs.rs)
const BODY_MARGIN_M: f32 = 0.6; // new: DECISIONS 2026-11-26 (keeps body inside sidewalk band)
const BODY_END_MARGIN_M: f32 = 1.5; // new: DECISIONS 2026-11-26 (keeps body off curb ramps)
const BODY_YAW_JITTER: f32 = 0.25;
const LOBBY_GRAFFITI_X_SHARE: f32 = 0.28; // new: DECISIONS 2026-11-26 (clears 1.4 m door half-width)
const SIDE_GRAFFITI_X_SHARE: f32 = 0.15; // new: DECISIONS 2026-11-26 (keeps 4-6 m decal inside wall)
const STREET_PROBE_STEP_M: f32 = 2.0; // new: DECISIONS 2026-11-26
const STREET_PROBE_MAX_M: f32 = 28.0; // new: DECISIONS 2026-11-26
pub(super) const POSTER_MODEL: &str = "poster_wall";
const SIDEWALK_BODY_MODELS: [&str; 2] = ["body_sheet", "body_sheet_b"];
const GRAFFITI_MODELS: [&str; 6] = [ // new: DECISIONS 2026-11-26
    "graffiti_dead_inside",
    "graffiti_turn_back",
    "graffiti_x",
    "graffiti_arrow",
    "graffiti_no_food",
    "graffiti_went_north",
];

/// One placed rooftop survivor camp (`--metro=camp` reads this).
#[derive(Debug, Clone, PartialEq)]
pub struct CampSite {
    pub pos: Vec3,
    pub yaw: f32,
    pub tower: usize,
}

/// Lists every rooftop camp across New Halden using the city's ground height.
pub fn camp_sites(metro: &MetroWorld) -> Vec<CampSite> {
    let cfg = default_traces();
    let styles = default_styles();
    let ground_y = metro.layout.site.ground_y;
    metro
        .towers
        .iter()
        .enumerate()
        .filter_map(|(idx, tower)| {
            if metro.damage.tower_is_replaced(idx) || metro.damage.tower_is_burnt(tower.block) {
                return None;
            }
            camp_for_tower(metro, idx, tower, cfg, styles, ground_y)
        })
        .collect()
}

pub(super) fn camp_for_tower(
    metro: &MetroWorld,
    idx: usize,
    tower: &TowerSpot,
    cfg: &TraceConfig,
    styles: &[StyleDef],
    ground_y: f32,
) -> Option<CampSite> {
    if tower.style != TowerStyle::Brick
        || !(cfg.camp_floors[0]..=cfg.camp_floors[1]).contains(&tower.floors)
        || (tower.size as f32) < cfg.camp_size_m + 2.0 * cfg.camp_margin_m
    {
        return None;
    }
    let seed = tower_seed(&metro.layout.site, idx, tower.block, b"traces_camp");
    let mut rng = DotnetRandom::new(seed);
    if rng.next_double() >= f64::from(cfg.camp_chance) {
        return None;
    }
    let style = style_for(styles, tower.style)?;
    let roof_y = roof_top_y(tower, style, ground_y, seed as u64)?;
    let yaw = tower.yaw + rng.next(4) as f32 * FRAC_PI_2;
    Some(CampSite { pos: Vec3::new(tower.pos.x, roof_y, tower.pos.y), yaw, tower: idx })
}

fn roof_top_y(spot: &TowerSpot, style: &StyleDef, ground_y: f32, seed: u64) -> Option<f32> {
    let parts = tower_parts(spot, style, ground_y, seed);
    let last_floor = parts.iter().rfind(|p| p.kind == PartKind::Floor)?;
    let fh = if style.floor_height > 0.0 { style.floor_height } else { DEFAULT_FLOOR_HEIGHT };
    Some(last_floor.pos.y + fh + style.roof_deck_height) // data: metro.json styles.roofDeckHeight
}

/// Rolls `lobbyDressing` entries for one tower's ground-floor lobby (not used by `tower_parts`;
/// `traces_props_in_chunk` places them with their `y` height).
pub fn lobby_dressing(_spot: &TowerSpot, cfg: &TraceConfig, seed: u64) -> Vec<LobbyProp> {
    lobby_dressing_with_y(cfg, seed).into_iter().map(|(prop, _)| prop).collect()
}

fn lobby_dressing_with_y(cfg: &TraceConfig, seed: u64) -> Vec<(LobbyProp, f32)> {
    let mut rng = DotnetRandom::new((seed ^ (seed >> 32)) as i32); // new: DECISIONS 2026-11-26
    let mut out = Vec::new();
    for entry in &cfg.lobby_dressing {
        if rng.next_double() < f64::from(entry.chance) {
            out.push((
                LobbyProp { model: entry.model.clone(), x: entry.x, z: entry.z, yaw: entry.yaw },
                entry.y,
            ));
        }
    }
    out
}

pub(super) fn append_lobby_dressing(
    metro: &MetroWorld,
    idx: usize,
    tower: &TowerSpot,
    cfg: &TraceConfig,
    chunk: (i32, i32),
    height: &dyn Fn(f32, f32) -> f32,
    out: &mut Vec<PropInstance>,
) {
    let seed = tower_seed(&metro.layout.site, idx, tower.block, b"traces_lobby") as u64;
    let (sin_y, cos_y) = tower.yaw.sin_cos();
    let size = tower.size as f32;
    for (prop, y_off) in lobby_dressing_with_y(cfg, seed) {
        let lx = prop.x * size;
        let lz = prop.z * size;
        let wx = tower.pos.x + lx * cos_y + lz * sin_y;
        let wz = tower.pos.y - lx * sin_y + lz * cos_y;
        if in_chunk(wx, wz, chunk) {
            let pos = Vec3::new(wx, height(wx, wz) + y_off, wz);
            out.push(PropInstance::new(prop.model, pos, tower.yaw + prop.yaw, PROP_SCALE));
        }
    }
}

pub(super) fn append_tower_facade_traces(
    metro: &MetroWorld,
    idx: usize,
    tower: &TowerSpot,
    cfg: &TraceConfig,
    chunk: (i32, i32),
    height: &dyn Fn(f32, f32) -> f32,
    out: &mut Vec<PropInstance>,
) {
    let mut rng = DotnetRandom::new(tower_seed(&metro.layout.site, idx, tower.block, b"traces_facade"));
    let f = forward_of(tower.yaw);
    let r = right_of(f);
    let wall_dist = tower.half.x + cfg.wall_offset_m;
    let lobby_side = if rng.next(2) == 0 { -1.0_f32 } else { 1.0_f32 };

    if rng.next_double() < f64::from(cfg.graffiti_lobby_chance) {
        let model = GRAFFITI_MODELS[rng.next(GRAFFITI_MODELS.len() as i32) as usize];
        let pos2 = tower.pos + f * wall_dist + r * (lobby_side * LOBBY_GRAFFITI_X_SHARE * tower.size as f32);
        let y_off = rng_range_f(&mut rng, cfg.graffiti_height_m);
        if in_chunk(pos2.x, pos2.y, chunk) {
            out.push(PropInstance::new(model, Vec3::new(pos2.x, height(pos2.x, pos2.y) + y_off, pos2.y), tower.yaw, PROP_SCALE));
        }
    }

    if rng.next_double() < f64::from(cfg.poster_chance) {
        let share = rng_range_f(&mut rng, cfg.poster_x_share);
        let pos2 = tower.pos + f * wall_dist + r * (-lobby_side * share * tower.size as f32);
        let y_off = rng_range_f(&mut rng, cfg.poster_height_m);
        if in_chunk(pos2.x, pos2.y, chunk) {
            out.push(PropInstance::new(POSTER_MODEL, Vec3::new(pos2.x, height(pos2.x, pos2.y) + y_off, pos2.y), tower.yaw, PROP_SCALE));
        }
    }

    let side_faces = street_facing_sides(metro, tower);
    if !side_faces.is_empty() && rng.next_double() < f64::from(cfg.graffiti_street_wall_chance) {
        let (n, t, yaw) = side_faces[rng.next(side_faces.len() as i32) as usize];
        let model = GRAFFITI_MODELS[rng.next(GRAFFITI_MODELS.len() as i32) as usize];
        let x_off = (rng.next_double() as f32 * 2.0 - 1.0) * (SIDE_GRAFFITI_X_SHARE * tower.size as f32);
        let pos2 = tower.pos + n * wall_dist + t * x_off;
        let y_off = rng_range_f(&mut rng, cfg.graffiti_height_m);
        if in_chunk(pos2.x, pos2.y, chunk) {
            out.push(PropInstance::new(model, Vec3::new(pos2.x, height(pos2.x, pos2.y) + y_off, pos2.y), yaw, PROP_SCALE));
        }
    }
}

fn street_facing_sides(metro: &MetroWorld, tower: &TowerSpot) -> Vec<(Vec2, Vec2, f32)> {
    let f = forward_of(tower.yaw);
    let r = right_of(f);
    let candidates = [(r, -f, tower.yaw - FRAC_PI_2), (-f, -r, tower.yaw + PI), (-r, f, tower.yaw + FRAC_PI_2)];
    let mut out = Vec::new();
    for (n, t, yaw) in candidates {
        let mut d = tower.half.x + STREET_PROBE_STEP_M;
        while d <= STREET_PROBE_MAX_M {
            let p = tower.pos + n * d;
            if metro.in_tower_footprint(p) {
                break;
            }
            if metro.layout.street_at(p).is_some() {
                out.push((n, t, yaw));
                break;
            }
            d += STREET_PROBE_STEP_M;
        }
    }
    out
}

pub(super) fn append_sidewalk_bodies(
    metro: &MetroWorld,
    block: &MetroBlock,
    cfg: &TraceConfig,
    entrance_aves: &HashSet<usize>,
    chunk: (i32, i32),
    height: &dyn Fn(f32, f32) -> f32,
    out: &mut Vec<PropInstance>,
) {
    let mut rng = DotnetRandom::new(mix_seed(&metro.layout.site, Some((block.ix, block.iz)), b"traces_bodies"));
    if rng.next_double() >= f64::from(cfg.body_sidewalk_chance) {
        return;
    }
    let edges = block_sidewalk_edges(metro, block, entrance_aves);
    if edges.is_empty() {
        return;
    }
    let (s, inward_sign) = edges[rng.next(edges.len() as i32) as usize];
    let ab = s.b - s.a;
    let len = ab.length();
    if len <= ENDPOINT_EPS {
        return;
    }
    let dir = ab / len;
    let perp = Vec2::new(-dir.y, dir.x) * inward_sign;
    let ramp = metro.look.curb_width;
    let tf0 = endpoint_trim(s.a, dir, &metro.layout.streets) + ramp + BODY_END_MARGIN_M;
    let tf1 = len - endpoint_trim(s.b, dir, &metro.layout.streets) - ramp - BODY_END_MARGIN_M;
    let d_in = (s.width * 0.5 - metro.look.sidewalk_width).max(0.0) + BODY_MARGIN_M;
    let d_out = s.width * 0.5 - BODY_MARGIN_M;
    if tf1 <= tf0 || d_out <= d_in {
        return;
    }
    let t = rng_range_f(&mut rng, [tf0, tf1]);
    let d = rng_range_f(&mut rng, [d_in, d_out]);
    let w = metro.layout.site.to_world(s.a + dir * t + perp * d);
    if !in_chunk(w.x, w.y, chunk) {
        return;
    }
    let local = metro.layout.site.to_local(w);
    if let Some(hit) = metro.layout.street_at(w) {
        let hit_d = hit.closest_point(local).distance(local);
        if hit_d <= hit.width * 0.5 - metro.look.sidewalk_width || hit_d >= hit.width * 0.5 {
            return;
        }
    }
    let model = SIDEWALK_BODY_MODELS[rng.next(SIDEWALK_BODY_MODELS.len() as i32) as usize];
    let world_dir = (metro.layout.site.to_world(s.b) - metro.layout.site.to_world(s.a)).normalize();
    let yaw = dir_yaw(world_dir) + (rng.next_double() as f32 * 2.0 - 1.0) * BODY_YAW_JITTER;
    let pos = Vec3::new(w.x, height(w.x, w.y) + metro.look.curb_height, w.y);
    out.push(PropInstance::new(model, pos, yaw, PROP_SCALE));
}

fn block_sidewalk_edges<'a>(
    metro: &'a MetroWorld,
    block: &MetroBlock,
    entrance_aves: &HashSet<usize>,
) -> Vec<(&'a MetroStreet, f32)> {
    let (lo, hi) = block.rect;
    let center = (lo + hi) * 0.5;
    let edge_mids = [Vec2::new(lo.x, center.y), Vec2::new(hi.x, center.y), Vec2::new(center.x, lo.y), Vec2::new(center.x, hi.y)];
    metro
        .layout
        .streets
        .iter()
        .enumerate()
        .filter(|(idx, s)| !entrance_aves.contains(idx) && !is_damage_street(metro, s) && edge_mids.iter().any(|&em| em.distance((s.a + s.b) * 0.5) < 1.0))
        .map(|(_, s)| {
            let dir = (s.b - s.a).normalize_or_zero();
            let perp = Vec2::new(-dir.y, dir.x);
            let sign = if (center - (s.a + s.b) * 0.5).dot(perp) >= 0.0 { 1.0 } else { -1.0 };
            (s, sign)
        })
        .collect()
}

fn endpoint_trim(p: Vec2, dir: Vec2, streets: &[MetroStreet]) -> f32 {
    streets
        .iter()
        .filter(|other| {
            let odir = (other.b - other.a).normalize_or_zero();
            odir.dot(dir).abs() < 0.5
                && (other.a.distance(p) < ENDPOINT_EPS
                    || other.b.distance(p) < ENDPOINT_EPS
                    || other.contains_point(p))
        })
        .map(|other| other.width * 0.5)
        .fold(0.0_f32, f32::max)
}
