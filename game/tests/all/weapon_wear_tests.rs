use std::sync::Arc;

use game::items::{Inventory, InventorySaveData, ItemRegistry, ItemStack};
use game::weapons::wear::{
    apply_hit_wear, apply_shot_wear, is_broken, jam_chance, jam_roll, melee_damage_mult,
    spread_mult, WearConfig,
};
use game::weapons::{WeaponKind, WeaponRegistry};

#[test]
fn weapon_wear_json_covers_all_weapons() {
    let cfg = WearConfig::from_repo().expect("load data/weapon_wear.json");
    let weapons = WeaponRegistry::load(&paths::repo_path("data/weapons.json"))
        .expect("load data/weapons.json");

    assert!((cfg.jam_below - 0.3).abs() < 1e-6);
    assert!((cfg.jam_chance_max - 0.15).abs() < 1e-6);
    assert!((cfg.spread_below - 0.5).abs() < 1e-6);
    assert!((cfg.spread_mult_max - 1.6).abs() < 1e-6);
    assert!((cfg.melee_damage_mult_at_zero - 0.5).abs() < 1e-6);

    for def in weapons.iter() {
        match def.kind {
            WeaponKind::Gun | WeaponKind::Bow => {
                assert!(
                    cfg.wear_per_shot(&def.id) > 0.0,
                    "gun/bow '{}' must have positive wearPerShot",
                    def.id
                );
            }
            WeaponKind::Melee => {
                assert!(
                    cfg.wear_per_hit(&def.id) > 0.0,
                    "melee '{}' must have positive wearPerHit",
                    def.id
                );
            }
            WeaponKind::Throwable => {}
        }
    }
}

#[test]
fn shot_and_hit_wear_degrade_smoothly_and_clamp_at_zero() {
    let cfg = WearConfig::from_repo().expect("load wear config");

    // Fresh stack counts as 1.0
    let mut pistol = ItemStack::new("pistol", 1);
    assert_eq!(pistol.wear, None);
    assert_eq!(pistol.condition(), 1.0);
    assert!(!is_broken(pistol.condition()));

    // ~400 shots for pistol
    for _ in 0..399 {
        apply_shot_wear(&mut pistol, &cfg);
    }
    assert!(pistol.condition() > 0.0);
    assert!(!is_broken(pistol.condition()));

    // Extra shots reach 0.0 and never go below 0.0
    for _ in 0..50 {
        apply_shot_wear(&mut pistol, &cfg);
    }
    assert_eq!(pistol.condition(), 0.0);
    assert_eq!(pistol.wear, Some(0.0));
    assert!(is_broken(pistol.condition()));

    // AK lasts ~600 shots, crossbow ~250 shots, bat ~300 hits
    let mut ak = ItemStack::new("ak47", 1);
    for _ in 0..450 {
        apply_shot_wear(&mut ak, &cfg);
    }
    assert!(ak.condition() > 0.2);

    let mut crossbow = ItemStack::new("crossbow", 1);
    for _ in 0..250 {
        apply_shot_wear(&mut crossbow, &cfg);
    }
    assert_eq!(crossbow.condition(), 0.0);

    let mut bat = ItemStack::new("bat", 1);
    for _ in 0..300 {
        apply_hit_wear(&mut bat, &cfg);
    }
    assert!(bat.condition() <= 0.01);

    // Non-weapon items do not acquire wear
    let mut beans = ItemStack::new("canned_beans", 3);
    apply_shot_wear(&mut beans, &cfg);
    apply_hit_wear(&mut beans, &cfg);
    assert_eq!(beans.wear, None);
}

#[test]
fn jam_only_occurs_below_jam_below() {
    let cfg = WearConfig::from_repo().expect("load wear config");

    // Even with a roll of 0.0, healthy weapons (>= jamBelow) never jam
    assert_eq!(jam_chance(1.0, &cfg), 0.0);
    assert_eq!(jam_chance(0.5, &cfg), 0.0);
    assert_eq!(jam_chance(0.3, &cfg), 0.0);
    assert!(!jam_roll(1.0, &cfg, || 0.0));
    assert!(!jam_roll(0.3, &cfg, || 0.0));

    // Below jamBelow, chance increases smoothly up to jamChanceMax
    let mid_chance = jam_chance(0.15, &cfg);
    assert!((mid_chance - 0.075).abs() < 1e-5);
    assert!(jam_roll(0.15, &cfg, || 0.01));
    assert!(!jam_roll(0.15, &cfg, || 0.10));

    let max_chance = jam_chance(0.0, &cfg);
    assert!((max_chance - 0.15).abs() < 1e-5);
    assert!(jam_roll(0.0, &cfg, || 0.14));
    assert!(!jam_roll(0.0, &cfg, || 0.16));
}

#[test]
fn spread_and_melee_damage_scale_smoothly() {
    let cfg = WearConfig::from_repo().expect("load wear config");

    assert_eq!(spread_mult(1.0, &cfg), 1.0);
    assert_eq!(spread_mult(0.5, &cfg), 1.0);
    assert!((spread_mult(0.25, &cfg) - 1.3).abs() < 1e-5);
    assert!((spread_mult(0.0, &cfg) - 1.6).abs() < 1e-5);
    assert!(spread_mult(0.1, &cfg) > spread_mult(0.3, &cfg));

    assert!((melee_damage_mult(1.0, &cfg) - 1.0).abs() < 1e-5);
    assert!((melee_damage_mult(0.5, &cfg) - 0.75).abs() < 1e-5);
    assert!((melee_damage_mult(0.0, &cfg) - 0.5).abs() < 1e-5);
}

#[test]
fn stack_mods_and_old_save_json_roundtrip() {
    let registry = Arc::new(ItemRegistry::from_repo().expect("load items"));

    // Old save JSON without wear or mods fields loads cleanly with condition 1.0
    let old_json = r#"{
        "stacks": [
            { "slot": 0, "id": "pistol", "count": 1 },
            { "slot": 1, "id": "canned_beans", "count": 4 }
        ]
    }"#;
    let old_save: InventorySaveData = serde_json::from_str(old_json).expect("parse old save");
    assert_eq!(old_save.stacks[0].wear, None);
    assert!(old_save.stacks[0].mods.is_empty());

    let mut inv = Inventory::with_capacity_and_registry(6, 100.0, registry.clone());
    inv.load_save(&old_save);
    let loaded_pistol = inv.slot(0).expect("slot 0 pistol");
    assert_eq!(loaded_pistol.condition(), 1.0);
    assert!(!loaded_pistol.has_mod("suppressor"));

    // Modified wear and mods survive save -> JSON -> load
    let mut modded = ItemStack::new("rifle", 1);
    modded.wear = Some(0.42);
    modded.mods = vec!["scope_4x".to_string(), "suppressor".to_string()];
    assert!(modded.has_mod("scope_4x"));
    assert!(modded.has_mod("suppressor"));
    assert!(!modded.has_mod("red_dot"));
    inv.slots_mut()[2] = Some(modded);

    let saved_json = serde_json::to_string(&inv.save()).expect("serialize inventory");
    let parsed_save: InventorySaveData = serde_json::from_str(&saved_json).expect("reparse save");
    let mut inv2 = Inventory::with_capacity_and_registry(6, 100.0, registry);
    inv2.load_save(&parsed_save);

    let r = inv2.slot(2).expect("slot 2 rifle");
    assert!((r.condition() - 0.42).abs() < 1e-6);
    assert!(r.has_mod("scope_4x"));
    assert!(r.has_mod("suppressor"));
}

#[test]
fn stacks_with_wear_or_mods_never_merge() {
    let registry = Arc::new(ItemRegistry::from_repo().expect("load items"));
    let mut inv = Inventory::with_capacity_and_registry(4, 100.0, registry);

    // Use a stackable item id to prove wear/mods block merging even when max_stack > 1
    let mut s0 = ItemStack::new("canned_beans", 2);
    s0.wear = Some(0.8);
    let mut s1 = ItemStack::new("canned_beans", 3);
    s1.wear = Some(0.5);
    let mut s2 = ItemStack::new("canned_beans", 1);
    s2.mods = vec!["ext_mag".to_string()];

    inv.slots_mut()[0] = Some(s0);
    inv.slots_mut()[1] = Some(s1);
    inv.slots_mut()[2] = Some(s2);

    // Swapping two stacks with wear swaps them instead of merging
    inv.swap_slots(0, 1);
    assert_eq!(inv.slot(0).unwrap().count, 3);
    assert_eq!(inv.slot(0).unwrap().wear, Some(0.5));
    assert_eq!(inv.slot(1).unwrap().count, 2);
    assert_eq!(inv.slot(1).unwrap().wear, Some(0.8));

    // Adding plain canned_beans cannot merge into worn/modded slots; only slot 3 (empty) is available
    assert!(inv.can_add("canned_beans", 10));
    assert!(!inv.can_add("canned_beans", 11));
    let leftover = inv.add("canned_beans", 12);
    assert_eq!(leftover, 2);
    assert_eq!(inv.slot(0).unwrap().count, 3);
    assert_eq!(inv.slot(1).unwrap().count, 2);
    assert_eq!(inv.slot(2).unwrap().count, 1);
    assert_eq!(inv.slot(3).unwrap().count, 10);
    assert_eq!(inv.slot(3).unwrap().wear, None);

    // Swapping modded slot with plain slot preserves mods without merging
    inv.swap_slots(2, 3);
    assert_eq!(inv.slot(2).unwrap().count, 10);
    assert!(inv.slot(2).unwrap().mods.is_empty());
    assert_eq!(inv.slot(3).unwrap().count, 1);
    assert!(inv.slot(3).unwrap().has_mod("ext_mag"));
}
