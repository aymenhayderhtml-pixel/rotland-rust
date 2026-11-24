//! Unit tests for `StanceTable`, `StanceController`, and `data/stances.json` coverage.

use glam::Vec3;

use crate::stance::{HoldTransform, StanceClass, StanceController, StanceTable};
use crate::weapon::{off_hand_grip_for, weapon_hold_for};

const EPS: f32 = 1e-4; // new: DECISIONS 2026-11-23
const STEP_BIG: f32 = 0.1; // new: DECISIONS 2026-11-23
const STEP_SMALL: f32 = 0.01; // new: DECISIONS 2026-11-23
const SMALL_STEPS: usize = 10; // new: DECISIONS 2026-11-23
const SETTLE_DT: f32 = 2.0; // new: DECISIONS 2026-11-23

fn load_real_table() -> StanceTable {
    let text = std::fs::read_to_string(paths::repo_path("data/stances.json"))
        .expect("read data/stances.json");
    StanceTable::from_json(&text).expect("parse data/stances.json")
}

fn rates_of(table: &StanceTable) -> (f32, f32, f32) {
    (table.ready_rate(), table.aim_rate(), table.ik_rate())
}

#[test]
fn real_stances_json_parses_and_matches_defaults() {
    let table = load_real_table();
    assert!((table.ready_rate() - 10.0).abs() < EPS); // data: stances.json readyBlendRate
    assert!((table.aim_rate() - 14.0).abs() < EPS); // data: stances.json aimBlendRate
    assert!((table.ik_rate() - 14.0).abs() < EPS); // data: stances.json ikBlendRate

    let bat = table.for_weapon("bat").expect("bat stance");
    let (bat_pos, bat_rot) = weapon_hold_for("bat");
    assert_eq!(bat.class, StanceClass::Melee2H);
    assert_eq!(bat.ready_hold, HoldTransform { pos: bat_pos, rot_deg: bat_rot });
    assert_eq!(bat.off_hand_grip, off_hand_grip_for("bat"));

    let rifle = table.for_weapon("rifle").expect("rifle stance");
    assert_eq!(rifle.class, StanceClass::Rifle);
    assert_eq!(rifle.off_hand_grip, off_hand_grip_for("rifle"));
    assert_eq!(rifle.aim_clip.as_deref(), Some("aim_rifle_shoulder"));
}

#[test]
fn every_weapon_and_fishing_rod_has_a_stance() {
    let table = load_real_table();
    let weapons_text = std::fs::read_to_string(paths::repo_path("data/weapons.json"))
        .expect("read data/weapons.json");
    let weapons_val: serde_json::Value =
        serde_json::from_str(&weapons_text).expect("parse data/weapons.json");
    let map = weapons_val.as_object().expect("weapons.json top-level object");
    for id in map.keys() {
        assert!(
            table.for_weapon(id).is_some(),
            "weapon '{id}' from data/weapons.json has no stance in data/stances.json"
        );
    }
    let rod = table.for_weapon("fishing_rod").expect("fishing_rod stance");
    assert_eq!(rod.class, StanceClass::Rod);
    assert!(rod.aim_clip.is_none());
}

#[test]
fn every_ready_clip_is_defined_in_poses_stance_py() {
    let stances_text = std::fs::read_to_string(paths::repo_path("data/stances.json"))
        .expect("read data/stances.json");
    let stances_val: serde_json::Value =
        serde_json::from_str(&stances_text).expect("parse data/stances.json");
    let py_text = std::fs::read_to_string(paths::repo_path(
        "tools/blender/characters/poses_stance.py",
    ))
    .expect("read poses_stance.py");

    let classes = stances_val["classes"].as_object().expect("classes object");
    assert!(!classes.is_empty());
    for (class_name, def) in classes {
        let clip = def["readyClip"].as_str().expect("readyClip string");
        let needle = format!("\"{clip}\":");
        assert!(
            py_text.contains(&needle),
            "readyClip '{clip}' for class '{class_name}' missing from poses_stance.py"
        );
    }
}

#[test]
fn unknown_weapon_falls_back_safely() {
    let table = load_real_table();
    assert!(table.for_weapon("unknown_laser_cannon").is_none());

    let mut ctrl = StanceController::new();
    ctrl.set_weapon(table.for_weapon("unknown_laser_cannon"));
    ctrl.set_aiming(true);
    ctrl.tick(STEP_BIG, rates_of(&table), false);
    assert!(!ctrl.is_aiming());
    assert!(ctrl.ready_clip().is_none());
    assert!(ctrl.aim_clip().is_none());
    assert_eq!(ctrl.ready_weight(), 0.0); // new: DECISIONS 2026-11-23
    assert_eq!(ctrl.aim_weight(), 0.0); // new: DECISIONS 2026-11-23
    assert_eq!(ctrl.left_ik_weight(), 0.0); // new: DECISIONS 2026-11-23
    assert_eq!(ctrl.hold(), HoldTransform::default());
}

#[test]
fn aim_is_ignored_for_spear_and_rod() {
    let table = load_real_table();
    let rates = rates_of(&table);
    // Knife and bat aim = block (A13b, DECISIONS 2026-11-24); spear throws, rod fishes.
    for id in ["spear", "fishing_rod"] {
        let mut ctrl = StanceController::new();
        ctrl.set_weapon(table.for_weapon(id));
        ctrl.set_aiming(true);
        ctrl.tick(SETTLE_DT, rates, false);
        assert!(!ctrl.is_aiming(), "{id} must ignore aiming");
        assert_eq!(ctrl.aim_weight(), 0.0, "{id} aim_weight must stay 0"); // new: DECISIONS 2026-11-23
        assert!(ctrl.aim_clip().is_none(), "{id} must have no aim_clip");
    }
}

#[test]
fn weights_fade_stay_in_range_and_respect_action_and_bow_grip() {
    let table = load_real_table();
    let rates = rates_of(&table);
    let mut ctrl = StanceController::new();

    ctrl.set_weapon(table.for_weapon("rifle"));
    ctrl.tick(STEP_BIG, rates, false);
    assert!(ctrl.ready_weight() > 0.0 && ctrl.ready_weight() < 1.0); // new: DECISIONS 2026-11-23
    assert!(ctrl.left_ik_weight() > 0.0 && ctrl.left_ik_weight() < 1.0); // new: DECISIONS 2026-11-23

    ctrl.tick(SETTLE_DT, rates, false);
    assert!(ctrl.ready_weight() <= 1.0 && ctrl.ready_weight() > 0.99); // new: DECISIONS 2026-11-23
    assert!(ctrl.left_ik_weight() <= 1.0 && ctrl.left_ik_weight() > 0.99); // new: DECISIONS 2026-11-23

    // Non-melee_2h action releases stance left-hand IK smoothly.
    ctrl.tick(STEP_BIG, rates, true);
    assert!(ctrl.left_ik_weight() >= 0.0 && ctrl.left_ik_weight() < 0.5); // new: DECISIONS 2026-11-23

    // Bow only engages left-hand IK while aiming.
    ctrl.set_weapon(table.for_weapon("bow"));
    ctrl.tick(SETTLE_DT, rates, false);
    assert_eq!(ctrl.left_ik_weight(), 0.0); // new: DECISIONS 2026-11-23
    ctrl.set_aiming(true);
    ctrl.tick(SETTLE_DT, rates, false);
    assert!(ctrl.left_ik_weight() > 0.99); // new: DECISIONS 2026-11-23
}

#[test]
fn frame_rate_independent_blending() {
    let table = load_real_table();
    let rates = rates_of(&table);
    let rifle = table.for_weapon("rifle");

    let mut big = StanceController::new();
    big.set_weapon(rifle);
    big.set_aiming(true);
    big.tick(STEP_BIG, rates, false);

    let mut small = StanceController::new();
    small.set_weapon(rifle);
    small.set_aiming(true);
    for _ in 0..SMALL_STEPS {
        small.tick(STEP_SMALL, rates, false);
    }

    assert!((big.ready_weight() - small.ready_weight()).abs() < EPS);
    assert!((big.aim_weight() - small.aim_weight()).abs() < EPS);
    assert!((big.left_ik_weight() - small.left_ik_weight()).abs() < EPS);
}

#[test]
fn weapon_swap_resets_aim_and_clears_old_clips() {
    let table = load_real_table();
    let rates = rates_of(&table);
    let mut ctrl = StanceController::new();

    ctrl.set_weapon(table.for_weapon("rifle"));
    ctrl.set_aiming(true);
    ctrl.tick(SETTLE_DT, rates, false);
    assert!(ctrl.is_aiming());
    assert!(ctrl.aim_weight() > 0.99); // new: DECISIONS 2026-11-23

    // Re-setting the same weapon keeps current aim state.
    ctrl.set_weapon(table.for_weapon("rifle"));
    assert!(ctrl.is_aiming());

    // Swapping to pistol resets aim and fades in the new stance from zero.
    ctrl.set_weapon(table.for_weapon("pistol"));
    assert!(!ctrl.is_aiming());
    assert_eq!(ctrl.ready_weight(), 0.0); // new: DECISIONS 2026-11-23
    assert_eq!(ctrl.aim_weight(), 0.0); // new: DECISIONS 2026-11-23
    assert_eq!(ctrl.left_ik_weight(), 0.0); // new: DECISIONS 2026-11-23
    assert_eq!(ctrl.ready_clip(), Some("stance_pistol_ready"));
    assert_eq!(ctrl.aim_clip(), Some("aim_pistol_two_hand"));

    // Swapping to melee swaps the aim clip for the block pose (A13b).
    ctrl.set_weapon(table.for_weapon("bat"));
    assert_eq!(ctrl.ready_clip(), Some("stance_melee2h_ready"));
    assert_eq!(ctrl.aim_clip(), Some("block_2h"));
}

#[test]
fn hold_blends_from_ready_to_aim() {
    let table = load_real_table();
    let rates = rates_of(&table);
    let rifle = table.for_weapon("rifle").expect("rifle stance");
    let mut ctrl = StanceController::new();
    ctrl.set_weapon(Some(rifle));

    assert_eq!(ctrl.hold(), rifle.ready_hold);

    ctrl.set_aiming(true);
    ctrl.tick(STEP_BIG, rates, false);
    let w = ctrl.aim_weight();
    let expected_rot = rifle.ready_hold.rot_deg.lerp(rifle.aim_hold.rot_deg, w);
    let mid = ctrl.hold();
    assert!((mid.rot_deg - expected_rot).length() < EPS);

    ctrl.tick(SETTLE_DT, rates, false);
    let aimed = ctrl.hold();
    assert!((aimed.pos - rifle.aim_hold.pos).length() < EPS);
    assert!((aimed.rot_deg - rifle.aim_hold.rot_deg).length() < EPS);
}

#[test]
fn overrides_replace_only_specified_fields() {
    let json = r#"{
        "version": 1,
        "readyBlendRate": 10.0,
        "aimBlendRate": 14.0,
        "ikBlendRate": 14.0,
        "classes": {
            "rifle": {
                "readyClip": "stance_rifle_ready",
                "aimClip": "aim_rifle_shoulder",
                "leftHandGrip": true,
                "gripWhenAiming": true,
                "readyHold": { "pos": [1.0, 2.0, 3.0], "rot": [90.0, 0.0, 0.0] },
                "aimHold": { "pos": [0.1, 0.2, 0.3], "rot": [5.0, 6.0, 7.0] }
            }
        },
        "weapons": { "rifle": "rifle", "ak47": "rifle" },
        "overrides": {
            "rifle": { "readyHold": { "pos": [4.0, 5.0, 6.0], "rot": [10.0, 20.0, 30.0] } },
            "ak47": { "offHandGrip": [0.0, 0.5, -0.5] }
        }
    }"#;
    let table = StanceTable::from_json(json).expect("parse override test");

    let rifle = table.for_weapon("rifle").expect("rifle");
    assert_eq!(rifle.ready_hold.pos, Vec3::new(4.0, 5.0, 6.0)); // new: DECISIONS 2026-11-23
    assert_eq!(rifle.ready_hold.rot_deg, Vec3::new(10.0, 20.0, 30.0)); // new: DECISIONS 2026-11-23
    assert_eq!(rifle.aim_hold.pos, Vec3::new(0.1, 0.2, 0.3)); // new: DECISIONS 2026-11-23
    assert_eq!(rifle.aim_hold.rot_deg, Vec3::new(5.0, 6.0, 7.0)); // new: DECISIONS 2026-11-23
    assert_eq!(rifle.off_hand_grip, off_hand_grip_for("rifle"));

    let ak = table.for_weapon("ak47").expect("ak47");
    assert_eq!(ak.ready_hold.pos, Vec3::new(1.0, 2.0, 3.0)); // new: DECISIONS 2026-11-23
    assert_eq!(ak.aim_hold.pos, Vec3::new(0.1, 0.2, 0.3)); // new: DECISIONS 2026-11-23
    assert_eq!(ak.off_hand_grip, Some(Vec3::new(0.0, 0.5, -0.5))); // new: DECISIONS 2026-11-23
}

#[test]
fn dt_zero_and_non_finite_produce_no_nan() {
    let table = load_real_table();
    let rates = rates_of(&table);
    let mut ctrl = StanceController::new();
    ctrl.set_weapon(table.for_weapon("rifle"));
    ctrl.set_aiming(true);

    for bad_dt in [0.0, -0.1, f32::NAN, f32::INFINITY] {
        // new: DECISIONS 2026-11-23
        ctrl.tick(bad_dt, rates, false);
        assert!(ctrl.ready_weight().is_finite());
        assert!(ctrl.aim_weight().is_finite());
        assert!(ctrl.left_ik_weight().is_finite());
        assert!(ctrl.hold().pos.is_finite());
        assert!(ctrl.hold().rot_deg.is_finite());
    }
}
