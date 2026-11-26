//! Poses for the `--metro=<spot>` test flags: where the player stands, which way it faces and,
//! for `top`, the camera pitch and spring length. Pure maths; `test_flags.rs` applies them.
//! Facing rule (AGENTS.md): yaw forward = (-sin yaw, -cos yaw), so yaw = atan2(-dx, -dz).

use glam::Vec2;
use world::metro::{MetroStreet, MetroWorld};

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

/// The five places `--metro=<spot>` can teleport to.
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

/// Where to teleport: world XZ, Godot yaw, and the camera override for `top`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MetroPose {
    pub pos: Vec2,
    pub yaw: f32,
    pub top: Option<TopView>,
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

/// `EDGE_INSET_M` along the avenue from `entrance` toward `far`, looking at `center`.
pub fn edge_pose(entrance: Vec2, far: Vec2, center: Vec2) -> MetroPose {
    let inward = (far - entrance).normalize_or_zero();
    let pos = entrance + inward * EDGE_INSET_M;
    MetroPose { pos, yaw: yaw_facing(center - pos), top: None }
}

/// The city centre, looking along the avenue direction `avenue_dir` (world XZ).
pub fn core_pose(center: Vec2, avenue_dir: Vec2) -> MetroPose {
    MetroPose { pos: center, yaw: yaw_facing(avenue_dir), top: None }
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
    Some(MetroPose { pos, yaw: yaw_facing(end - pos), top: None })
}

/// `distance` metres from the town centre on the side facing the city, looking at `center`.
pub fn town_view_pose(town_center: Vec2, center: Vec2, distance: f32) -> MetroPose {
    let toward_city = (center - town_center).normalize_or(Vec2::X);
    let pos = town_center + toward_city * distance;
    MetroPose { pos, yaw: yaw_facing(center - pos), top: None }
}

/// Above the Core centre, looking down `TOP_PITCH_DEG` toward the city from the highway side
/// (`from` is the highway's end on the city edge). The camera sits on the spring behind the pivot.
pub fn top_pose(center: Vec2, from: Vec2, eye_height: f32) -> MetroPose {
    let pitch = -TOP_PITCH_DEG.to_radians();
    let spring = top_spring(TOP_HEIGHT_M, eye_height, pitch);
    MetroPose { pos: center, yaw: yaw_facing(center - from), top: Some(TopView { pitch, spring }) }
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
}
