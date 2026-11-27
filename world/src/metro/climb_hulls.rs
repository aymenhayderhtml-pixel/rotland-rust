//! R43-01: Tower-local collider boxes for climbable towers (replaces the single upper box).
//! Pure logic, no GPU.

use glam::Vec3;

use crate::metro::climb::{ClimbTower, module_size_at, roof_deck_y};
use crate::metro::config::{MetroConfig, SetbackSpec};
use crate::metro::styles::StyleDef;
use crate::metro::towers::TowerSpot;

const DEFAULT_FLOOR_HEIGHT: f32 = 3.8; // data: metro.json floorHeight
const SHAFT_HALF_X: f32 = 2.2; // new: DECISIONS 2026-11-27 (shaftSize[0] * 0.5)
const SHAFT_HALF_Z: f32 = 3.2; // new: DECISIONS 2026-11-27 (shaftSize[1] * 0.5)
const SHAFT_Z_SHARE: f32 = 0.25; // data: metro.json shaftCentreZShare
const HATCH_X: f32 = 3.4; // data: metro.json hatch
const HATCH_HALF: f32 = 0.7; // new: DECISIONS 2026-11-27 (1.4 x 1.4 m hatch hole)
const SLAB_THICK: f32 = 0.3; // new: DECISIONS 2026-11-27 (stop-floor slab thickness)
const WALL_THICK: f32 = 0.3; // new: DECISIONS 2026-11-27 (stop-floor outer wall thickness)

/// Tower-local `[min, max]` boxes from `baseHeight` to `roof_deck_y(t, style, 0.0)`.
/// Leaves the stairwell shaft hollow, opens each stop floor (slab + 4 outer walls), and
/// leaves the roof hatch hole open above the top floor.
pub fn climb_hull_boxes(t: &TowerSpot, style: &StyleDef, plan: &ClimbTower) -> Vec<[Vec3; 2]> {
    let setbacks = default_setbacks();
    climb_hull_boxes_with_setbacks(t, style, plan, &setbacks)
}

pub(crate) fn climb_hull_boxes_with_setbacks(
    t: &TowerSpot,
    style: &StyleDef,
    plan: &ClimbTower,
    setbacks: &[SetbackSpec],
) -> Vec<[Vec3; 2]> {
    let fh = if style.floor_height > 0.0 { style.floor_height } else { DEFAULT_FLOOR_HEIGHT };
    let shaft_z = SHAFT_Z_SHARE * t.size as f32;
    let hole = [(-SHAFT_HALF_X, SHAFT_HALF_X), (shaft_z - SHAFT_HALF_Z, shaft_z + SHAFT_HALF_Z)];
    let mut out = Vec::new();

    for i in 0..t.floors {
        let y0 = style.base_height + i as f32 * fh;
        let y1 = y0 + fh;
        let half = module_size_at(t, i, style, setbacks) as f32 * 0.5;
        if plan.stop_floors.contains(&i) {
            ring_around_hole([-half, half], [-half, half], hole[0], hole[1], y0, y0 + SLAB_THICK, &mut out);
            let inner = half - WALL_THICK;
            ring_around_hole([-half, half], [-half, half], (-inner, inner), (-inner, inner), y0 + SLAB_THICK, y1, &mut out);
        } else {
            ring_around_hole([-half, half], [-half, half], hole[0], hole[1], y0, y1, &mut out);
        }
    }

    let top_ceil = style.base_height + t.floors as f32 * fh;
    let deck_y = roof_deck_y(t, style, 0.0);
    if deck_y > top_ceil {
        let top_half = module_size_at(t, t.floors.saturating_sub(1), style, setbacks) as f32 * 0.5;
        let mut right_band = Vec::new();
        // Front, back, and left of shaft:
        ring_around_hole([-top_half, top_half], [-top_half, top_half], hole[0], hole[1], top_ceil, deck_y, &mut right_band);
        // Split only the +X strip (which contains the hatch at x=3.4, z=shaft_z) around the hatch hole.
        for b in right_band {
            if b[0].x >= SHAFT_HALF_X - 1e-4 {
                ring_around_hole(
                    [b[0].x, b[1].x],
                    [b[0].z, b[1].z],
                    (HATCH_X - HATCH_HALF, HATCH_X + HATCH_HALF),
                    (shaft_z - HATCH_HALF, shaft_z + HATCH_HALF),
                    top_ceil,
                    deck_y,
                    &mut out,
                );
            } else {
                out.push(b);
            }
        }
    }
    out
}

fn ring_around_hole(
    rx: [f32; 2],
    rz: [f32; 2],
    hx: (f32, f32),
    hz: (f32, f32),
    y0: f32,
    y1: f32,
    out: &mut Vec<[Vec3; 2]>,
) {
    let hx0 = hx.0.clamp(rx[0], rx[1]);
    let hx1 = hx.1.clamp(rx[0], rx[1]);
    let hz0 = hz.0.clamp(rz[0], rz[1]);
    let hz1 = hz.1.clamp(rz[0], rz[1]);
    let candidates = [
        [Vec3::new(rx[0], y0, rz[0]), Vec3::new(rx[1], y1, hz0)],
        [Vec3::new(rx[0], y0, hz1), Vec3::new(rx[1], y1, rz[1])],
        [Vec3::new(rx[0], y0, hz0), Vec3::new(hx0, y1, hz1)],
        [Vec3::new(hx1, y0, hz0), Vec3::new(rx[1], y1, hz1)],
    ];
    for b in candidates {
        if b[1].x - b[0].x > 1e-4 && b[1].y - b[0].y > 1e-4 && b[1].z - b[0].z > 1e-4 {
            out.push(b);
        }
    }
}

fn default_setbacks() -> Vec<SetbackSpec> {
    MetroConfig::load_from_file(paths::repo_path("data/metro.json"))
        .map(|c| c.towers.setbacks)
        .unwrap_or_default()
}
