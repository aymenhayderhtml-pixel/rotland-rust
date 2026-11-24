use std::f32::consts::FRAC_PI_2;

use game::{DeathAnimConfig, DeathCause};
use glam::Vec3;

#[test]
fn config_from_repo_matches_default() {
    let cfg = DeathAnimConfig::from_repo().expect("load data/death_anim.json");
    assert_eq!(cfg, DeathAnimConfig::default());
}

#[test]
fn zombie_death_picks_opposite_fall_clip_for_each_side() {
    let cfg = DeathAnimConfig::default();
    let player = Vec3::ZERO;
    let yaw = 0.0; // facing -Z

    // Attacker in front (-Z) -> fall backward
    assert_eq!(
        cfg.pick(DeathCause::Zombie, player, yaw, Some(Vec3::new(0.0, 0.0, -2.0))).clip,
        "death_back"
    );
    // Attacker behind (+Z) -> fall forward
    assert_eq!(
        cfg.pick(DeathCause::Zombie, player, yaw, Some(Vec3::new(0.0, 0.0, 2.0))).clip,
        "death_forward"
    );
    // Attacker on left (-X) -> fall right
    assert_eq!(
        cfg.pick(DeathCause::Zombie, player, yaw, Some(Vec3::new(-2.0, 0.0, 0.0))).clip,
        "death_right"
    );
    // Attacker on right (+X) -> fall left
    assert_eq!(
        cfg.pick(DeathCause::Zombie, player, yaw, Some(Vec3::new(2.0, 0.0, 0.0))).clip,
        "death_left"
    );
}

#[test]
fn zombie_death_respects_player_yaw() {
    let cfg = DeathAnimConfig::default();
    let player = Vec3::ZERO;
    let yaw = -FRAC_PI_2; // facing +X

    assert_eq!(
        cfg.pick(DeathCause::Zombie, player, yaw, Some(Vec3::new(2.0, 0.0, 0.0))).clip,
        "death_back"
    );
    assert_eq!(
        cfg.pick(DeathCause::Zombie, player, yaw, Some(Vec3::new(-2.0, 0.0, 0.0))).clip,
        "death_forward"
    );
}

#[test]
fn drowned_and_other_or_unknown_causes_pick_expected_clips() {
    let cfg = DeathAnimConfig::default();
    let drowned = cfg.pick(DeathCause::Drowned, Vec3::ZERO, 0.0, None);
    assert_eq!(drowned.clip, "death_drown");
    assert_eq!(drowned.speed, 1.0);

    let other = cfg.pick(DeathCause::Other, Vec3::ZERO, 0.0, None);
    assert_eq!(other.clip, "die");
    assert_eq!(other.speed, 1.0);

    let unknown_zombie = cfg.pick(DeathCause::Zombie, Vec3::ZERO, 0.0, None);
    assert_eq!(unknown_zombie.clip, "die");

    let unknown_key = cfg.pick_cause_key("radiation");
    assert_eq!(unknown_key.clip, "die");
}
