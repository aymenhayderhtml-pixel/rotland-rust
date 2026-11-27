//! R41 spot poses for `--metro=<spot>`: the herd, crows (and crows_up) and army cache spots.
//! Split out of `metro_flags.rs` to keep that file short. Pure maths.

use glam::{Vec2, Vec3Swizzles};
use world::metro::{
    MetroWorld, PerchKind, crow_perches_in_chunk, herd_routes_avoiding, metro_loot_in_chunk,
    route_pos,
};

use super::metro_flags::{MetroPose, MetroSpot, yaw_facing};

/// Herd spot pose: 25 m ahead of the first street herd, facing it.
pub(super) fn herd_pose(metro: &MetroWorld) -> Option<MetroPose> {
    let routes = herd_routes_avoiding(&metro.layout, &metro.config.herds, &[], 42);
    let route = routes.first()?;
    let (hpos, hyaw) = route_pos(route, 0.0);
    let dir = Vec2::new(-hyaw.sin(), -hyaw.cos());
    let pos = hpos + dir * 25.0; // new: DECISIONS 2026-11-27
    Some(MetroPose { pos, yaw: yaw_facing(hpos - pos), top: None, y: None })
}

/// Crows spot pose (`crows` 14 m, `crows_up` 4 m) from the first body perch, facing it.
pub(super) fn crows_pose(metro: &MetroWorld, spot: MetroSpot) -> Option<MetroPose> {
    let site = &metro.layout.site;
    let center = site.center;
    let dist = if spot == MetroSpot::CrowsUp { 4.0 } else { 14.0 }; // new: DECISIONS 2026-11-27
    let ground_y = site.ground_y;
    let cx0 = (center.x / 64.0).floor() as i32;
    let cz0 = (center.y / 64.0).floor() as i32;
    let mut found = None;
    'outer: for dx in -8..=8 {
        for dz in -8..=8 {
            for p in crow_perches_in_chunk(metro, (cx0 + dx, cz0 + dz), &|_, _| ground_y) {
                if p.kind == PerchKind::Body {
                    found = Some(p);
                    break 'outer;
                }
            }
        }
    }
    let perch = found?;
    let pxz = perch.pos.xz();
    let dir = Vec2::new(-perch.yaw.sin(), -perch.yaw.cos());
    let pos = pxz + dir * dist;
    Some(MetroPose { pos, yaw: yaw_facing(pxz - pos), top: None, y: None })
}

/// Army spot pose: 2 m in front of the first Core army cache crate, facing it.
pub(super) fn army_pose(metro: &MetroWorld) -> Option<MetroPose> {
    let site = &metro.layout.site;
    let center = site.center;
    let ground_y = site.ground_y;
    let cx0 = (center.x / 64.0).floor() as i32;
    let cz0 = (center.y / 64.0).floor() as i32;
    let mut found = None;
    'army: for dx in -8..=8 {
        for dz in -8..=8 {
            for s in metro_loot_in_chunk(metro, (cx0 + dx, cz0 + dz), &|_, _| ground_y, 42) {
                if s.table.as_deref() == Some("metro_army") {
                    found = Some(s);
                    break 'army;
                }
            }
        }
    }
    let crate_spot = found?;
    let cxz = crate_spot.position.xz();
    let dir = Vec2::new(-crate_spot.yaw.sin(), -crate_spot.yaw.cos());
    let pos = cxz + dir * 2.0; // new: DECISIONS 2026-11-27
    Some(MetroPose { pos, yaw: yaw_facing(cxz - pos), top: None, y: None })
}
