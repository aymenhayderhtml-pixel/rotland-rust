//! Integration tests for tree felling, axe strikes, wood rewards, falling animation, and persistence.
// Traced constants:
// - FELL_HITS = 4 (TreeFelling.cs:15)
// - WoodMin = 3, WoodMax = 5 (TreeFelling.cs:16-17)
// - DropOffset = 1.5 (TreeFelling.cs:19)
// - Axe has tag "chop" in data/weapons.json:21
// - Bat has no "chop" tag in data/weapons.json:2
// - Wood item in data/items.json:50 (weight: 1.0, maxStack: 20)
// - Max tilt angle: FRAC_PI_2 rad = 90.0 deg

use std::f32::consts::FRAC_PI_2;
use glam::{IVec2, Vec3};

use game::{
    find_tree_strike, is_tree_kind, parse_tree_id, tree_id, Inventory, TreeCandidate,
    TreeFelling, TreeHit, TreeStrikeResult, TREE_DROP_OFFSET, TREE_FALL_DURATION,
    TREE_FELL_HITS, TREE_WOOD_MAX, TREE_WOOD_MIN,
};

#[test]
fn axe_strike_traces_hits_and_fells_tree() {
    let mut felling = TreeFelling::new();
    felling.set_fixed_wood(Some(4));

    let tree_pos = Vec3::new(10.0, 0.0, 10.0);
    let player_pos = Vec3::new(10.0, 0.0, 8.0);
    let id = tree_id(IVec2::new(0, 0), 1);
    assert_eq!(id, "tree_0_0_1");

    // Hit 1: 3 hits remaining, no wood, not felled, plays hit_wood
    let res1: TreeStrikeResult =
        felling.strike(&id, tree_pos, "tree_pine_01", Some("axe"), None, player_pos);
    assert_eq!(res1.hits_left, 3);
    assert_eq!(res1.wood_given, 0);
    assert!(!res1.felled);
    assert_eq!(res1.sound_event, Some("hit_wood"));
    assert!(!felling.is_felled(&id));

    // Hit 2: 2 hits remaining
    let res2 = felling.strike(&id, tree_pos, "tree_pine_01", Some("axe"), None, player_pos);
    assert_eq!(res2.hits_left, 2);
    assert_eq!(res2.wood_given, 0);
    assert!(!res2.felled);
    assert_eq!(res2.sound_event, Some("hit_wood"));
    assert!(!felling.is_felled(&id));

    // Hit 3: 1 hit remaining
    let res3 = felling.strike(&id, tree_pos, "tree_pine_01", Some("axe"), None, player_pos);
    assert_eq!(res3.hits_left, 1);
    assert_eq!(res3.wood_given, 0);
    assert!(!res3.felled);
    assert_eq!(res3.sound_event, Some("hit_wood"));
    assert!(!felling.is_felled(&id));

    // Hit 4: Felled! 0 hits remaining, fixed 4 wood given, tree is felled
    let res4 = felling.strike(&id, tree_pos, "tree_pine_01", Some("axe"), None, player_pos);
    assert_eq!(res4.hits_left, 0);
    assert_eq!(res4.wood_given, 4);
    assert!(res4.felled);
    assert_eq!(res4.sound_event, Some("hit_wood"));
    assert!(felling.is_felled(&id));

    // Hit 5 (after felling): tree is already felled, no more wood or sound
    let res5 = felling.strike(&id, tree_pos, "tree_pine_01", Some("axe"), None, player_pos);
    assert_eq!(res5.hits_left, 0);
    assert_eq!(res5.wood_given, 0);
    assert!(res5.felled);
    assert_eq!(res5.sound_event, None);
}

#[test]
fn bare_hands_does_not_chop_tree() {
    let mut felling = TreeFelling::new();
    let tree_pos = Vec3::new(5.0, 0.0, 5.0);
    let player_pos = Vec3::new(5.0, 0.0, 3.5);
    let id = "tree_0_0_2";

    // 4 strikes with bare hands (None)
    for _ in 1..=4 {
        let res = felling.strike(id, tree_pos, "tree_broadleaf_01", None, None, player_pos);
        assert_eq!(res.hits_left, TREE_FELL_HITS);
        assert_eq!(res.wood_given, 0);
        assert!(!res.felled);
        assert_eq!(res.sound_event, None);
    }
    assert!(!felling.is_felled(id));

    // Empty weapon id string is also bare hands
    let res_empty = felling.strike_weapon(
        id,
        tree_pos,
        "tree_broadleaf_01",
        "",
        None,
        player_pos,
    );
    assert_eq!(res_empty.hits_left, TREE_FELL_HITS);
    assert!(!res_empty.felled);
    assert_eq!(res_empty.sound_event, None);
}

#[test]
fn non_axe_weapons_do_not_chop_tree() {
    let mut felling = TreeFelling::new();
    let tree_pos = Vec3::new(2.0, 0.0, 2.0);
    let player_pos = Vec3::new(2.0, 0.0, 0.5);
    let id = "tree_1_1_4";

    let non_axe_weapons = ["bat", "machete", "knife", "pistol", "shotgun", "rifle"];

    for weapon in non_axe_weapons {
        let res = felling.strike(
            id,
            tree_pos,
            "tree_pine_02",
            Some(weapon),
            None,
            player_pos,
        );
        assert_eq!(res.hits_left, TREE_FELL_HITS, "weapon {weapon} should not reduce hits");
        assert_eq!(res.wood_given, 0);
        assert!(!res.felled);
        assert_eq!(res.sound_event, None);
    }
    assert!(!felling.is_felled(id));
}

#[test]
fn wood_count_range_and_inventory_addition() {
    let mut felling = TreeFelling::with_seed(12345);
    let tree_pos = Vec3::new(0.0, 0.0, 0.0);
    let player_pos = Vec3::new(0.0, 0.0, -1.5);

    // Verify random wood drop range across multiple felled trees
    for i in 0..15 {
        let tid = format!("tree_0_0_{i}");
        let mut last_res = None;
        for _ in 1..=TREE_FELL_HITS {
            last_res = Some(felling.strike(
                &tid,
                tree_pos,
                "tree_pine_01",
                Some("axe"),
                None,
                player_pos,
            ));
        }
        let res = last_res.unwrap();
        assert!(res.felled);
        assert!(
            res.wood_given >= TREE_WOOD_MIN && res.wood_given <= TREE_WOOD_MAX,
            "wood count {} must be within {}..={}",
            res.wood_given,
            TREE_WOOD_MIN,
            TREE_WOOD_MAX
        );
    }

    // Test depositing wood into player Inventory
    felling.set_fixed_wood(Some(4));
    let inv_tree_id = "tree_9_9_9";
    for _ in 1..=3 {
        felling.strike(inv_tree_id, tree_pos, "tree_pine_01", Some("axe"), None, player_pos);
    }
    let res = felling.strike(inv_tree_id, tree_pos, "tree_pine_01", Some("axe"), None, player_pos);
    assert_eq!(res.wood_given, 4);

    let mut inventory = Inventory::new();
    assert_eq!(inventory.count("wood"), 0);
    let leftover = res.add_to_inventory(&mut inventory);
    assert_eq!(leftover, 0);
    assert_eq!(inventory.count("wood"), 4);
    // Wood weight is 1.0 kg per item in data/items.json
    assert!((inventory.total_weight() - 4.0).abs() < 0.001);
}

#[test]
fn fall_animation_advances_tilt_and_tracks_direction() {
    let mut felling = TreeFelling::new();
    let player_pos = Vec3::new(10.0, 0.0, 0.0);
    let tree_pos = Vec3::new(13.0, 0.0, 4.0);
    let id = "tree_2_2_0";

    // Expected direction away from player: (13 - 10, 0, 4 - 0) = (3, 0, 4). Normalized: (0.6, 0.0, 0.8)
    let expected_dir = Vec3::new(0.6, 0.0, 0.8);

    for _ in 1..=3 {
        felling.strike(id, tree_pos, "tree_oak_big", Some("axe"), None, player_pos);
    }
    let res = felling.strike(id, tree_pos, "tree_oak_big", Some("axe"), None, player_pos);
    assert!(res.felled);

    // Initial animation state before updates
    assert_eq!(felling.falling.len(), 1);
    let active = &felling.falling[0];
    assert_eq!(active.id, id);
    assert_eq!(active.position, tree_pos);
    assert!((active.fall_dir - expected_dir).length() < 0.0001);
    assert_eq!(active.tilt_angle, 0.0);
    assert_eq!(active.progress, 0.0);
    assert!(!active.finished);

    // Update halfway (dt = 0.6s out of 1.2s duration)
    let half_dt = TREE_FALL_DURATION * 0.5;
    let list_half = felling.update(half_dt);
    assert_eq!(list_half.len(), 1);
    let falling_half = &list_half[0];
    assert!((falling_half.progress - 0.5).abs() < 0.001);
    assert!((falling_half.tilt_angle - 0.5 * FRAC_PI_2).abs() < 0.001);
    assert!((falling_half.tilt_degrees() - 45.0).abs() < 0.1);
    assert!(!falling_half.finished);

    // Update remaining half to reach the end
    let list_end = felling.update(half_dt);
    assert_eq!(list_end.len(), 1);
    let falling_end = &list_end[0];
    assert!((falling_end.progress - 1.0).abs() < 0.001);
    assert!((falling_end.tilt_angle - FRAC_PI_2).abs() < 0.001);
    assert!((falling_end.tilt_degrees() - 90.0).abs() < 0.1);
    assert!(falling_end.finished);

    // Next update prunes the finished animation
    let list_pruned = felling.update(0.1);
    assert!(list_pruned.is_empty());

    // Tree remains felled in manager
    assert!(felling.is_felled(id));
}

#[test]
fn felled_trees_persist_across_save_roundtrip() {
    let mut felling = TreeFelling::new();
    let tree_pos = Vec3::new(0.0, 0.0, 0.0);
    let player_pos = Vec3::new(0.0, 0.0, -2.0);

    let id_a = "tree_0_0_1";
    let id_b = "tree_-2_3_5";
    let id_c = "tree_1_1_0";

    // Fell tree A and B
    for _ in 1..=4 {
        felling.strike(id_a, tree_pos, "tree_pine_01", Some("axe"), None, player_pos);
        felling.strike(id_b, tree_pos, "tree_pine_02", Some("axe"), None, player_pos);
    }
    // Partial hit on tree C (2 hits)
    felling.strike(id_c, tree_pos, "tree_birch", Some("axe"), None, player_pos);
    felling.strike(id_c, tree_pos, "tree_birch", Some("axe"), None, player_pos);

    assert!(felling.is_felled(id_a));
    assert!(felling.is_felled(id_b));
    assert!(!felling.is_felled(id_c));

    // Save to Godot JSON structure
    let save_data = felling.to_save();
    let fallen_arr = save_data
        .get("fallen_trees")
        .and_then(|v| v.as_array())
        .expect("fallen_trees key must be present in save data");

    let saved_ids: Vec<&str> = fallen_arr.iter().filter_map(|v| v.as_str()).collect();
    assert_eq!(saved_ids, vec![id_b, id_a]); // Sorted order

    // Fresh instance loads save data
    let mut restored = TreeFelling::new();
    assert!(!restored.is_felled(id_a));
    assert!(!restored.is_felled(id_b));

    restored.from_save(&save_data);
    assert!(restored.is_felled(id_a));
    assert!(restored.is_felled(id_b));
    // Partial hits on tree C were not saved
    assert!(!restored.is_felled(id_c));

    // Direct array load fallback
    let mut array_restored = TreeFelling::new();
    array_restored.from_save(&serde_json::json!(["tree_4_4_1"]));
    assert!(array_restored.is_felled("tree_4_4_1"));
}

#[test]
fn spatial_tree_strike_queries_arc_and_range() {
    let origin = Vec3::new(0.0, 1.0, 0.0);
    let forward = Vec3::new(0.0, 0.0, 1.0); // Facing +Z
    let range = 2.0;
    let arc_deg = 80.0;

    let candidates = vec![
        // Tree 1: directly in front at distance 1.5 (inside range and arc) -> Target
        TreeCandidate::new(IVec2::new(0, 0), 1, Vec3::new(0.0, 0.0, 1.5), "tree_pine_01"),
        // Tree 2: directly in front at distance 2.5 (outside range 2.0)
        TreeCandidate::new(IVec2::new(0, 0), 2, Vec3::new(0.0, 0.0, 2.5), "tree_pine_02"),
        // Tree 3: at +X 1.8 (90 deg angle, outside 80 deg arc / 40 deg half-arc)
        TreeCandidate::new(IVec2::new(0, 0), 3, Vec3::new(1.8, 0.0, 0.0), "tree_broadleaf_01"),
        // Prop 4: directly in front at distance 0.8, but not a tree (rock)
        TreeCandidate::new(IVec2::new(0, 0), 4, Vec3::new(0.0, 0.0, 0.8), "rock_01"),
    ];

    let hit = find_tree_strike(origin, forward, range, arc_deg, candidates);
    assert!(hit.is_some());
    let h: TreeHit = hit.unwrap();
    assert_eq!(h.chunk, IVec2::new(0, 0));
    assert_eq!(h.index, 1);
    assert_eq!(h.tree_id(), "tree_0_0_1");
}

#[test]
fn non_tree_props_are_not_chopped() {
    let mut felling = TreeFelling::new();
    let pos = Vec3::new(0.0, 0.0, 0.0);
    let player_pos = Vec3::new(0.0, 0.0, -1.0);

    let non_trees = ["rock_01", "bush_01", "fern", "flowers_01", "car_01"];
    for model in non_trees {
        assert!(!is_tree_kind(model));
        let res = felling.strike("prop_0_0_1", pos, model, Some("axe"), None, player_pos);
        assert!(!res.felled);
        assert_eq!(res.wood_given, 0);
        assert_eq!(res.sound_event, None);
    }
}

#[test]
fn tree_id_formatting_and_parsing() {
    let coord = IVec2::new(3, -7);
    let idx = 42;
    let id = tree_id(coord, idx);
    assert_eq!(id, "tree_3_-7_42");

    let parsed = parse_tree_id(&id);
    assert_eq!(parsed, Some((coord, idx)));

    assert_eq!(parse_tree_id("invalid_id"), None);
    assert_eq!(parse_tree_id("tree_3_a_1"), None);
}

#[test]
fn apply_fallen_identifies_trees_to_hide() {
    let mut felling = TreeFelling::new();
    let chunk = IVec2::new(1, 2);
    felling.fallen.insert(tree_id(chunk, 0));
    felling.fallen.insert(tree_id(chunk, 3));

    let chunk_props = [
        (0, "tree_pine_01"),
        (1, "tree_pine_02"),
        (2, "rock_01"),
        (3, "tree_broadleaf_01"),
    ];

    let to_hide = felling.apply_fallen(chunk, chunk_props);
    assert_eq!(to_hide, vec![0, 3]);
    assert_eq!(TREE_DROP_OFFSET, 1.5);
}
