//! The failed army line on its reserved 4x1 strip (R42-02): site, layout data and per-chunk
//! placement. Pure logic, no GPU.
//!
//! Local data frame (`data/places/army_line.json`): `x` runs along the avenue the line blocks
//! (the strip centre = 0), `z` runs across it (0 = avenue centre line, + = toward the strip),
//! `y` is up from the ground, all in game metres.

use std::sync::OnceLock;

use glam::{Vec2, Vec3};
use serde::Deserialize;

use crate::metro::layout::{MetroBlock, MetroLayout, MetroStreet, Reserved};
use crate::metro::loot_helpers::{in_chunk, make_spot};
use crate::metro::state::MetroWorld;
use crate::poi::PoiInstance;
use crate::scatter::PropInstance;

/// Project path of the layout data.
pub const ARMY_LINE_DATA: &str = "data/places/army_line.json";
/// The one army-line POI (its zombie spawn points come from `data/poi.json`, R42-07).
pub const ARMY_LINE_POI_ID: &str = "metro_army_line";
/// Loot table of that POI. // data: loot_tables.json metro_army
pub const ARMY_LINE_POI_TABLE: &str = "metro_army";

const EPS: f32 = 1e-3; // new: DECISIONS 2026-11-27 (grid-line comparison slack)
const SCALE_ONE: f32 = 1.0; // new: DECISIONS 2026-11-27 (props are built at full size)

use crate::metro::places::PlaceChunk;

/// Where the army line sits: `origin` is the strip centre point on its avenue (world XZ),
/// `dir` is the avenue direction (local +x, unit), `yaw` is the Godot yaw of that direction,
/// and `strip` lists the reserved blocks.
#[derive(Debug, Clone, PartialEq)]
pub struct ArmyLineSite {
    pub origin: Vec2,
    pub dir: Vec2,
    pub yaw: f32,
    pub strip: Vec<(i32, i32)>,
}

impl ArmyLineSite {
    /// Local data `(x, z)` to world XZ: `x` along `dir`, `z` across it toward the strip.
    pub fn to_world(&self, x: f32, z: f32) -> Vec2 {
        self.origin + self.dir * x + right_of(self.dir) * z
    }
}

/// World yaw for a prop's data yaw (0 = the prop's front points up the avenue, `+x`).
pub fn army_line_yaw(site: &ArmyLineSite, data_yaw: f32) -> f32 {
    site.yaw + data_yaw
}

/// Finds the avenue (or, failing that, the plain street) along the reserved `army_line` strip's
/// long side and returns the line's frame. `None` when the strip is absent.
pub fn army_line_site(layout: &MetroLayout) -> Option<ArmyLineSite> {
    let blocks: Vec<&MetroBlock> = layout
        .blocks
        .iter()
        .filter(|b| b.reserved == Some(Reserved::ArmyLine))
        .collect();
    let first = *blocks.first()?;
    let (mut min_x, mut max_x) = (first.rect.0.x, first.rect.1.x);
    let (mut min_y, mut max_y) = (first.rect.0.y, first.rect.1.y);
    for b in &blocks {
        min_x = min_x.min(b.rect.0.x);
        max_x = max_x.max(b.rect.1.x);
        min_y = min_y.min(b.rect.0.y);
        max_y = max_y.max(b.rect.1.y);
    }
    let long_is_x = (max_x - min_x) >= (max_y - min_y);
    let (side, _avenue) = long_side_street(layout, long_is_x, (min_x, max_x), (min_y, max_y))?;

    let mid_x = (min_x + max_x) * 0.5;
    let mid_y = (min_y + max_y) * 0.5;
    let local_origin = if long_is_x { Vec2::new(mid_x, side) } else { Vec2::new(side, mid_y) };
    let origin = layout.site.to_world(local_origin);
    let along = if long_is_x { Vec2::X } else { Vec2::Y };
    let mut dir = (layout.site.to_world(local_origin + along) - origin).normalize_or(Vec2::X);
    // Keep local +z pointing at the strip, so "behind the barrier" is always the strip side.
    let strip_center = layout.site.to_world(Vec2::new(mid_x, mid_y));
    if right_of(dir).dot(strip_center - origin) < 0.0 {
        dir = -dir;
    }
    Some(ArmyLineSite {
        origin,
        dir,
        yaw: dir_yaw(dir),
        strip: blocks.iter().map(|b| (b.ix, b.iz)).collect(),
    })
}

/// POI-local zombie spawn offsets for `metro_army_line` (which has no building `.glb`), derived
/// from `data/places/army_line.json` around the barrier, tank, crashed helicopter and evac tents.
pub fn army_line_spawn_offsets() -> Vec<Vec3> {
    let layout = default_army_line();
    let bx = layout.barrier_x;
    // POI yaw is `site.yaw` (local -Z = avenue `dir`, local +X = `right_of(dir)`).
    layout
        .props
        .iter()
        .filter(|p| {
            matches!(
                p.model.as_str(),
                "evac_tent" | "army_tank" | "heli_crashed" | "checkpoint_booth" | "sandbag_corner"
            )
        })
        .map(|p| Vec3::new(p.z, p.y, -(p.x - bx)))
        .collect()
}

/// The barrier wall plane plus its two ends: points herds and the car-wall placement must avoid.
pub fn army_line_blocked_points(metro: &MetroWorld) -> Vec<Vec2> {
    let Some(site) = army_line_site(&metro.layout) else {
        return Vec::new();
    };
    let layout = default_army_line();
    let x = layout.barrier_x;
    let half_w = metro
        .layout
        .street_at(site.to_world(x, 0.0))
        .map_or(metro.config.avenue_width, |s| s.width)
        * 0.5;
    vec![site.to_world(x, -half_w), site.to_world(x, 0.0), site.to_world(x, half_w)]
}

/// Every army-line POI, prop and crate whose centre falls in `chunk`, at `height(x, z) + data y`.
pub fn army_line_in_chunk(
    metro: &MetroWorld,
    chunk: (i32, i32),
    height: &dyn Fn(f32, f32) -> f32,
) -> PlaceChunk {
    let Some(site) = army_line_site(&metro.layout) else {
        return PlaceChunk::default();
    };
    let layout = default_army_line();
    let mut out = PlaceChunk::default();

    let barrier_mid = site.to_world(layout.barrier_x, 0.0);
    if in_chunk(barrier_mid.x, barrier_mid.y, chunk) {
        let y = height(barrier_mid.x, barrier_mid.y);
        out.pois.push(
            PoiInstance::new(
                ARMY_LINE_POI_ID,
                ARMY_LINE_POI_ID,
                Vec3::new(barrier_mid.x, y, barrier_mid.y),
                site.yaw,
            )
            .with_loot_table(ARMY_LINE_POI_TABLE),
        );
    }

    for p in &layout.props {
        let w = site.to_world(p.x, p.z);
        if !in_chunk(w.x, w.y, chunk) {
            continue;
        }
        let pos = Vec3::new(w.x, height(w.x, w.y) + p.y, w.y);
        out.props
            .push(PropInstance::new(p.model.as_str(), pos, army_line_yaw(&site, p.yaw), p.scale));
    }

    for c in &layout.crates {
        let w = site.to_world(c.x, c.z);
        if !in_chunk(w.x, w.y, chunk) {
            continue;
        }
        let pos = Vec3::new(w.x, height(w.x, w.y) + c.y, w.y);
        out.crates.push(make_spot("army_line", pos, army_line_yaw(&site, c.yaw), &c.table, Some(&c.model)));
    }
    out
}

/// One prop entry of `data/places/army_line.json`. `len` is the model's length along its own
/// long axis (model X); the layout check uses it for the barrier coverage, nothing else.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ArmyProp {
    pub x: f32,
    #[serde(default)]
    pub y: f32,
    pub z: f32,
    pub yaw: f32,
    pub model: String,
    #[serde(default = "scale_one")]
    pub scale: f32,
    #[serde(default)]
    pub len: f32,
}

/// One loot container entry (`army_crate` with `metro_army`, `med_cabinet` with `metro_medical`).
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ArmyCrate {
    pub x: f32,
    #[serde(default)]
    pub y: f32,
    pub z: f32,
    pub yaw: f32,
    pub model: String,
    pub table: String,
}

/// The whole army-line layout as data.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArmyLineLayout {
    /// Where the barrier wall stands along the avenue (`x` of its middle).
    #[serde(default)]
    pub barrier_x: f32,
    /// Models that make up the barrier wall (`sandbag_wall`, `concrete_barrier`).
    #[serde(default)]
    pub barrier_models: Vec<String>,
    #[serde(default)]
    pub props: Vec<ArmyProp>,
    #[serde(default)]
    pub crates: Vec<ArmyCrate>,
}

fn scale_one() -> f32 {
    SCALE_ONE
}

/// Parses an `army_line.json` string.
pub fn load_army_line(json: &str) -> Result<ArmyLineLayout, String> {
    let layout: ArmyLineLayout =
        serde_json::from_str(json).map_err(|e| format!("army_line.json: {e}"))?;
    validate_army_line(&layout)?;
    Ok(layout)
}

/// Checks that the data file has entries and no empty model names.
pub fn validate_army_line(layout: &ArmyLineLayout) -> Result<(), String> {
    if layout.props.is_empty() || layout.crates.is_empty() {
        return Err("army_line.json needs at least one prop and one crate".into());
    }
    let unnamed = layout
        .props
        .iter()
        .map(|p| &p.model)
        .chain(layout.crates.iter().map(|c| &c.model))
        .any(|m| m.is_empty());
    if unnamed {
        return Err("army_line.json: every entry needs a model name".into());
    }
    if layout.props.iter().any(|p| p.scale <= 0.0) || layout.crates.iter().any(|c| c.table.is_empty()) {
        return Err("army_line.json: scale > 0 and a loot table are required".into());
    }
    Ok(())
}

/// Cached layout loaded from `data/places/army_line.json`.
pub fn default_army_line() -> &'static ArmyLineLayout {
    static CACHE: OnceLock<ArmyLineLayout> = OnceLock::new();
    CACHE.get_or_init(|| {
        let path = paths::repo_path(ARMY_LINE_DATA);
        let json = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
        load_army_line(&json).expect("data/places/army_line.json must be valid")
    })
}

/// The first street running along the strip's long side, preferring an avenue.
fn long_side_street(
    layout: &MetroLayout,
    long_is_x: bool,
    x: (f32, f32),
    y: (f32, f32),
) -> Option<(f32, &MetroStreet)> {
    let sides = if long_is_x { [y.0, y.1] } else { [x.0, x.1] };
    let (lo, hi) = if long_is_x { x } else { y };
    let mut plain: Option<(f32, &MetroStreet)> = None;
    for side in sides {
        for s in &layout.streets {
            let along = if long_is_x { (s.b.y - s.a.y).abs() < EPS } else { (s.b.x - s.a.x).abs() < EPS };
            let line = if long_is_x { s.a.y } else { s.a.x };
            if !along || (line - side).abs() > EPS {
                continue;
            }
            let (t0, t1) = if long_is_x {
                (s.a.x.min(s.b.x), s.a.x.max(s.b.x))
            } else {
                (s.a.y.min(s.b.y), s.a.y.max(s.b.y))
            };
            if t1 <= lo + EPS || t0 >= hi - EPS {
                continue; // the segment does not touch the strip
            }
            if s.avenue {
                return Some((side, s));
            }
            plain.get_or_insert((side, s));
        }
    }
    plain
}

fn right_of(d: Vec2) -> Vec2 {
    Vec2::new(-d.y, d.x)
}

/// Godot yaw that faces `dir` (forward = `(-sin yaw, -cos yaw)`).
fn dir_yaw(d: Vec2) -> f32 {
    (-d.x).atan2(-d.y)
}
