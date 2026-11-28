//! Unit tests for R44-03 (`safehouse.rs`): safe house tower, rooftop camp, rescue floor seeds.

use glam::{Vec2, Vec3};

use crate::metro::climb::{ClimbKind, StopKind, climb_in_chunk, climb_towers, roof_deck_y};
use crate::metro::config::MetroConfig;
use crate::metro::damage::{damage_sites, default_damage};
use crate::metro::fire_escapes::fire_escape_towers;
use crate::metro::highway::{Highway, HighwayHeightField};
use crate::metro::jam::plan_jam;
use crate::metro::layout::MetroLayout;
use crate::metro::paint::MetroLook;
use crate::metro::place::MetroSite;
use crate::metro::safehouse::{
    SafeHouseLayout, rescue_floor_seeds, rooftop_camp_in_chunk, rooftop_camp_tower, safe_house_layout,
    safe_house_spots, safe_house_tower, safehouse_in_chunk,
};
use crate::metro::state::{MetroWorld, index_towers_by_block};
use crate::metro::styles::{default_styles, style_for};
use crate::metro::towers::{TowerSpot, plan_towers};

/// Copy of the R44-common blocks (`shack`, `ladder`, `rooftopCamp`) used as the test layout.
const SAMPLE: &str = r#"{
  "travelHours": 3.0,
  "shack": { "centreLocal": [-4.0, -5.0], "bedLocal": [-4.6, -5.6], "chestLocal": [-3.2, -5.9],
             "signLocal": [-2.2, -4.0], "travelPointLocal": [-4.0, -3.0] },
  "ladder": { "footOutM": 4.0, "deckInsetM": 0.6, "centreX": 0.0, "segmentM": 1.0 },
  "rooftopCamp": {
    "mapLabel": "Rooftop Camp",
    "props": [
      { "model": "evac_tent", "local": [-5.0, 3.0], "yaw": 0.4 },
      { "model": "evac_tent", "local": [-5.0, -1.0], "yaw": -0.3 },
      { "model": "green_smoke_barrel", "local": [0.0, 2.0], "yaw": 0.0 },
      { "model": "rooftop_trader_stall", "local": [4.0, 0.0], "yaw": -1.5708 },
      { "model": "fire_barrel", "local": [0.5, -1.0], "yaw": 0.0 }
    ],
    "npcs": [
      { "id": "rooftop_survivor_1", "model": "npc_survivor", "local": [-3.0, 2.5], "yaw": 0.6, "sitting": true },
      { "id": "rooftop_survivor_2", "model": "npc_survivor", "local": [-1.0, -2.5], "yaw": 1.2, "sitting": false },
      { "id": "rooftop_survivor_3", "model": "npc_survivor", "local": [2.0, 3.0], "yaw": 3.0, "sitting": true },
      { "id": "rooftop_trader", "model": "npc_trader", "local": [2.8, 0.0], "yaw": -1.5708, "sitting": false, "role": "trader" }
    ]
  }
}"#;

fn test_metro() -> MetroWorld {
    let cfg = MetroConfig::load_from_file(paths::repo_path("data/metro.json"))
        .expect("data/metro.json must load");
    let site = MetroSite {
        key: cfg.key.clone(),
        name: cfg.name.clone(),
        center: Vec2::new(6000.0, 0.0),
        yaw: 0.3,
        half_size: (cfg.size_blocks as f32 * cfg.block_size) * 0.5,
        ground_y: 0.0,
    };
    let layout = MetroLayout::generate(site.clone(), &cfg);
    let towers = plan_towers(&layout, &cfg);
    let entrance = site.to_world(Vec2::new(0.0, site.half_size));
    let out = (entrance - site.center).normalize();
    let highway = Highway {
        from_town: "blackwater".into(),
        points: vec![entrance + out * 500.0, entrance],
        width: cfg.highway.highway_width,
    };
    let highway_field = HighwayHeightField::build(&highway, 0.0, &|_x, _z| 0.0, &cfg);
    let jam = plan_jam(&highway, &layout, &cfg);
    let towers_by_block = index_towers_by_block(&towers);
    let damage = damage_sites(&layout, &towers, default_damage());
    let look = MetroLook::load_from_file(paths::repo_path("data/metro.json")).expect("look");
    MetroWorld { config: cfg, layout, towers, highway, highway_field, jam, look, towers_by_block, damage }
}

fn sample() -> SafeHouseLayout {
    SafeHouseLayout::from_json(SAMPLE).expect("sample layout parses")
}

fn flat(_x: f32, _z: f32) -> f32 {
    0.0
}

fn chunk_of(pos: Vec2) -> (i32, i32) {
    ((pos.x / 64.0).floor() as i32, (pos.y / 64.0).floor() as i32)
}

/// Inverse of the tower-local frame: world XZ to tower-local `(lx, lz)`.
fn local_of(t: &TowerSpot, p: Vec3) -> (f32, f32) {
    let (sin_y, cos_y) = t.yaw.sin_cos();
    let dx = p.x - t.pos.x;
    let dz = p.z - t.pos.y;
    (dx * cos_y - dz * sin_y, dx * sin_y + dz * cos_y)
}

fn count_model(props: &[crate::scatter::PropInstance], model: &str) -> usize {
    props.iter().filter(|p| p.model == model).count()
}

#[test]
fn safe_house_tower_is_nearest_fire_escape_tower() {
    let m = test_metro();
    let picks = fire_escape_towers(&m);
    assert!(!picks.is_empty(), "fixture needs fire-escape towers");
    let st = safe_house_tower(&m).expect("safe house tower");
    assert!(picks.contains(&st));
    let centre = m.layout.site.center;
    let d = |i: usize| m.towers[i].pos.distance_squared(centre);
    for &p in &picks {
        assert!(d(st) <= d(p), "tower {p} is nearer than {st}");
    }
}

#[test]
fn safe_house_tower_is_plain_brick_not_climb_or_damaged() {
    let m = test_metro();
    let st = safe_house_tower(&m).expect("safe house tower");
    let t = &m.towers[st];
    assert!(!t.landmark);
    assert!(!climb_towers(&m).iter().any(|p| p.tower == st));
    assert!(!m.damage.tower_is_replaced(st));
    assert!(!m.damage.tower_is_burnt(t.block));
}

#[test]
fn rooftop_camp_tower_is_farthest_core_and_not_safe_house() {
    let m = test_metro();
    let st = safe_house_tower(&m).expect("safe house tower");
    let rc = rooftop_camp_tower(&m).expect("rooftop camp tower");
    assert_ne!(rc, st);
    let cores: Vec<usize> = climb_towers(&m)
        .iter()
        .filter(|p| p.kind == ClimbKind::Core)
        .map(|p| p.tower)
        .collect();
    assert!(!cores.is_empty(), "fixture needs Core climb towers");
    assert!(cores.contains(&rc));
    let safe = m.towers[st].pos;
    let d = |i: usize| m.towers[i].pos.distance_squared(safe);
    for &c in &cores {
        assert!(d(c) <= d(rc), "core tower {c} is farther than {rc}");
    }
}

#[test]
fn safe_house_spots_sit_on_the_roof_with_ladder_on_the_back_face() {
    let m = test_metro();
    let lay = sample();
    let s = safe_house_spots(&m, &lay, &flat).expect("spots");
    let t = &m.towers[s.tower];
    let style = style_for(default_styles(), t.style).expect("style");
    assert_eq!(s.deck_y, roof_deck_y(t, style, m.layout.site.ground_y));
    let half = t.half.x;
    let (_, foot_z) = local_of(t, s.ladder_foot);
    let (_, deck_z) = local_of(t, s.ladder_deck);
    assert!(foot_z > half, "ladder foot must be on the back face: {foot_z} vs {half}");
    assert!(deck_z < half, "ladder deck must be on the roof: {deck_z} vs {half}");
    for p in [s.shack, s.bed, s.chest, s.sign] {
        let (lx, lz) = local_of(t, p);
        assert!(lx.abs() < half && lz.abs() < half, "({lx}, {lz}) off the roof");
        assert_eq!(p.y, s.deck_y);
    }
}

#[test]
fn safehouse_chunk_emits_four_props_only_in_its_own_chunk() {
    let m = test_metro();
    let lay = sample();
    let s = safe_house_spots(&m, &lay, &flat).expect("spots");
    let home = chunk_of(m.towers[s.tower].pos);
    let c = safehouse_in_chunk(&m, &lay, home, &flat, true);
    let models: Vec<&str> = c
        .props
        .iter()
        .map(|p| p.model.as_str())
        .filter(|name| !name.starts_with("rope_ladder"))
        .collect();
    assert_eq!(models, ["safehouse_shack", "safehouse_bed", "safehouse_chest", "travel_sign"]);
    assert!(c.pois.is_empty() && c.crates.is_empty() && c.ladders.is_empty() && c.boxes.is_empty());
    for (i, t) in m.towers.iter().enumerate() {
        let ch = chunk_of(t.pos);
        if ch == home {
            continue;
        }
        let other = safehouse_in_chunk(&m, &lay, ch, &flat, true);
        assert!(other.props.is_empty(), "tower {i} got safe-house props");
    }
}

#[test]
fn ladder_up_is_one_coil_and_no_segments() {
    let m = test_metro();
    let lay = sample();
    let s = safe_house_spots(&m, &lay, &flat).expect("spots");
    let c = safehouse_in_chunk(&m, &lay, chunk_of(m.towers[s.tower].pos), &flat, false);
    assert_eq!(count_model(&c.props, "rope_ladder_coil"), 1);
    assert_eq!(count_model(&c.props, "rope_ladder_seg"), 0);
}

#[test]
fn ladder_down_segment_count_matches_height() {
    let m = test_metro();
    let lay = sample();
    let s = safe_house_spots(&m, &lay, &flat).expect("spots");
    let c = safehouse_in_chunk(&m, &lay, chunk_of(m.towers[s.tower].pos), &flat, true);
    let segs: Vec<&Vec3> = c.props.iter().filter(|p| p.model == "rope_ladder_seg").map(|p| &p.position).collect();
    let rise = s.ladder_deck.y - s.ladder_foot.y;
    let expected = (rise / lay.ladder.segment_m).ceil() as usize;
    assert!(segs.len().abs_diff(expected) <= 1, "segments {} vs expected {expected}", segs.len());
    assert_eq!(count_model(&c.props, "rope_ladder_coil"), 0);
    for p in segs {
        assert!(p.y >= s.ladder_foot.y && p.y <= s.ladder_deck.y);
    }
}

#[test]
fn rooftop_camp_chunk_has_five_props_and_four_npcs() {
    let m = test_metro();
    let lay = sample();
    let rc = rooftop_camp_tower(&m).expect("rooftop camp tower");
    let ch = chunk_of(m.towers[rc].pos);
    let (c, npcs) = rooftop_camp_in_chunk(&m, &lay, ch);
    assert_eq!(c.props.len(), 5);
    assert_eq!(npcs.len(), 4);
    assert_eq!(npcs.iter().filter(|n| n.role == "trader").count(), 1);
    assert_eq!(npcs.iter().filter(|n| n.role.is_empty()).count(), 3);
    let (far, far_npcs) = rooftop_camp_in_chunk(&m, &lay, (ch.0 + 500, ch.1 + 500));
    assert!(far.props.is_empty() && far_npcs.is_empty());
}

#[test]
fn rescue_floor_seeds_are_three_valid_stop_floors() {
    let m = test_metro();
    let seeds = rescue_floor_seeds(&m);
    assert_eq!(seeds.len(), 3);
    let plans = climb_towers(&m);
    let mut towers_used = Vec::new();
    for s in &seeds {
        let rest = s.poi_id.strip_prefix("metro_floor_").expect("poi id prefix");
        let mut parts = rest.split('_');
        let tower: usize = parts.next().and_then(|v| v.parse().ok()).expect("tower");
        let floor: u32 = parts.next().and_then(|v| v.parse().ok()).expect("floor");
        let plan = plans.iter().find(|p| p.tower == tower).expect("climb tower");
        assert_ne!(floor, m.towers[tower].floors - 1, "top floor is not a rescue floor");
        let idx = plan.stop_floors.iter().position(|&f| f == floor).expect("stop floor");
        assert_ne!(plan.stop_kinds[idx], StopKind::Camp);
        let pois = climb_in_chunk(&m, chunk_of(m.towers[tower].pos), &flat).pois;
        assert!(
            pois.iter().any(|p| p.id == s.poi_id && p.building_id == s.building_id),
            "seed {} not in climb POIs",
            s.poi_id
        );
        towers_used.push(tower);
    }
    towers_used.sort_unstable();
    towers_used.dedup();
    assert_eq!(towers_used.len(), 3, "seeds should sit on different towers");
}

#[test]
fn rescue_floor_seeds_are_deterministic() {
    let m = test_metro();
    assert_eq!(rescue_floor_seeds(&m), rescue_floor_seeds(&m));
}

#[test]
fn sample_layout_parses_every_block() {
    let lay = sample();
    assert_eq!(lay.shack.centre_local, [-4.0, -5.0]);
    assert_eq!(lay.ladder.segment_m, 1.0);
    assert_eq!(lay.rooftop_camp.map_label, "Rooftop Camp");
    let roles: Vec<Option<&str>> = lay.rooftop_camp.npcs.iter().map(|n| n.role.as_deref()).collect();
    assert_eq!(roles, [None, None, None, Some("trader")]);
}

#[test]
fn shipped_safe_house_json_parses_when_present() {
    let path = paths::repo_path("data/safe_house.json");
    if let Ok(raw) = std::fs::read_to_string(&path) {
        assert!(SafeHouseLayout::from_json(&raw).is_ok(), "{} must parse", path.display());
        assert!(safe_house_layout().is_some());
    }
}
