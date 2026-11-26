use game::gun_feel::{GunFeelConfig, GunFeelState};

#[test]
fn loads_all_nine_weapons_and_defaults() {
    let cfg = GunFeelConfig::load().expect("data/gun_feel.json should load");

    assert_eq!(cfg.defaults.recover_deg_per_sec, 12.0);
    assert_eq!(cfg.defaults.fp_back, 0.06);
    assert_eq!(cfg.defaults.fp_tilt_deg, 4.0);
    assert_eq!(cfg.defaults.flash_light, 4.0);
    assert_eq!(cfg.defaults.aim_mul, 0.6);
    assert_eq!(cfg.defaults.kick_yaw_deg, 0.4);
    assert_eq!(cfg.defaults.fp_recover_sec, 0.12);
    assert_eq!(cfg.defaults.flash_duration_sec, 0.05);

    let expected = [
        ("pistol", 2.5, false, 0.0, false),
        ("revolver", 5.0, false, 0.0, false),
        ("rifle", 6.0, false, 0.0, false),
        ("hunting_rifle", 7.0, false, 0.0, false),
        ("shotgun", 8.0, false, 0.0, false),
        ("ak47", 1.4, true, 9.0, false),
        ("smg", 0.9, true, 7.0, false),
        ("bow", 0.6, false, 0.0, true),
        ("crossbow", 1.5, false, 0.0, true),
    ];

    for (id, kick_deg, climb, max_climb_deg, no_flash) in expected {
        let def = cfg
            .get(id)
            .unwrap_or_else(|| panic!("missing weapon {id} in gun_feel.json"));
        assert_eq!(def.kick_deg, kick_deg, "{id} kick_deg");
        assert_eq!(def.climb, climb, "{id} climb");
        assert_eq!(def.max_climb_deg, max_climb_deg, "{id} max_climb_deg");
        assert_eq!(def.no_flash, no_flash, "{id} no_flash");
        assert_eq!(def.aim_mul, 0.6, "{id} aim_mul");
        assert_eq!(def.recover_deg_per_sec, 12.0, "{id} recover_deg_per_sec");
    }
}

#[test]
fn shot_applies_view_and_fp_kick_and_aim_multiplier() {
    let cfg = GunFeelConfig::load().unwrap();
    let pistol = cfg.get("pistol").unwrap();

    let mut hip = GunFeelState::default();
    hip.on_shot(pistol, false);
    let (hip_pitch, hip_yaw) = hip.view_kick_deg();
    assert!((hip_pitch - 2.5).abs() < 1e-5);
    assert!(hip_yaw.abs() <= pistol.kick_yaw_deg + 1e-5);
    let (hip_back, hip_tilt) = hip.fp_kick();
    assert!((hip_back - 0.06).abs() < 1e-5);
    assert!((hip_tilt - 4.0).abs() < 1e-5);

    let mut aimed = GunFeelState::default();
    aimed.on_shot(pistol, true);
    let (aim_pitch, aim_yaw) = aimed.view_kick_deg();
    assert!((aim_pitch - 2.5 * 0.6).abs() < 1e-5);
    assert!(aim_yaw.abs() <= pistol.kick_yaw_deg * 0.6 + 1e-5);
    let (aim_back, aim_tilt) = aimed.fp_kick();
    assert!((aim_back - 0.06 * 0.6).abs() < 1e-5);
    assert!((aim_tilt - 4.0 * 0.6).abs() < 1e-5);
}

#[test]
fn automatic_climb_clamps_to_max_climb_deg() {
    let cfg = GunFeelConfig::load().unwrap();
    let ak47 = cfg.get("ak47").unwrap();
    let smg = cfg.get("smg").unwrap();

    let mut ak_state = GunFeelState::default();
    for _ in 0..20 {
        ak_state.on_shot(ak47, false);
    }
    assert!((ak_state.climb_deg - 9.0).abs() < 1e-5);
    assert!((ak_state.view_kick_deg().0 - 9.0).abs() < 1e-5);

    let mut smg_state = GunFeelState::default();
    for _ in 0..20 {
        smg_state.on_shot(smg, false);
    }
    assert!((smg_state.climb_deg - 7.0).abs() < 1e-5);
    assert!((smg_state.view_kick_deg().0 - 7.0).abs() < 1e-5);
}

#[test]
fn view_kick_and_fp_kick_recover_over_dt() {
    let cfg = GunFeelConfig::load().unwrap();
    let revolver = cfg.get("revolver").unwrap();

    let mut state = GunFeelState::default();
    state.on_shot(revolver, false);
    assert!((state.view_kick_deg().0 - 5.0).abs() < 1e-5);

    // After 0.12 s at 12 deg/s: 5.0 - 1.44 = 3.56 deg; FP kick decayed > 95%.
    state.update(0.12, Some(revolver));
    let (pitch_mid, _) = state.view_kick_deg();
    assert!((pitch_mid - 3.56).abs() < 1e-4, "got {pitch_mid}");
    let (back_mid, tilt_mid) = state.fp_kick();
    assert!(back_mid < 0.005, "expected fp_back mostly recovered, got {back_mid}");
    assert!(tilt_mid < 0.2, "expected fp_tilt mostly recovered, got {tilt_mid}");

    // After another 1.0 s with no def (uses defaults), everything reaches 0.
    state.update(1.0, None);
    assert_eq!(state.view_kick_deg(), (0.0, 0.0));
    assert_eq!(state.fp_kick(), (0.0, 0.0));
}

#[test]
fn bow_and_crossbow_have_no_flash_and_small_kick() {
    let cfg = GunFeelConfig::load().unwrap();
    let bow = cfg.get("bow").unwrap();
    let crossbow = cfg.get("crossbow").unwrap();

    assert!(bow.no_flash);
    assert!(crossbow.no_flash);
    assert_eq!(bow.kick_deg, 0.6);
    assert_eq!(crossbow.kick_deg, 1.5);
}
