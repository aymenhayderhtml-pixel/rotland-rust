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
    assert_eq!(MetroSpot::parse("hospital"), Some(MetroSpot::Hospital));
    assert_eq!(MetroSpot::parse("hospital_in"), Some(MetroSpot::HospitalIn));
    assert_eq!(MetroSpot::parse("police"), Some(MetroSpot::Police));
    assert_eq!(MetroSpot::parse("police_in"), Some(MetroSpot::PoliceIn));
    assert_eq!(MetroSpot::parse("fire"), Some(MetroSpot::Fire));
    assert_eq!(MetroSpot::parse("fire_in"), Some(MetroSpot::FireIn));
    assert_eq!(MetroSpot::parse("armyline"), Some(MetroSpot::Armyline));
    assert_eq!(MetroSpot::parse("armyline_top"), Some(MetroSpot::ArmylineTop));
    assert_eq!(MetroSpot::parse("climb"), Some(MetroSpot::Climb));
    assert_eq!(MetroSpot::parse("climb_stairs"), Some(MetroSpot::ClimbStairs));
    assert_eq!(MetroSpot::parse("climb_floor"), Some(MetroSpot::ClimbFloor));
    assert_eq!(MetroSpot::parse("climb_roof"), Some(MetroSpot::ClimbRoof));
    assert_eq!(MetroSpot::parse("roof_planks"), Some(MetroSpot::RoofPlanks));
    assert_eq!(MetroSpot::parse("fire_escape"), Some(MetroSpot::FireEscape));
    assert_eq!(MetroSpot::parse("radio"), Some(MetroSpot::Radio));
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

fn test_metro_world() -> MetroWorld {
    let cfg = world::metro::MetroConfig::load_from_file(paths::repo_path("data/metro.json"))
        .expect("data/metro.json must load");
    let ground_y = 10.0;
    let site = world::metro::MetroSite {
        key: cfg.key.clone(),
        name: cfg.name.clone(),
        center: Vec2::new(6000.0, 0.0),
        yaw: 0.3,
        half_size: (cfg.size_blocks as f32 * cfg.block_size) * 0.5,
        ground_y,
    };
    let layout = world::metro::MetroLayout::generate(site.clone(), &cfg);
    let towers = world::metro::plan_towers(&layout, &cfg);
    let entrance = site.to_world(Vec2::new(0.0, site.half_size));
    let out = (entrance - site.center).normalize();
    let highway = world::metro::Highway {
        from_town: "blackwater".into(),
        points: vec![entrance + out * 500.0, entrance],
        width: cfg.highway.highway_width,
    };
    let highway_field = world::metro::HighwayHeightField::build(&highway, ground_y, &|_x, _z| ground_y, &cfg);
    let jam = world::metro::plan_jam(&highway, &layout, &cfg);
    let towers_by_block = world::metro::state::index_towers_by_block(&towers);
    let damage = world::metro::damage_sites(&layout, &towers, world::metro::default_damage());
    let look = world::metro::MetroLook::load_from_file(paths::repo_path("data/metro.json")).expect("look");
    MetroWorld { config: cfg, layout, towers, highway, highway_field, jam, look, towers_by_block, damage }
}

#[test]
fn r43_poses_stand_at_right_height_and_face_targets() {
    use world::metro::{
        ClimbKind, climb_towers, default_styles, fire_escape_towers, roof_deck_y, roof_links,
        style_for,
    };

    let metro = test_metro_world();
    let ground_y = metro.layout.site.ground_y;
    let styles = default_styles();
    let plans = climb_towers(&metro);
    assert!(!plans.is_empty());

    let first_t = &metro.towers[plans[0].tower];
    let first_style = style_for(styles, first_t.style).unwrap();

    // 1. climb: lobby floor height, facing local +Z (yaw = t.yaw + PI)
    let climb = metro_pose(MetroSpot::Climb, &metro, None, 25.0, EYE).expect("climb");
    assert!(close(climb.y.unwrap(), ground_y + 0.15));
    let local_plus_z = Vec2::new(first_t.yaw.sin(), first_t.yaw.cos());
    assert!(forward_of(climb.yaw).dot(local_plus_z) > 0.999);

    // 2. climb_stairs: mid-storey flight A height, facing up flight A (local +Z)
    let stairs = metro_pose(MetroSpot::ClimbStairs, &metro, None, 25.0, EYE).expect("climb_stairs");
    let mid_floor = (first_t.floors / 2).min(first_t.floors.saturating_sub(2));
    let expected_stairs_y = ground_y + first_style.base_height + mid_floor as f32 * first_style.floor_height + 1.0;
    assert!(close(stairs.y.unwrap(), expected_stairs_y));
    assert!(forward_of(stairs.yaw).dot(local_plus_z) > 0.999);

    // 3. climb_floor: first stop floor slab + 0.05, facing local -Z (t.yaw)
    let floor = metro_pose(MetroSpot::ClimbFloor, &metro, None, 25.0, EYE).expect("climb_floor");
    let fi = plans[0].stop_floors[0];
    let expected_floor_y = ground_y + first_style.base_height + fi as f32 * first_style.floor_height + 0.35;
    assert!(close(floor.y.unwrap(), expected_floor_y));
    assert!(forward_of(floor.yaw).dot(tower_front_dir(first_t.yaw)) > 0.999);

    // 4. climb_roof: tallest non-landmark climb tower deck, facing tallest landmark
    let roof = metro_pose(MetroSpot::ClimbRoof, &metro, None, 25.0, EYE).expect("climb_roof");
    let non_lm = plans
        .iter()
        .filter(|p| p.kind != ClimbKind::Landmark)
        .max_by(|a, b| metro.towers[a.tower].height.total_cmp(&metro.towers[b.tower].height))
        .unwrap();
    let non_lm_t = &metro.towers[non_lm.tower];
    let non_lm_style = style_for(styles, non_lm_t.style).unwrap();
    assert!(close(roof.y.unwrap(), roof_deck_y(non_lm_t, non_lm_style, ground_y) + 0.1));
    let lm_t = &metro.towers[plans.iter().find(|p| p.kind == ClimbKind::Landmark).unwrap().tower];
    assert!(forward_of(roof.yaw).dot((lm_t.pos - roof.pos).normalize()) > 0.999);

    // 5. roof_planks: on from_tower deck, 4 m back from plank, facing along plank
    let planks = metro_pose(MetroSpot::RoofPlanks, &metro, None, 25.0, EYE).expect("roof_planks");
    let link = roof_links(&metro).into_iter().next().unwrap();
    let link_t = &metro.towers[link.from_tower];
    let link_style = style_for(styles, link_t.style).unwrap();
    assert!(close(planks.y.unwrap(), roof_deck_y(link_t, link_style, ground_y) + 0.1));
    let plank_dir = Vec2::new(link.plank_yaw.sin(), link.plank_yaw.cos());
    assert!(forward_of(planks.yaw).dot(plank_dir) > 0.999);

    // 6. fire_escape: on street 14 m from first fire-escape tower, facing it
    let fe = metro_pose(MetroSpot::FireEscape, &metro, None, 25.0, EYE).expect("fire_escape");
    assert!(fe.y.is_none());
    let fe_t = &metro.towers[fire_escape_towers(&metro)[0]];
    // 14 m out from the front face, looking straight at the escape's landing
    let front = tower_front_dir(fe_t.yaw);
    assert!(close((fe.pos - fe_t.pos).dot(front), fe_t.half.y + 14.0));
    let lateral = (fe.pos - fe_t.pos).dot(Vec2::new(front.y, -front.x));
    assert!(close(lateral.abs(), 2.0) || close(lateral.abs(), 4.0), "lateral {lateral}");
    let side = Vec2::new(front.y, -front.x);
    let landing = fe_t.pos + front * fe_t.half.y + side * lateral;
    assert!(forward_of(fe.yaw).dot((landing - fe.pos).normalize()) > 0.9999);

    // 7. radio: on mast platform (+18.22 m above crown), 2 m from console, facing hut
    let radio = metro_pose(MetroSpot::Radio, &metro, None, 25.0, EYE).expect("radio");
    let lm_style = style_for(styles, lm_t.style).unwrap();
    let crown_y = lm_style.base_height + lm_t.floors as f32 * lm_style.floor_height;
    let expected_radio_y = ground_y + crown_y + metro.config.climb.mast.platform_y + 0.22 + 0.05;
    assert!(close(radio.y.unwrap(), expected_radio_y));
    let (s, c) = lm_t.yaw.sin_cos();
    let console_xz = Vec2::new(lm_t.pos.x + (-0.2) * c + (-2.1) * s, lm_t.pos.y - (-0.2) * s + (-2.1) * c);
    assert!(close(radio.pos.distance(console_xz), 1.5));
    let hut_xz = Vec2::new(lm_t.pos.x + (-1.1) * c + (-0.6) * s, lm_t.pos.y - (-1.1) * s + (-0.6) * c);
    assert!(forward_of(radio.yaw).dot((hut_xz - radio.pos).normalize()) > 0.999);
}
