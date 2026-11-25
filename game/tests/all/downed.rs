use game::downed::{BANDAGE_ITEM, Downed, DownedConfig, DownedEvent, DownedPhase, ReviveBy};

#[test]
fn config_parses_and_loads_from_repo() {
    let cfg = DownedConfig::from_repo().expect("data/downed.json should load");
    assert_eq!(cfg, DownedConfig::default());
    assert_eq!(cfg.bleed_out_seconds, 30.0);
    assert_eq!(cfg.bite_seconds, 4.0);
    assert_eq!(cfg.teammate_range, 12.0);
    assert_eq!(cfg.revive_seconds, 4.0);
    assert_eq!(cfg.self_revive_seconds, 6.0);
    assert_eq!(cfg.revive_health, 25.0);
    assert_eq!(cfg.crawl_speed_mul, 0.35);
    assert_eq!(cfg.re_down_window_seconds, 60.0);
    assert_eq!(BANDAGE_ITEM, "bandage");
}

#[test]
fn bleed_out_transitions_to_dead_and_emits_bled_out() {
    let mut downed = Downed::default();
    assert!(downed.is_up());
    assert!(!downed.is_down());
    assert!(!downed.is_dead());
    assert_eq!(downed.phase(), DownedPhase::Up);
    assert_eq!(downed.bleed_remaining(), 0.0);
    assert_eq!(downed.revive_progress(), 0.0);
    assert_eq!(downed.revive_fraction(), 0.0);
    assert_eq!(downed.revive_by(), ReviveBy::None);

    let phase = downed.on_lethal_hit(10.0);
    assert!(matches!(
        phase,
        DownedPhase::Down { bleed, revive_progress, by: ReviveBy::None }
            if (bleed - 30.0).abs() < 1e-4 && revive_progress == 0.0
    ));
    assert!(downed.is_down());
    assert!(!downed.is_up());
    assert_eq!(downed.bleed_remaining(), 30.0);

    // Partial bleed-out emits None.
    assert_eq!(downed.tick(10.0, false, false, false), DownedEvent::None);
    assert!((downed.bleed_remaining() - 20.0).abs() < 1e-4);

    // Remaining 20s finishes bleed-out -> BledOut + Dead.
    let ev = downed.tick(20.0, false, false, false);
    assert_eq!(ev, DownedEvent::BledOut);
    assert!(!ev.is_revived());
    assert!(downed.is_dead());
    assert_eq!(downed.phase(), DownedPhase::Dead);
    assert_eq!(downed.bleed_remaining(), 0.0);

    // Further ticks while Dead emit None.
    assert_eq!(downed.tick(1.0, true, true, true), DownedEvent::None);
}

#[test]
fn zombie_bites_shorten_bleed_out_timer() {
    let mut downed = Downed::default();
    downed.on_lethal_hit(0.0);

    downed.bite();
    assert!((downed.bleed_remaining() - 26.0).abs() < 1e-4);

    // 7 more bites (8 * 4s = 32s >= 30s) bring bleed to 0.
    for _ in 0..7 {
        downed.bite();
    }
    assert_eq!(downed.bleed_remaining(), 0.0);

    let ev = downed.tick(0.016, false, false, false);
    assert_eq!(ev, DownedEvent::BledOut);
    assert!(downed.is_dead());
}

#[test]
fn teammate_revives_within_revive_seconds() {
    let mut downed = Downed::default();
    downed.on_lethal_hit(5.0);

    let ev = downed.tick(2.0, true, false, false);
    assert_eq!(ev, DownedEvent::None);
    assert_eq!(downed.revive_by(), ReviveBy::Teammate);
    assert!((downed.revive_progress() - 2.0).abs() < 1e-4);
    assert!((downed.revive_fraction() - 0.5).abs() < 1e-4);

    let ev = downed.tick(2.0, true, false, false);
    assert_eq!(ev, DownedEvent::Revived);
    assert!(ev.is_revived());
    assert!(downed.is_up());
    assert_eq!(downed.config().revive_health, 25.0);
}

#[test]
fn self_revive_requires_bandage_and_continuous_hold() {
    let mut downed = Downed::default();
    downed.on_lethal_hit(0.0);

    // Without a bandage, holding E makes no progress.
    let ev = downed.tick(6.0, false, true, false);
    assert_eq!(ev, DownedEvent::None);
    assert_eq!(downed.revive_by(), ReviveBy::None);
    assert_eq!(downed.revive_progress(), 0.0);
    assert!(downed.is_down());

    // With a bandage, progress advances.
    let ev = downed.tick(3.0, false, true, true);
    assert_eq!(ev, DownedEvent::None);
    assert_eq!(downed.revive_by(), ReviveBy::SelfBandage);
    assert!((downed.revive_fraction() - 0.5).abs() < 1e-4);

    // Releasing E interrupts and resets progress.
    let ev = downed.tick(0.5, false, false, true);
    assert_eq!(ev, DownedEvent::None);
    assert_eq!(downed.revive_by(), ReviveBy::None);
    assert_eq!(downed.revive_progress(), 0.0);

    // Holding E for full 6s completes and emits BandageUsed.
    assert_eq!(downed.tick(3.0, false, true, true), DownedEvent::None);
    let ev = downed.tick(3.0, false, true, true);
    assert_eq!(ev, DownedEvent::BandageUsed);
    assert!(ev.is_revived());
    assert!(downed.is_up());
}

#[test]
fn re_down_within_60_seconds_kills_immediately_while_after_60_allows_down() {
    // Within 60s of getting up -> immediate Dead.
    let mut downed = Downed::default();
    downed.on_lethal_hit(10.0);
    assert_eq!(downed.tick(4.0, true, false, false), DownedEvent::Revived);
    assert!(downed.is_up());

    // Revived at 14.0s; lethal hit at 73.9s (59.9s later < 60.0s) -> Dead.
    let phase = downed.on_lethal_hit(73.9);
    assert_eq!(phase, DownedPhase::Dead);
    assert!(downed.is_dead());

    // Reset clears re-down history (e.g. after respawn).
    downed.reset();
    assert!(downed.is_up());

    // At or after 60s of getting up -> knocked Down again.
    downed.on_lethal_hit(10.0);
    assert_eq!(downed.tick(4.0, true, false, false), DownedEvent::Revived);
    // Revived at 14.0s; lethal hit at 74.0s (60.0s later >= 60.0s) -> Down.
    let phase = downed.on_lethal_hit(74.0);
    assert!(matches!(phase, DownedPhase::Down { .. }));
    assert!(downed.is_down());
}
