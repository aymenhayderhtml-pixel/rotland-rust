use game::weapons::{WeaponAnim, WeaponDef, WeaponKind};
use game::{MeleeComboState, MeleeMovesConfig};

const INLINE_CFG: &str = r#"{
  "version": 1,
  "bufferSeconds": 0.25,
  "minClipSpeed": 0.6,
  "maxClipSpeed": 2.2,
  "weapons": { "bat": "twoHand", "hammer": "heavy" },
  "classes": {
    "twoHand": {
      "comboWindow": 0.5,
      "missRecover": 0.2,
      "steps": [
        { "clip": "c0", "impact": 0.40, "windupScale": 1.0, "cooldownScale": 0.9, "damageScale": 1.0, "knockbackScale": 1.0, "arcScale": 1.0 },
        { "clip": "c1", "impact": 0.30, "windupScale": 1.0, "cooldownScale": 0.9, "damageScale": 1.1, "knockbackScale": 1.2, "arcScale": 1.0 },
        { "clip": "c2", "impact": 0.50, "windupScale": 1.5, "cooldownScale": 1.1, "damageScale": 1.3, "knockbackScale": 1.5, "arcScale": 0.8 }
      ]
    },
    "heavy": {
      "comboWindow": 0.6,
      "missRecover": 0.45,
      "steps": [
        { "clip": "h0", "impact": 0.50, "windupScale": 2.0, "cooldownScale": 1.0, "damageScale": 1.0, "knockbackScale": 1.4, "arcScale": 0.8 },
        { "clip": "h1", "impact": 0.35, "windupScale": 0.1, "cooldownScale": 1.0, "damageScale": 1.3, "knockbackScale": 1.6, "arcScale": 1.2 }
      ]
    }
  }
}"#;

fn make_weapon(id: &str, windup: f32) -> WeaponDef {
    WeaponDef {
        id: id.to_string(),
        name: id.to_string(),
        kind: WeaponKind::Melee,
        damage: 20.0,       // new: DECISIONS 2026-11-24
        range: 2.0,         // new: DECISIONS 2026-11-24
        arc_deg: 90.0,      // new: DECISIONS 2026-11-24
        windup,
        cooldown: 0.6,      // new: DECISIONS 2026-11-24
        stamina_cost: 12.0, // new: DECISIONS 2026-11-24
        knockback: 3.0,     // new: DECISIONS 2026-11-24
        anim: WeaponAnim {
            attack: Some("melee_1h".to_string()),
            ..WeaponAnim::default()
        },
        ..WeaponDef::default()
    }
}

fn cfg() -> MeleeMovesConfig {
    MeleeMovesConfig::from_json(INLINE_CFG).expect("inline melee_moves json")
}

#[test]
fn repo_melee_moves_json_is_valid() {
    let repo_cfg = MeleeMovesConfig::from_repo().expect("load data/melee_moves.json");
    assert_eq!(repo_cfg.version, 1);
    assert!(!repo_cfg.classes.is_empty());
    for (name, class) in &repo_cfg.classes {
        assert!(
            (2..=3).contains(&class.steps.len()),
            "class {name} has {} steps",
            class.steps.len()
        );
    }
    for (weapon, class_name) in &repo_cfg.weapons {
        assert!(
            repo_cfg.classes.contains_key(class_name),
            "weapon {weapon} maps to unknown class {class_name}"
        );
    }
}

#[test]
fn unknown_weapon_falls_back_unscaled() {
    let c = cfg();
    let w = make_weapon("frying_pan", 0.2);
    let state = MeleeComboState::default();
    let mv = state.next_move(&w, &c);
    assert_eq!(mv.clip, "melee_1h");
    assert!((mv.clip_speed - 1.0).abs() < 1e-5);
    assert_eq!(mv.step, 0);
    assert!((mv.weapon.damage - w.damage).abs() < 1e-5);
    assert!((mv.weapon.windup - w.windup).abs() < 1e-5);
    assert!((mv.weapon.range - w.range).abs() < 1e-5);
    assert!((mv.weapon.stamina_cost - w.stamina_cost).abs() < 1e-5);
}

#[test]
fn combo_advances_wraps_and_resets_after_window() {
    let c = cfg();
    let bat = make_weapon("bat", 0.2);
    let mut s = MeleeComboState::default();

    let m0 = s.next_move(&bat, &c);
    assert_eq!((m0.step, m0.clip.as_str()), (0, "c0"));
    s.on_started(&m0, &c);
    s.on_resolved(true, &bat.id, &c);

    s.tick(0.2);
    let m1 = s.next_move(&bat, &c);
    assert_eq!((m1.step, m1.clip.as_str()), (1, "c1"));
    s.on_started(&m1, &c);
    s.on_resolved(true, &bat.id, &c);

    s.tick(0.2);
    let m2 = s.next_move(&bat, &c);
    assert_eq!((m2.step, m2.clip.as_str()), (2, "c2"));
    assert!((m2.weapon.damage - 26.0).abs() < 1e-4);
    assert!((m2.weapon.knockback - 4.5).abs() < 1e-4);
    assert!((m2.weapon.range - bat.range).abs() < 1e-5);
    assert!((m2.weapon.stamina_cost - bat.stamina_cost).abs() < 1e-5);
    s.on_started(&m2, &c);
    s.on_resolved(true, &bat.id, &c);

    // Wraps to step 0 after the last step.
    s.tick(0.1);
    let m3 = s.next_move(&bat, &c);
    assert_eq!(m3.step, 0);
    s.on_started(&m3, &c);
    s.on_resolved(true, &bat.id, &c);

    // Letting the combo window expire resets to step 0.
    s.tick(0.6);
    assert_eq!(s.next_move(&bat, &c).step, 0);
}

#[test]
fn input_buffer_fires_when_idle_and_expires() {
    let c = cfg();
    let mut s = MeleeComboState::default();

    // Click while not idle is buffered.
    assert!(!s.wants_swing(true, false, &c));
    s.tick(0.1);
    assert!(s.wants_swing(false, true, &c));

    // Expire buffer if idle comes too late.
    let mut s2 = MeleeComboState::default();
    assert!(!s2.wants_swing(true, false, &c));
    s2.tick(0.3);
    assert!(!s2.wants_swing(false, true, &c));
}

#[test]
fn miss_recover_blocks_only_on_miss() {
    let c = cfg();
    let hammer = make_weapon("hammer", 0.5);
    let mut s = MeleeComboState::default();

    let mv = s.next_move(&hammer, &c);
    s.on_started(&mv, &c);
    s.on_resolved(false, &hammer.id, &c);
    assert!(!s.wants_swing(true, true, &c));
    s.tick(0.2);
    assert!(!s.wants_swing(false, true, &c));
    s.tick(0.3);
    assert!(s.wants_swing(true, true, &c));

    // A hit does not start miss recovery.
    let mut s_hit = MeleeComboState::default();
    s_hit.on_started(&mv, &c);
    s_hit.on_resolved(true, &hammer.id, &c);
    assert!(s_hit.wants_swing(true, true, &c));
}

#[test]
fn clip_speed_clamps_both_ends_and_reset_clears_state() {
    let c = cfg();
    let hammer = make_weapon("hammer", 0.5);
    let mut s = MeleeComboState::default();

    // Step 0: impact 0.50 / (0.5 * 2.0) = 0.50 -> clamped to minClipSpeed 0.6.
    let m0 = s.next_move(&hammer, &c);
    assert!((m0.clip_speed - 0.6).abs() < 1e-5);
    s.on_started(&m0, &c);
    s.on_resolved(true, &hammer.id, &c);

    // Step 1: impact 0.35 / (0.5 * 0.1) = 7.0 -> clamped to maxClipSpeed 2.2.
    let m1 = s.next_move(&hammer, &c);
    assert!((m1.clip_speed - 2.2).abs() < 1e-5);

    // Buffer a click, then reset: step returns to 0 and buffer is cleared.
    assert!(!s.wants_swing(true, false, &c));
    s.reset();
    assert_eq!(s.next_move(&hammer, &c).step, 0);
    assert!(!s.wants_swing(false, true, &c));
}
