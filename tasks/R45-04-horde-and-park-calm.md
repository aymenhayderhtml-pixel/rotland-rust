# R45-04: The stadium horde timetable and the park calm rule (pure logic, data)

Runs at the same time as R45-01, 02, 03. Read first: `tasks/R45-common.md` (whole). Style: `tasks/R41-02-street-herds.md` (the street herd logic this copies in spirit) and `tasks/R44-01-safehouse-logic.md` (a pure module with a JSON config).

Owns:
- `game_zombie/src/stadium_horde.rs` (NEW, pure): `StadiumHordeConfig`, `HordePhase`, `phase_at`, `HordeSplit`, `split_counts`, `gate_for_member`, `wants_real`, `may_despawn`.
- `game_zombie/src/stadium_horde_tests.rs` (NEW, `#[cfg(test)]`).
- `game_zombie/src/metro_spawn.rs` (ONLY: the `ParkCalm` struct, the `park_calm` field on `MetroSpawnConfig`, `metro_profile_near_park`).
- `game_zombie/src/metro_spawn_park_tests.rs` (NEW, `#[cfg(test)]`: tests for the park calm and the real spawning.json). Do NOT edit `metro_spawn_tests.rs`.
- `data/stadium_horde.json` (NEW).
- `data/spawning.json` (ONLY the new key `metro.parkCalm`; add it next to `placeDanger`, change nothing else).

Not owned: `game_zombie/src/lib.rs` (lead adds `pub mod stadium_horde;` and `#[cfg(test)] mod stadium_horde_tests;` and `#[cfg(test)] mod metro_spawn_park_tests;`), `game_zombie/src/street_herds.rs` (read only), `rotland/*` (the spawn glue is R45-05), `world/*` (R45-02/03).

Read (in this order):
- `game_zombie/src/metro_spawn.rs` (the whole file, ~200 lines): `ZoneSpawn`, `TypeWeight`, `PlaceDanger` (copy its serde style: `rename_all = "camelCase"`, `deny_unknown_fields`, `_why` as `Option<String>`), `MetroSpawnConfig`, `metro_profile`, `metro_profile_near_place` (the BOOST you mirror with a calm, not a copy of its numbers).
- `game_zombie/src/metro_spawn_tests.rs` (how the real `data/spawning.json` is loaded in a test, ~line 158).
- `game_zombie/src/street_herds.rs` (`HerdState`, `update_with_day`, the `activeRadius` / `despawnRadius` checks) and `world/src/metro/herds.rs` `HerdsConfig` (the `activeRadius` 170 and `despawnRadius` 210 are the herd numbers; the stadium horde uses its own numbers in its data file).
- `world/src/time.rs` (`Clock`, the hour as f32, 24 h day) to confirm the hour range.
- `data/spawning.json` (the `metro` block: `zones`, `nightMaxActiveMultiplier`, `placeDanger`).

## The stadium horde timetable (`game_zombie/src/stadium_horde.rs`)
Pure. The horde is a COUNT of `total` virtual members. Their position is decided by the phase. Real zombies are made elsewhere (R45-05), only for members near the player.

`data/stadium_horde.json` (create exactly these keys):
```json
{
  "_why": "R45: the stadium horde lives on the pitch by day, leaves through the gates at night and walks back at dawn. Hours are the game clock (0-24). Claude default 2026-11-28.",
  "total": 30,
  "insideHour": 7.0,
  "leaveStartHour": 20.0,
  "outsideHour": 22.0,
  "returnStartHour": 5.0,
  "activeRadiusM": 120.0,
  "despawnRadiusM": 150.0
}
```

- `StadiumHordeConfig` (serde camelCase, `from_json(&str) -> Result<Self, String>`, `from_repo()` reads `data/stadium_horde.json` with `paths::repo_path`, `OnceLock` cache; `from_json` validates: `total >= 1`; every hour in `[0, 24)`; `inside < leaveStart < outside`; `returnStart < inside`; `activeRadiusM > 0`; `despawnRadiusM >= activeRadiusM`).
- `HordePhase { Inside, Leaving, Outside, Returning }` with the hour windows:
  - `h = hour.rem_euclid(24)`.
  - `Leaving`: `leaveStart <= h < outside`.
  - `Outside`: `h >= outside` OR `h < returnStart` (wraps midnight).
  - `Returning`: `returnStart <= h < inside`.
  - `Inside`: everything else (`inside <= h < leaveStart`).
- `HordeSplit { inside, outside }` and `split_counts(hour, cfg)`:
  - Inside: all `total` inside. Outside: all `total` outside.
  - Leaving: `t = (h - leaveStart) / (outside - leaveStart)`; `outside = round(total * t)` clamped to `0..=total`; `inside = total - outside`.
  - Returning: `t = (h - returnStart) / (inside - returnStart)`; `inside = round(total * t)`; `outside = total - inside`.
  - Invariant: `inside + outside == total` for every hour.
- `gate_for_member(member, gates) -> usize`: `member % gates` (gates > 0; with `gates == 0` return 0).
- `wants_real(dist_m, cfg) -> bool`: `dist_m <= activeRadiusM`.
- `may_despawn(dist_m, chasing, cfg) -> bool`: `!chasing && dist_m > despawnRadiusM`.
- No randomness here. No Rust numbers except as `// data: stadium_horde.json <key>` tags (the tests may use literal hours and counts).

## The park calm rule (`game_zombie/src/metro_spawn.rs`)
- Add `ParkCalm { pub density_mult: f32, pub special_bonus: f32 }` (serde camelCase, `deny_unknown_fields`, `_why` optional like `PlaceDanger`).
- Add to `MetroSpawnConfig`: `#[serde(default)] pub park_calm: Option<ParkCalm>` (the file already has `deny_unknown_fields`, so the field is required in the struct for the key to load).
- `pub fn metro_profile_near_park(cfg: &MetroSpawnConfig, zone: MetroZone, is_night: bool, in_park: bool) -> MetroSpawnProfile`:
  - Not in the park, or no `park_calm`: return `metro_profile(cfg, zone, is_night)` unchanged.
  - In the park: `density *= densityMult`; `max_active = round(max_active * densityMult)` (the spawner's hard cap still applies); `special_chance_bonus = max(0, special_chance_bonus + specialBonus)`.
- `data/spawning.json` `metro.parkCalm`: `{ "_why": "R45: the wild park is calm: fewer and fewer-special zombies inside it. Claude default 2026-11-28.", "densityMult": 0.4, "specialBonus": -0.05 }`.
- Keep every other field of `data/spawning.json` byte-for-byte as it is (only the new key).

## Tests
`game_zombie/src/stadium_horde_tests.rs`:
- Hour 12: `split_counts` = (30, 0) and `phase_at` = Inside. Hour 23: (0, 30) and Outside. Hour 2 and 0: Outside. Hour 6: Returning, inside between 0 and 30 exclusive at 6.0? (at exactly `returnStart` it is 0 inside; check 6.0 is strictly between). Hour 21: Leaving with outside between 0 and 30.
- For every hour in steps of 0.25 over 0..24: `inside + outside == total`.
- Hour wrap: `phase_at(-1.0)` equals `phase_at(23.0)`; `phase_at(24.0)` equals `phase_at(0.0)`.
- Leaving is monotonic: outside never decreases from 20 to 22; Returning: inside never decreases from 5 to 7.
- `gate_for_member` cycles 0,1,2,3,0 for 4 gates; 0 gates gives 0.
- `wants_real`: 119.9 true, 120.0 true, 120.1 false (with the real config file loaded via `from_repo()`).
- `may_despawn`: chasing = never despawn; 151 m and not chasing = true; 149 m = false.
- `from_json` rejects: `total` 0; `inside` after `leaveStart`; an hour of 24.0.

`game_zombie/src/metro_spawn_park_tests.rs`:
- The real `data/spawning.json` loads with `park_calm` Some (`densityMult` 0.4, `specialBonus` -0.05).
- Inside the park: `density` is 0.4 of the street profile for each zone and both day and night (zone edge, midtown, core).
- Outside the park: `metro_profile_near_park(.., false)` equals `metro_profile(..)`.
- `max_active` inside is `round(street * 0.4)`; `special_chance_bonus` never negative.
- A spawning.json without `parkCalm` (built from the current file with the key removed in the test) still loads and gives the street profile in the park.
- The existing `metro_profile_near_place` numbers are unchanged (the R42 tests in `metro_spawn_tests.rs` keep passing).

## Number tags
Every number from a JSON file: `// data: stadium_horde.json <key>` or `// data: spawning.json metro.parkCalm <key>`. Code-only constants: `// new: DECISIONS 2026-11-28`.

## Done when
The pure module, the calm function, the two JSON edits and the tests are in place; the gate passes (`tools/gate.sh --task tasks/R45-04-horde-and-park-calm.md`). Fill `## Worker notes` with:
- Files changed.
- The `mod` / `pub use` lines for the lead (`game_zombie/src/lib.rs`: `pub mod stadium_horde;`, the two `#[cfg(test)]` test mods; `pub use stadium_horde::*` only if you think the wiring needs it).
- The real-zombie state the horde needs: check `game_zombie/src/zombie/state.rs` for an Idle state and say which one the inside members should use (`ZombieState::Idle` is expected).
- Any premise here that did not hold in the code.

## Worker notes
