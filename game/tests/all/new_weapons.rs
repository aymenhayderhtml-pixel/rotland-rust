use game::items::{ItemCategory, ItemRegistry};
use game::loot::LootTableRegistry;
use game::weapons::{WeaponKind, WeaponRegistry};

#[test]
fn new_weapons_load_and_validate_items() {
    let weapons = WeaponRegistry::load(&paths::repo_path("data/weapons.json"))
        .expect("load data/weapons.json");
    let items = ItemRegistry::from_repo().expect("load data/items.json");

    items
        .validate_weapons(&weapons)
        .expect("all weapon items validate against weapons.json");

    let revolver = weapons.get("revolver").expect("revolver exists");
    assert_eq!(revolver.kind, WeaponKind::Gun);
    assert_eq!(revolver.damage, 55.0);
    assert_eq!(revolver.magazine_size, 6);
    assert_eq!(revolver.ammo_item, "revolver_ammo");

    let hunting_rifle = weapons.get("hunting_rifle").expect("hunting_rifle exists");
    assert_eq!(hunting_rifle.kind, WeaponKind::Gun);
    assert_eq!(hunting_rifle.damage, 110.0);
    assert_eq!(hunting_rifle.magazine_size, 4);
    assert_eq!(hunting_rifle.ammo_item, "rifle_rounds");
    let scope = hunting_rifle.scope.as_ref().expect("hunting_rifle has scope");
    assert_eq!(scope.fov, 18.0);
    assert_eq!(scope.sensitivity, 0.3);

    let crossbow = weapons.get("crossbow").expect("crossbow exists");
    assert_eq!(crossbow.kind, WeaponKind::Bow);
    assert_eq!(crossbow.damage, 80.0);
    assert_eq!(crossbow.arrow_speed, 60.0);
    assert_eq!(crossbow.gravity, 6.0);
    assert_eq!(crossbow.reload_time, 2.0);
    assert_eq!(crossbow.ammo_item, "crossbow_bolt");
    assert_eq!(crossbow.projectile_model, "crossbow_bolt");
    assert_eq!(crossbow.anim.reload.as_deref(), Some("crossbow_reload"));

    let grenade = weapons.get("grenade").expect("grenade exists");
    assert_eq!(grenade.kind, WeaponKind::Throwable);
    assert!(grenade.is_throwable());
    assert!(grenade.throwable);
    assert_eq!(grenade.throw_speed, 14.0);
    assert_eq!(grenade.bounce_restitution, 0.3);
    assert_eq!(grenade.fuse, 2.5);
    assert_eq!(grenade.blast_radius, 6.0);
    assert_eq!(grenade.damage, 150.0);
    assert_eq!(grenade.min_edge_frac, 0.2);

    let rev_ammo = items.get("revolver_ammo").expect("revolver_ammo item");
    assert_eq!(rev_ammo.category, ItemCategory::Ammo);
    assert_eq!(rev_ammo.max_stack, 36);

    let bolt = items.get("crossbow_bolt").expect("crossbow_bolt item");
    assert_eq!(bolt.category, ItemCategory::Ammo);
    assert_eq!(bolt.max_stack, 20);

    let grenade_item = items.get("grenade").expect("grenade item");
    assert_eq!(grenade_item.category, ItemCategory::Weapon);
    assert_eq!(grenade_item.max_stack, 5);
}

#[test]
fn new_weapons_loot_tables_placement() {
    let loot = LootTableRegistry::from_repo().expect("load data/loot_tables.json");

    let has = |table: &str, id: &str| {
        loot.get(table)
            .unwrap_or_else(|| panic!("missing loot table {table}"))
            .entries
            .iter()
            .any(|e| e.id == id)
    };

    assert!(has("house", "revolver"));
    assert!(has("house", "revolver_ammo"));
    assert!(has("police", "revolver"));
    assert!(has("police", "revolver_ammo"));

    assert!(has("cabin", "hunting_rifle"));
    assert!(has("cabin", "crossbow"));
    assert!(has("cabin", "crossbow_bolt"));
    assert!(has("barn", "crossbow"));
    assert!(has("barn", "crossbow_bolt"));

    // Grenade must ONLY appear in police, town_station, and metro_army tables, count 1-2
    for (name, table) in loot.iter() {
        let entry = table.entries.iter().find(|e| e.id == "grenade");
        if name == "police" || name == "town_station" || name == "metro_army" {
            let e = entry.unwrap_or_else(|| panic!("grenade missing from {name}"));
            assert_eq!(e.count, [1, 2], "grenade count in {name} must be [1, 2]");
        } else {
            assert!(entry.is_none(), "grenade must not appear in {name}");
        }
    }
}
