//! Zombie vocalizations logic (port of scripts/Audio/ZombieVoices.cs).
//!
//! Evaluates live zombies periodically (every 0.5 s) to emit 3D sound events
//! (attack snarls and ambient groans) within audible range (25 m) of the player.
//! Uses a seeded DotnetRandom to match Godot's random groan timing, and caps
//! simultaneous voices to prevent overwhelming the audio mixer.

use std::collections::HashMap;

use glam::Vec3;
use world::DotnetRandom;

use super::types::{VoiceEvent, VoiceState, VoiceZombie};

/// Interval in seconds between zombie voice scans (ZombieVoices.ScanInterval).
pub const SCAN_INTERVAL: f32 = 0.5;
/// Minimum gap in seconds between subsequent groans for a zombie (ZombieVoices.MinGroanGap).
pub const MIN_GROAN_GAP: f32 = 6.0;
/// Maximum gap in seconds between subsequent groans for a zombie (ZombieVoices.MaxGroanGap).
pub const MAX_GROAN_GAP: f32 = 14.0;
/// Maximum distance in meters at which zombie vocalizations are audible (ZombieVoices.AudibleRange).
pub const AUDIBLE_RANGE: f32 = 25.0;
/// Squared audible distance threshold (ZombieVoices.AudibleRangeSq).
pub const AUDIBLE_RANGE_SQ: f32 = AUDIBLE_RANGE * AUDIBLE_RANGE;
/// Default maximum number of concurrent 3D spatial voices (AudioManager.SpatialVoices / VoicePool).
pub const MAX_VOICES: usize = 16;
/// Estimated playback duration in seconds for zombie_groan WAV.
pub const GROAN_DURATION: f32 = 1.6;
/// Estimated playback duration in seconds for zombie_attack WAV.
pub const ATTACK_DURATION: f32 = 0.9;

/// Per-zombie tracking record across voice scans.
#[derive(Clone, Debug, PartialEq)]
struct Record {
    last_state: VoiceState,
    next_groan_at: f32,
}

/// An actively playing voice tracked to enforce the concurrent voice cap.
#[derive(Clone, Debug, PartialEq)]
pub struct ActiveVoice {
    pub sound_id: String,
    pub position: Vec3,
    pub remaining: f32,
}

/// Periodic scanner and state machine for zombie groans and attack snarls.
pub struct ZombieVoices {
    rng: DotnetRandom,
    records: HashMap<u64, Record>,
    next_records: HashMap<u64, Record>,
    clock: f32,
    scan_timer: f32,
    max_voices: usize,
    active_voices: Vec<ActiveVoice>,
}

impl ZombieVoices {
    /// Creates a new `ZombieVoices` state machine with a given RNG seed.
    pub fn new(seed: i32) -> Self {
        Self::with_max_voices(seed, MAX_VOICES)
    }

    /// Creates a new `ZombieVoices` state machine with a specific max voice cap.
    pub fn with_max_voices(seed: i32, max_voices: usize) -> Self {
        Self {
            rng: DotnetRandom::new(seed),
            records: HashMap::new(),
            next_records: HashMap::new(),
            clock: 0.0,
            scan_timer: 0.0,
            max_voices,
            active_voices: Vec::new(),
        }
    }

    /// Advances the voice clock and scan timer, evaluating zombies when the scan interval elapses.
    ///
    /// Returns a list of `VoiceEvent`s to be played at specific 3D positions in the world.
    pub fn update(
        &mut self,
        dt: f32,
        player: Vec3,
        zombies: &[VoiceZombie],
    ) -> Vec<VoiceEvent> {
        self.clock += dt;
        self.scan_timer += dt;

        // Advance active voice playback timers
        for v in &mut self.active_voices {
            v.remaining -= dt;
        }
        self.active_voices.retain(|v| v.remaining > 0.0);

        if self.scan_timer < SCAN_INTERVAL {
            return Vec::new();
        }
        self.scan_timer = 0.0;
        self.scan(player, zombies)
    }

    /// Performs an immediate scan of the provided zombies without waiting for `SCAN_INTERVAL`.
    pub fn force_scan(&mut self, player: Vec3, zombies: &[VoiceZombie]) -> Vec<VoiceEvent> {
        self.scan_timer = 0.0;
        self.scan(player, zombies)
    }

    /// Scans the given zombies against the player's position (Godot ZombieVoices.Scan).
    fn scan(&mut self, player: Vec3, zombies: &[VoiceZombie]) -> Vec<VoiceEvent> {
        self.next_records.clear();
        let mut events = Vec::new();

        for zombie in zombies {
            // Dead zombies drop out of records and emit no sounds
            if zombie.state == VoiceState::Dead {
                continue;
            }

            let mut record = self
                .records
                .remove(&zombie.id)
                .unwrap_or_else(|| Record {
                    last_state: VoiceState::Idle,
                    next_groan_at: self.clock + Self::range(&mut self.rng, 1.0, MAX_GROAN_GAP),
                });

            let audible = zombie.position.distance_squared(player) <= AUDIBLE_RANGE_SQ;

            // Attack snarl: triggers upon entering VoiceState::Attack while in audible range
            if audible && zombie.state == VoiceState::Attack && record.last_state != VoiceState::Attack {
                events.push(VoiceEvent {
                    sound_id: "zombie_attack".to_string(),
                    position: zombie.position,
                });
            }
            record.last_state = zombie.state;

            // Ambient groan: triggers periodically when the scheduled groan time is reached
            if self.clock >= record.next_groan_at {
                record.next_groan_at =
                    self.clock + Self::range(&mut self.rng, MIN_GROAN_GAP, MAX_GROAN_GAP);
                if audible {
                    events.push(VoiceEvent {
                        sound_id: "zombie_groan".to_string(),
                        position: zombie.position,
                    });
                }
            }

            self.next_records.insert(zombie.id, record);
        }

        // Drop records of zombies that were killed, despawned, or are no longer in the list
        std::mem::swap(&mut self.records, &mut self.next_records);

        // Enforce the concurrent voice cap (prioritizing sounds closest to the player)
        if events.len() > self.max_voices {
            events.sort_by(|a, b| {
                let da = a.position.distance_squared(player);
                let db = b.position.distance_squared(player);
                da.total_cmp(&db)
            });
            events.truncate(self.max_voices);
        }

        // Register new events in the active voices pool (stealing oldest if capacity is exceeded)
        for ev in &events {
            let duration = if ev.sound_id == "zombie_attack" {
                ATTACK_DURATION
            } else {
                GROAN_DURATION
            };
            self.active_voices.push(ActiveVoice {
                sound_id: ev.sound_id.clone(),
                position: ev.position,
                remaining: duration,
            });
        }
        if self.active_voices.len() > self.max_voices {
            let drop_count = self.active_voices.len() - self.max_voices;
            self.active_voices.drain(0..drop_count);
        }

        events
    }

    /// Computes a random floating-point value in `[min, max)` matching C# `min + (float)_rng.NextDouble() * (max - min)`.
    fn range(rng: &mut DotnetRandom, min: f32, max: f32) -> f32 {
        min + (rng.next_double() as f32) * (max - min)
    }

    /// Current accumulated time on the zombie voice clock.
    pub fn clock(&self) -> f32 {
        self.clock
    }

    /// Current timer since the last scan.
    pub fn scan_timer(&self) -> f32 {
        self.scan_timer
    }

    /// Number of actively playing voice slots tracked.
    pub fn active_voices(&self) -> usize {
        self.active_voices.len()
    }

    /// Maximum concurrent voice cap.
    pub fn max_voices(&self) -> usize {
        self.max_voices
    }

    /// Sets a new maximum concurrent voice cap.
    pub fn set_max_voices(&mut self, max: usize) {
        self.max_voices = max;
    }

    /// Sets the voice clock manually (useful for testing).
    pub fn set_clock(&mut self, clock: f32) {
        self.clock = clock;
    }

    /// Sets the scan timer manually (useful for testing).
    pub fn set_scan_timer(&mut self, timer: f32) {
        self.scan_timer = timer;
    }

    /// Resets the RNG with a new seed.
    pub fn set_seed(&mut self, seed: i32) {
        self.rng = DotnetRandom::new(seed);
    }

    /// Number of live tracked zombie records.
    pub fn records_count(&self) -> usize {
        self.records.len()
    }

    /// Returns whether a record exists for the given zombie id.
    pub fn has_record(&self, id: u64) -> bool {
        self.records.contains_key(&id)
    }

    /// Returns the scheduled next groan time for a given zombie id, if tracked.
    pub fn record_next_groan_at(&self, id: u64) -> Option<f32> {
        self.records.get(&id).map(|r| r.next_groan_at)
    }
}

impl Default for ZombieVoices {
    fn default() -> Self {
        Self::new(12345)
    }
}
