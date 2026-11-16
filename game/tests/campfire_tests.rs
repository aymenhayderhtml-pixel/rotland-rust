use std::sync::Arc;

use glam::Vec3;
use pack::Marker;

use game::campfire::{
    find_lit_near, is_flat, near_campfire, place_campfire, settlement_campfire_from_markers,
    Campfire, CampfireInteractResult, CampfireManager, PlaceCampfireError, BURN_HOURS,
    CAMPFIRE_KIT_ID, CAMPFIRE_RANGE, PLACE_DISTANCE, ROOT_LIFT,
    SECONDS_PER_HOUR, WOOD_ID,
};
use game::crafting::{CraftingSystem, RecipeRegistry, CAMPFIRE_STATION};
use game::items::{Inventory, ItemRegistry};

#[test]
fn test_placing_campfire_on_flat_ground_refused_on_steep_or_too_close() {
    let player_pos = Vec3::new(0.0, 0.0, 0.0);
    let facing = Vec3::NEG_Z; // Facing -Z forward

    let flat_ground = |p: Vec3| Some((Vec3::new(p.x, 0.0, p.z), Vec3::Y));
    let mut fires: Vec<Campfire> = Vec::new();

    // 1. Successful placement on flat ground
    let fire1 = place_campfire(player_pos, facing, &fires, None, flat_ground)
        .expect("Placement on flat ground must succeed");
    assert_eq!(
        fire1.position,
        Vec3::new(0.0, ROOT_LIFT, -PLACE_DISTANCE),
        "Placed campfire must be at 1.5m ahead with 0.4m root lift"
    );
    assert_eq!(fire1.remaining_hours, BURN_HOURS);
    assert!(fire1.is_lit());
    assert!(!fire1.permanent);
    fires.push(fire1);

    // 2. Refused when too close to existing fire (< 2.0m spacing)
    let err_too_close = place_campfire(player_pos, facing, &fires, None, flat_ground);
    assert_eq!(
        err_too_close,
        Err(PlaceCampfireError::TooCloseToAnotherFire)
    );
    assert_eq!(
        PlaceCampfireError::TooCloseToAnotherFire.message(),
        "Too close to another campfire"
    );

    // 3. Refused on steep ground (slope > ~23 degrees, normal.y < 0.92)
    let steep_normal = Vec3::new(1.0, 1.0, 0.0).normalize();
    assert!(!is_flat(steep_normal));
    let steep_ground = move |p: Vec3| Some((p, steep_normal));
    let far_player = Vec3::new(20.0, 0.0, 0.0);
    let err_steep = place_campfire(far_player, facing, &fires, None, steep_ground);
    assert_eq!(err_steep, Err(PlaceCampfireError::GroundTooSteep));
    assert_eq!(
        PlaceCampfireError::GroundTooSteep.message(),
        "The ground there is too steep"
    );

    // 4. Refused when no ground is hit
    let no_ground = |_| None;
    let err_no_ground = place_campfire(far_player, facing, &fires, None, no_ground);
    assert_eq!(err_no_ground, Err(PlaceCampfireError::NoGround));

    // 5. Inventory kit requirement
    let items = Arc::new(ItemRegistry::from_repo().expect("ItemRegistry"));
    let mut bag = Inventory::with_capacity_and_registry(10, 50.0, items);

    // Empty bag has no kit
    let err_no_kit = place_campfire(far_player, facing, &fires, Some(&mut bag), flat_ground);
    assert_eq!(err_no_kit, Err(PlaceCampfireError::NoCampfireKit));

    // Bag with 1 campfire_kit succeeds and consumes the kit
    bag.add(CAMPFIRE_KIT_ID, 1);
    assert_eq!(bag.count(CAMPFIRE_KIT_ID), 1);
    let fire2 = place_campfire(far_player, facing, &fires, Some(&mut bag), flat_ground)
        .expect("Placing with kit in bag must succeed");
    assert_eq!(bag.count(CAMPFIRE_KIT_ID), 0);
    assert_eq!(fire2.remaining_hours, BURN_HOURS);
}

#[test]
fn test_fuel_burns_down_over_seconds_and_hours() {
    let mut fire = Campfire::create(Vec3::new(0.0, 0.0, 0.0));
    assert_eq!(fire.remaining_hours, 5.0);
    assert!(fire.is_lit());
    assert!(fire.visuals.is_lit);
    assert!(fire.visuals.light_energy > 0.0);

    // Burn 2 in-game hours -> 3 hours left
    fire.advance_hours(2.0);
    assert_eq!(fire.remaining_hours, 3.0);
    assert!(fire.is_lit());

    // Burn 3 more hours (total 5) -> burns out
    fire.advance_hours(3.0);
    assert_eq!(fire.remaining_hours, 0.0);
    assert!(!fire.is_lit());
    assert!(!fire.visuals.is_lit);
    assert_eq!(fire.visuals.light_energy, 0.0);
    assert_eq!(fire.visuals.flame_scale, Vec3::ZERO);

    // Prompt without wood
    assert_eq!(fire.prompt(None), "Campfire is out");

    // Relight with 1 wood from inventory
    let items = Arc::new(ItemRegistry::from_repo().expect("ItemRegistry"));
    let mut bag = Inventory::with_capacity_and_registry(10, 50.0, items);
    bag.add(WOOD_ID, 1);

    assert_eq!(fire.prompt(Some(&bag)), "[E] Add wood to relight");
    let res = fire.interact(&mut bag);
    assert_eq!(res, CampfireInteractResult::Relit);
    assert_eq!(bag.count(WOOD_ID), 0);
    assert_eq!(fire.remaining_hours, 5.0);
    assert!(fire.is_lit());
    assert_eq!(fire.prompt(Some(&bag)), "[E] Cook");

    // Real seconds tick: 60 real seconds = 1 in-game hour
    fire.tick(SECONDS_PER_HOUR); // 60s
    assert!((fire.remaining_hours - 4.0).abs() < 1e-4);
}

#[test]
fn test_near_campfire_radius() {
    let fire = Campfire::create(Vec3::new(10.0, 0.0, 20.0));
    let fires = [fire];

    // Inside radius (CAMPFIRE_RANGE = 4.0m)
    // Horizontal distance: 2.0m
    let close_player = Vec3::new(10.0, 0.0, 22.0);
    assert!(near_campfire(close_player, &fires));
    assert!(find_lit_near(close_player, &fires, CAMPFIRE_RANGE).is_some());

    // Exactly at boundary
    let boundary_player = Vec3::new(10.0, 0.0, 24.0);
    assert!(near_campfire(boundary_player, &fires));

    // Outside radius: 4.5m
    let far_player = Vec3::new(10.0, 0.0, 24.5);
    assert!(!near_campfire(far_player, &fires));
    assert!(find_lit_near(far_player, &fires, CAMPFIRE_RANGE).is_none());

    // When fire is extinguished, even close player is not near lit campfire
    let mut out_fire = fires[0].clone();
    out_fire.advance_hours(BURN_HOURS);
    let out_fires = [out_fire];
    assert!(!near_campfire(close_player, &out_fires));
}

#[test]
fn test_settlement_fixed_campfire() {
    let mut fire = Campfire::create_permanent(Vec3::new(0.0, 0.0, 0.0));
    assert!(fire.permanent);
    assert!(fire.is_lit());

    // Advance 10 hours: permanent settlement fire never burns out
    fire.advance_hours(10.0);
    assert_eq!(fire.remaining_hours, BURN_HOURS);
    assert!(fire.is_lit());

    // Marker pairing: marker named "campfire" in model markers
    let markers = vec![
        Marker {
            name: "npc_trader".to_string(),
            translation: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0, 1.0, 1.0],
        },
        Marker {
            name: "campfire".to_string(),
            translation: [3.0, 0.0, 4.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0, 1.0, 1.0],
        },
    ];

    let settlement_fire =
        settlement_campfire_from_markers(&markers, glam::Mat4::IDENTITY).expect("Settlement fire");
    assert!(settlement_fire.permanent);
    assert_eq!(
        settlement_fire.position,
        Vec3::new(3.0, ROOT_LIFT, 4.0)
    );
}

#[test]
fn test_save_and_load_round_trip() {
    let mut manager = CampfireManager::new();

    // Fire 1 in chunk (0, 0)
    let fire1 = Campfire::restore(Vec3::new(10.0, 0.4, 10.0), 3.5, false);
    // Fire 2 in chunk (1, 2) since 80/64=1, 150/64=2
    let fire2 = Campfire::restore(Vec3::new(80.0, 0.4, 150.0), 1.5, false);
    // Permanent fire in camp: must not be saved
    let camp_fire = Campfire::create_permanent(Vec3::new(5.0, 0.0, 5.0));

    manager.add(fire1);
    manager.add(fire2);
    manager.add(camp_fire);
    assert_eq!(manager.campfires().len(), 3);

    let saved = manager.to_save();
    let obj = saved.as_object().expect("Saved campfires must be JSON object");

    // Only placed fires are saved, grouped by chunk keys
    assert_eq!(obj.len(), 2, "Must have exactly 2 chunk keys");
    assert!(obj.contains_key("0_0"), "Must contain chunk 0_0");
    assert!(obj.contains_key("1_2"), "Must contain chunk 1_2");

    // Restore into a fresh manager
    let mut loaded_manager = CampfireManager::new();
    // Keep permanent camp fire
    loaded_manager.add(Campfire::create_permanent(Vec3::new(5.0, 0.0, 5.0)));
    loaded_manager
        .from_save(&saved)
        .expect("Load campfires must succeed");

    assert_eq!(loaded_manager.campfires().len(), 3);

    let restored_1 = loaded_manager
        .campfires()
        .iter()
        .find(|f| (f.position.x - 10.0).abs() < 1e-4)
        .expect("Fire 1 restored");
    assert!((restored_1.remaining_hours - 3.5).abs() < 1e-4);
    assert!(!restored_1.permanent);

    let restored_2 = loaded_manager
        .campfires()
        .iter()
        .find(|f| (f.position.x - 80.0).abs() < 1e-4)
        .expect("Fire 2 restored");
    assert!((restored_2.remaining_hours - 1.5).abs() < 1e-4);
    assert!(!restored_2.permanent);
}

#[test]
fn test_crafting_at_campfire_boil_dirty_water_and_cook_meat() {
    let items = Arc::new(ItemRegistry::from_repo().expect("ItemRegistry"));
    let recipes = RecipeRegistry::from_repo().expect("RecipeRegistry");
    let mut crafting = CraftingSystem::new();
    let mut bag = Inventory::with_capacity_and_registry(10, 50.0, items);

    let fire = Campfire::create(Vec3::new(0.0, 0.0, 0.0));
    let player_pos = Vec3::new(1.0, 0.0, 1.0);
    let near = near_campfire(player_pos, &[fire]);
    assert!(near, "Player at 1.41m must be near lit campfire");

    // 1. Boil dirty water (BoilDirtyWater.cs behaviour):
    // 1. Boil dirty water (BoilDirtyWater.cs behaviour): 10s -> 6 cooking XP
    bag.add("dirty_water", 1);
    assert!(crafting.can_craft("water_bottle", CAMPFIRE_STATION, &bag, near, false, &recipes));
    assert!(!crafting.can_craft("water_bottle", CAMPFIRE_STATION, &bag, false, false, &recipes));
    assert!(crafting.start_craft("water_bottle", CAMPFIRE_STATION, &mut bag, near, false, &recipes));
    assert_eq!(bag.count("dirty_water"), 0);

    let update = crafting.update(10.0, &mut bag, near, false);
    match update {
        game::crafting::CraftUpdate::Finished(res) => {
            assert_eq!(res.recipe_id, "water_bottle");
            let xp = res.xp.expect("Cooking XP reward");
            assert_eq!(xp.skill, "cooking");
            assert!((xp.amount - 6.0).abs() < 1e-4, "Must grant 6 cooking XP");
        }
        other => panic!("Expected Finished, got {other:?}"),
    }
    assert_eq!(bag.count("water_bottle"), 1);

    // 2. Cook meat (CampfirePlaceAndCook.cs behaviour): 8s -> 10 cooking XP
    bag.add("raw_meat", 1);
    assert!(crafting.start_craft("cooked_meat", CAMPFIRE_STATION, &mut bag, near, false, &recipes));
    assert_eq!(bag.count("raw_meat"), 0);

    let update_cook = crafting.update(8.0, &mut bag, near, false);
    match update_cook {
        game::crafting::CraftUpdate::Finished(res) => {
            assert_eq!(res.recipe_id, "cooked_meat");
            let xp = res.xp.expect("Cooking XP reward");
            assert_eq!(xp.skill, "cooking");
            assert!((xp.amount - 10.0).abs() < 1e-4, "Must grant 10 cooking XP");
        }
        other => panic!("Expected Finished, got {other:?}"),
    }
    assert_eq!(bag.count("cooked_meat"), 1);
}
