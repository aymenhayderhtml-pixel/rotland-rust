use std::sync::Arc;

use game::gunsmith::*;
use game::items::{Inventory, ItemRegistry, ItemStack};
use game::weapons::AttachmentRegistry;

fn setup() -> (Arc<ItemRegistry>, AttachmentRegistry, GunsmithConfig, GunsmithStock) {
    let items = Arc::new(ItemRegistry::from_repo().expect("ItemRegistry::from_repo"));
    let attachments = AttachmentRegistry::from_repo().expect("AttachmentRegistry::from_repo");
    let cfg = GunsmithConfig::from_repo().expect("GunsmithConfig::from_repo");
    cfg.validate(Some(&items)).expect("gunsmith.json validates against items.json");
    let stock = GunsmithStock::new(&cfg);
    (items, attachments, cfg, stock)
}

#[test]
fn config_loads_validates_and_checks_tiers_and_exclusives() {
    let (_items, _att, cfg, stock) = setup();

    // Camp tier 0: pistol + pistol_ammo only
    assert_eq!(cfg.required_tier(&ShopKind::Camp { tier: 0 }, "pistol"), Some(0));
    assert_eq!(cfg.required_tier(&ShopKind::Camp { tier: 0 }, "pistol_ammo"), Some(0));
    assert_eq!(cfg.required_tier(&ShopKind::Camp { tier: 0 }, "shotgun"), Some(1));
    assert_eq!(cfg.required_tier(&ShopKind::Camp { tier: 0 }, "ak47"), Some(3));

    // Town exclusives exist only in their respective towns (not in camp or other towns)
    let towns = ["dusthollow", "frostmere", "blackwater", "ridgeback_station"];
    let exclusives = [
        ("katana", "dusthollow"),
        ("drum_mag", "frostmere"),
        ("bolt_quiver", "blackwater"),
        ("hunting_rifle", "ridgeback_station"),
        ("scope_4x", "ridgeback_station"),
    ];
    for (item_id, owner_town) in exclusives {
        assert!(cfg.required_tier(&ShopKind::Camp { tier: 3 }, item_id).is_none(), "{item_id} must not be in camp");
        for town in towns {
            let kind = ShopKind::Town { key: town.to_string() };
            if town == owner_town {
                assert!(cfg.required_tier(&kind, item_id).is_some(), "{item_id} missing from {town}");
                assert!(stock.qty(&kind, item_id) > 0);
            } else {
                assert!(cfg.required_tier(&kind, item_id).is_none(), "{item_id} leaked into {town}");
            }
        }
    }
}

#[test]
fn buy_happy_path_and_refusals_tier_soldout_scrap_full_heavy() {
    let (items, _att, cfg, mut stock) = setup();
    let mut bag = Inventory::with_capacity_and_registry(24, 120.0, items.clone());

    // Tier locked at camp tier 0
    bag.add("scrap_metal", 60);
    let camp0 = ShopKind::Camp { tier: 0 };
    assert_eq!(
        buy(&camp0, &mut stock, "shotgun", &mut bag, &cfg, &items),
        Err(GunsmithError::TierLocked { required: 1 })
    );
    assert_eq!(buy(&camp0, &mut stock, "hunting_rifle", &mut bag, &cfg, &items), Err(GunsmithError::NotInShop));

    // Tier 1 unlocks shotgun (45 scrap)
    let camp1 = ShopKind::Camp { tier: 1 };
    let bought = buy(&camp1, &mut stock, "shotgun", &mut bag, &cfg, &items).unwrap();
    assert_eq!(bought.count_added, 1);
    assert_eq!(bought.scrap_spent, 45);
    assert_eq!(bag.count("shotgun"), 1);
    assert_eq!(bag.count("scrap_metal"), 15);

    // Sold out on second shotgun
    assert_eq!(
        buy(&camp1, &mut stock, "shotgun", &mut bag, &cfg, &items),
        Err(GunsmithError::SoldOut)
    );

    // Not enough scrap for pistol (costs 25, have 15)
    assert_eq!(
        buy(&camp0, &mut stock, "pistol", &mut bag, &cfg, &items),
        Err(GunsmithError::NotEnoughScrap { needed: 25, have: 15 })
    );

    // Buy ammo pack at tier 0 (3 scrap -> 12 pistol_ammo)
    let ammo = buy(&camp0, &mut stock, "pistol_ammo", &mut bag, &cfg, &items).unwrap();
    assert_eq!(ammo.count_added, 12);
    assert_eq!(bag.count("pistol_ammo"), 12);
    assert_eq!(bag.count("scrap_metal"), 12);

    // Bag full refusal (1-slot bag with 1 crowbar, plenty of scrap would be needed; use 2-slot bag)
    let mut tiny_bag = Inventory::with_capacity_and_registry(2, 40.0, items.clone());
    tiny_bag.add("scrap_metal", 20); // 1 slot
    tiny_bag.add("crowbar", 1); // 2nd slot
    // Buying revolver_ammo (4 scrap) leaves 16 scrap still in slot 0 -> no free slot for ammo
    assert_eq!(
        buy(&camp1, &mut stock, "revolver_ammo", &mut tiny_bag, &cfg, &items),
        Err(GunsmithError::BagFull)
    );

    // Too heavy refusal on buy (bag overweight after scrap removal)
    let mut heavy_bag = Inventory::with_capacity_and_registry(24, 20.0, items.clone());
    heavy_bag.add("fuel_can", 1); // 10.0 kg
    heavy_bag.add("scrap_metal", 4); // 3.2 kg
    heavy_bag.set_max_weight(10.0); // after removing 4 scrap, still at 10.0 kg so adding ammo exceeds cap
    assert_eq!(
        buy(&camp1, &mut stock, "revolver_ammo", &mut heavy_bag, &cfg, &items),
        Err(GunsmithError::TooHeavy)
    );

    // Too heavy refusal on sell (scrap returned exceeds weight limit)
    let mut light_bag = Inventory::with_capacity_and_registry(24, 5.0, items.clone());
    light_bag.add("katana", 1); // 1.2 kg, sells for 38 scrap (30.4 kg)
    assert_eq!(
        sell_slot(0, 1, &mut light_bag, &cfg, &items),
        Err(GunsmithError::TooHeavy)
    );
}

#[test]
fn sell_splits_scrap_across_stacks_and_returns_mods_first() {
    let (items, att, cfg, _stock) = setup();
    let mut bag = Inventory::with_capacity_and_registry(24, 80.0, items.clone());

    // Give a modded, pristine AK-47 (price 110 * 0.4 = 44 scrap -> splits into 20, 20, 4!)
    let mut ak = ItemStack::new("ak47", 1);
    bag.add("scrap_metal", 60);
    fit(&mut ak, "suppressor", &mut bag, &cfg, &items, &att).unwrap();
    bag.remove("scrap_metal", bag.count("scrap_metal")); // empty scrap first
    bag.slots_mut()[0] = Some(ak);

    let res = sell_slot(0, 1, &mut bag, &cfg, &items).unwrap();
    assert_eq!(res.scrap_gained, 44);
    assert_eq!(res.mods_returned, vec!["suppressor".to_string()]);
    assert_eq!(bag.count("ak47"), 0);
    assert_eq!(bag.count("suppressor"), 1);
    assert_eq!(bag.count("scrap_metal"), 44);

    // Verify scrap_metal split into stacks of max 20
    let scrap_counts: Vec<u32> = bag
        .slots()
        .iter()
        .flatten()
        .filter(|s| s.item_id == "scrap_metal")
        .map(|s| s.count)
        .collect();
    assert_eq!(scrap_counts, vec![20, 20, 4]);

    // Refuses selling scrap_metal itself
    let scrap_slot = bag.slots().iter().position(|s| s.as_ref().is_some_and(|x| x.item_id == "scrap_metal")).unwrap();
    assert_eq!(sell_slot(scrap_slot, 1, &mut bag, &cfg, &items), Err(GunsmithError::CannotSell));
}

#[test]
fn repair_cost_and_repair_all_and_refusals() {
    let (items, _att, cfg, _stock) = setup();
    let mut bag = Inventory::with_capacity_and_registry(24, 40.0, items.clone());

    let mut pistol = ItemStack::new("pistol", 1);
    assert_eq!(repair_cost(&pistol, &cfg, &items), 0);
    assert_eq!(repair(&mut pistol, &mut bag, &cfg, &items), Err(GunsmithError::AlreadyRepaired));

    // Worn to 0.0: repair_cost = ceil(0.5 * 25 * 1.0) = 13
    pistol.wear = Some(0.0);
    let cost = repair_cost(&pistol, &cfg, &items);
    assert_eq!(cost, 13);

    // Refused without scrap
    assert_eq!(
        repair(&mut pistol, &mut bag, &cfg, &items),
        Err(GunsmithError::NotEnoughScrap { needed: 13, have: 0 })
    );

    // Repair succeeds with scrap
    bag.add("scrap_metal", 15);
    let spent = repair(&mut pistol, &mut bag, &cfg, &items).unwrap();
    assert_eq!(spent, 13);
    assert!((pistol.condition() - 1.0).abs() < 1e-6);
    assert_eq!(bag.count("scrap_metal"), 2);

    // Not a weapon refusal
    let mut beans = ItemStack::new("canned_beans", 1);
    assert_eq!(repair(&mut beans, &mut bag, &cfg, &items), Err(GunsmithError::NotAWeapon));

    // Repair all across multiple slots
    let mut p1 = ItemStack::new("pistol", 1);
    p1.wear = Some(0.5); // ceil(0.5 * 25 * 0.5) = 7
    let mut bat = ItemStack::new("bat", 1);
    bat.wear = Some(0.0); // ceil(0.5 * 7 * 1.0) = 4
    bag.slots_mut()[5] = Some(p1);
    bag.slots_mut()[6] = Some(bat);
    bag.add("scrap_metal", 20); // have 22
    let (repaired_n, total_spent) = repair_all(&mut bag, &cfg, &items).unwrap();
    assert_eq!(repaired_n, 2);
    assert_eq!(total_spent, 11);
    assert!((bag.slot(5).unwrap().condition() - 1.0).abs() < 1e-6);
    assert!((bag.slot(6).unwrap().condition() - 1.0).abs() < 1e-6);
}

#[test]
fn fit_and_remove_mod_and_slot_rules() {
    let (items, att, cfg, mut stock) = setup();
    let mut bag = Inventory::with_capacity_and_registry(24, 120.0, items.clone());
    bag.add("scrap_metal", 80);

    let mut ak = ItemStack::new("ak47", 1);

    // Incompatible mod refused (bolt_quiver is crossbow-only)
    assert_eq!(
        fit(&mut ak, "bolt_quiver", &mut bag, &cfg, &items, &att),
        Err(GunsmithError::IncompatibleAttachment)
    );

    // Fit red_dot (20 + 5 labour = 25)
    assert_eq!(fit_cost("red_dot", &cfg, &att), 25);
    let spent = fit(&mut ak, "red_dot", &mut bag, &cfg, &items, &att).unwrap();
    assert_eq!(spent, 25);
    assert!(ak.has_mod("red_dot"));

    // Duplicate mod refused
    assert_eq!(
        fit(&mut ak, "red_dot", &mut bag, &cfg, &items, &att),
        Err(GunsmithError::AlreadyFitted)
    );

    // Second Optic in same slot refused
    assert_eq!(
        fit(&mut ak, "scope_2x", &mut bag, &cfg, &items, &att),
        Err(GunsmithError::SlotOccupied { slot_mod: "red_dot".to_string() })
    );

    // Remove red_dot -> returned to bag as item
    remove_mod(&mut ak, "red_dot", &mut bag, &items).unwrap();
    assert!(!ak.has_mod("red_dot"));
    assert_eq!(bag.count("red_dot"), 1);

    // Removing again refused
    assert_eq!(
        remove_mod(&mut ak, "red_dot", &mut bag, &items),
        Err(GunsmithError::ModNotFitted)
    );

    // Re-fitting from bag consumes the item and only charges fit_labour (5)
    let before_scrap = bag.count("scrap_metal");
    let labour_spent = fit(&mut ak, "red_dot", &mut bag, &cfg, &items, &att).unwrap();
    assert_eq!(labour_spent, cfg.fit_labour);
    assert_eq!(bag.count("red_dot"), 0);
    assert_eq!(bag.count("scrap_metal"), before_scrap - cfg.fit_labour);

    // Town exclusive enforcement in fit_at_shop
    let mut rifle = ItemStack::new("rifle", 1);
    let camp3 = ShopKind::Camp { tier: 3 };
    assert_eq!(
        fit_at_shop(&camp3, &mut stock, &mut rifle, "scope_4x", &mut bag, &cfg, &items, &att),
        Err(GunsmithError::NotInShop)
    );
    let ridgeback = ShopKind::Town { key: "ridgeback_station".to_string() };
    bag.add("scrap_metal", 60);
    let sniper_cost = fit_at_shop(&ridgeback, &mut stock, &mut rifle, "scope_4x", &mut bag, &cfg, &items, &att).unwrap();
    assert_eq!(sniper_cost, fit_cost("scope_4x", &cfg, &att));
    assert!(rifle.has_mod("scope_4x"));
    assert_eq!(stock.qty(&ridgeback, "scope_4x"), 0);
}

#[test]
fn stock_restocks_each_morning_and_roundtrips_save() {
    let (_items, _att, cfg, mut stock) = setup();
    let dusthollow = ShopKind::Town { key: "dusthollow".to_string() };
    let camp1 = ShopKind::Camp { tier: 1 };

    assert!(stock.take(&dusthollow, "katana", 1));
    assert_eq!(stock.qty(&dusthollow, "katana"), 0);
    assert!(stock.take(&camp1, "shotgun", 1));
    assert_eq!(stock.qty(&camp1, "shotgun"), 0);

    // Before restock hour (5 < 6): stays empty
    stock.tick_with_config(1, 5, &cfg);
    assert_eq!(stock.qty(&dusthollow, "katana"), 0);

    // At restock hour (6): refills all shops
    stock.tick_with_config(1, 6, &cfg);
    assert_eq!(stock.qty(&dusthollow, "katana"), 1);
    assert_eq!(stock.qty(&camp1, "shotgun"), 1);

    // Deplete again and save/load roundtrip
    assert!(stock.take(&dusthollow, "katana", 1));
    let saved = stock.to_save();
    let loaded = GunsmithStock::from_save_value(&saved, &cfg);
    assert_eq!(loaded.qty(&dusthollow, "katana"), 0);
    assert_eq!(loaded.qty(&camp1, "shotgun"), 1);
    assert_eq!(loaded.shop(&dusthollow).unwrap().last_restock_day(), 1);
}
