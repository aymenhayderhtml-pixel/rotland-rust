//! Unit tests for the R44 `--metro=` flags (`metro_flags_r44.rs`). Kept in a sibling file.

use glam::{Vec2, Vec3Swizzles};
use world::metro::safehouse::{rescue_floor_seeds, safe_house_layout, safe_house_spots};
use world::metro::{MetroWorld, TowerSpot};

use super::metro_flags::{MetroSpot, metro_pose};

/// Pose XZ must be this close to the ladder deck. // new: DECISIONS 2026-11-28 (test tolerance)
const DECK_TOLERANCE_M: f32 = 0.5;

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

/// Whether a world XZ point lies inside the city's square site.
fn inside_city(metro: &MetroWorld, p: Vec2) -> bool {
    let site = &metro.layout.site;
    let local = site.to_local(p);
    local.x.abs() <= site.half_size && local.y.abs() <= site.half_size
}

#[test]
fn r44_spot_names_parse_and_unknown_is_none() {
    assert_eq!(MetroSpot::parse("safehouse"), Some(MetroSpot::SafeHouse));
    assert_eq!(MetroSpot::parse("SAFEHOUSE_CLAIMED"), Some(MetroSpot::SafeHouseClaimed));
    assert_eq!(MetroSpot::parse("safehouse_bed"), Some(MetroSpot::SafeHouseBed));
    assert_eq!(MetroSpot::parse("rooftop_camp"), Some(MetroSpot::RooftopCamp));
    assert_eq!(MetroSpot::parse("rooftop_trader"), Some(MetroSpot::RooftopTrader));
    assert_eq!(MetroSpot::parse("rescue_floor"), Some(MetroSpot::RescueFloor));
    assert_eq!(MetroSpot::parse("safe_house"), None);
    assert_eq!(MetroSpot::parse("rooftop"), None);
}

#[test]
fn r44_poses_are_inside_the_city_and_exist() {
    let metro = test_metro_world();
    for spot in [
        MetroSpot::SafeHouse,
        MetroSpot::SafeHouseClaimed,
        MetroSpot::SafeHouseBed,
        MetroSpot::RooftopCamp,
        MetroSpot::RooftopTrader,
        MetroSpot::RescueFloor,
    ] {
        let pose = metro_pose(spot, &metro, None, 25.0, 1.6)
            .unwrap_or_else(|| panic!("{spot:?} has no pose"));
        assert!(inside_city(&metro, pose.pos), "{spot:?} at {:?} is outside the city", pose.pos);
        assert!(pose.y.is_some(), "{spot:?} needs a height");
    }
}

#[test]
fn safehouse_claimed_stands_on_the_ladder_deck_edge() {
    let metro = test_metro_world();
    let layout = safe_house_layout().expect("data/safe_house.json must load");
    let spots = safe_house_spots(&metro, layout, &|_, _| 0.0).expect("safe house tower");
    let pose = metro_pose(MetroSpot::SafeHouseClaimed, &metro, None, 25.0, 1.6).expect("pose");
    assert!(pose.pos.distance(spots.ladder_deck.xz()) <= DECK_TOLERANCE_M);
    let y = pose.y.expect("height");
    assert!((y - spots.deck_y).abs() < 0.2, "roof height {y} vs deck {}", spots.deck_y);
}

#[test]
fn safehouse_unclaimed_stands_at_the_travel_point_facing_the_sign() {
    let metro = test_metro_world();
    let layout = safe_house_layout().expect("layout");
    let spots = safe_house_spots(&metro, layout, &|_, _| 0.0).expect("spots");
    let pose = metro_pose(MetroSpot::SafeHouse, &metro, None, 25.0, 1.6).expect("pose");
    // Stands 0.5 m off the travel point toward the shack, so the shack and the sign are in reach.
    assert!(pose.pos.distance(spots.travel_point.xz()) < 0.51);
    assert!(pose.pos.distance(spots.shack.xz()) <= 2.0 && pose.pos.distance(spots.sign.xz()) <= 2.0);
    let to_sign = (spots.sign.xz() - pose.pos).normalize();
    let forward = Vec2::new(-pose.yaw.sin(), -pose.yaw.cos());
    assert!(forward.dot(to_sign) > 0.999, "faces the sign");
}

#[test]
fn rescue_floor_stands_on_the_seed_floor() {
    let metro = test_metro_world();
    let seed = rescue_floor_seeds(&metro).into_iter().next().expect("3 seeds");
    let pose = metro_pose(MetroSpot::RescueFloor, &metro, None, 25.0, 1.6).expect("pose");
    assert!(pose.pos.distance(seed.position.xz()) < 1e-3);
    assert!((pose.y.unwrap() - seed.position.y).abs() < 1e-4);
}

#[test]
fn rooftop_trader_pose_is_within_interact_reach_of_the_trader() {
    let metro = test_metro_world();
    let layout = safe_house_layout().expect("layout");
    let trader = layout.rooftop_camp.npcs.iter().find(|n| n.role.as_deref() == Some("trader")).expect("trader");
    let pose = metro_pose(MetroSpot::RooftopTrader, &metro, None, 25.0, 1.6).expect("pose");
    let tower: &TowerSpot = &metro.towers[world::metro::safehouse::rooftop_camp_tower(&metro).expect("camp")];
    let (sin_y, cos_y) = tower.yaw.sin_cos();
    let [lx, lz] = trader.local;
    let trader_xz = Vec2::new(tower.pos.x + lx * cos_y + lz * sin_y, tower.pos.y - lx * sin_y + lz * cos_y);
    assert!(pose.pos.distance(trader_xz) < 2.0, "trader within the 2 m use radius");
}

#[test]
fn safehouse_bed_pose_is_within_reach_of_the_bed_only() {
    let metro = test_metro_world();
    let layout = safe_house_layout().expect("layout");
    let spots = safe_house_spots(&metro, layout, &|_, _| 0.0).expect("spots");
    let pose = metro_pose(MetroSpot::SafeHouseBed, &metro, None, 25.0, 1.6).expect("pose");
    let reach = 2.0; // data: safe_house.json useRadiusM
    assert!(pose.pos.distance(spots.bed.xz()) <= reach);
    assert!(pose.pos.distance(spots.chest.xz()) > reach, "the chest must not be the nearest spot");
    assert!(pose.pos.distance(spots.ladder_deck.xz()) > reach);
}
