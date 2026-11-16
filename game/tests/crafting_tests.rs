use std::sync::Arc;

use game::crafting::*;
use game::items::{Inventory, ItemRegistry};

#[test]
fn recipes_load_from_repo_and_validate() {
    let items = ItemRegistry::from_repo().expect("ItemRegistry::from_repo");
    let recipes = RecipeRegistry::from_repo().expect("RecipeRegistry::from_repo");

    assert_eq!(recipes.len(), 7);
    assert!(!recipes.is_empty());

    recipes.validate(&items).expect("recipes validate with items");

    let expected_ids = [
        "bandage",
        "campfire_kit",
        "cooked_meat",
        "water_bottle",
        "bread",
        "baked_potato",
        "vegetable_stew",
    ];

    let mut compared = 0;
    for id in &expected_ids {
        let recipe = recipes.get(id).expect("recipe exists");
        assert_eq!(recipe.id, *id);
        assert!(!recipe.name.is_empty());
        assert!(recipe.time > 0.0);
        assert!(!recipe.inputs.is_empty());
        assert!(!recipe.outputs.is_empty());

        for input in &recipe.inputs {
            assert!(items.contains(&input.id));
            assert!(input.count >= 1);
        }
        for output in &recipe.outputs {
            assert!(items.contains(&output.id));
            assert!(output.count >= 1);
        }

        compared += 1;
    }

    assert_eq!(compared, 7);
    assert!(compared > 0);

    // Check specific known recipes
    let bandage = recipes.get("bandage").unwrap();
    assert_eq!(bandage.station, HAND_STATION);
    assert_eq!(bandage.inputs.len(), 1);
    assert_eq!(bandage.inputs[0].id, "cloth");
    assert_eq!(bandage.inputs[0].count, 2);
    assert_eq!(bandage.outputs[0].id, "bandage");
    assert_eq!(bandage.outputs[0].count, 1);
    assert!((bandage.time - 3.0).abs() < 1e-6);
    assert!(bandage.xp.is_none());

    let cooked_meat = recipes.get("cooked_meat").unwrap();
    assert_eq!(cooked_meat.station, CAMPFIRE_STATION);
    assert_eq!(cooked_meat.inputs[0].id, "raw_meat");
    assert_eq!(cooked_meat.outputs[0].id, "cooked_meat");
    assert!((cooked_meat.time - 8.0).abs() < 1e-6);
    let xp = cooked_meat.xp.as_ref().unwrap();
    assert_eq!(xp.skill, "cooking");
    assert!((xp.amount - 10.0).abs() < 1e-6);
}

#[test]
fn recipe_can_craft_checks_inputs_station_and_alive() {
    let items = ItemRegistry::from_repo().expect("ItemRegistry::from_repo");
    let reg_arc = Arc::new(items);
    let recipes = RecipeRegistry::from_repo().expect("RecipeRegistry::from_repo");

    let system = CraftingSystem::new();
    let mut bag = Inventory::with_capacity_and_registry(10, 40.0, reg_arc);

    // Bandage needs 2 cloth, hand station
    // 1. Missing inputs
    assert!(!system.can_craft("bandage", HAND_STATION, &bag, false, false, &recipes));

    // Add 2 cloth
    bag.add("cloth", 2);
    assert!(system.can_craft("bandage", HAND_STATION, &bag, false, false, &recipes));

    // 2. Wrong station (bandage at campfire station)
    assert!(!system.can_craft("bandage", CAMPFIRE_STATION, &bag, true, false, &recipes));

    // 3. Dead player cannot craft
    assert!(!system.can_craft("bandage", HAND_STATION, &bag, false, true, &recipes));

    // 4. Campfire recipe (cooked_meat needs 1 raw_meat and campfire)
    bag.add("raw_meat", 1);
    // Not near campfire -> cannot craft
    assert!(!system.can_craft("cooked_meat", CAMPFIRE_STATION, &bag, false, false, &recipes));
    // Near lit campfire -> can craft
    assert!(system.can_craft("cooked_meat", CAMPFIRE_STATION, &bag, true, false, &recipes));
}

#[test]
fn crafting_removes_inputs_and_adds_outputs_with_xp() {
    let items = ItemRegistry::from_repo().expect("ItemRegistry::from_repo");
    let reg_arc = Arc::new(items);
    let recipes = RecipeRegistry::from_repo().expect("RecipeRegistry::from_repo");

    let mut system = CraftingSystem::new();
    let mut bag = Inventory::with_capacity_and_registry(10, 40.0, reg_arc);

    // Setup for campfire_kit (4 wood -> 1 campfire_kit, 2s time)
    bag.add("wood", 4);
    assert_eq!(bag.count("wood"), 4);

    let started = system.start_craft("campfire_kit", HAND_STATION, &mut bag, false, false, &recipes);
    assert!(started);
    assert!(system.is_crafting());
    assert_eq!(bag.count("wood"), 0); // Inputs taken on start

    // Advance 1s (halfway)
    let update1 = system.update(1.0, &mut bag, false, false);
    match update1 {
        CraftUpdate::Progress(p) => {
            assert_eq!(p.recipe_id, "campfire_kit");
            assert!((p.progress - 0.5).abs() < 1e-4);
        }
        _ => panic!("expected Progress"),
    }

    // Advance another 1s (finishes at 2s)
    let update2 = system.update(1.0, &mut bag, false, false);
    match update2 {
        CraftUpdate::Finished(res) => {
            assert_eq!(res.recipe_id, "campfire_kit");
            assert_eq!(res.outputs_added.len(), 1);
            assert_eq!(res.outputs_added[0].item_id, "campfire_kit");
            assert_eq!(res.outputs_added[0].count, 1);
            assert!(res.leftovers.is_empty());
            assert!(res.xp.is_none());
        }
        _ => panic!("expected Finished"),
    }

    assert_eq!(bag.count("campfire_kit"), 1);
    assert!(!system.is_crafting());

    // Test campfire recipe with XP (cooked_meat, 8s, 10 cooking XP)
    bag.add("raw_meat", 1);
    let started_meat = system.start_craft("cooked_meat", CAMPFIRE_STATION, &mut bag, true, false, &recipes);
    assert!(started_meat);

    let finish_meat = system.update(8.5, &mut bag, true, false);
    match finish_meat {
        CraftUpdate::Finished(res) => {
            assert_eq!(res.recipe_id, "cooked_meat");
            assert_eq!(res.outputs_added[0].item_id, "cooked_meat");
            let xp = res.xp.expect("cooked_meat gives cooking XP");
            assert_eq!(xp.skill, "cooking");
            assert!((xp.amount - 10.0).abs() < 1e-6);
        }
        _ => panic!("expected Finished with XP"),
    }
    assert_eq!(bag.count("cooked_meat"), 1);
}

#[test]
fn crafting_respects_inventory_weight_and_stack_limits() {
    let items = ItemRegistry::from_repo().expect("ItemRegistry::from_repo");
    let reg_arc = Arc::new(items);
    let recipes = RecipeRegistry::from_repo().expect("RecipeRegistry::from_repo");

    // Bag with 1.2 kg capacity (data/items.json: cloth 0.1 kg, wood 1.0 kg, bandage 0.1 kg)
    let mut bag = Inventory::with_capacity_and_registry(5, 1.2, reg_arc);

    // 2 cloth = 0.2 kg + 1 wood = 1.0 kg: 1.2 kg, the bag is full by weight
    bag.add("cloth", 2);
    bag.add("wood", 1);
    assert!((bag.total_weight() - 1.2).abs() < 1e-4);

    let mut system = CraftingSystem::new();
    // Starting the craft removes 2 cloth (0.2 kg): 1.0 kg
    let started = system.start_craft("bandage", HAND_STATION, &mut bag, false, false, &recipes);
    assert!(started);
    assert!((bag.total_weight() - 1.0).abs() < 1e-4);

    // While the craft runs, 2 more cloth fill the bag again
    bag.add("cloth", 2);
    assert!((bag.total_weight() - 1.2).abs() < 1e-4);

    // Craft completes (bandage weighs 0.1 kg, cannot fit in 1.0 kg limit)
    let finished = system.update(3.0, &mut bag, false, false);
    match finished {
        CraftUpdate::Finished(res) => {
            assert_eq!(res.recipe_id, "bandage");
            // Output could not fit, lands in leftovers
            assert_eq!(res.leftovers.len(), 1);
            assert_eq!(res.leftovers[0].item_id, "bandage");
            assert_eq!(res.leftovers[0].count, 1);
            assert!(res.outputs_added.is_empty());
        }
        _ => panic!("expected Finished"),
    }
}

#[test]
fn crafting_cancellation_returns_inputs() {
    let items = ItemRegistry::from_repo().expect("ItemRegistry::from_repo");
    let reg_arc = Arc::new(items);
    let recipes = RecipeRegistry::from_repo().expect("RecipeRegistry::from_repo");

    let mut system = CraftingSystem::new();
    let mut bag = Inventory::with_capacity_and_registry(10, 40.0, reg_arc);

    bag.add("cloth", 2);
    assert_eq!(bag.count("cloth"), 2);

    let started = system.start_craft("bandage", HAND_STATION, &mut bag, false, false, &recipes);
    assert!(started);
    assert_eq!(bag.count("cloth"), 0);

    // Cancel manually
    let cancelled = system.cancel_craft(&mut bag, CancelReason::Manual);
    assert!(cancelled.is_some());
    let res = cancelled.unwrap();
    assert_eq!(res.recipe_id, "bandage");
    assert_eq!(res.returned_inputs.len(), 1);
    assert_eq!(res.returned_inputs[0].item_id, "cloth");
    assert_eq!(res.returned_inputs[0].count, 2);

    assert_eq!(bag.count("cloth"), 2);
    assert!(!system.is_crafting());
}

#[test]
fn crafting_interrupted_by_distance_or_death_cancels() {
    let items = ItemRegistry::from_repo().expect("ItemRegistry::from_repo");
    let reg_arc = Arc::new(items);
    let recipes = RecipeRegistry::from_repo().expect("RecipeRegistry::from_repo");

    let mut system = CraftingSystem::new();
    let mut bag = Inventory::with_capacity_and_registry(10, 40.0, reg_arc);

    // 1. Interrupted by moving away from campfire
    bag.add("raw_meat", 1);
    let started1 = system.start_craft("cooked_meat", CAMPFIRE_STATION, &mut bag, true, false, &recipes);
    assert!(started1);
    assert_eq!(bag.count("raw_meat"), 0);

    // update with near_lit_campfire = false
    let update1 = system.update(1.0, &mut bag, false, false);
    match update1 {
        CraftUpdate::Cancelled(res) => {
            assert_eq!(res.recipe_id, "cooked_meat");
            assert_eq!(res.reason, CancelReason::TooFarFromCampfire);
            assert_eq!(res.returned_inputs[0].item_id, "raw_meat");
            assert_eq!(res.returned_inputs[0].count, 1);
        }
        _ => panic!("expected Cancelled"),
    }
    assert_eq!(bag.count("raw_meat"), 1);
    assert!(!system.is_crafting());

    // 2. Interrupted by death
    bag.add("cloth", 2);
    let started2 = system.start_craft("bandage", HAND_STATION, &mut bag, false, false, &recipes);
    assert!(started2);
    assert_eq!(bag.count("cloth"), 0);

    // update with is_dead = true
    let update2 = system.update(1.0, &mut bag, false, true);
    match update2 {
        CraftUpdate::Cancelled(res) => {
            assert_eq!(res.recipe_id, "bandage");
            assert_eq!(res.reason, CancelReason::Dead);
            assert_eq!(res.returned_inputs[0].item_id, "cloth");
            assert_eq!(res.returned_inputs[0].count, 2);
        }
        _ => panic!("expected Cancelled"),
    }
    assert_eq!(bag.count("cloth"), 2);
    assert!(!system.is_crafting());
}
