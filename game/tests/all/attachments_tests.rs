use std::path::Path;

use game::items::{ItemCategory, ItemRegistry};
use game::weapons::{
    AttachEffects, AttachSlot, AttachmentDef, AttachmentLoadError, AttachmentRegistry,
    WeaponRegistry, can_fit, effective_stats, kick_mult,
};

fn load_weapons() -> WeaponRegistry {
    let path = paths::repo_path("data/weapons.json");
    WeaponRegistry::load(&path).expect("data/weapons.json loads")
}

#[test]
fn attachment_registry_loads_all_nine_and_matches_items_and_weapons() {
    let reg = AttachmentRegistry::from_repo().expect("data/attachments.json loads");
    assert_eq!(reg.len(), 9);
    assert!(!reg.is_empty());

    let expected_ids = [
        "scope_2x",
        "scope_4x",
        "red_dot",
        "suppressor",
        "ext_mag",
        "drum_mag",
        "foregrip",
        "stock_pad",
        "bolt_quiver",
    ];
    let weapons = load_weapons();
    let items = ItemRegistry::from_repo().expect("data/items.json loads");

    for id in expected_ids {
        assert!(reg.contains(id), "missing attachment {id}");
        let def: &AttachmentDef = reg.get(id).expect("attachment def");
        assert_eq!(def.id, id);
        assert!(def.price_scrap > 0, "{id} should cost > 0 scrap");
        assert!(!def.fits.is_empty(), "{id} should fit at least one weapon");
        for weapon_id in &def.fits {
            assert!(
                weapons.get(weapon_id).is_some(),
                "{id} lists unknown weapon '{weapon_id}'"
            );
            assert!(can_fit(id, weapon_id, &reg));
        }
        let item = items
            .get(id)
            .unwrap_or_else(|| panic!("attachment '{id}' missing from data/items.json"));
        assert_eq!(item.category, ItemCategory::Attachment);
        assert_eq!(item.max_stack, 1);
    }

    assert_eq!(reg.iter().count(), 9);
    assert!(reg.for_weapon("ak47").count() >= 5);
    assert_eq!(
        AttachSlot::ALL.map(|s| s.as_str()),
        ["optic", "muzzle", "magazine", "grip", "stock"]
    );
}

#[test]
fn attachment_registry_rejects_invalid_data() {
    assert!(matches!(
        AttachmentRegistry::load(Path::new("/nonexistent/attachments.json")),
        Err(AttachmentLoadError::Io(_))
    ));
    assert!(matches!(
        AttachmentRegistry::from_json("{invalid"),
        Err(AttachmentLoadError::Parse(_))
    ));
    assert!(matches!(
        AttachmentRegistry::from_json("{}"),
        Err(AttachmentLoadError::Validation(_))
    ));
    let bad_mult = r#"{ "bad": { "name": "Bad", "effects": { "spreadMult": 0.0 } } }"#;
    assert!(matches!(
        AttachmentRegistry::from_json(bad_mult),
        Err(AttachmentLoadError::Validation(_))
    ));
}

#[test]
fn effective_stats_applies_each_attachment_effect() {
    let reg = AttachmentRegistry::from_repo().unwrap();
    let weapons = load_weapons();

    // scope_2x + suppressor on rifle: scope_fov=40, aim spread*0.75, hip spread unchanged, noise*0.35
    let rifle = weapons.get("rifle").unwrap();
    let rifle_2x = effective_stats(rifle, &["scope_2x".into()], &reg);
    assert_eq!(rifle_2x.scope.as_ref().map(|s| s.fov), Some(40.0));
    assert_eq!(rifle_2x.scope.as_ref().map(|s| s.sensitivity), Some(0.4));
    assert!((rifle_2x.spread_aim_deg - rifle.spread_aim_deg * 0.75).abs() < 1e-5);
    assert_eq!(rifle_2x.spread_hip_deg, rifle.spread_hip_deg);

    // scope_4x + suppressor on rifle
    let rifle_4x_sup = effective_stats(
        rifle,
        &["scope_4x".into(), "suppressor".into()],
        &reg,
    );
    assert_eq!(rifle_4x_sup.scope.as_ref().map(|s| s.fov), Some(20.0));
    assert!((rifle_4x_sup.spread_aim_deg - rifle.spread_aim_deg * 0.5).abs() < 1e-5);
    assert!((rifle_4x_sup.noise_radius - rifle.noise_radius * 0.35).abs() < 1e-4);

    // red_dot + ext_mag on pistol: both hip and aim spread tighten, no zoom scope, +50% mag (12 -> 18)
    let pistol = weapons.get("pistol").unwrap();
    let pistol_mod = effective_stats(pistol, &["red_dot".into(), "ext_mag".into()], &reg);
    assert_eq!(pistol_mod.scope, None);
    assert!((pistol_mod.spread_hip_deg - pistol.spread_hip_deg * 0.85).abs() < 1e-5);
    assert!((pistol_mod.spread_aim_deg - pistol.spread_aim_deg * 0.85).abs() < 1e-5);
    assert_eq!(pistol_mod.magazine_size, 18);

    // drum_mag on ak47: +150% mag (30 -> 75), reload * 1.3
    let ak47 = weapons.get("ak47").unwrap();
    let ak_drum = effective_stats(ak47, &["drum_mag".into()], &reg);
    assert_eq!(ak_drum.magazine_size, 75);
    assert!((ak_drum.reload_time - ak47.reload_time * 1.3).abs() < 1e-5);

    // bolt_quiver + scope_2x on crossbow: reload * 0.75, adds 2x scope
    let crossbow = weapons.get("crossbow").unwrap();
    let xbow_mod = effective_stats(
        crossbow,
        &["bolt_quiver".into(), "scope_2x".into()],
        &reg,
    );
    assert!((xbow_mod.reload_time - crossbow.reload_time * 0.75).abs() < 1e-5);
    assert_eq!(xbow_mod.scope.as_ref().map(|s| s.fov), Some(40.0));
    assert_eq!(xbow_mod.magazine_size, 0);
}

#[test]
fn stacking_mods_and_kick_mult() {
    let reg = AttachmentRegistry::from_repo().unwrap();
    let weapons = load_weapons();
    let ak47 = weapons.get("ak47").unwrap();

    // foregrip (0.7) + stock_pad (0.8) stack multiplicatively -> 0.56
    let mods = vec![
        "foregrip".to_string(),
        "stock_pad".to_string(),
        "suppressor".to_string(),
        "drum_mag".to_string(),
        "red_dot".to_string(),
    ];
    let km = kick_mult(&mods, "ak47", &reg);
    assert!((km - 0.56).abs() < 1e-5, "got {km}");

    let eff = effective_stats(ak47, &mods, &reg);
    assert!((eff.recoil_climb - ak47.recoil_climb * 0.56).abs() < 1e-5);
    assert!((eff.noise_radius - ak47.noise_radius * 0.35).abs() < 1e-4);
    assert_eq!(eff.magazine_size, 75);
    assert!((eff.spread_hip_deg - ak47.spread_hip_deg * 0.85).abs() < 1e-5);
}

#[test]
fn ignores_wrong_weapon_unknown_ids_and_duplicate_slots() {
    let reg = AttachmentRegistry::from_repo().unwrap();
    let weapons = load_weapons();
    let pistol = weapons.get("pistol").unwrap();

    assert!(!can_fit("scope_4x", "pistol", &reg));
    assert!(!can_fit("bolt_quiver", "pistol", &reg));
    assert!(!can_fit("nonexistent_mod", "pistol", &reg));

    // Wrong-weapon (scope_4x, foregrip, drum_mag) and unknown ids leave pistol stats untouched.
    let ignored = effective_stats(
        pistol,
        &[
            "scope_4x".into(),
            "foregrip".into(),
            "drum_mag".into(),
            "unknown_id".into(),
        ],
        &reg,
    );
    assert_eq!(ignored, *pistol);
    assert_eq!(
        kick_mult(&["foregrip".into(), "unknown_id".into()], "pistol", &reg),
        1.0
    );

    // At most one mod per AttachSlot: second optic / second magazine is ignored.
    let ak47 = weapons.get("ak47").unwrap();
    let dup_slots = effective_stats(
        ak47,
        &[
            "scope_2x".into(),
            "scope_4x".into(),
            "ext_mag".into(),
            "drum_mag".into(),
        ],
        &reg,
    );
    assert_eq!(dup_slots.scope.as_ref().map(|s| s.fov), Some(40.0));
    assert_eq!(dup_slots.magazine_size, 45);
    assert_eq!(dup_slots.reload_time, ak47.reload_time);

    // Verify AttachEffects default values match identity multipliers.
    let def_fx = AttachEffects::default();
    assert_eq!(def_fx.spread_mult, 1.0);
    assert_eq!(def_fx.kick_mult, 1.0);
    assert_eq!(def_fx.noise_mult, 1.0);
    assert_eq!(def_fx.mag_bonus, 0.0);
    assert_eq!(def_fx.reload_mult, 1.0);
    assert_eq!(def_fx.scope_fov, None);
}
