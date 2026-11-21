use game::items::{ItemCategory, ItemRegistry};
use game::weapons::{WeaponKind, WeaponRegistry};

#[test]
fn projectile_weapons_new_all_weapons_load() {
    let weapon_path = paths::repo_path("data/weapons.json");
    let weapons = WeaponRegistry::load(&weapon_path).expect("load weapons.json");

    let required = [
        "bat", "machete", "knife", "axe", "crowbar", "katana",
        "pistol", "shotgun", "rifle", "ak47", "smg",
        "dagger", "spear", "hammer", "pipe", "fire_axe", "bow",
    ];

    for id in &required {
        assert!(weapons.get(id).is_some(), "weapon {} must exist in weapons.json", id);
    }
}

#[test]
fn projectile_weapons_new_items_validate() {
    let item_path = paths::repo_path("data/items.json");
    let items = ItemRegistry::load(&item_path).expect("load items.json");
    let weapon_path = paths::repo_path("data/weapons.json");
    let weapons = WeaponRegistry::load(&weapon_path).expect("load weapons.json");

    assert!(items.validate_weapons(&weapons).is_ok());

    let required_items = [
        "dagger", "spear", "hammer", "pipe", "fire_axe", "bow",
        "arrow", "arrow_broken",
    ];

    for id in &required_items {
        let item = items.get(id).expect("item present");
        assert_eq!(&item.id, id);
    }

    let arrow = items.get("arrow").unwrap();
    assert_eq!(arrow.category, ItemCategory::Ammo);
    assert_eq!(arrow.max_stack, 30);
    assert_eq!(arrow.ammo_type, "arrow");

    let broken = items.get("arrow_broken").unwrap();
    assert_eq!(broken.category, ItemCategory::Material);
    assert_eq!(broken.max_stack, 30);
}

#[test]
fn projectile_weapons_new_unknown_kind_fails() {
    let json = r#"{
        "blaster": {
            "name": "Laser Blaster",
            "kind": "plasma_blaster",
            "damage": 100
        }
    }"#;
    let res = WeaponRegistry::from_json(json);
    assert!(res.is_err(), "unknown weapon kind must fail to parse");
}

#[test]
fn projectile_weapons_new_bow_properties() {
    let weapon_path = paths::repo_path("data/weapons.json");
    let weapons = WeaponRegistry::load(&weapon_path).expect("load weapons.json");
    let bow = weapons.get("bow").expect("bow exists");

    assert!(bow.is_bow());
    assert!(!bow.is_gun());
    assert_eq!(bow.kind, WeaponKind::Bow);
    assert_eq!(bow.damage, 50.0);
    assert_eq!(bow.headshot_multiplier, 2.5);
    assert_eq!(bow.draw_time, 1.0);
    assert_eq!(bow.arrow_speed, 45.0);
    assert_eq!(bow.gravity, 9.8);
    assert_eq!(bow.pickup_chance, 0.6);
    assert_eq!(bow.max_flight_seconds, 10.0);
    assert_eq!(bow.ammo_item, "arrow");
    assert!(bow.noise_radius <= 3.0, "bow must be silent (low noiseRadius)");
}

#[test]
fn projectile_weapons_new_spear_properties() {
    let weapon_path = paths::repo_path("data/weapons.json");
    let weapons = WeaponRegistry::load(&weapon_path).expect("load weapons.json");
    let spear = weapons.get("spear").expect("spear exists");

    assert_eq!(spear.kind, WeaponKind::Melee);
    assert_eq!(spear.range, 3.0);
    assert!(spear.arc_deg <= 40.0, "spear has narrow thrust arc");
    assert!(spear.throwable);
    assert_eq!(spear.throw_speed, 20.0);
    assert_eq!(spear.throw_damage, 45.0);
}

#[test]
fn projectile_weapons_new_dagger_stats() {
    let weapon_path = paths::repo_path("data/weapons.json");
    let weapons = WeaponRegistry::load(&weapon_path).expect("load weapons.json");
    let dagger = weapons.get("dagger").expect("dagger exists");
    let knife = weapons.get("knife").expect("knife exists");

    assert_eq!(dagger.kind, WeaponKind::Melee);
    assert!(dagger.damage < knife.damage, "dagger damage lower than knife");
    assert!(dagger.range < knife.range, "dagger range shorter than knife");
    assert!(dagger.windup < knife.windup, "dagger windup faster than knife");
    assert!(dagger.cooldown < knife.cooldown, "dagger cooldown faster than knife");
    assert!(dagger.stamina_cost < knife.stamina_cost, "dagger stamina cost cheaper than knife");
}

#[test]
fn projectile_weapons_new_hammer_stats() {
    let weapon_path = paths::repo_path("data/weapons.json");
    let weapons = WeaponRegistry::load(&weapon_path).expect("load weapons.json");
    let hammer = weapons.get("hammer").expect("hammer exists");

    assert_eq!(hammer.kind, WeaponKind::Melee);
    assert!(hammer.stagger >= 0.7, "hammer has high stagger");
    assert!(hammer.cooldown >= 0.9, "hammer is slow");
}

#[test]
fn projectile_weapons_new_pipe_stats() {
    let weapon_path = paths::repo_path("data/weapons.json");
    let weapons = WeaponRegistry::load(&weapon_path).expect("load weapons.json");
    let pipe = weapons.get("pipe").expect("pipe exists");

    assert_eq!(pipe.kind, WeaponKind::Melee);
    assert_eq!(pipe.damage, 22.0);
}

#[test]
fn projectile_weapons_new_fire_axe_stats() {
    let weapon_path = paths::repo_path("data/weapons.json");
    let weapons = WeaponRegistry::load(&weapon_path).expect("load weapons.json");
    let fire_axe = weapons.get("fire_axe").expect("fire_axe exists");
    let axe = weapons.get("axe").expect("axe exists");

    assert_eq!(fire_axe.kind, WeaponKind::Melee);
    assert!(fire_axe.damage > axe.damage, "fire axe has higher damage than standard axe");
    assert!(fire_axe.cooldown > axe.cooldown, "fire axe is slower than standard axe");
    assert!(fire_axe.tags.contains(&"chop".to_string()));
}
