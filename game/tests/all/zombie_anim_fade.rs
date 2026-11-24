use std::f32::consts::FRAC_PI_2;

use game::zombie::anim_fade::{
    AnimFade, ZombieAnimConfig, clip_key, death_clip, locomotion_rate,
};
use glam::Vec3;

#[test]
fn config_from_repo_matches_default() {
    let cfg = ZombieAnimConfig::from_repo().expect("data/zombie_anim.json should parse");
    assert_eq!(cfg, ZombieAnimConfig::default());
}

#[test]
fn same_key_does_not_fade() {
    let mut fade = AnimFade::default();
    let k = clip_key("zombie_shamble");
    fade.update(0.1, k, 1.0, 0.25);
    assert!(fade.blend().is_none());
    fade.update(0.1, k, 1.0, 0.25);
    assert!(fade.blend().is_none());
    assert!((fade.time() - 0.2).abs() < 1e-5);
}

#[test]
fn key_change_fades_from_zero_to_one_and_ends() {
    let mut fade = AnimFade::default();
    let walk = clip_key("walk");
    let attack = clip_key("zombie_attack");
    fade.update(0.5, walk, 1.0, 0.2);

    fade.update(0.0, attack, 1.0, 0.2);
    let (pk, pt, w0) = fade.blend().expect("fade should start");
    assert_eq!(pk, walk);
    assert!((pt - 0.5).abs() < 1e-5);
    assert!(w0.abs() < 1e-5);

    fade.update(0.1, attack, 1.0, 0.2);
    let (_, _, w_mid) = fade.blend().expect("mid-fade");
    assert!((w_mid - 0.5).abs() < 1e-5);

    fade.update(0.1, attack, 1.0, 0.2);
    assert!(fade.blend().is_none());
}

#[test]
fn fade_during_fade_uses_dominant_clip_and_finishes() {
    let mut fade = AnimFade::default();
    let a = clip_key("idle");
    let b = clip_key("walk");
    let c = clip_key("zombie_attack");

    fade.update(0.3, a, 1.0, 0.2);
    fade.update(0.05, b, 1.0, 0.2); // weight 0.25 -> dominant is a
    fade.update(0.0, c, 1.0, 0.1);
    let (pk, _, w) = fade.blend().expect("re-fade active");
    assert_eq!(pk, a);
    assert!(w.is_finite());

    fade.update(0.1, c, 1.0, 0.1);
    assert!(fade.blend().is_none());
}

#[test]
fn time_advances_by_rate() {
    let mut fade = AnimFade::default();
    let k = clip_key("run");
    fade.update(0.2, k, 1.5, 0.2);
    fade.update(0.2, k, 0.5, 0.2);
    assert!((fade.time() - 0.4).abs() < 1e-5);
}

#[test]
fn locomotion_rate_clamps_to_config() {
    let cfg = ZombieAnimConfig::default();
    assert!((locomotion_rate(0.1, 1.4, &cfg) - cfg.min_rate).abs() < 1e-5);
    assert!((locomotion_rate(10.0, 1.4, &cfg) - cfg.max_rate).abs() < 1e-5);
    assert!((locomotion_rate(1.4, 1.4, &cfg) - 1.0).abs() < 1e-5);
}

#[test]
fn death_clip_directions_yaw_zero_and_half_pi() {
    // yaw = 0 -> forward = (0, 0, -1)
    assert_eq!(death_clip(Vec3::new(0.0, 0.0, 1.0), 0.0), "zombie_die_back");
    assert_eq!(death_clip(Vec3::new(0.0, 0.0, -1.0), 0.0), "zombie_die_forward");
    assert_eq!(death_clip(Vec3::new(1.0, 0.0, 0.0), 0.0), "die");
    assert_eq!(death_clip(Vec3::ZERO, 0.0), "die");

    // yaw = PI/2 -> forward = (-1, 0, 0)
    assert_eq!(death_clip(Vec3::new(1.0, 0.0, 0.0), FRAC_PI_2), "zombie_die_back");
    assert_eq!(death_clip(Vec3::new(-1.0, 0.0, 0.0), FRAC_PI_2), "zombie_die_forward");
    assert_eq!(death_clip(Vec3::new(0.0, 0.0, 1.0), FRAC_PI_2), "die");
}

#[test]
fn clip_key_stable_and_distinct() {
    assert_eq!(clip_key("zombie_shamble"), clip_key("zombie_shamble"));
    assert_ne!(clip_key("zombie_shamble"), clip_key("zombie_lurch"));
    assert_ne!(clip_key("die"), clip_key("zombie_die_back"));
}

#[test]
fn finite_at_zero_and_large_dt() {
    let mut fade = AnimFade::default();
    let a = clip_key("walk");
    let b = clip_key("zombie_stagger");
    fade.update(0.0, a, 1.0, 0.2);
    assert!(fade.time().is_finite());
    fade.update(0.25, b, 1.2, 0.08);
    assert!(fade.time().is_finite());
    if let Some((_, pt, w)) = fade.blend() {
        assert!(pt.is_finite() && w.is_finite());
    }
}
