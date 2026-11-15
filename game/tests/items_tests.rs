use std::sync::Arc;

use game::items::*;
use game::weapons::WeaponRegistry;

#[test]
fn every_item_in_items_json_loads() {
    let path = paths::repo_path("data/items.json");
    let text = std::fs::read_to_string(&path).expect("read data/items.json");
    let raw: serde_json::Value = serde_json::from_str(&text).expect("valid json");
    let json_map = raw.as_object().expect("json object");
    let expected_count = json_map.len();
    assert!(expected_count > 0, "items.json must not be empty");

    let registry = ItemRegistry::load(&path).expect("ItemRegistry::load succeeds");
    assert_eq!(registry.len(), expected_count);
    assert!(!registry.is_empty());

    let mut compared = 0;
    for (id, val) in json_map {
        let def = registry.get(id).expect("item present in registry");
        assert_eq!(&def.id, id);
        assert_eq!(def.name, val["name"].as_str().unwrap());
        if let Some(cat) = val.get("category").and_then(|c| c.as_str()) {
            assert_eq!(def.category.as_str(), cat);
        }
        if let Some(w) = val.get("weight").and_then(|w| w.as_f64()) {
            assert!((def.weight - w as f32).abs() < 1e-6);
        }
        if let Some(s) = val.get("maxStack").and_then(|s| s.as_u64()) {
            assert_eq!(def.max_stack, s as u32);
        }
        compared += 1;
    }

    println!("compared {} items from data/items.json, all matched", compared);
    assert!(compared > 0);
    assert_eq!(compared, expected_count);
}

#[test]
fn stacking_respects_max_stack_and_returns_leftover() {
    let registry = ItemRegistry::from_repo().expect("items registry");
    let reg_arc = Arc::new(registry);
    // Create an inventory with 2 slots and generous weight capacity so weight is not a factor
    let mut inv = Inventory::with_capacity_and_registry(2, 1000.0, reg_arc.clone());

    // Canned beans has max_stack = 10
    let beans_def = reg_arc.get("canned_beans").expect("canned_beans exists");
    assert_eq!(beans_def.max_stack, 10);

    // Add 15 canned beans: slot 0 gets 10, slot 1 gets 5, leftover is 0
    let leftover1 = inv.add("canned_beans", 15);
    assert_eq!(leftover1, 0);
    assert_eq!(inv.count("canned_beans"), 15);
    assert_eq!(inv.slot(0).unwrap().count, 10);
    assert_eq!(inv.slot(1).unwrap().count, 5);

    // Add 8 more canned beans: slot 1 takes 5 (reaches max_stack 10), leftover is 3
    let leftover2 = inv.add("canned_beans", 8);
    assert_eq!(leftover2, 3);
    assert_eq!(inv.count("canned_beans"), 20);
    assert_eq!(inv.slot(0).unwrap().count, 10);
    assert_eq!(inv.slot(1).unwrap().count, 10);

    // Adding any more to a full inventory returns all of them as leftover
    let leftover3 = inv.add("canned_beans", 5);
    assert_eq!(leftover3, 5);
    assert_eq!(inv.count("canned_beans"), 20);
}

#[test]
fn weight_limit_refuses_overflow() {
    let registry = ItemRegistry::from_repo().expect("items registry");
    let reg_arc = Arc::new(registry);

    // 10 slots, 10.0 kg max weight
    let mut inv = Inventory::with_capacity_and_registry(10, 10.0, reg_arc.clone());
    assert_eq!(inv.max_weight(), 10.0);

    // raw_meat: weight 0.5 kg, max_stack 10
    let meat_def = reg_arc.get("raw_meat").expect("raw_meat exists");
    assert_eq!(meat_def.weight, 0.5);

    // 10 kg / 0.5 kg = exactly 20 pieces fit
    let leftover = inv.add("raw_meat", 25);
    assert_eq!(leftover, 5);
    assert_eq!(inv.count("raw_meat"), 20);
    assert!((inv.total_weight() - 10.0).abs() < 1e-4);

    // Adding 1 more meat refuses it because inventory is at weight capacity
    let leftover2 = inv.add("raw_meat", 1);
    assert_eq!(leftover2, 1);
    assert_eq!(inv.count("raw_meat"), 20);
    assert!((inv.total_weight() - 10.0).abs() < 1e-4);
}

#[test]
fn remove_and_count_work() {
    let registry = ItemRegistry::from_repo().expect("items registry");
    let mut inv = Inventory::with_capacity_and_registry(5, 100.0, Arc::new(registry));

    assert_eq!(inv.count("wood"), 0);
    assert!(!inv.has("wood", 1));

    // wood has max_stack 20
    let leftover = inv.add("wood", 35); // 20 in slot 0, 15 in slot 1
    assert_eq!(leftover, 0);
    assert_eq!(inv.count("wood"), 35);
    assert!(inv.has("wood", 35));
    assert!(!inv.has("wood", 36));

    // Remove 10 wood: removes from last stacks first (slot 1 has 15 -> becomes 5)
    let ok = inv.remove("wood", 10);
    assert!(ok);
    assert_eq!(inv.count("wood"), 25);
    assert_eq!(inv.slot(0).unwrap().count, 20);
    assert_eq!(inv.slot(1).unwrap().count, 5);

    // Remove 10 more wood: slot 1 emptied (5 removed), slot 0 loses 5 (becomes 15)
    let ok2 = inv.remove("wood", 10);
    assert!(ok2);
    assert_eq!(inv.count("wood"), 15);
    assert_eq!(inv.slot(0).unwrap().count, 15);
    assert!(inv.slot(1).is_none());

    // Trying to remove more than carried fails completely and changes nothing (all or nothing)
    let ok3 = inv.remove("wood", 20);
    assert!(!ok3);
    assert_eq!(inv.count("wood"), 15);
    assert_eq!(inv.slot(0).unwrap().count, 15);
}

#[test]
fn starting_inventory_matches_json() {
    let reg_path = paths::repo_path("data/items.json");
    let registry = ItemRegistry::load(&reg_path).expect("load items registry");

    let start_path = paths::repo_path("data/start_inventory.json");
    let start_text = std::fs::read_to_string(&start_path).expect("read start_inventory.json");
    let entries: Vec<StartItemEntry> =
        serde_json::from_str(&start_text).expect("parse start_inventory.json");
    assert!(!entries.is_empty(), "start_inventory must not be empty");

    let inv = start_inventory(&registry);

    let mut checked = 0;
    for entry in &entries {
        assert!(
            inv.has(&entry.id, entry.count),
            "starting inventory missing item {} (expected count {})",
            entry.id,
            entry.count
        );
        assert_eq!(
            inv.count(&entry.id),
            entry.count,
            "starting inventory count mismatch for {}",
            entry.id
        );
        checked += 1;
    }

    println!("checked {} starting inventory entries", checked);
    assert!(checked > 0);
    assert_eq!(checked, entries.len());
}

#[test]
fn hotbar_slot_1_selects_bat_and_resolves_weapon() {
    let reg_path = paths::repo_path("data/items.json");
    let registry = ItemRegistry::load(&reg_path).expect("load items");
    let weapon_path = paths::repo_path("data/weapons.json");
    let weapons = WeaponRegistry::load(&weapon_path).expect("load weapons");

    let inv = start_inventory(&registry);
    let mut hotbar = Hotbar::new();

    // Verify start item in slot 0 is indeed the bat
    let first_slot = inv.slot(0).expect("slot 0 has start item");
    assert_eq!(first_slot.item_id, "bat");
    assert_eq!(first_slot.count, 1);

    // Verify hotbar shows bat in slot 1 (key 1 -> index 0)
    let hotbar_slot_1 = hotbar.slot_entry(1, &inv).expect("hotbar slot 1 entry");
    assert_eq!(hotbar_slot_1.item_id, "bat");

    // Select hotbar slot 1
    let selected = hotbar.select(1, &inv, &registry);
    assert_eq!(selected, Some("bat"));
    assert_eq!(hotbar.selected_item(), Some("bat"));
    assert_eq!(hotbar.equipped_item_id(), Some("bat"));

    // Weapon link resolves through WeaponRegistry
    let item_def = registry.get("bat").expect("bat in ItemRegistry");
    assert_eq!(item_def.category, ItemCategory::Weapon);
    assert_eq!(item_def.weapon_id, "bat");

    let weapon_def = weapons.get(&item_def.weapon_id).expect("bat in WeaponRegistry");
    assert_eq!(weapon_def.id, "bat");
    assert_eq!(weapon_def.name, "Baseball Bat");

    // Also verify helper methods on Hotbar and ItemDef
    let resolved_weapon = hotbar
        .selected_weapon(&registry, &weapons)
        .expect("hotbar selected_weapon resolves");
    assert_eq!(resolved_weapon.id, "bat");

    let item_weapon = item_def.weapon(&weapons).expect("item_def.weapon resolves");
    assert_eq!(item_weapon.id, "bat");

    // Selecting slot 1 again toggles it off (holsters the weapon)
    let toggled = hotbar.select(1, &inv, &registry);
    assert_eq!(toggled, None);
    assert_eq!(hotbar.selected_item(), None);
}

#[test]
fn hotbar_auto_holsters_when_weapon_removed() {
    let registry = ItemRegistry::from_repo().expect("load items");
    let mut inv = start_inventory(&registry);
    let mut hotbar = Hotbar::new();

    // Select slot 1 (bat)
    hotbar.select(1, &inv, &registry);
    assert_eq!(hotbar.selected_item(), Some("bat"));

    // Remove the bat from inventory
    inv.remove("bat", 1);
    assert_eq!(inv.count("bat"), 0);

    // Notify hotbar of inventory change
    hotbar.on_inventory_changed(&inv);
    assert_eq!(hotbar.selected_item(), None);
}

#[test]
fn slot_swapping_and_merging() {
    let registry = ItemRegistry::from_repo().expect("load items");
    let mut inv = Inventory::with_capacity_and_registry(4, 100.0, Arc::new(registry));

    inv.add("canned_beans", 6); // slot 0 has 6 beans (max 10)
    inv.add("water_bottle", 2); // slot 1 has 2 water

    // Swap slots 0 and 1 (different items)
    inv.swap_slots(0, 1);
    assert_eq!(inv.slot(0).unwrap().item_id, "water_bottle");
    assert_eq!(inv.slot(1).unwrap().item_id, "canned_beans");

    // Add 8 beans to slot 2
    inv.add("canned_beans", 8); // slot 2 has 8 beans
    assert_eq!(inv.slot(2).unwrap().count, 8);

    // Swap slots 1 (6 beans) and 2 (8 beans): merges up to max_stack 10!
    // Slot 1 gets 6 + 4 = 10, slot 2 retains 8 - 4 = 4
    inv.swap_slots(1, 2);
    assert_eq!(inv.slot(1).unwrap().count, 10);
    assert_eq!(inv.slot(2).unwrap().count, 4);
}

#[test]
fn take_slot_and_remove_from_slot() {
    let registry = ItemRegistry::from_repo().expect("load items");
    let mut inv = Inventory::with_capacity_and_registry(4, 100.0, Arc::new(registry));

    inv.add("bandage", 5);
    assert_eq!(inv.count("bandage"), 5);

    let removed = inv.remove_from_slot(0, 2);
    assert_eq!(removed, 2);
    assert_eq!(inv.slot(0).unwrap().count, 3);

    let taken = inv.take_slot(0).expect("slot 0 has bandage");
    assert_eq!(taken.item_id, "bandage");
    assert_eq!(taken.count, 3);
    assert!(inv.slot(0).is_none());
}

#[test]
fn inventory_save_and_load() {
    let registry = ItemRegistry::from_repo().expect("load items");
    let mut inv = Inventory::with_capacity_and_registry(4, 100.0, Arc::new(registry.clone()));
    inv.add("canned_beans", 7);
    inv.add("water_bottle", 3);

    let save = inv.save();
    assert_eq!(save.stacks.len(), 2);

    let mut inv2 = Inventory::with_capacity_and_registry(4, 100.0, Arc::new(registry));
    inv2.load_save(&save);

    assert_eq!(inv2.count("canned_beans"), 7);
    assert_eq!(inv2.count("water_bottle"), 3);
    assert_eq!(inv2.slot(0), inv.slot(0));
    assert_eq!(inv2.slot(1), inv.slot(1));
}

#[test]
fn all_weapons_in_items_json_exist_in_weapons_json() {
    let items = ItemRegistry::from_repo().expect("items registry");
    let weapon_path = paths::repo_path("data/weapons.json");
    let weapons = WeaponRegistry::load(&weapon_path).expect("weapons registry");

    let result = items.validate_weapons(&weapons);
    assert!(result.is_ok(), "all weapon items must validate against weapons.json: {:?}", result.err());

    let mut weapon_count = 0;
    for def in items.iter() {
        if def.category == ItemCategory::Weapon {
            weapon_count += 1;
            let weapon = weapons.get(&def.weapon_id);
            assert!(weapon.is_some(), "weapon id {} not found", def.weapon_id);
        }
    }
    println!("validated {} weapon items against weapons.json", weapon_count);
    assert!(weapon_count > 0);
}
