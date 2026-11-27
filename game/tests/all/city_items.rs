use game::armor::{armor_reduction, apply_armor_damage, ArmorConfig};
use game::downed::Downed;
use game::items::{Inventory, ItemRegistry, ItemStack};
use game::night_vision::{NightVision, BATTERY_ITEM};
use game::survival::{adrenaline_revive, Boosts, BoostsConfig};
use game::vehicles::{Cars, VehicleConfigData};
use game::weapons::wear::{repair_with_parts, GUN_PARTS_REPAIR_AMOUNT};
use game::weapons::WeaponRegistry;
use glam::Vec3;

const CITY_ITEM_IDS: &[&str] = &[
    "body_armor",
    "army_helmet",
    "army_rifle",
    "ammo_556",
    "army_ration",
    "night_vision",
    "battery",
    "antibiotics",
    "surgery_kit",
    "adrenaline_shot",
    "gun_parts",
    "car_parts",
];

#[test]
fn all_12_city_items_load_and_army_rifle_is_auto() {
    let items = ItemRegistry::load(&paths::repo_path("data/items.json")).expect("data/items.json");
    let weapons =
        WeaponRegistry::load(&paths::repo_path("data/weapons.json")).expect("data/weapons.json");

    for id in CITY_ITEM_IDS {
        assert!(items.get(id).is_some(), "missing item '{id}' in data/items.json");
    }

    for def in items.iter() {
        if !def.weapon_id.is_empty() {
            assert!(
                weapons.get(&def.weapon_id).is_some(),
                "item '{}' references missing weaponId '{}'",
                def.id,
                def.weapon_id
            );
        }
    }

    for w in weapons.iter() {
        if !w.ammo_item.is_empty() {
            assert!(
                items.get(&w.ammo_item).is_some(),
                "weapon '{}' references missing ammoItem '{}'",
                w.id,
                w.ammo_item
            );
        }
    }

    let army = weapons.get("army_rifle").expect("army_rifle in weapons.json");
    assert!(army.auto);
    assert_eq!(army.ammo_item, "ammo_556");
    assert_eq!(army.magazine_size, 30);
    assert_eq!(army.damage, 32.0);
    assert!((army.fire_interval - 0.085).abs() < 1e-6);
    assert_eq!(army.noise_radius, 70.0);
    let ak = weapons.get("ak47").expect("ak47 in weapons.json");
    assert!(army.spread_aim_deg < ak.spread_aim_deg);
}

#[test]
fn armor_reduction_rules() {
    let cfg = ArmorConfig::from_repo().expect("data/armor.json");
    let mut inv = Inventory::new();

    assert_eq!(armor_reduction(&inv, &cfg), 0.0);

    inv.add("body_armor", 1);
    assert!((armor_reduction(&inv, &cfg) - 0.35).abs() < 1e-6);

    inv.add("body_armor", 1);
    assert!((armor_reduction(&inv, &cfg) - 0.35).abs() < 1e-6);

    inv.add("army_helmet", 1);
    assert!((armor_reduction(&inv, &cfg) - 0.5).abs() < 1e-6);
    assert!((apply_armor_damage(40.0, armor_reduction(&inv, &cfg)) - 20.0).abs() < 1e-6);
}

#[test]
fn antibiotics_heal_and_stack_in_time() {
    let cfg = BoostsConfig::from_repo().expect("data/boosts.json");
    let mut boosts = Boosts::new();
    boosts.start_antibiotics(&cfg.antibiotics);

    let mut healed = 0.0;
    for _ in 0..60 {
        healed += boosts.update(1.0);
    }
    assert!((healed - 60.0).abs() < 0.5);

    boosts.start_antibiotics(&cfg.antibiotics);
    boosts.start_antibiotics(&cfg.antibiotics);
    assert!((boosts.heal_seconds_remaining() - 120.0).abs() < 1e-4);
}

#[test]
fn adrenaline_revives_downed_player() {
    let mut downed = Downed::default();
    assert!(!adrenaline_revive(&mut downed));

    downed.on_lethal_hit(5.0);
    assert!(downed.is_down());
    assert!(adrenaline_revive(&mut downed));
    assert!(downed.is_up());
}

#[test]
fn night_vision_battery_cycle() {
    let cfg = BoostsConfig::from_repo().expect("data/boosts.json");
    let mut inv = Inventory::new();
    let mut nv = NightVision::new();

    assert!(!nv.toggle(&mut inv, &cfg.night_vision));
    assert!(!nv.on);

    inv.add(BATTERY_ITEM, 1);
    assert!(nv.toggle(&mut inv, &cfg.night_vision));
    nv.update(299.0, &mut inv, &cfg.night_vision);
    assert!(nv.on);
    nv.update(2.0, &mut inv, &cfg.night_vision);
    assert!(!nv.on);

    inv.add(BATTERY_ITEM, 2);
    assert!(nv.toggle(&mut inv, &cfg.night_vision));
    nv.update(300.0, &mut inv, &cfg.night_vision);
    assert!(nv.on);
    assert_eq!(inv.count(BATTERY_ITEM), 0);
    nv.update(300.0, &mut inv, &cfg.night_vision);
    assert!(!nv.on);
}

#[test]
fn gun_parts_and_car_parts_repair() {
    let mut gun = ItemStack::new("army_rifle", 1);
    gun.wear = Some(0.6);
    assert!(repair_with_parts(&mut gun, GUN_PARTS_REPAIR_AMOUNT));
    assert_eq!(gun.condition(), 1.0);
    assert_eq!(gun.wear, None);

    let cfg = VehicleConfigData::from_repo().expect("data/vehicles.json");
    let mut cars = Cars::new(cfg);
    cars.spawn_car("c1", "red", Vec3::ZERO, 0.0, 20.0, 0.0);
    assert!(cars.get("c1").unwrap().condition.is_wrecked());

    let mut inv = Inventory::new();
    inv.add("car_parts", 2);
    assert!(cars.repair_with_parts("c1", &mut inv));
    let car = cars.get("c1").unwrap();
    assert!(!car.condition.is_wrecked());
    assert!((car.condition.current() - 150.0).abs() < 1e-4);
    assert_eq!(inv.count("car_parts"), 1);
}
