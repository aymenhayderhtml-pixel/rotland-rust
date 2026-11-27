//! R42 spot poses for `--metro=<spot>`: hospital, police station, fire station and army line.
//! Split out of `metro_flags.rs` to keep that file short. Pure maths.

use glam::Vec2;
use world::metro::{MetroWorld, Reserved, army_line_site, place_site};

use super::metro_flags::{MetroPose, MetroSpot, TopView, yaw_facing};

const PLACE_OUTSIDE_DIST_M: f32 = 35.0; // new: DECISIONS 2026-11-27
const POLICE_OUTSIDE_DIST_M: f32 = 24.0; // new: DECISIONS 2026-11-27
const FIRE_OUTSIDE_DIST_M: f32 = 26.0; // new: DECISIONS 2026-11-27
const ARMYLINE_OUTSIDE_DIST_M: f32 = 14.0; // new: DECISIONS 2026-11-27
const ARMYLINE_BARRIER_X_M: f32 = 20.0; // data: places/army_line.json barrierX
const ARMYLINE_TOP_HEIGHT_M: f32 = 60.0; // new: DECISIONS 2026-11-27
const ARMYLINE_TOP_PITCH_DEG: f32 = 55.0; // new: DECISIONS 2026-11-27
const INDOOR_SLAB_Y_OFFSET_M: f32 = 0.25; // new: DECISIONS 2026-11-27

/// Resolves an R42 place or army-line `--metro=<spot>` pose, or `None` if not an R42 spot.
pub(super) fn r42_pose(
    spot: MetroSpot,
    metro: &MetroWorld,
    eye_height: f32,
) -> Option<MetroPose> {
    let ground_y = metro.layout.site.ground_y;
    match spot {
        MetroSpot::Hospital => {
            let site = place_site(&metro.layout, Reserved::Hospital)?;
            let pos = site.pos + site.front() * PLACE_OUTSIDE_DIST_M;
            Some(MetroPose { pos, yaw: yaw_facing(site.pos - pos), top: None, y: None })
        }
        MetroSpot::HospitalIn => {
            let site = place_site(&metro.layout, Reserved::Hospital)?;
            let pos = site.pos + site.front() * 9.0; // new: DECISIONS 2026-11-27 (reception hall)
            Some(MetroPose {
                pos,
                yaw: yaw_facing(-site.front()),
                top: None,
                y: Some(ground_y + INDOOR_SLAB_Y_OFFSET_M),
            })
        }
        MetroSpot::Police => {
            let site = place_site(&metro.layout, Reserved::Police)?;
            let pos = site.pos + site.front() * POLICE_OUTSIDE_DIST_M;
            Some(MetroPose { pos, yaw: yaw_facing(site.pos - pos), top: None, y: None })
        }
        MetroSpot::PoliceIn => {
            let site = place_site(&metro.layout, Reserved::Police)?;
            let pos = site.pos - site.front() * 2.0; // new: DECISIONS 2026-11-27 (open office)
            Some(MetroPose {
                pos,
                yaw: yaw_facing(-site.front()),
                top: None,
                y: Some(ground_y + INDOOR_SLAB_Y_OFFSET_M),
            })
        }
        MetroSpot::Fire => {
            let site = place_site(&metro.layout, Reserved::FireStation)?;
            let pos = site.pos + site.front() * FIRE_OUTSIDE_DIST_M;
            Some(MetroPose { pos, yaw: yaw_facing(site.pos - pos), top: None, y: None })
        }
        MetroSpot::FireIn => {
            let site = place_site(&metro.layout, Reserved::FireStation)?;
            let front = site.front();
            let right = Vec2::new(-front.y, front.x);
            let pos = site.pos + front * 4.0 + right * 3.0; // new: DECISIONS 2026-11-27 (bay hall)
            Some(MetroPose {
                pos,
                yaw: yaw_facing(-front - right * 0.25),
                top: None,
                y: Some(ground_y + INDOOR_SLAB_Y_OFFSET_M),
            })
        }
        MetroSpot::Armyline => {
            let site = army_line_site(&metro.layout)?;
            let barrier = site.to_world(ARMYLINE_BARRIER_X_M, 0.0);
            let pos = site.to_world(ARMYLINE_BARRIER_X_M - ARMYLINE_OUTSIDE_DIST_M, 0.0);
            Some(MetroPose { pos, yaw: yaw_facing(barrier - pos), top: None, y: None })
        }
        MetroSpot::ArmylineTop => {
            let site = army_line_site(&metro.layout)?;
            let pos = site.to_world(0.0, 10.0); // new: DECISIONS 2026-11-27
            let aim = site.to_world(ARMYLINE_BARRIER_X_M, 15.0);
            let pitch = -ARMYLINE_TOP_PITCH_DEG.to_radians();
            let spring = ((ARMYLINE_TOP_HEIGHT_M - eye_height) / (-pitch).sin()).max(1.0);
            Some(MetroPose {
                pos,
                yaw: yaw_facing(aim - pos),
                top: Some(TopView { pitch, spring }),
                y: None,
            })
        }
        _ => None,
    }
}
