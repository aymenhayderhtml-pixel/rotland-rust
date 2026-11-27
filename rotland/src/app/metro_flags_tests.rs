//! Unit tests for `metro_flags.rs` (kept in a sibling file to stay under the size rule).

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
