//! Integration tests for zombie vocalizations (ports of scripts/Audio/ZombieVoices.cs).
//!
//! Tests:
//! 1. Nothing beyond the audible range (25m) emits sound.
//! 2. A chasing zombie in range groans after the hand-traced time from DotnetRandom seed 42.
//! 3. The concurrent voice cap holds when many zombies attack at once.
//! 4. Dead zombies are completely silent and drop out of tracking.
//! 5. Emitted sound IDs exist in data/sounds.json.
//! 6. Attack snarls trigger only on state entry into VoiceState::Attack.

use game::{
    VoiceState, VoiceZombie, ZombieVoices, AUDIBLE_RANGE, AUDIBLE_RANGE_SQ,
    MAX_VOICES, SCAN_INTERVAL,
};
use glam::Vec3;

#[test]
fn test_nothing_beyond_the_range() {
    let mut voices = ZombieVoices::new(42);
    let player = Vec3::ZERO;

    // Zombie 1: at distance 30m (> 25m AUDIBLE_RANGE), entering Attack
    let far_attacker = VoiceZombie::new(1, Vec3::new(30.0, 0.0, 0.0), VoiceState::Attack);
    // Zombie 2: at distance 50m (> 25m), chasing
    let far_chaser = VoiceZombie::new(2, Vec3::new(0.0, 0.0, 50.0), VoiceState::Chase);

    let events = voices.update(SCAN_INTERVAL, player, &[far_attacker, far_chaser]);
    assert!(
        events.is_empty(),
        "Zombies beyond audible range (25m) must emit no sound events"
    );

    // Advance clock past any initial groan delay (e.g. 20s)
    let mut total_far_events = 0;
    for _ in 0..40 {
        let evs = voices.update(SCAN_INTERVAL, player, &[far_attacker, far_chaser]);
        total_far_events += evs.len();
    }
    assert_eq!(
        total_far_events, 0,
        "Far zombies must remain completely silent even when groan timers elapse"
    );

    // Verify distance squared threshold constant
    assert_eq!(AUDIBLE_RANGE, 25.0);
    assert_eq!(AUDIBLE_RANGE_SQ, 625.0);

    // When a zombie is brought within audible range (e.g. 10m), it can be heard
    let near_attacker = VoiceZombie::new(10, Vec3::new(10.0, 0.0, 0.0), VoiceState::Attack);
    let near_events = voices.update(SCAN_INTERVAL, player, &[near_attacker]);
    assert_eq!(
        near_events.len(),
        1,
        "Attacking zombie inside range must emit sound"
    );
    assert_eq!(near_events[0].sound_id, "zombie_attack");
    assert_eq!(near_events[0].position, Vec3::new(10.0, 0.0, 0.0));
}

#[test]
fn test_chasing_zombie_in_range_groans_after_traced_time() {
    // Traced numbers with DotnetRandom::new(42):
    // - Call 1 next_double(): 0.6681064659115423 (from DotnetRandom Knuth subtractive PRNG)
    // - Cast to f32: 0.66810644_f32
    // - Range(1.0, 14.0): 1.0 + 0.66810644 * (14.0 - 1.0) = 1.0 + 8.6853838 = 9.685384_f32
    // - Step 1 at dt = 0.5s: clock = 0.5s, next_groan_at = 0.5 + 9.685384 = 10.185384_f32
    // - During steps 1..=20 (clock = 0.5s .. 10.0s): clock < next_groan_at (10.185384) -> 0 groans
    // - Step 21 at dt = 0.5s: clock = 10.5s -> clock (10.5) >= 10.185384 is TRUE -> exactly 1 groan!
    let mut voices = ZombieVoices::new(42);
    let player = Vec3::ZERO;
    let chaser = VoiceZombie::new(1, Vec3::new(5.0, 0.0, 0.0), VoiceState::Chase);

    // First scan at t = 0.5s
    let ev1 = voices.update(SCAN_INTERVAL, player, &[chaser]);
    assert!(ev1.is_empty(), "First scan should not groan immediately");
    assert!(voices.has_record(1), "Zombie record must be created");

    let scheduled = voices
        .record_next_groan_at(1)
        .expect("Record must have scheduled next groan");
    assert!(
        (scheduled - 10.185384).abs() < 1e-3,
        "Expected traced next_groan_at around 10.185s, got {scheduled}"
    );

    // Steps 2 to 20: clock goes from 1.0s to 10.0s (all < 10.185384)
    for step in 2..=20 {
        let evs = voices.update(SCAN_INTERVAL, player, &[chaser]);
        assert!(
            evs.is_empty(),
            "Step {step} (t = {:.1}s) should be silent before 10.185s",
            voices.clock()
        );
    }
    assert!((voices.clock() - 10.0).abs() < 1e-4);

    // Step 21: clock reaches 10.5s >= 10.185384 -> groan fires!
    let ev21 = voices.update(SCAN_INTERVAL, player, &[chaser]);
    assert_eq!(
        ev21.len(),
        1,
        "Step 21 (t = 10.5s) must emit the scheduled groan"
    );
    assert_eq!(ev21[0].sound_id, "zombie_groan");
    assert_eq!(ev21[0].position, Vec3::new(5.0, 0.0, 0.0));

    // Next scheduled groan was rolled with Range(6.0, 14.0):
    let next_scheduled = voices
        .record_next_groan_at(1)
        .expect("Record must have next groan");
    assert!(
        next_scheduled >= 10.5 + 6.0,
        "Subsequent groan must be at least 6s away (MinGroanGap)"
    );
}

#[test]
fn test_cap_holds_with_many_zombies() {
    let mut voices = ZombieVoices::new(42);
    let player = Vec3::ZERO;

    // Spawn 100 zombies within audible range, all entering Attack state simultaneously
    let zombies: Vec<VoiceZombie> = (0..100)
        .map(|i| {
            let dist = 1.0 + (i as f32) * 0.15; // all between 1.0m and 16.0m (< 25m)
            VoiceZombie::new(i, Vec3::new(dist, 0.0, 0.0), VoiceState::Attack)
        })
        .collect();

    let events = voices.update(SCAN_INTERVAL, player, &zombies);

    // Verify the concurrent voice cap holds
    assert_eq!(
        events.len(),
        MAX_VOICES,
        "Emitted events must be capped at MAX_VOICES ({MAX_VOICES})"
    );
    assert_eq!(
        voices.active_voices(),
        MAX_VOICES,
        "Active voices tracked must not exceed MAX_VOICES ({MAX_VOICES})"
    );

    // Closest zombies must be prioritized
    for ev in &events {
        assert_eq!(ev.sound_id, "zombie_attack");
        // All capped events should be from the closest zombies (x <= 1.0 + 16 * 0.15 = 3.4m)
        assert!(
            ev.position.x <= 4.0,
            "Capped voices must prioritize closest zombies to player"
        );
    }

    // Verify custom cap configuration
    let mut small_cap_voices = ZombieVoices::with_max_voices(42, 4);
    let small_events = small_cap_voices.update(SCAN_INTERVAL, player, &zombies);
    assert_eq!(
        small_events.len(),
        4,
        "Custom cap of 4 must limit output events to 4"
    );
    assert_eq!(small_cap_voices.active_voices(), 4);
}

#[test]
fn test_dead_zombies_are_silent() {
    let mut voices = ZombieVoices::new(42);
    let player = Vec3::ZERO;

    // Dead zombie right in front of the player
    let dead_zombie = VoiceZombie::new(99, Vec3::new(1.0, 0.0, 0.0), VoiceState::Dead);

    for _ in 0..30 {
        let evs = voices.update(SCAN_INTERVAL, player, &[dead_zombie]);
        assert!(evs.is_empty(), "Dead zombies must emit no sounds");
    }
    assert_eq!(
        voices.records_count(),
        0,
        "Dead zombies must not be kept in records"
    );

    // Zombie that was previously alive then dies
    let mut zombie = VoiceZombie::new(100, Vec3::new(2.0, 0.0, 0.0), VoiceState::Idle);
    let _ = voices.update(SCAN_INTERVAL, player, &[zombie]);
    assert!(voices.has_record(100), "Alive zombie was tracked");

    // Zombie dies: should drop out of records on next scan
    zombie.state = VoiceState::Dead;
    let dead_events = voices.update(SCAN_INTERVAL, player, &[zombie]);
    assert!(dead_events.is_empty(), "Killed zombie must be silent");
    assert!(
        !voices.has_record(100),
        "Killed zombie must drop out of records"
    );
}

#[test]
fn test_sound_ids_exist_in_sounds_json() {
    let repo_sounds_path = paths::repo_path("data/sounds.json");
    let content = std::fs::read_to_string(&repo_sounds_path)
        .expect("data/sounds.json must exist in repo");
    let json: serde_json::Value =
        serde_json::from_str(&content).expect("data/sounds.json must be valid JSON");

    let sounds = json
        .get("sounds")
        .expect("data/sounds.json must contain 'sounds' object");

    // Both sound IDs emitted by ZombieVoices must exist in data/sounds.json
    assert!(
        sounds.get("zombie_groan").is_some(),
        "'zombie_groan' must exist in data/sounds.json"
    );
    assert!(
        sounds.get("zombie_attack").is_some(),
        "'zombie_attack' must exist in data/sounds.json"
    );
}

#[test]
fn test_attack_snarl_transitions() {
    let mut voices = ZombieVoices::new(42);
    let player = Vec3::ZERO;

    let mut zombie = VoiceZombie::new(5, Vec3::new(3.0, 0.0, 0.0), VoiceState::Idle);

    // Scan 1: Idle state
    let ev1 = voices.update(SCAN_INTERVAL, player, &[zombie]);
    assert!(ev1.is_empty(), "Idle start produces no attack snarl");

    // Scan 2: Transitions to Attack -> emits zombie_attack
    zombie.state = VoiceState::Attack;
    let ev2 = voices.update(SCAN_INTERVAL, player, &[zombie]);
    assert_eq!(ev2.len(), 1);
    assert_eq!(ev2[0].sound_id, "zombie_attack");

    // Scan 3: Stays in Attack -> does NOT re-emit zombie_attack
    let ev3 = voices.update(SCAN_INTERVAL, player, &[zombie]);
    let attack_sounds_in_ev3 = ev3.iter().filter(|e| e.sound_id == "zombie_attack").count();
    assert_eq!(
        attack_sounds_in_ev3, 0,
        "Staying in Attack state must not repeat attack snarl"
    );

    // Scan 4: Transitions to Chase
    zombie.state = VoiceState::Chase;
    let ev4 = voices.update(SCAN_INTERVAL, player, &[zombie]);
    let attack_sounds_in_ev4 = ev4.iter().filter(|e| e.sound_id == "zombie_attack").count();
    assert_eq!(attack_sounds_in_ev4, 0);

    // Scan 5: Re-enters Attack -> emits zombie_attack again
    zombie.state = VoiceState::Attack;
    let ev5 = voices.update(SCAN_INTERVAL, player, &[zombie]);
    assert_eq!(ev5.len(), 1);
    assert_eq!(ev5[0].sound_id, "zombie_attack");
}
