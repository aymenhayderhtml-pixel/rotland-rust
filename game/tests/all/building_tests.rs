use std::f32::consts::{FRAC_PI_2, PI};

use game::building::*;
use game::dir::yaw_forward;
use glam::{IVec2, Vec2, Vec3};

fn test_ground_flat(_x: f32, _z: f32) -> f32 {
    0.0
}

fn test_unblocked(_min: Vec2, _max: Vec2) -> bool {
    false
}

#[test]
fn building_default_config_loads() {
    let cfg = BuildConfig::load_default().expect("failed to load data/build_parts.json");
    assert_eq!(cfg.grid_size, 2.0);
    assert_eq!(cfg.floor_height, 3.0);
    assert_eq!(cfg.max_floors, 2);
    assert!((cfg.max_slope - 0.8).abs() < 1e-4);
    assert!(!cfg.only_near_camp);
    assert!((cfg.camp_radius - 60.0).abs() < 1e-4);
    assert_eq!(cfg.camp_pos_vec2(), Vec2::ZERO);

    let expected_parts = [
        "floor",
        "wall",
        "wall_window",
        "wall_door",
        "half_wall",
        "roof",
        "stairs",
        "bed",
        "storage_box",
        "workbench",
        "light",
        "spikes",
        "barricade",
    ];

    for part_id in expected_parts {
        let def = cfg.get(part_id).unwrap_or_else(|| panic!("missing part {part_id}"));
        assert_eq!(def.id, part_id);
        assert!(!def.name.is_empty());
        assert!(def.hp > 0.0);
        assert!(def.height > 0.0);
        assert!(!def.footprint.is_empty());
    }

    assert_eq!(cfg.get("floor").unwrap().snap, SnapTarget::Cell);
    assert_eq!(cfg.get("wall").unwrap().snap, SnapTarget::Edge);
    assert!(cfg.get("floor").unwrap().is_floor);
    assert!(cfg.get("roof").unwrap().is_roof);
    assert!(cfg.get("wall_door").unwrap().is_door);
}

#[test]
fn building_placement_success_floor() {
    let cfg = BuildConfig::load_default().unwrap();
    let state = BaseState::default();

    let res = BuildPlan::check(
        &state,
        &cfg,
        "floor",
        IVec2::new(0, 0),
        0,
        0,
        test_ground_flat,
        test_unblocked,
    );

    let placement = res.expect("floor should place on flat ground");
    assert_eq!(placement.part, "floor");
    assert_eq!(placement.cell, IVec2::new(0, 0));
    assert_eq!(placement.floor, 0);
    assert_eq!(placement.rot, 0);
    assert_eq!(placement.position, Vec3::new(0.0, 0.0, 0.0));
    assert_eq!(placement.yaw, 0.0);
    assert_eq!(placement.world_transform(), (Vec3::new(0.0, 0.0, 0.0), 0.0));
    assert_eq!(placement.cost.get("wood"), Some(&4));
}

#[test]
fn building_placement_error_unknown_part() {
    let cfg = BuildConfig::load_default().unwrap();
    let state = BaseState::default();

    let res = BuildPlan::check(
        &state,
        &cfg,
        "non_existent_part",
        IVec2::ZERO,
        0,
        0,
        test_ground_flat,
        test_unblocked,
    );
    assert_eq!(res, Err(Invalid::UnknownPart));
}

#[test]
fn building_placement_error_max_floors() {
    let cfg = BuildConfig::load_default().unwrap();
    let state = BaseState::default();

    let res = BuildPlan::check(
        &state,
        &cfg,
        "floor",
        IVec2::ZERO,
        2, // cfg.max_floors is 2 (floors 0 and 1 are valid)
        0,
        test_ground_flat,
        test_unblocked,
    );
    assert_eq!(res, Err(Invalid::MaxFloorsExceeded));
}

#[test]
fn building_placement_error_too_far_from_camp() {
    let mut cfg = BuildConfig::load_default().unwrap();
    cfg.only_near_camp = true;
    cfg.camp_radius = 60.0;
    cfg.camp_pos = [0.0, 0.0];
    let state = BaseState::default();

    // cell (10, 0) -> world position (20, 0) -> distance 20 <= 60 (valid)
    let ok = BuildPlan::check(
        &state,
        &cfg,
        "floor",
        IVec2::new(10, 0),
        0,
        0,
        test_ground_flat,
        test_unblocked,
    );
    assert!(ok.is_ok());

    // cell (35, 0) -> world position (70, 0) -> distance 70 > 60 (too far)
    let err = BuildPlan::check(
        &state,
        &cfg,
        "floor",
        IVec2::new(35, 0),
        0,
        0,
        test_ground_flat,
        test_unblocked,
    );
    assert_eq!(err, Err(Invalid::TooFarFromCamp));
}

#[test]
fn building_placement_error_blocked() {
    let cfg = BuildConfig::load_default().unwrap();
    let state = BaseState::default();

    let blocked_fn = |_min: Vec2, _max: Vec2| true;
    let res = BuildPlan::check(
        &state,
        &cfg,
        "floor",
        IVec2::ZERO,
        0,
        0,
        test_ground_flat,
        blocked_fn,
    );
    assert_eq!(res, Err(Invalid::Blocked));
}

#[test]
fn building_placement_error_overlapping_cell() {
    let cfg = BuildConfig::load_default().unwrap();
    let mut state = BaseState::default();

    let p = BuildPlan::check(
        &state,
        &cfg,
        "floor",
        IVec2::new(1, 1),
        0,
        0,
        test_ground_flat,
        test_unblocked,
    )
    .unwrap();
    state.place(p);

    let res = BuildPlan::check(
        &state,
        &cfg,
        "floor",
        IVec2::new(1, 1),
        0,
        0,
        test_ground_flat,
        test_unblocked,
    );
    assert_eq!(res, Err(Invalid::OverlappingPart));
}

#[test]
fn building_placement_error_overlapping_object() {
    let cfg = BuildConfig::load_default().unwrap();
    let mut state = BaseState::default();

    let p_floor = BuildPlan::check(
        &state,
        &cfg,
        "floor",
        IVec2::ZERO,
        0,
        0,
        test_ground_flat,
        test_unblocked,
    )
    .unwrap();
    state.place(p_floor);

    let p_bench = BuildPlan::check(
        &state,
        &cfg,
        "workbench",
        IVec2::ZERO,
        0,
        0,
        test_ground_flat,
        test_unblocked,
    )
    .unwrap();
    state.place(p_bench);

    // Trying to place another object (bed) in the same cell at floor 0 should fail
    let res = BuildPlan::check(
        &state,
        &cfg,
        "bed",
        IVec2::ZERO,
        0,
        0,
        test_ground_flat,
        test_unblocked,
    );
    assert_eq!(res, Err(Invalid::OverlappingPart));
}

#[test]
fn building_placement_error_needs_floor_below_upper() {
    let cfg = BuildConfig::load_default().unwrap();
    let state = BaseState::default();

    // Trying to place a floor at level 1 with no floor at level 0
    let res = BuildPlan::check(
        &state,
        &cfg,
        "floor",
        IVec2::ZERO,
        1,
        0,
        test_ground_flat,
        test_unblocked,
    );
    assert_eq!(res, Err(Invalid::NeedsFloorBelow));
}

#[test]
fn building_placement_error_needs_floor_below_roof() {
    let cfg = BuildConfig::load_default().unwrap();
    let state = BaseState::default();

    // Trying to place a roof on ground with no floor below it
    let res = BuildPlan::check(
        &state,
        &cfg,
        "roof",
        IVec2::ZERO,
        0,
        0,
        test_ground_flat,
        test_unblocked,
    );
    assert_eq!(res, Err(Invalid::NeedsFloorBelow));

    // Also at floor 1 without floor below
    let res1 = BuildPlan::check(
        &state,
        &cfg,
        "roof",
        IVec2::ZERO,
        1,
        0,
        test_ground_flat,
        test_unblocked,
    );
    assert_eq!(res1, Err(Invalid::NeedsFloorBelow));
}

#[test]
fn building_placement_error_slope_too_steep() {
    let cfg = BuildConfig::load_default().unwrap();
    let state = BaseState::default();

    // Ground with steep slope: height differences > max_slope (0.8)
    let steep_ground = |x: f32, _z: f32| x * 2.0;

    let res = BuildPlan::check(
        &state,
        &cfg,
        "floor",
        IVec2::ZERO,
        0,
        0,
        steep_ground,
        test_unblocked,
    );
    assert_eq!(res, Err(Invalid::SlopeTooSteep));
}

#[test]
fn building_wall_on_edge_between_two_cells() {
    let cfg = BuildConfig::load_default().unwrap();
    let mut state = BaseState::default();

    // Helper checks
    assert_eq!(neighbor_cell(IVec2::new(5, 5), 0), IVec2::new(5, 4));
    assert_eq!(neighbor_cell(IVec2::new(5, 5), 1), IVec2::new(6, 5));
    assert_eq!(neighbor_cell(IVec2::new(5, 5), 2), IVec2::new(5, 6));
    assert_eq!(neighbor_cell(IVec2::new(5, 5), 3), IVec2::new(4, 5));

    let edge_a = canonical_edge(0, IVec2::new(5, 5), 0); // North edge
    let edge_b = canonical_edge(0, IVec2::new(5, 4), 2); // South edge from neighbor
    assert_eq!(edge_a, edge_b);

    // Place a wall on cell (5, 5) with rot 0 (North edge)
    let p1 = BuildPlan::check(
        &state,
        &cfg,
        "wall",
        IVec2::new(5, 5),
        0,
        0,
        test_ground_flat,
        test_unblocked,
    )
    .unwrap();
    state.place(p1);

    // Attempt to place a wall on the exact same edge from the adjacent cell (5, 4) with rot 2 (South edge)
    let p2 = BuildPlan::check(
        &state,
        &cfg,
        "wall",
        IVec2::new(5, 4),
        0,
        2,
        test_ground_flat,
        test_unblocked,
    );
    assert_eq!(p2, Err(Invalid::OverlappingPart));

    // Also attempt to place another wall on (5, 5) with rot 0
    let p3 = BuildPlan::check(
        &state,
        &cfg,
        "wall_window",
        IVec2::new(5, 5),
        0,
        0,
        test_ground_flat,
        test_unblocked,
    );
    assert_eq!(p3, Err(Invalid::OverlappingPart));

    // But placing on East edge (rot 1) of (5, 5) should succeed
    let p4 = BuildPlan::check(
        &state,
        &cfg,
        "wall",
        IVec2::new(5, 5),
        0,
        1,
        test_ground_flat,
        test_unblocked,
    );
    assert!(p4.is_ok());
}

#[test]
fn building_door_toggles() {
    let cfg = BuildConfig::load_default().unwrap();
    let mut state = BaseState::default();

    let p = BuildPlan::check(
        &state,
        &cfg,
        "wall_door",
        IVec2::ZERO,
        0,
        0,
        test_ground_flat,
        test_unblocked,
    )
    .unwrap();
    let id = state.place(p);

    assert!(!state.is_door_open(id));
    assert!(state.blocks_zombies(&cfg, id));

    // Toggle open
    let now_open = state.toggle_door(id);
    assert!(now_open);
    assert!(state.is_door_open(id));
    assert!(!state.blocks_zombies(&cfg, id));

    // Toggle closed
    let now_open_again = state.toggle_door(id);
    assert!(!now_open_again);
    assert!(!state.is_door_open(id));
    assert!(state.blocks_zombies(&cfg, id));
}

#[test]
fn building_support_collapse() {
    let cfg = BuildConfig::load_default().unwrap();
    let mut state = BaseState::default();

    // 1. Place floor at (0, 0), floor 0
    let p0 = BuildPlan::check(
        &state,
        &cfg,
        "floor",
        IVec2::ZERO,
        0,
        0,
        test_ground_flat,
        test_unblocked,
    )
    .unwrap();
    let id_f0 = state.place(p0);

    // 2. Place floor at (0, 0), floor 1
    let p1 = BuildPlan::check(
        &state,
        &cfg,
        "floor",
        IVec2::ZERO,
        1,
        0,
        test_ground_flat,
        test_unblocked,
    )
    .unwrap();
    let id_f1 = state.place(p1);

    // 3. Place bed at (0, 0), floor 1
    let p_bed = BuildPlan::check(
        &state,
        &cfg,
        "bed",
        IVec2::ZERO,
        1,
        0,
        test_ground_flat,
        test_unblocked,
    )
    .unwrap();
    let id_bed = state.place(p_bed);

    // 4. Place roof at (0, 0), floor 1
    let p_roof = BuildPlan::check(
        &state,
        &cfg,
        "roof",
        IVec2::ZERO,
        1,
        0,
        test_ground_flat,
        test_unblocked,
    )
    .unwrap();
    let id_roof = state.place(p_roof);

    assert_eq!(state.len(), 4);
    assert!(state.is_supported(state.get(id_f0).unwrap()));
    assert!(state.is_supported(state.get(id_f1).unwrap()));
    assert!(state.is_supported(state.get(id_bed).unwrap()));
    assert!(state.is_supported(state.get(id_roof).unwrap()));

    // Now remove floor 0. All upper floor structures must collapse!
    let refund = state.remove(&cfg, id_f0).expect("floor 0 should be removed");
    assert!(refund.contains_key("wood"));

    assert_eq!(state.len(), 0);
    assert!(state.is_empty());
}

#[test]
fn building_support_collapse_furniture_ground() {
    let cfg = BuildConfig::load_default().unwrap();
    let mut state = BaseState::default();

    // Place floor at (2, 2)
    let p_floor = BuildPlan::check(
        &state,
        &cfg,
        "floor",
        IVec2::new(2, 2),
        0,
        0,
        test_ground_flat,
        test_unblocked,
    )
    .unwrap();
    let floor_id = state.place(p_floor);

    // Place workbench on floor at (2, 2)
    let p_bench = BuildPlan::check(
        &state,
        &cfg,
        "workbench",
        IVec2::new(2, 2),
        0,
        0,
        test_ground_flat,
        test_unblocked,
    )
    .unwrap();
    let bench_id = state.place(p_bench);

    // Place spikes on ground at (4, 4) (no floor)
    let p_spikes = BuildPlan::check(
        &state,
        &cfg,
        "spikes",
        IVec2::new(4, 4),
        0,
        0,
        test_ground_flat,
        test_unblocked,
    )
    .unwrap();
    let spikes_id = state.place(p_spikes);

    assert_eq!(state.len(), 3);

    // Removing the floor at (2, 2) destroys the workbench that stood on it
    state.remove(&cfg, floor_id);

    assert!(state.get(floor_id).is_none());
    assert!(state.get(bench_id).is_none());
    // Spikes on ground remain intact!
    assert!(state.get(spikes_id).is_some());
    assert_eq!(state.len(), 1);
}

#[test]
fn building_refund() {
    let cfg = BuildConfig::load_default().unwrap();
    let mut state = BaseState::default();

    // floor costs {"wood": 4}, refund_share = 0.5
    let p = BuildPlan::check(
        &state,
        &cfg,
        "floor",
        IVec2::ZERO,
        0,
        0,
        test_ground_flat,
        test_unblocked,
    )
    .unwrap();
    let id = state.place(p);

    let refund = state.remove(&cfg, id).unwrap();
    assert_eq!(refund.get("wood"), Some(&2)); // 4 * 0.5 = 2
}

#[test]
fn building_save_round_trip() {
    let cfg = BuildConfig::load_default().unwrap();
    let mut state = BaseState::default();

    let p1 = BuildPlan::check(
        &state,
        &cfg,
        "floor",
        IVec2::new(0, 0),
        0,
        0,
        test_ground_flat,
        test_unblocked,
    )
    .unwrap();
    state.place(p1);

    let p2 = BuildPlan::check(
        &state,
        &cfg,
        "wall_door",
        IVec2::new(0, 0),
        0,
        1,
        test_ground_flat,
        test_unblocked,
    )
    .unwrap();
    let door_id = state.place(p2);
    state.toggle_door(door_id);

    let p3 = BuildPlan::check(
        &state,
        &cfg,
        "bed",
        IVec2::new(0, 0),
        0,
        2,
        test_ground_flat,
        test_unblocked,
    )
    .unwrap();
    state.place(p3);

    // Save round-trip
    let json = serde_json::to_string_pretty(&state).expect("serialize BaseState");
    let loaded: BaseState = serde_json::from_str(&json).expect("deserialize BaseState");
    assert_eq!(state, loaded);

    // Test that empty JSON `{}` deserializes to default BaseState
    let empty_loaded: BaseState = serde_json::from_str("{}").expect("empty json deserialize");
    assert_eq!(empty_loaded, BaseState::default());
}

#[test]
fn building_transforms_for_each_rotation() {
    let cfg = BuildConfig::load_default().unwrap();
    let state = BaseState::default();

    // Test rot_to_yaw and Godot rule 7 parity
    assert_eq!(rot_to_yaw(0), 0.0);
    assert!((rot_to_yaw(1) - (-FRAC_PI_2)).abs() < 1e-6);
    assert!((rot_to_yaw(2) - PI).abs() < 1e-6);
    assert!((rot_to_yaw(3) - FRAC_PI_2).abs() < 1e-6);

    // Check forward unit vectors for each rotation
    let f0 = yaw_forward(rot_to_yaw(0));
    assert!((f0 - Vec3::new(0.0, 0.0, -1.0)).length() < 1e-5); // -Z (North)

    let f1 = yaw_forward(rot_to_yaw(1));
    assert!((f1 - Vec3::new(1.0, 0.0, 0.0)).length() < 1e-5); // +X (East, -PI/2)

    let f2 = yaw_forward(rot_to_yaw(2));
    assert!((f2 - Vec3::new(0.0, 0.0, 1.0)).length() < 1e-5); // +Z (South, PI)

    let f3 = yaw_forward(rot_to_yaw(3));
    assert!((f3 - Vec3::new(-1.0, 0.0, 0.0)).length() < 1e-5); // -X (West, PI/2)

    // Check wall transforms for cell (0, 0)
    for rot in 0..4 {
        let p = BuildPlan::check(
            &state,
            &cfg,
            "wall",
            IVec2::ZERO,
            0,
            rot,
            test_ground_flat,
            test_unblocked,
        )
        .unwrap();

        let expected_yaw = rot_to_yaw(rot);
        assert!((p.yaw - expected_yaw).abs() < 1e-6);

        let half = cfg.grid_size * 0.5;
        let expected_pos = Vec3::new(
            -expected_yaw.sin() * half,
            0.0,
            -expected_yaw.cos() * half,
        );
        assert!((p.position - expected_pos).length() < 1e-5);
    }
}

#[test]
fn building_damage_and_destroy() {
    let cfg = BuildConfig::load_default().unwrap();
    let mut state = BaseState::default();

    let p = BuildPlan::check(
        &state,
        &cfg,
        "wall",
        IVec2::ZERO,
        0,
        0,
        test_ground_flat,
        test_unblocked,
    )
    .unwrap();
    let id = state.place(p);

    // Initial HP is 150
    assert_eq!(state.get(id).unwrap().hp, 150.0);

    // Damage by 50 -> not destroyed
    let destroyed = state.damage(id, 50.0);
    assert!(!destroyed);
    assert_eq!(state.get(id).unwrap().hp, 100.0);

    // Damage by 100 -> destroyed
    let destroyed = state.damage(id, 100.0);
    assert!(destroyed);
    assert!(state.get(id).is_none());
}

#[test]
fn building_repair() {
    let cfg = BuildConfig::load_default().unwrap();
    let mut state = BaseState::default();

    let p = BuildPlan::check(
        &state,
        &cfg,
        "workbench",
        IVec2::ZERO,
        0,
        0,
        test_ground_flat,
        test_unblocked,
    )
    .unwrap();
    let id = state.place(p);

    state.damage(id, 50.0);
    assert_eq!(state.get(id).unwrap().hp, 70.0);

    // Repair by amount
    state.repair_amount(&cfg, id, 20.0);
    assert_eq!(state.get(id).unwrap().hp, 90.0);

    // Repair to full
    state.repair(&cfg, id);
    assert_eq!(state.get(id).unwrap().hp, 120.0);
}

#[test]
fn building_parts_in_radius() {
    let cfg = BuildConfig::load_default().unwrap();
    let mut state = BaseState::default();

    let p1 = BuildPlan::check(
        &state,
        &cfg,
        "floor",
        IVec2::new(0, 0),
        0,
        0,
        test_ground_flat,
        test_unblocked,
    )
    .unwrap();
    let id1 = state.place(p1);

    let p2 = BuildPlan::check(
        &state,
        &cfg,
        "floor",
        IVec2::new(2, 0), // pos = (4.0, 0.0, 0.0)
        0,
        0,
        test_ground_flat,
        test_unblocked,
    )
    .unwrap();
    let id2 = state.place(p2);

    let p3 = BuildPlan::check(
        &state,
        &cfg,
        "floor",
        IVec2::new(10, 0), // pos = (20.0, 0.0, 0.0)
        0,
        0,
        test_ground_flat,
        test_unblocked,
    )
    .unwrap();
    let id3 = state.place(p3);

    let found = state.parts_in_radius(Vec3::ZERO, 5.0);
    assert_eq!(found.len(), 2);
    let ids = state.parts_in_radius_ids(Vec3::ZERO, 5.0);
    assert_eq!(ids, vec![id1, id2]);
    assert!(!ids.contains(&id3));
}

#[test]
fn building_place_part_direct_and_helpers() {
    let mut state = BaseState::default();
    let part = PlacedPart {
        id: 0,
        part: "wall".to_string(),
        cell: IVec2::new(1, 2),
        floor: 0,
        rot: 1,
        hp: 150.0,
        door_open: false,
        position: Vec3::ZERO,
        yaw: 0.0,
    };
    let id = state.place_part(part);
    assert_eq!(id, 1);
    assert!(is_edge_part("wall"));
    assert!(is_edge_part("wall_door"));
    assert!(!is_edge_part("floor"));

    let transform = state.world_transform_id(id).unwrap();
    assert!((transform.1 - (-FRAC_PI_2)).abs() < 1e-6);

    let part_mut = state.get_mut(id).unwrap();
    part_mut.hp = 120.0;
    assert_eq!(state.get(id).unwrap().hp, 120.0);

    let removed = state.remove_simple(id);
    assert!(removed);
    assert!(state.is_empty());
}

#[test]
fn building_from_json_and_error_display() {
    let json = r#"{
        "gridSize": 2.0,
        "floorHeight": 3.0,
        "maxFloors": 2,
        "maxSlope": 0.8,
        "onlyNearCamp": false,
        "campRadius": 60.0,
        "campPos": [0.0, 0.0],
        "refundShare": 0.5,
        "parts": {}
    }"#;
    let cfg = BuildConfig::from_json(json).unwrap();
    assert_eq!(cfg.grid_size, 2.0);

    // Display implementation checks for Invalid / PlacementError
    let err = Invalid::NeedsFloorBelow;
    let err_str = format!("{err}");
    assert_eq!(err_str, "Needs a floor below");

    let _alias_check: PlacementError = Invalid::SlopeTooSteep;
}
