use std::f32::consts::{FRAC_PI_2, PI};
use glam::Vec3;
use crate::camp_layout::buildings::camp_gunsmith_prop_with_config;
use crate::camp_layout::{
    camp_upgrade_props, camp_upgrade_props_with_config, wall_section_count,
    CampLayoutConfigData, CampLevels, TowerCorner,
};
use crate::settlement::SettlementSite;

#[test]
fn camp_layout_level_0_empty() {
    let site = SettlementSite::new("camp_test", Vec3::ZERO, 0.0);
    let props = camp_upgrade_props(&site, CampLevels::default());
    assert!(props.is_empty(), "level 0 should yield empty props");
}

#[test]
fn camp_layout_wall_ring_closed_except_gate_gap() {
    let site = SettlementSite::new("camp_test", Vec3::ZERO, 0.0);
    let levels = CampLevels {
        walls: 1,
        guards: 0,
        workshop: 0,
        traders: 0,
        gunsmith: 0,
    };
    let props = camp_upgrade_props(&site, levels);

    let gate = props
        .iter()
        .find(|p| p.model == "camp_gate_wood")
        .expect("gate prop missing");
    assert!(gate.position.x.abs() < 1e-4);
    assert!((gate.position.z - (-20.0)).abs() < 1e-4);
    assert!((gate.yaw - 0.0).abs() < 1e-4);

    let north_walls: Vec<_> = props
        .iter()
        .filter(|p| (p.position.z - (-20.0)).abs() < 1e-4 && p.model == "camp_wall_wood")
        .collect();
    assert_eq!(north_walls.len(), 8);
    for w in &north_walls {
        assert!(w.position.x.abs() >= 4.0 - 1e-4, "wall inside gate opening");
    }

    let south_walls: Vec<_> = props
        .iter()
        .filter(|p| (p.position.z - 20.0).abs() < 1e-4)
        .collect();
    assert_eq!(south_walls.len(), 10);
    for w in &south_walls {
        assert_eq!(w.model, "camp_wall_wood");
        assert!((w.yaw - PI).abs() < 1e-4);
    }

    let east_walls: Vec<_> = props
        .iter()
        .filter(|p| (p.position.x - 20.0).abs() < 1e-4)
        .collect();
    assert_eq!(east_walls.len(), 10);
    for w in &east_walls {
        assert_eq!(w.model, "camp_wall_wood");
        assert!((w.yaw - (-FRAC_PI_2)).abs() < 1e-4);
    }

    let west_walls: Vec<_> = props
        .iter()
        .filter(|p| (p.position.x - (-20.0)).abs() < 1e-4)
        .collect();
    assert_eq!(west_walls.len(), 10);
    for w in &west_walls {
        assert_eq!(w.model, "camp_wall_wood");
        assert!((w.yaw - FRAC_PI_2).abs() < 1e-4);
    }
}

#[test]
fn camp_layout_section_count_matches_perimeter() {
    let cfg = CampLayoutConfigData::load_default();
    let perimeter = 8.0 * cfg.wall_radius;
    let total_4m_sections = (perimeter / cfg.wall_section_length).round() as usize;
    let gate_equiv_sections = (cfg.gate_width / cfg.wall_section_length).round() as usize;
    let expected_walls = total_4m_sections - gate_equiv_sections;

    assert_eq!(
        wall_section_count(cfg.wall_radius, cfg.wall_section_length, cfg.gate_width),
        expected_walls
    );

    let site = SettlementSite::new("camp_test", Vec3::ZERO, 0.0);
    for lvl in 1..=3 {
        let levels = CampLevels {
            walls: lvl,
            guards: 0,
            workshop: 0,
            traders: 0,
            gunsmith: 0,
        };
        let props = camp_upgrade_props(&site, levels);
        let walls: Vec<_> = props.iter().filter(|p| p.model.starts_with("camp_wall_")).collect();
        let gates: Vec<_> = props.iter().filter(|p| p.model.starts_with("camp_gate_")).collect();
        assert_eq!(walls.len(), expected_walls, "wall section count mismatch at level {lvl}");
        assert_eq!(gates.len(), 1, "gate count mismatch at level {lvl}");

        let covered = walls.len() as f32 * cfg.wall_section_length + cfg.gate_width;
        assert!((covered - perimeter).abs() < 1e-4, "total length covered does not match perimeter");
    }
}

#[test]
fn camp_layout_towers_at_right_corners_per_level() {
    let site = SettlementSite::new("camp_test", Vec3::ZERO, 0.0);

    let props0 = camp_upgrade_props(&site, CampLevels { guards: 0, ..Default::default() });
    assert!(props0.iter().all(|p| !p.model.contains("tower")));

    let props1 = camp_upgrade_props(&site, CampLevels { guards: 1, ..Default::default() });
    let towers1: Vec<_> = props1.iter().filter(|p| p.model.contains("tower")).collect();
    assert_eq!(towers1.len(), 1);
    assert_eq!(towers1[0].model, "camp_tower_wood");
    assert!((towers1[0].position.x - 17.5).abs() < 1e-4);
    assert!((towers1[0].position.z - (-17.5)).abs() < 1e-4);

    let props2 = camp_upgrade_props(&site, CampLevels { guards: 2, ..Default::default() });
    let towers2: Vec<_> = props2.iter().filter(|p| p.model.contains("tower")).collect();
    assert_eq!(towers2.len(), 1);
    assert_eq!(towers2[0].model, "camp_tower_wood");
    assert_eq!(towers2[0].position, towers1[0].position);

    let props3 = camp_upgrade_props(&site, CampLevels { guards: 3, ..Default::default() });
    let towers3: Vec<_> = props3.iter().filter(|p| p.model.contains("tower")).collect();
    assert_eq!(towers3.len(), 2);
    for t in &towers3 {
        assert_eq!(t.model, "camp_tower_metal");
    }
    let ne = towers3.iter().find(|t| (t.position.x - 17.5).abs() < 1e-4 && (t.position.z - (-17.5)).abs() < 1e-4);
    let sw = towers3.iter().find(|t| (t.position.x - (-17.5)).abs() < 1e-4 && (t.position.z - 17.5).abs() < 1e-4);
    assert!(ne.is_some(), "missing NE tower");
    assert!(sw.is_some(), "missing SW tower");
}

#[test]
fn camp_layout_nothing_overlaps() {
    let site = SettlementSite::new("camp_test", Vec3::ZERO, 0.0);
    let levels = CampLevels {
        walls: 3,
        guards: 3,
        workshop: 3,
        traders: 3,
        gunsmith: 3,
    };
    let cfg = CampLayoutConfigData::load_default();
    let mut props = camp_upgrade_props_with_config(&site, levels, &cfg);
    props.push(camp_gunsmith_prop_with_config(&site, &cfg));

    struct Box2D {
        name: &'static str,
        min_x: f32,
        max_x: f32,
        min_z: f32,
        max_z: f32,
    }
    impl Box2D {
        fn overlaps(&self, other: &Box2D) -> bool {
            !(self.max_x <= other.min_x
                || self.min_x >= other.max_x
                || self.max_z <= other.min_z
                || self.min_z >= other.max_z)
        }
    }

    let existing = vec![
        Box2D { name: "trader_stall", min_x: -11.0, max_x: -6.0, min_z: -9.5, max_z: -5.5 },
        Box2D { name: "bank_shack", min_x: 5.8, max_x: 10.2, min_z: -9.5, max_z: -5.5 },
        Box2D { name: "guard_platform_1", min_x: -8.8, max_x: -5.8, min_z: -19.3, max_z: -16.2 },
        Box2D { name: "guard_platform_2", min_x: 5.8, max_x: 8.8, min_z: -19.3, max_z: -16.2 },
        Box2D { name: "tent_1", min_x: -14.3, max_x: -11.7, min_z: 6.4, max_z: 9.6 },
        Box2D { name: "tent_2", min_x: 10.7, max_x: 13.3, min_z: 6.4, max_z: 9.6 },
        Box2D { name: "campfire", min_x: -1.6, max_x: 1.6, min_z: -1.6, max_z: 1.6 },
        Box2D { name: "water_barrel", min_x: -5.0, max_x: -4.0, min_z: 2.0, max_z: 3.0 },
        Box2D { name: "crates_east", min_x: 7.8, max_x: 10.4, min_z: -3.6, max_z: -1.4 },
        Box2D { name: "flagpole", min_x: -0.5, max_x: 0.5, min_z: 14.5, max_z: 15.5 },
        Box2D { name: "woodpile", min_x: -19.0, max_x: -17.0, min_z: -0.5, max_z: 3.5 },
        Box2D { name: "gate_lane", min_x: -3.0, max_x: 3.0, min_z: -19.0, max_z: -2.0 },
    ];

    let mut upgrade_boxes = Vec::new();
    for p in &props {
        let (hw, hd) = match p.model.as_str() {
            "camp_workshop_3" => (3.0, 2.0),
            "camp_workshop_2" => (1.5, 1.25),
            "camp_workshop_1" => (1.3, 1.0),
            "camp_doctor" => (2.5, 2.0),
            "camp_mechanic" => (3.5, 2.5),
            "gunsmith_bench" => (1.25, 0.75),
            "camp_tower_wood" | "camp_tower_metal" => (1.5, 1.5),
            _ => continue,
        };
        let b = Box2D {
            name: "upgrade",
            min_x: p.position.x - hw,
            max_x: p.position.x + hw,
            min_z: p.position.z - hd,
            max_z: p.position.z + hd,
        };

        assert!(b.min_x > -20.0 && b.max_x < 20.0, "{}: exceeds wall X", p.model);
        assert!(b.min_z > -20.0 && b.max_z < 20.0, "{}: exceeds wall Z", p.model);

        for ex in &existing {
            assert!(
                !b.overlaps(ex),
                "{} at ({}, {}) overlaps existing {}",
                p.model, p.position.x, p.position.z, ex.name
            );
        }

        upgrade_boxes.push((p.model.clone(), b));
    }

    for i in 0..upgrade_boxes.len() {
        for j in (i + 1)..upgrade_boxes.len() {
            let (m1, b1) = &upgrade_boxes[i];
            let (m2, b2) = &upgrade_boxes[j];
            assert!(
                !b1.overlaps(b2),
                "upgrade {} overlaps upgrade {}",
                m1, m2
            );
        }
    }
}

#[test]
fn camp_layout_yaw_rotates_everything() {
    let site0 = SettlementSite::new("c0", Vec3::new(100.0, 50.0, 200.0), 0.0);
    let site_rot = SettlementSite::new("cr", Vec3::new(100.0, 50.0, 200.0), FRAC_PI_2);
    let levels = CampLevels {
        walls: 2,
        guards: 3,
        workshop: 2,
        traders: 3,
        gunsmith: 0,
    };

    let p0 = camp_upgrade_props(&site0, levels);
    let pr = camp_upgrade_props(&site_rot, levels);

    assert_eq!(p0.len(), pr.len());
    for (a, b) in p0.iter().zip(pr.iter()) {
        assert_eq!(a.model, b.model);

        let local_a = a.position - site0.center;
        let local_b = b.position - site_rot.center;

        assert!((local_b.x - local_a.z).abs() < 1e-3, "model {} pos.x rotation failed", a.model);
        assert!((local_b.z - (-local_a.x)).abs() < 1e-3, "model {} pos.z rotation failed", a.model);

        let diff = (b.yaw - a.yaw - FRAC_PI_2).rem_euclid(2.0 * PI);
        assert!(
            diff.abs() < 1e-3 || (diff - 2.0 * PI).abs() < 1e-3,
            "model {} yaw rotation failed",
            a.model
        );
    }
}

#[test]
fn camp_layout_workshop_tiers() {
    let site = SettlementSite::new("camp_test", Vec3::ZERO, 0.0);
    for (lvl, expected_model) in [(1, "camp_workshop_1"), (2, "camp_workshop_2"), (3, "camp_workshop_3")] {
        let props = camp_upgrade_props(&site, CampLevels { workshop: lvl, ..Default::default() });
        assert_eq!(props.len(), 1);
        assert_eq!(props[0].model, expected_model);
    }
}

#[test]
fn camp_layout_trader_tiers() {
    let site = SettlementSite::new("camp_test", Vec3::ZERO, 0.0);
    let p1 = camp_upgrade_props(&site, CampLevels { traders: 1, ..Default::default() });
    assert!(p1.is_empty());

    let p2 = camp_upgrade_props(&site, CampLevels { traders: 2, ..Default::default() });
    assert_eq!(p2.len(), 1);
    assert_eq!(p2[0].model, "camp_doctor");

    let p3 = camp_upgrade_props(&site, CampLevels { traders: 3, ..Default::default() });
    assert_eq!(p3.len(), 2);
    let models: Vec<_> = p3.iter().map(|p| p.model.as_str()).collect();
    assert!(models.contains(&"camp_doctor"));
    assert!(models.contains(&"camp_mechanic"));
}

#[test]
fn camp_layout_config_loading() {
    let path = paths::repo_path("data/camp_layout.json");
    let loaded = CampLayoutConfigData::load_from_file(&path).expect("failed to load from file");
    assert_eq!(loaded.wall_radius, 20.0);

    let json = std::fs::read_to_string(&path).expect("failed to read json");
    let from_str = CampLayoutConfigData::from_json(&json).expect("failed to parse json");
    assert_eq!(from_str.wall_radius, 20.0);

    let default_cfg = CampLayoutConfigData::load_default();
    assert_eq!(default_cfg.wall_radius, 20.0);

    let corner = TowerCorner { x: 10.0, z: 20.0, yaw: 0.0 };
    assert_eq!(corner.x, 10.0);

    let site = SettlementSite::new("camp_test", Vec3::ZERO, 0.0);
    let custom_props = camp_upgrade_props_with_config(&site, CampLevels::default(), &loaded);
    assert!(custom_props.is_empty());
}

#[test]
fn camp_layout_gunsmith_bench_placed_and_rotates() {
    let cfg = CampLayoutConfigData::load_from_file(paths::repo_path("data/camp_layout.json"))
        .expect("data/camp_layout.json must load");
    let site0 = SettlementSite::new("camp_0", Vec3::new(100.0, 50.0, 200.0), 0.0);
    let bench0 = camp_gunsmith_prop_with_config(&site0, &cfg);
    assert_eq!(bench0.model, "gunsmith_bench");
    assert!((bench0.position.x - (100.0 + cfg.gunsmith_offset[0])).abs() < 1e-4);
    assert!((bench0.position.z - (200.0 + cfg.gunsmith_offset[1])).abs() < 1e-4);

    let site_rot = SettlementSite::new("camp_r", Vec3::new(100.0, 50.0, 200.0), FRAC_PI_2);
    let bench_r = camp_gunsmith_prop_with_config(&site_rot, &cfg);
    let local0 = bench0.position - site0.center;
    let local_r = bench_r.position - site_rot.center;
    assert!((local_r.x - local0.z).abs() < 1e-3);
    assert!((local_r.z - (-local0.x)).abs() < 1e-3);
    assert!((bench_r.yaw - (bench0.yaw + FRAC_PI_2)).abs() < 1e-4);
}
