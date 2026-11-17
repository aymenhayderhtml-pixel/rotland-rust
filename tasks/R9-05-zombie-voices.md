# R9-05: Zombie groans and shrieks

Read `tasks/R9-common.md` first. Reference: `scripts/Audio/ZombieVoices.cs`, `data/sounds.json` (the zombie sound ids), `scripts/Audio/VoicePool.cs`.

Write `rust/game/src/zombie_voices/` (logic only, no audio calls):
- `ZombieVoices::update(dt, player: Vec3, zombies: &[VoiceZombie {id, position, state: VoiceState (Idle/Wander/Chase/Attack/Dead)}]) -> Vec<VoiceEvent {sound_id: String, position: Vec3}>`: ports ZombieVoices.cs exactly: scan range, timers, which sound for which state, the cap on voices at once, random spacing from a seeded RNG (`world::DotnetRandom`).
- Make sure every sound id you emit exists in data/sounds.json (list them in your notes).
Tests (`rust/game/tests/zombie_voices_tests.rs`): nothing beyond the range; a chasing zombie in range groans after the traced time; the cap holds with many zombies; dead zombies are silent.

## Worker notes

### Files changed
- `rust/game/src/zombie_voices/types.rs` (created): `VoiceState`, `VoiceZombie`, `VoiceEvent`.
- `rust/game/src/zombie_voices/system.rs` (created): `ZombieVoices` state machine, constants, `ActiveVoice`.
- `rust/game/src/zombie_voices/mod.rs` (created): module re-exports.
- `rust/game/src/lib.rs` (modified): added `pub mod zombie_voices;` and `pub use zombie_voices::*;`.
- `rust/game/tests/zombie_voices_tests.rs` (created): integration tests covering all 4 task requirements plus state transition and sound validation.
- `tasks/R9-05-zombie-voices.md` (modified): added worker report.

### Public API
- `game::VoiceState`: enum with variants `Idle`, `Wander`, `Chase`, `Attack`, `Dead`. Derives `Clone, Copy, Debug, PartialEq, Eq, Hash`. Implements `From<crate::zombie::ZombieState>`.
- `game::VoiceZombie`: struct `{ pub id: u64, pub position: glam::Vec3, pub state: VoiceState }`. Derives `Clone, Copy, Debug, PartialEq`. Constructor `VoiceZombie::new(id, position, state)`.
- `game::VoiceEvent`: struct `{ pub sound_id: String, pub position: glam::Vec3 }`. Derives `Clone, Debug, PartialEq`. Constructor `VoiceEvent::new(sound_id, position)`.
- `game::ActiveVoice`: struct `{ pub sound_id: String, pub position: glam::Vec3, pub remaining: f32 }`. Derives `Clone, Debug, PartialEq`.
- `game::ZombieVoices`:
  - `pub fn new(seed: i32) -> Self`
  - `pub fn with_max_voices(seed: i32, max_voices: usize) -> Self`
  - `pub fn default() -> Self` (seed 12345)
  - `pub fn update(&mut self, dt: f32, player: Vec3, zombies: &[VoiceZombie]) -> Vec<VoiceEvent>`
  - `pub fn force_scan(&mut self, player: Vec3, zombies: &[VoiceZombie]) -> Vec<VoiceEvent>`
  - `pub fn clock(&self) -> f32`
  - `pub fn scan_timer(&self) -> f32`
  - `pub fn active_voices(&self) -> usize`
  - `pub fn max_voices(&self) -> usize`
  - `pub fn set_max_voices(&mut self, max: usize)`
  - `pub fn set_clock(&mut self, clock: f32)`
  - `pub fn set_scan_timer(&mut self, timer: f32)`
  - `pub fn set_seed(&mut self, seed: i32)`
  - `pub fn records_count(&self) -> usize`
  - `pub fn has_record(&self, id: u64) -> bool`
  - `pub fn record_next_groan_at(&self, id: u64) -> Option<f32>`
- Constants:
  - `SCAN_INTERVAL: f32 = 0.5` (from `ZombieVoices.cs:ScanInterval = 0.5f`)
  - `MIN_GROAN_GAP: f32 = 6.0` (from `ZombieVoices.cs:MinGroanGap = 6f`)
  - `MAX_GROAN_GAP: f32 = 14.0` (from `ZombieVoices.cs:MaxGroanGap = 14f`)
  - `AUDIBLE_RANGE: f32 = 25.0` (from `ZombieVoices.cs:AudibleRange = 25f`)
  - `AUDIBLE_RANGE_SQ: f32 = 625.0` (from `ZombieVoices.cs:AudibleRangeSq`)
  - `MAX_VOICES: usize = 16` (from `AudioManager.cs:SpatialVoices = 16` and `VoicePool.cs`)
  - `GROAN_DURATION: f32 = 1.6` (from `synth.py:zombie_groan`)
  - `ATTACK_DURATION: f32 = 0.9` (from `synth.py:zombie_attack`)

### Emitted Sound IDs in `data/sounds.json`
Every emitted sound ID exists in `data/sounds.json`:
1. `"zombie_attack"`: line 60 of `data/sounds.json` (`res://assets/audio/sfx/zombie_attack.wav`, volumeDb: -2, maxDistance: 30, spatial: true, bus: "SFX").
2. `"zombie_groan"`: line 48 of `data/sounds.json` (`res://assets/audio/sfx/zombie_groan_1..3.wav`, volumeDb: -4, maxDistance: 25, spatial: true, bus: "SFX").

### Traced test numbers
1. `test_nothing_beyond_the_range`:
   - Threshold `AUDIBLE_RANGE` = 25.0m; `AUDIBLE_RANGE_SQ` = 625.0.
   - Far attacking zombie at x = 30.0m (dist_sq = 900.0 > 625.0) and far chasing zombie at z = 50.0m (dist_sq = 2500.0 > 625.0) emit 0 events, even after 40 scans (20.0s).
   - Moving within audible range (x = 10.0m, dist_sq = 100.0 <= 625.0) emits 1 `"zombie_attack"` event.
2. `test_chasing_zombie_in_range_groans_after_traced_time`:
   - Seed: 42.
   - `DotnetRandom` call 1 `next_double()`: `0.6681064659115423` (`0.66810644_f32`).
   - Hand-traced initial groan delay: `Range(1.0, 14.0)` = `1.0 + 0.66810644 * 13.0 = 9.685384_f32`.
   - First scan at `dt = 0.5s`: `_clock` = 0.5s. Scheduled `next_groan_at = 0.5 + 9.685384 = 10.185384_f32`.
   - Steps 1..=20 (`t = 0.5s` to `10.0s`): `clock < 10.185384`, emits 0 events.
   - Step 21 at `dt = 0.5s` (`t = 10.5s`): `clock (10.5) >= 10.185384` is TRUE. Exactly 1 `"zombie_groan"` event is emitted at position `(5.0, 0.0, 0.0)`.
   - Next groan rescheduled with `Range(6.0, 14.0)`, verified to be >= `10.5 + 6.0` (16.5s).
3. `test_cap_holds_with_many_zombies`:
   - 100 zombies spawned within audible range (distances 1.0m to 16.0m), all entering `VoiceState::Attack`.
   - Emitted events length: exactly 16 (`MAX_VOICES`).
   - Active voices tracked: exactly 16 (`MAX_VOICES`).
   - Distance prioritization: closest zombies (x <= 4.0m) kept, distant ones truncated.
   - Custom cap test: `with_max_voices(42, 4)` with 100 zombies returns exactly 4 events.
4. `test_dead_zombies_are_silent`:
   - Zombie in `VoiceState::Dead` within range (1.0m) emits 0 events across 30 scans and has 0 records retained.
   - Active zombie killed (`VoiceState::Dead`) drops out of `records` on next scan and stays silent.
5. `test_sound_ids_exist_in_sounds_json`:
   - Validates `"zombie_groan"` and `"zombie_attack"` exist under `"sounds"` in `data/sounds.json`.
6. `test_attack_snarl_transitions`:
   - Start in `Idle`: 0 events.
   - Transition to `Attack`: 1 `"zombie_attack"` event.
   - Stays in `Attack` on next scan: 0 events (no re-trigger).
   - Transition to `Chase`, then back to `Attack`: 1 `"zombie_attack"` event.

### Wiring for Claude (Orchestrator)
In `rust/rotland/src/main.rs`:
1. In `GameApp` struct:
   Add field `zombie_voices: game::ZombieVoices,`
   Initialize in `GameApp::new(...)`:
   `zombie_voices: game::ZombieVoices::default(),`
2. Each frame in the gameplay update loop (near zombie AI updates around line 1440-1465):
   ```rust
   if let (Some(audio), Some(z)) = (self.audio.as_mut(), self.zombies.as_ref()) {
       let voice_zombies: Vec<game::VoiceZombie> = z.list.iter().map(|q| {
           let state = if q.health.is_dead() {
               game::VoiceState::Dead
           } else {
               q.brain.state().into()
           };
           game::VoiceZombie::new(q.id, q.position, state)
       }).collect();

       for ev in self.zombie_voices.update(dt, self.player, &voice_zombies) {
           audio.play_at(&ev.sound_id, ev.position);
       }
   }
   ```
