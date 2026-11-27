//! R43 spot poses for `--metro=<spot>`: climbable towers, rooftops, fire escapes and the radio.
//! Split out of `metro_flags.rs` to keep that file short. Pure maths.

use glam::{Vec2, Vec3Swizzles};
use world::metro::{
    ClimbKind, MetroWorld, StyleDef, TowerSpot, climb_towers, default_styles, fire_escape_towers,
    roof_deck_y, roof_links, style_for,
};

use super::metro_flags::{MetroPose, MetroSpot, tower_front_dir, yaw_facing};

const DEFAULT_FLOOR_HEIGHT_M: f32 = 3.8; // data: metro.json floorHeight
const SHAFT_HALF_Z_M: f32 = 3.2; // new: DECISIONS 2026-11-27 (shaftSize[1] * 0.5)
const LOBBY_SLAB_Y_M: f32 = 0.15; // new: DECISIONS 2026-11-26 (above 0.1 m lobby floor slab)
const CLIMB_DOOR_DIST_M: f32 = 6.0; // new: DECISIONS 2026-11-27 (6 m in front of open stair door)
const FLIGHT_A_MID_X_M: f32 = -1.05; // new: DECISIONS 2026-11-27 (mid of flight A x -2.0..-0.1)
const FLIGHT_A_MID_Y_M: f32 = 0.95; // new: DECISIONS 2026-11-27 (mid of flight A y 0..1.9)
const STOP_FLOOR_SLAB_T_M: f32 = 0.3; // new: DECISIONS 2026-11-27 (stop-floor slab thickness)
const STAND_CLEAR_M: f32 = 0.05; // new: DECISIONS 2026-11-27 (clearance above slab/ramp)
const ROOF_STAND_CLEAR_M: f32 = 0.1; // new: DECISIONS 2026-11-27 (clearance above roof deck)
const ROOF_EDGE_INSET_M: f32 = 1.8; // new: DECISIONS 2026-11-27 (inside parapet on roof edge)
const FLOOR_IN_DIST_M: f32 = 3.0; // new: DECISIONS 2026-11-27 (3 m in from shaft door)
const PLANK_BACK_DIST_M: f32 = 4.0; // new: DECISIONS 2026-11-27 (4 m back from plank end)
const FIRE_ESCAPE_DIST_M: f32 = 14.0; // new: DECISIONS 2026-11-27 (14 m in front of fire escape)
const MAST_PLATFORM_SLAB_T_M: f32 = 0.22; // new: DECISIONS 2026-11-27 (landmarks.py pz..pz+0.22)
const RADIO_STAND_LOCAL: (f32, f32) = (1.3, -2.1); // new: DECISIONS 2026-11-27 (1.5 m from console at (-0.2, -2.1), inside useRadiusM 2.0)
const RADIO_HUT_LOCAL: (f32, f32) = (-1.1, -0.6); // new: DECISIONS 2026-11-27 (landmarks.py radio_hut centre)

/// Converts tower-local `(lx, lz)` to world XZ using the tower's yaw and centre.
fn tower_local_xz(t: &TowerSpot, lx: f32, lz: f32) -> Vec2 {
    let (sin_y, cos_y) = t.yaw.sin_cos();
    Vec2::new(
        t.pos.x + lx * cos_y + lz * sin_y,
        t.pos.y - lx * sin_y + lz * cos_y,
    )
}

/// Converts a tower-local XZ direction `(dx, dz)` to world XZ.
fn tower_local_dir(t: &TowerSpot, dx: f32, dz: f32) -> Vec2 {
    let (sin_y, cos_y) = t.yaw.sin_cos();
    Vec2::new(dx * cos_y + dz * sin_y, -dx * sin_y + dz * cos_y)
}

fn floor_height(style: &StyleDef) -> f32 {
    if style.floor_height > 0.0 {
        style.floor_height
    } else {
        DEFAULT_FLOOR_HEIGHT_M
    }
}

/// Resolves an R43 `--metro=<spot>` pose, or `None` if not an R43 spot or unavailable.
pub(super) fn r43_pose(spot: MetroSpot, metro: &MetroWorld) -> Option<MetroPose> {
    let ground_y = metro.layout.site.ground_y;
    let styles = default_styles();
    match spot {
        MetroSpot::Climb => {
            let plan = climb_towers(metro).into_iter().next()?;
            let t = &metro.towers[plan.tower];
            let shaft_z = metro.config.climb.shaft_centre_z_share * t.size as f32;
            let door_lz = shaft_z - SHAFT_HALF_Z_M;
            let pos = tower_local_xz(t, 0.0, door_lz - CLIMB_DOOR_DIST_M);
            let door_world = tower_local_xz(t, 0.0, door_lz);
            Some(MetroPose {
                pos,
                yaw: yaw_facing(door_world - pos),
                top: None,
                y: Some(ground_y + LOBBY_SLAB_Y_M),
            })
        }
        MetroSpot::ClimbStairs => {
            let plan = climb_towers(metro).into_iter().next()?;
            let t = &metro.towers[plan.tower];
            let style = style_for(styles, t.style)?;
            let fh = floor_height(style);
            let mid_floor = (t.floors / 2).min(t.floors.saturating_sub(2));
            let fy = style.base_height + mid_floor as f32 * fh;
            let shaft_z = metro.config.climb.shaft_centre_z_share * t.size as f32;
            let pos = tower_local_xz(t, FLIGHT_A_MID_X_M, shaft_z);
            Some(MetroPose {
                pos,
                yaw: yaw_facing(tower_local_dir(t, 0.0, 1.0)),
                top: None,
                y: Some(ground_y + fy + FLIGHT_A_MID_Y_M + STAND_CLEAR_M),
            })
        }
        MetroSpot::ClimbFloor => {
            let plan = climb_towers(metro).into_iter().next()?;
            let fi = *plan.stop_floors.first()?;
            let t = &metro.towers[plan.tower];
            let style = style_for(styles, t.style)?;
            let fh = floor_height(style);
            let fy = style.base_height + fi as f32 * fh;
            let shaft_z = metro.config.climb.shaft_centre_z_share * t.size as f32;
            let door_lz = shaft_z - SHAFT_HALF_Z_M;
            let pos = tower_local_xz(t, 0.0, door_lz - FLOOR_IN_DIST_M);
            Some(MetroPose {
                pos,
                yaw: yaw_facing(tower_local_dir(t, 0.0, -1.0)),
                top: None,
                y: Some(ground_y + fy + STOP_FLOOR_SLAB_T_M + STAND_CLEAR_M),
            })
        }
        MetroSpot::ClimbRoof => {
            let plans = climb_towers(metro);
            let plan = plans
                .iter()
                .filter(|p| p.kind != ClimbKind::Landmark)
                .max_by(|a, b| metro.towers[a.tower].height.total_cmp(&metro.towers[b.tower].height))?;
            let t = &metro.towers[plan.tower];
            let style = style_for(styles, t.style)?;
            let deck_y = roof_deck_y(t, style, ground_y);
            let target_xz = plans
                .iter()
                .find(|p| p.kind == ClimbKind::Landmark)
                .map(|p| metro.towers[p.tower].pos)
                .unwrap_or(metro.layout.site.center);
            let to_target = (target_xz - t.pos).normalize_or(tower_front_dir(t.yaw));
            let (sin_y, cos_y) = t.yaw.sin_cos();
            let local_dir = Vec2::new(
                to_target.x * cos_y - to_target.y * sin_y,
                to_target.x * sin_y + to_target.y * cos_y,
            );
            let max_comp = local_dir.x.abs().max(local_dir.y.abs()).max(1e-4);
            let edge_half = (t.half.x - ROOF_EDGE_INSET_M).max(1.0);
            let local_edge = local_dir * (edge_half / max_comp);
            let pos = tower_local_xz(t, local_edge.x, local_edge.y);
            Some(MetroPose {
                pos,
                yaw: yaw_facing(target_xz - pos),
                top: None,
                y: Some(deck_y + ROOF_STAND_CLEAR_M),
            })
        }
        MetroSpot::RoofPlanks => {
            let link = roof_links(metro).into_iter().next()?;
            let t = &metro.towers[link.from_tower];
            let style = style_for(styles, t.style)?;
            let deck_y = roof_deck_y(t, style, ground_y);
            let dir = Vec2::new(link.plank_yaw.sin(), link.plank_yaw.cos());
            let plank_start = link.plank_pos.xz() - dir * (link.plank_length as f32 * 0.5);
            let pos = plank_start - dir * PLANK_BACK_DIST_M;
            Some(MetroPose {
                pos,
                yaw: yaw_facing(dir),
                top: None,
                y: Some(deck_y + ROOF_STAND_CLEAR_M),
            })
        }
        MetroSpot::FireEscape => {
            let idx = fire_escape_towers(metro).into_iter().next()?;
            let t = &metro.towers[idx];
            let front = tower_front_dir(t.yaw);
            let pos = t.pos + front * (t.half.y + FIRE_ESCAPE_DIST_M);
            Some(MetroPose {
                pos,
                yaw: yaw_facing(-front),
                top: None,
                y: None,
            })
        }
        MetroSpot::Radio => {
            let plan = climb_towers(metro)
                .into_iter()
                .find(|p| p.kind == ClimbKind::Landmark)?;
            let t = &metro.towers[plan.tower];
            let style = style_for(styles, t.style)?;
            let fh = floor_height(style);
            let crown_y = style.base_height + t.floors as f32 * fh;
            let plat_top = crown_y + metro.config.climb.mast.platform_y + MAST_PLATFORM_SLAB_T_M;
            let pos = tower_local_xz(t, RADIO_STAND_LOCAL.0, RADIO_STAND_LOCAL.1);
            let hut = tower_local_xz(t, RADIO_HUT_LOCAL.0, RADIO_HUT_LOCAL.1);
            Some(MetroPose {
                pos,
                yaw: yaw_facing(hut - pos),
                top: None,
                y: Some(ground_y + plat_top + STAND_CLEAR_M),
            })
        }
        _ => None,
    }
}
