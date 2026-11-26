//! Pure, deterministic assembly of a `TowerSpot` into stacked modular kit parts.

use std::sync::OnceLock;

use glam::Vec3;

use crate::metro::config::{LobbyProp, MetroConfig, SetbackSpec};
use crate::metro::styles::StyleDef;
use crate::metro::towers::TowerSpot;
use crate::random::DotnetRandom;

/// Fallback floor height when `StyleDef.floor_height` is unset.
const DEFAULT_FLOOR_HEIGHT: f32 = 3.8; // new: DECISIONS 2026-11-26 (fallback for metro.json towers.floorHeight)

/// Landmark crown model ids (non-tallest).
const LANDMARK_CROWNS: [&str; 3] = [
    "tw_landmark_spire",
    "tw_landmark_stepped",
    "tw_landmark_glass_top",
];

/// Tallest-landmark crown model id (radio goal).
const TALLEST_LANDMARK_CROWN: &str = "tw_landmark_mast";

/// Role of one modular piece in a tower stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PartKind {
    Base,
    Floor,
    Crown,
    Setback,
    LandmarkCrown,
    Interior,
}

/// One placed modular kit piece of a tower in world space.
#[derive(Debug, Clone, PartialEq)]
pub struct TowerPart {
    pub model: String,
    pub pos: Vec3,
    pub yaw: f32,
    pub kind: PartKind,
}

struct DefaultTowerExtras {
    setbacks: Vec<SetbackSpec>,
    lobby_props: Vec<LobbyProp>,
}

fn default_extras() -> &'static DefaultTowerExtras {
    static CACHE: OnceLock<DefaultTowerExtras> = OnceLock::new();
    CACHE.get_or_init(|| {
        let cfg = MetroConfig::load_from_file(paths::repo_path("data/metro.json"))
            .expect("data/metro.json must load");
        DefaultTowerExtras {
            setbacks: cfg.towers.setbacks,
            lobby_props: cfg.towers.lobby_props,
        }
    })
}

/// Builds every stacked part (base, floors/setbacks, crown, lobby interior) for `spot`.
/// Pure and deterministic for a given `(spot, style, ground_y, rng_seed)`.
pub fn tower_parts(
    spot: &TowerSpot,
    style: &StyleDef,
    ground_y: f32,
    rng_seed: u64,
) -> Vec<TowerPart> {
    let extras = default_extras();
    tower_parts_with_extras(
        spot,
        style,
        ground_y,
        rng_seed,
        &extras.setbacks,
        &extras.lobby_props,
        false,
    )
}

/// Same as `tower_parts`, with explicit setbacks, lobby props and tallest-landmark flag.
pub fn tower_parts_with_extras(
    spot: &TowerSpot,
    style: &StyleDef,
    ground_y: f32,
    rng_seed: u64,
    setbacks: &[SetbackSpec],
    lobby_props: &[LobbyProp],
    is_tallest_landmark: bool,
) -> Vec<TowerPart> {
    let mut parts = Vec::with_capacity(spot.floors as usize + lobby_props.len() + 2);
    let style_key = style.key.key();
    let base_size = spot.size;
    let fh = if style.floor_height > 0.0 {
        style.floor_height
    } else {
        DEFAULT_FLOOR_HEIGHT
    };

    parts.push(TowerPart {
        model: format!("tw_{style_key}_{base_size}_base"),
        pos: Vec3::new(spot.pos.x, ground_y, spot.pos.y),
        yaw: spot.yaw,
        kind: PartKind::Base,
    });

    let (sin_y, cos_y) = spot.yaw.sin_cos();
    for prop in lobby_props {
        let lx = prop.x * base_size as f32;
        let lz = prop.z * base_size as f32;
        let wx = spot.pos.x + lx * cos_y + lz * sin_y;
        let wz = spot.pos.y - lx * sin_y + lz * cos_y;
        parts.push(TowerPart {
            model: prop.model.clone(),
            pos: Vec3::new(wx, ground_y, wz),
            yaw: spot.yaw + prop.yaw,
            kind: PartKind::Interior,
        });
    }

    let mut rng = DotnetRandom::new((rng_seed ^ (rng_seed >> 32)) as i32); // new: DECISIONS 2026-11-26
    let mut sb_points: Vec<(u32, u32)> = if spot.landmark {
        setbacks
            .iter()
            .map(|sb| {
                let floor = (spot.floors as f32 * sb.at_floor).round() as u32;
                let target = base_size as f32 * sb.footprint_share;
                let size = snap_setback_size(target, style);
                (floor, size)
            })
            .collect()
    } else {
        Vec::new()
    };
    sb_points.sort_by_key(|&(f, _)| f);

    for i in 0..spot.floors {
        let active_sb = sb_points.iter().rfind(|&&(f, _)| i >= f);
        let (cur_size, kind) = match active_sb {
            Some(&(_, sz)) => (sz, PartKind::Setback),
            None => (base_size, PartKind::Floor),
        };
        let suffix = if rng.next(2) == 0 { "_floor" } else { "_floor_b" };
        let y = ground_y + style.base_height + i as f32 * fh;
        parts.push(TowerPart {
            model: format!("tw_{style_key}_{cur_size}{suffix}"),
            pos: Vec3::new(spot.pos.x, y, spot.pos.y),
            yaw: spot.yaw,
            kind,
        });
    }

    let crown_y = ground_y + style.base_height + spot.floors as f32 * fh;
    if spot.landmark {
        let crown_model = if is_tallest_landmark {
            TALLEST_LANDMARK_CROWN.to_string()
        } else {
            let idx = rng.next(LANDMARK_CROWNS.len() as i32) as usize;
            LANDMARK_CROWNS[idx].to_string()
        };
        parts.push(TowerPart {
            model: crown_model,
            pos: Vec3::new(spot.pos.x, crown_y, spot.pos.y),
            yaw: spot.yaw,
            kind: PartKind::LandmarkCrown,
        });
    } else {
        parts.push(TowerPart {
            model: format!("tw_{style_key}_{base_size}_crown"),
            pos: Vec3::new(spot.pos.x, crown_y, spot.pos.y),
            yaw: spot.yaw,
            kind: PartKind::Crown,
        });
    }

    parts
}

/// Snaps `target` down to the largest module size `<= target` in `style.sizes`,
/// or the smallest size in `style.sizes` so we never emit a missing model id.
fn snap_setback_size(target: f32, style: &StyleDef) -> u32 {
    if let Some(&sz) = style
        .sizes
        .iter()
        .filter(|&&s| (s as f32) <= target)
        .max()
    {
        return sz;
    }
    *style.sizes.iter().min().unwrap_or(&24) // new: DECISIONS 2026-11-26
}
