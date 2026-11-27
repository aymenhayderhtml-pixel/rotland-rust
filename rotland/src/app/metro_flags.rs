//! Poses for the `--metro=<spot>` test flags: where the player stands, which way it faces and,
//! for `top`, the camera pitch and spring length. Pure maths; `test_flags.rs` applies them.
//! Facing rule (AGENTS.md): yaw forward = (-sin yaw, -cos yaw), so yaw = atan2(-dx, -dz).

use glam::{Vec2, Vec3Swizzles};
use world::metro::{
    MetroStreet, MetroWorld, PerchKind, TowerSpot, TowerStyle, camp_sites, car_wall_sites,
    crow_perches_in_chunk, herd_routes_avoiding, metro_loot_in_chunk, route_pos,
};

/// Edge spot: metres inside the first avenue entrance. // new: DECISIONS 2026-11-26
const EDGE_INSET_M: f32 = 10.0;
/// Highway spot: metres back along the highway from its city end (inside the jam, skyline ahead). // new: DECISIONS 2026-11-26
const HIGHWAY_BACK_M: f32 = 350.0;
/// Highway spot: metres in from the highway's outer edge (on the shoulder, clear of the lanes). // new: DECISIONS 2026-11-26
const SHOULDER_IN_M: f32 = 1.0;
/// Top spot: camera height above the city ground in metres (above the ~160 m landmarks). // new: DECISIONS 2026-11-26
const TOP_HEIGHT_M: f32 = 260.0;
/// Top spot: how far the camera looks down, in degrees. // new: DECISIONS 2026-11-26
const TOP_PITCH_DEG: f32 = 60.0;
/// Landmark spot: metres from the landmark base along the nearest avenue. // new: DECISIONS 2026-11-26
const LANDMARK_VIEW_DIST_M: f32 = 60.0;
/// Leaning spot: metres from the leaning tower base along the nearest avenue. // new: DECISIONS 2026-11-26
const LEANING_VIEW_DIST_M: f32 = 40.0;
/// Fallen spot: metres from the fallen section centre along its street. // new: DECISIONS 2026-11-26
const FALLEN_VIEW_DIST_M: f32 = 55.0;
/// Burnt spot: metres from the burnt block centre on the nearest street. // new: DECISIONS 2026-11-26
const BURNT_VIEW_DIST_M: f32 = 35.0;
/// Carwall spot: metres before the first car wall on its street. // new: DECISIONS 2026-11-26
const CARWALL_VIEW_DIST_M: f32 = 12.0;
/// Camp spot: metres off the rooftop camp centre on the roof. // new: DECISIONS 2026-11-26
const CAMP_VIEW_OFFSET_M: f32 = 6.0;
/// Landmark aim share of total tower height so the crown sits in the upper view. // new: DECISIONS 2026-11-26
const LANDMARK_AIM_SHARE: f32 = 0.85;
/// Landmark spot: default third-person camera spring length in metres. // new: DECISIONS 2026-11-26
const LANDMARK_SPRING_M: f32 = 4.0;
/// Style spot: metres in front of the tower's facade on its front street. // new: DECISIONS 2026-11-26
const STYLE_STREET_DIST_M: f32 = 12.0;

/// The twelve places `--metro=<spot>` can teleport to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetroSpot {
    /// Just inside the city's first avenue entrance, looking in.
    Edge,
    /// The middle of the city, looking along the nearest avenue.
    Core,
    /// On the jam, looking at the skyline.
    Highway,
    /// Beside the nearest town, looking at the city.
    TownView,
    /// High above the Core, looking down.
    Top,
    /// Inside the ground-floor lobby of the tower nearest the Core centre, facing its door.
    Lobby,
    /// On the nearest avenue ~60 m from the tallest landmark, looking up at its crown.
    Landmark,
    /// On the street 30 m from the fallen tower section, facing its middle.
    Fallen,
    /// On the nearest avenue 40 m from the leaning tower, looking up at its top.
    Leaning,
    /// On the nearest street 35 m from the burnt block centre, facing it.
    Burnt,
    /// 12 m before the first stacked-car wall on its street, facing it.
    Carwall,
    /// On the first rooftop camp, 6 m off the camp centre, looking across it.
    Camp,
    /// On an avenue 25 m ahead of an active street herd walking toward the camera (hour 10).
    Herd,
    /// 15 m from a body perch with pecking crows.
    Crows,
    /// Inside the startle radius (6 m) of a body perch so crows burst into the air.
    CrowsUp,
    /// 2 m in front of a Core army cache crate.
    Army,
    /// Core centre at hour 23 (night runners).
    Night,
}

impl MetroSpot {
    /// Parses a spot name (any case). None for an unknown name.
    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "edge" => Some(Self::Edge),
            "core" => Some(Self::Core),
            "highway" => Some(Self::Highway),
            "town_view" => Some(Self::TownView),
            "top" => Some(Self::Top),
            "lobby" => Some(Self::Lobby),
            "landmark" => Some(Self::Landmark),
            "fallen" => Some(Self::Fallen),
            "leaning" => Some(Self::Leaning),
            "burnt" => Some(Self::Burnt),
            "carwall" => Some(Self::Carwall),
            "camp" => Some(Self::Camp),
            "herd" => Some(Self::Herd),
            "crows" => Some(Self::Crows),
            "crows_up" => Some(Self::CrowsUp),
            "army" => Some(Self::Army),
            "night" => Some(Self::Night),
            _ => None,
        }
    }
}

/// Camera settings for the `top` spot: the rig's pitch (radians, negative = down) and spring length.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TopView {
    pub pitch: f32,
    pub spring: f32,
}

/// Where to teleport: world XZ, Godot yaw, optional camera override, and optional roof/lobby Y.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MetroPose {
    pub pos: Vec2,
    pub yaw: f32,
    pub top: Option<TopView>,
    pub y: Option<f32>,
}

/// Yaw that faces along `dir` (world XZ): forward = (-sin yaw, -cos yaw).
pub fn yaw_facing(dir: Vec2) -> f32 {
    (-dir.x).atan2(-dir.y)
}

/// Rig spring length that puts the camera `height` metres above the feet when the camera looks
/// down at `pitch` (negative radians). The camera sits behind the pivot along `-forward`.
pub fn top_spring(height: f32, eye_height: f32, pitch: f32) -> f32 {
    (height - eye_height) / -pitch.sin()
}

/// The avenue whose centreline is closest to `p` (site-local). None when the city has no avenue.
pub fn nearest_avenue(streets: &[MetroStreet], p: Vec2) -> Option<&MetroStreet> {
    let dist = |s: &MetroStreet| s.closest_point(p).distance_squared(p);
    streets.iter().filter(|s| s.avenue).min_by(|a, b| dist(a).total_cmp(&dist(b)))
}

/// The street (or avenue) whose centreline is closest to `p` (site-local).
fn nearest_street(streets: &[MetroStreet], p: Vec2) -> Option<&MetroStreet> {
    let dist = |s: &MetroStreet| s.closest_point(p).distance_squared(p);
    streets.iter().min_by(|a, b| dist(a).total_cmp(&dist(b)))
}

/// `EDGE_INSET_M` along the avenue from `entrance` toward `far`, looking at `center`.
pub fn edge_pose(entrance: Vec2, far: Vec2, center: Vec2) -> MetroPose {
    let inward = (far - entrance).normalize_or_zero();
    let pos = entrance + inward * EDGE_INSET_M;
    MetroPose { pos, yaw: yaw_facing(center - pos), top: None, y: None }
}

/// The city centre, looking along the avenue direction `avenue_dir` (world XZ).
pub fn core_pose(center: Vec2, avenue_dir: Vec2) -> MetroPose {
    MetroPose { pos: center, yaw: yaw_facing(avenue_dir), top: None, y: None }
}

/// The point `back` metres before the polyline's end, and the travel direction there.
pub fn point_back_from_end(points: &[Vec2], back: f32) -> Option<(Vec2, Vec2)> {
    let mut left = back;
    for w in points.windows(2).rev() {
        let seg = w[0].distance(w[1]);
        let dir = (w[1] - w[0]).normalize_or_zero();
        if seg > 0.0 && seg >= left {
            return Some((w[1] - dir * left, dir));
        }
        left -= seg;
    }
    let first = *points.first()?;
    let dir = points.get(1).map_or(Vec2::X, |b| (*b - first).normalize_or(Vec2::X));
    Some((first, dir))
}

/// On the highway shoulder `HIGHWAY_BACK_M` before the city, looking along the road at the city.
pub fn highway_pose(points: &[Vec2], width: f32) -> Option<MetroPose> {
    let (on_road, dir) = point_back_from_end(points, HIGHWAY_BACK_M)?;
    let side = Vec2::new(-dir.y, dir.x);
    let pos = on_road + side * (width * 0.5 - SHOULDER_IN_M);
    let end = *points.last()?;
    Some(MetroPose { pos, yaw: yaw_facing(end - pos), top: None, y: None })
}

/// `distance` metres from the town centre on the side facing the city, looking at `center`.
pub fn town_view_pose(town_center: Vec2, center: Vec2, distance: f32) -> MetroPose {
    let toward_city = (center - town_center).normalize_or(Vec2::X);
    let pos = town_center + toward_city * distance;
    MetroPose { pos, yaw: yaw_facing(center - pos), top: None, y: None }
}

/// Above the Core centre, looking down `TOP_PITCH_DEG` toward the city from the highway side
/// (`from` is the highway's end on the city edge). The camera sits on the spring behind the pivot.
pub fn top_pose(center: Vec2, from: Vec2, eye_height: f32) -> MetroPose {
    let pitch = -TOP_PITCH_DEG.to_radians();
    let spring = top_spring(TOP_HEIGHT_M, eye_height, pitch);
    MetroPose { pos: center, yaw: yaw_facing(center - from), top: Some(TopView { pitch, spring }), y: None }
}

/// World XZ unit direction that a tower's front (its lobby door, Blender +Y / glTF -Z) faces.
pub fn tower_front_dir(yaw: f32) -> Vec2 {
    Vec2::new(-yaw.sin(), -yaw.cos())
}

/// Inside the ground-floor lobby at `tower.pos`, facing its street door.
pub fn lobby_pose(tower: &TowerSpot) -> MetroPose {
    let front_dir = tower_front_dir(tower.yaw);
    MetroPose { pos: tower.pos, yaw: yaw_facing(front_dir), top: None, y: None }
}

/// Point on segment `(seg_a, seg_b)` at `dist_m` from `target` (closest point if perpendicular >= `dist_m`).
fn point_on_segment_at_dist(target: Vec2, seg_a: Vec2, seg_b: Vec2, dist_m: f32) -> Vec2 {
    let ab = seg_b - seg_a;
    let len_sq = ab.length_squared();
    let closest = if len_sq < 1e-6 {
        seg_a
    } else {
        let t = ((target - seg_a).dot(ab) / len_sq).clamp(0.0, 1.0);
        seg_a + ab * t
    };
    let d_perp = closest.distance(target);
    if d_perp < dist_m && len_sq >= 1e-6 {
        let along = (dist_m * dist_m - d_perp * d_perp).sqrt();
        let dir = ab / len_sq.sqrt();
        let mid = (seg_a + seg_b) * 0.5;
        let p1 = closest + dir * along;
        let p2 = closest - dir * along;
        if p1.distance_squared(mid) <= p2.distance_squared(mid) { p1 } else { p2 }
    } else {
        closest
    }
}

/// On the avenue `(ave_a, ave_b)` `dist_m` from `tower_pos`, facing the tower and aiming at 85% of `tower_height`.
pub fn landmark_pose(
    tower_pos: Vec2,
    tower_height: f32,
    ave_a: Vec2,
    ave_b: Vec2,
    dist_m: f32,
    eye_height: f32,
) -> MetroPose {
    let pos = point_on_segment_at_dist(tower_pos, ave_a, ave_b, dist_m);
    let horiz = pos.distance(tower_pos).max(1.0);
    let pitch = ((tower_height * LANDMARK_AIM_SHARE - eye_height) / horiz).atan();
    MetroPose {
        pos,
        yaw: yaw_facing(tower_pos - pos),
        top: Some(TopView { pitch, spring: LANDMARK_SPRING_M }),
        y: None,
    }
}

/// Nearest tower of `style` to `center`. None when the city has no tower of that style.
pub fn nearest_tower_of_style(
    towers: &[TowerSpot],
    center: Vec2,
    style: TowerStyle,
) -> Option<&TowerSpot> {
    towers
        .iter()
        .filter(|t| t.style == style)
        .min_by(|a, b| a.pos.distance_squared(center).total_cmp(&b.pos.distance_squared(center)))
}

/// On the street in front of `tower`, facing its facade.
pub fn style_pose(tower: &TowerSpot) -> MetroPose {
    let front_dir = tower_front_dir(tower.yaw);
    let pos = tower.pos + front_dir * (tower.half.y + STYLE_STREET_DIST_M);
    MetroPose { pos, yaw: yaw_facing(-front_dir), top: None, y: None }
}

/// The pose for `spot` in the metro city. None when the city lacks what the spot needs
/// (no avenue, no highway, no jam prop, or no town for `town_view`).
pub fn metro_pose(
    spot: MetroSpot,
    metro: &MetroWorld,
    town_center: Option<Vec2>,
    town_distance: f32,
    eye_height: f32,
) -> Option<MetroPose> {
    let site = &metro.layout.site;
    let center = site.center;
    let streets = &metro.layout.streets;
    match spot {
        MetroSpot::Edge => {
            let end = site.to_local(*metro.highway.points.last()?);
            let avenue = nearest_avenue(streets, end)?;
            let (entrance, far) = if avenue.a.distance_squared(end) <= avenue.b.distance_squared(end) {
                (avenue.a, avenue.b)
            } else {
                (avenue.b, avenue.a)
            };
            Some(edge_pose(site.to_world(entrance), site.to_world(far), center))
        }
        MetroSpot::Core => {
            let avenue = nearest_avenue(streets, Vec2::ZERO)?;
            let dir = site.to_world(avenue.b) - site.to_world(avenue.a);
            Some(core_pose(center, dir))
        }
        MetroSpot::Highway => highway_pose(&metro.highway.points, metro.highway.width),
        MetroSpot::TownView => {
            // Where the highway leaves the town: the road cuts through the forest toward the skyline
            let _ = town_distance;
            let start = *metro.highway.points.first().or(town_center.as_ref())?;
            Some(town_view_pose(start, center, 0.0))
        }
        MetroSpot::Top => {
            // Look along the central avenue: the camera's wall check then finds open air, not a tower
            let avenue = nearest_avenue(streets, Vec2::ZERO)?;
            let back = center - (site.to_world(avenue.b) - site.to_world(avenue.a)).normalize_or(Vec2::X);
            Some(top_pose(center, back, eye_height))
        }
        MetroSpot::Lobby => {
            let tower = metro
                .towers
                .iter()
                .min_by(|a, b| a.pos.distance_squared(center).total_cmp(&b.pos.distance_squared(center)))?;
            Some(lobby_pose(tower))
        }
        MetroSpot::Landmark => {
            let tallest = crate::metro_render::parts::tallest_landmark_index(&metro.towers);
            let tower = tallest
                .and_then(|i| metro.towers.get(i))
                .or_else(|| metro.towers.iter().max_by(|a, b| a.height.total_cmp(&b.height)))?;
            let avenue = nearest_avenue(streets, site.to_local(tower.pos))?;
            Some(landmark_pose(
                tower.pos,
                tower.height,
                site.to_world(avenue.a),
                site.to_world(avenue.b),
                LANDMARK_VIEW_DIST_M,
                eye_height,
            ))
        }
        MetroSpot::Fallen => {
            let target = metro.damage.fallen.pos;
            let street = nearest_street(streets, site.to_local(target))?;
            let pos = point_on_segment_at_dist(
                target,
                site.to_world(street.a),
                site.to_world(street.b),
                FALLEN_VIEW_DIST_M,
            );
            // Raised camera so the whole broken section and its rubble read at once.
            let top = TopView { pitch: -18f32.to_radians(), spring: 14.0 }; // new: DECISIONS 2026-11-26
            Some(MetroPose { pos, yaw: yaw_facing(target - pos), top: Some(top), y: None })
        }
        MetroSpot::Leaning => {
            let target = metro.damage.leaning.pos;
            let avenue = nearest_avenue(streets, site.to_local(target))?;
            let height = metro
                .towers
                .iter()
                .filter(|t| metro.damage.tower_is_replaced(
                    metro.towers.iter().position(|x| std::ptr::eq(x, *t)).unwrap_or(usize::MAX)
                ))
                .map(|t| t.height)
                .fold(60.0f32, f32::max);
            Some(landmark_pose(
                target,
                height,
                site.to_world(avenue.a),
                site.to_world(avenue.b),
                LEANING_VIEW_DIST_M,
                eye_height,
            ))
        }
        MetroSpot::Burnt => {
            let (bx, bz) = metro.damage.burnt_block;
            let block = metro.layout.blocks.iter().find(|b| b.ix == bx && b.iz == bz)?;
            let local_center = (block.rect.0 + block.rect.1) * 0.5;
            let target = site.to_world(local_center);
            let street = nearest_street(streets, local_center)?;
            let pos = point_on_segment_at_dist(
                target,
                site.to_world(street.a),
                site.to_world(street.b),
                BURNT_VIEW_DIST_M,
            );
            Some(MetroPose { pos, yaw: yaw_facing(target - pos), top: None, y: None })
        }
        MetroSpot::Carwall => {
            let (cw_pos, _) = car_wall_sites(metro).into_iter().next()?;
            let street = nearest_street(streets, site.to_local(cw_pos))?;
            let pos = point_on_segment_at_dist(
                cw_pos,
                site.to_world(street.a),
                site.to_world(street.b),
                CARWALL_VIEW_DIST_M,
            );
            Some(MetroPose { pos, yaw: yaw_facing(cw_pos - pos), top: None, y: None })
        }
        MetroSpot::Camp => {
            let camp = camp_sites(metro).into_iter().next()?;
            let dir = -tower_front_dir(camp.yaw);
            let camp_xz = camp.pos.xz();
            let pos = camp_xz + dir * CAMP_VIEW_OFFSET_M;
            Some(MetroPose {
                pos,
                yaw: yaw_facing(camp_xz - pos),
                top: Some(TopView { pitch: -30f32.to_radians(), spring: 10.0 }),
                y: Some(camp.pos.y + 0.2), // new: DECISIONS 2026-11-26 (on the roof deck, the hull top)
            })
        }
        MetroSpot::Night => {
            let avenue = nearest_avenue(streets, Vec2::ZERO)?;
            let dir = site.to_world(avenue.b) - site.to_world(avenue.a);
            Some(core_pose(center, dir))
        }
        MetroSpot::Herd => {
            let routes = herd_routes_avoiding(&metro.layout, &metro.config.herds, &[], 42);
            let route = routes.first()?;
            let (hpos, hyaw) = route_pos(route, 0.0);
            let dir = Vec2::new(-hyaw.sin(), -hyaw.cos());
            let pos = hpos + dir * 25.0; // new: DECISIONS 2026-11-27
            Some(MetroPose { pos, yaw: yaw_facing(hpos - pos), top: None, y: None })
        }
        MetroSpot::Crows | MetroSpot::CrowsUp => {
            let dist = if spot == MetroSpot::CrowsUp { 6.0 } else { 15.0 }; // new: DECISIONS 2026-11-27
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
        MetroSpot::Army => {
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::FRAC_PI_2;

    const EYE: f32 = 1.6;

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-3
    }

    /// Forward direction (world XZ) of a yaw: the inverse of `yaw_facing`.
    fn forward_of(yaw: f32) -> Vec2 {
        Vec2::new(-yaw.sin(), -yaw.cos())
    }

    #[test]
    fn spot_names_parse_any_case_and_reject_unknown() {
        assert_eq!(MetroSpot::parse("edge"), Some(MetroSpot::Edge));
        assert_eq!(MetroSpot::parse(" Core "), Some(MetroSpot::Core));
        assert_eq!(MetroSpot::parse("HIGHWAY"), Some(MetroSpot::Highway));
        assert_eq!(MetroSpot::parse("town_view"), Some(MetroSpot::TownView));
        assert_eq!(MetroSpot::parse("top"), Some(MetroSpot::Top));
        assert_eq!(MetroSpot::parse("lobby"), Some(MetroSpot::Lobby));
        assert_eq!(MetroSpot::parse("LANDMARK"), Some(MetroSpot::Landmark));
        assert_eq!(MetroSpot::parse("fallen"), Some(MetroSpot::Fallen));
        assert_eq!(MetroSpot::parse("Leaning"), Some(MetroSpot::Leaning));
        assert_eq!(MetroSpot::parse("BURNT"), Some(MetroSpot::Burnt));
        assert_eq!(MetroSpot::parse("carwall"), Some(MetroSpot::Carwall));
        assert_eq!(MetroSpot::parse("camp"), Some(MetroSpot::Camp));
        assert_eq!(MetroSpot::parse("roof"), None);
        assert_eq!(MetroSpot::parse(""), None);
    }

    #[test]
    fn yaw_follows_the_agents_rule() {
        // Facing east (+X) is yaw -PI/2 (Godot convention)
        assert!(close(yaw_facing(Vec2::X), -FRAC_PI_2));
        for dir in [Vec2::X, Vec2::NEG_Y, Vec2::new(0.6, -0.8), Vec2::new(-0.3, 0.95)] {
            let f = forward_of(yaw_facing(dir));
            assert!(f.dot(dir.normalize()) > 0.9999, "dir {dir:?} forward {f:?}");
        }
    }

    #[test]
    fn edge_is_inset_along_the_avenue_and_looks_at_the_centre() {
        let center = Vec2::new(100.0, 100.0);
        // A radial avenue from the city edge (x = 0) toward the centre
        let entrance = Vec2::new(0.0, 100.0);
        let far = Vec2::new(200.0, 100.0);
        let pose = edge_pose(entrance, far, center);
        assert!(close(pose.pos.distance(entrance), EDGE_INSET_M));
        assert!(close(pose.pos.y, 100.0));
        let to_centre = (center - pose.pos).normalize();
        assert!(forward_of(pose.yaw).dot(to_centre) > 0.9);
        assert!(pose.top.is_none());
    }

    #[test]
    fn core_is_the_centre_looking_along_the_avenue() {
        let center = Vec2::new(-40.0, 12.0);
        let pose = core_pose(center, Vec2::new(0.0, -5.0));
        assert_eq!(pose.pos, center);
        assert!(forward_of(pose.yaw).dot(Vec2::NEG_Y) > 0.9999);
    }

    #[test]
    fn highway_stands_on_the_shoulder_before_the_city_and_looks_at_it() {
        let points = [Vec2::new(0.0, 0.0), Vec2::new(1000.0, 0.0), Vec2::new(2000.0, 0.0)];
        let pose = highway_pose(&points, 22.0).expect("pose");
        assert!(close(pose.pos.x, 2000.0 - HIGHWAY_BACK_M), "pos {:?}", pose.pos);
        assert!(close(pose.pos.y.abs(), 11.0 - SHOULDER_IN_M), "pos {:?}", pose.pos);
        let to_end = (points[2] - pose.pos).normalize();
        assert!(forward_of(pose.yaw).dot(to_end) > 0.9999);
        assert_eq!(highway_pose(&[], 22.0), None);
    }

    #[test]
    fn town_view_is_25_metres_from_the_town_on_the_city_side() {
        let town = Vec2::new(0.0, 0.0);
        let center = Vec2::new(6000.0, 0.0);
        let pose = town_view_pose(town, center, 25.0);
        assert!(close(pose.pos.distance(town), 25.0));
        assert!(close(pose.pos.x, 25.0));
        assert!(forward_of(pose.yaw).dot(Vec2::X) > 0.9999);
    }

    #[test]
    fn top_is_above_the_core_centre_and_looks_down_60_degrees() {
        let center = Vec2::new(6000.0, 400.0);
        let pose = top_pose(center, Vec2::new(4000.0, 400.0), EYE);
        assert_eq!(pose.pos, center);
        let top = pose.top.expect("top sets the camera");
        assert!(close(top.pitch, -60f32.to_radians()));
        // The camera height above the feet: eye + spring * sin(-pitch) = TOP_HEIGHT_M
        let camera_height = EYE + top.spring * -top.pitch.sin();
        assert!(close(camera_height, TOP_HEIGHT_M), "height {camera_height}");
        // The view points from the highway side into the city
        assert!(forward_of(pose.yaw).dot(Vec2::X) > 0.9999);
    }

    #[test]
    fn nearest_avenue_ignores_plain_streets() {
        let street = |a: Vec2, b: Vec2, avenue: bool| MetroStreet { a, b, width: 12.0, avenue };
        let streets = [
            street(Vec2::new(0.0, 1.0), Vec2::new(100.0, 1.0), false),
            street(Vec2::new(0.0, 50.0), Vec2::new(100.0, 50.0), true),
            street(Vec2::new(-30.0, 0.0), Vec2::new(-30.0, 100.0), true),
        ];
        let best = nearest_avenue(&streets, Vec2::new(10.0, 0.0)).unwrap();
        assert_eq!(best.a, Vec2::new(-30.0, 0.0));
        assert!(nearest_avenue(&streets[..1], Vec2::ZERO).is_none());
    }

    fn fake_tower(pos: Vec2, yaw: f32, style: TowerStyle, landmark: bool) -> TowerSpot {
        TowerSpot {
            pos,
            half: Vec2::splat(12.0),
            yaw,
            height: 80.0,
            floors: 18,
            kind: world::metro::TowerKind::Highrise,
            landmark,
            block: (0, 0),
            style,
            size: 24,
        }
    }

    #[test]
    fn lobby_pose_is_inside_footprint_and_faces_the_door() {
        let tower = fake_tower(Vec2::new(100.0, -50.0), FRAC_PI_2, TowerStyle::Stone, false);
        let pose = lobby_pose(&tower);
        assert!(
            (pose.pos.x - tower.pos.x).abs() <= tower.half.x
                && (pose.pos.y - tower.pos.y).abs() <= tower.half.y
        );
        let door = tower.pos + tower_front_dir(tower.yaw) * tower.half.y;
        let to_door = (door - pose.pos).normalize();
        let dot = forward_of(pose.yaw).dot(to_door);
        assert!(dot > 5f32.to_radians().cos(), "yaw must face door, dot={dot}");
        assert!(pose.top.is_none());
    }

    #[test]
    fn landmark_pose_is_60m_from_base_and_frames_crown() {
        let tower_pos = Vec2::new(20.0, 100.0);
        let height = 160.0;
        let pose = landmark_pose(
            tower_pos,
            height,
            Vec2::new(0.0, 0.0),
            Vec2::new(0.0, 200.0),
            LANDMARK_VIEW_DIST_M,
            EYE,
        );
        assert!(close(pose.pos.distance(tower_pos), LANDMARK_VIEW_DIST_M));
        let to_landmark = (tower_pos - pose.pos).normalize();
        assert!(forward_of(pose.yaw).dot(to_landmark) > 0.99);
        let top = pose.top.expect("landmark sets upward pitch");
        assert!(top.pitch > 0.0);
        let crown_angle = ((height - EYE) / LANDMARK_VIEW_DIST_M).atan();
        assert!((crown_angle - top.pitch).abs() < 30f32.to_radians());
    }

    #[test]
    fn style_pose_picks_requested_style_and_faces_tower() {
        let center = Vec2::ZERO;
        let towers = [
            fake_tower(Vec2::new(10.0, 0.0), 0.0, TowerStyle::Brick, false),
            fake_tower(Vec2::new(50.0, 0.0), FRAC_PI_2, TowerStyle::Deco, false),
            fake_tower(Vec2::new(200.0, 0.0), 0.0, TowerStyle::Deco, false),
        ];
        let picked = nearest_tower_of_style(&towers, center, TowerStyle::Deco).expect("deco");
        assert_eq!(picked.style, TowerStyle::Deco);
        assert_eq!(picked.pos, Vec2::new(50.0, 0.0));

        let pose = style_pose(picked);
        let to_tower = (picked.pos - pose.pos).normalize();
        assert!(forward_of(pose.yaw).dot(to_tower) > 0.99);
    }
}
