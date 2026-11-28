# R44-01: Safe house logic, chase check, data (pure, no models)

Runs at the same time as R44-02, 03, 04. Read first: `tasks/R44-common.md` (whole). Style: `tasks/R43-01-climb-plan.md`.

Owns:
- `game_world/src/safe_house.rs` (NEW, pure): `SafeHouseConfig`, `SafeHousePrompts`, `SafeHouseState`, `travel_blocked`.
- `game_zombie/src/chasing.rs` (NEW, pure): `count_chasing_near`.
- `data/safe_house.json` (NEW, exact content in `R44-common.md`, "Data file").
- `game/tests/all/safe_house.rs` (NEW) and nothing else in tests. The lead adds `mod safe_house;` to `game/tests/all/main.rs`.

Not owned: anything in `world/`, `rotland/`, `rotland_ui/`, `tools/`, `assets/`, `game_camp/`. Do not edit `game_world/src/lib.rs` or `game_zombie/src/lib.rs`; write the `pub mod` lines in Worker notes.

Read:
- `game_world/src/radio.rs` (the pattern to copy: `RadioConfig::from_repo` at lines 23-40, `RadioState` with `to_json`/`from_json` at lines 43-60, and its tests). Use `paths::repo_path("data/safe_house.json")` for the file.
- `game_zombie/src/zombie/state.rs` (`ZombieState::Chase`, `Attack`).
- `game_zombie/src/zombie/herd.rs` `find_chasing_near` (a similar helper; do NOT change it).

## What to write
1. **`SafeHouseConfig`** (serde, `#[serde(rename_all = "camelCase")]`, with `#[serde(default)]` on each block):
   - `travel_hours: f32` (`travelHours`), `rest_hour: f32` (`restHour`), `chase_radius_m: f32` (`chaseRadiusM`), `use_radius_m: f32` (`useRadiusM`), `map_label: String` (`mapLabel`).
   - `prompts: SafeHousePrompts` (every key in the `prompts` block of the JSON, as `String`).
   - The geometry blocks `shack`, `ladder` and `rooftopCamp` are NOT read here. They belong to `world` (R44-03 defines `SafeHouseLayout`). serde ignores unknown keys, so leave them out of this struct.
   - `from_repo() -> Result<Self, String>`, `from_json(&str) -> Result<Self, String>`, `from_path(&Path)`. Mirror `RadioConfig`.
2. **`SafeHouseState { pub claimed: bool, pub ladder_down: bool }`** (`Default`, `Clone`, `Copy`, `PartialEq`, `Debug`):
   - `claim(&mut self)`: sets `claimed = true`, `ladder_down = true`. A second claim changes nothing.
   - `toggle_ladder(&mut self) -> bool`: if not claimed, returns `false` and changes nothing. Else flips `ladder_down` and returns the new value.
   - `to_json(&self) -> serde_json::Value` as `{"claimed": bool, "ladderDown": bool}`.
   - `from_json(&Value) -> Self`: a missing or broken value gives the default (unclaimed, ladder up).
3. **`pub fn travel_blocked(chasing: usize) -> bool`**: `chasing > 0`. (Comment: "new: DECISIONS 2026-11-28 (any zombie chasing blocks travel)".)
4. **`game_zombie/src/chasing.rs`**: `pub fn count_chasing_near(zombies: impl IntoIterator<Item = (Vec3, bool)>, pos: Vec3, radius: f32) -> usize`. It counts entries whose bool is true and whose flat (XZ) distance to `pos` is `<= radius`. Use `radius * radius` (no sqrt). Pure, no game types.

## Number tags
Every number in Rust must come from `data/safe_house.json` (read it, do not hard-code). The only Rust constants allowed are the ones already in the JSON defaults, and those get `// data: safe_house.json key`. The `0`/`1` counts and `true`/`false` are fine.

## Tests (`game/tests/all/safe_house.rs`, use the public API)
- The repo config loads (`SafeHouseConfig::from_repo()` is `Ok`); `travel_hours == 3.0`, `rest_hour == 7.0`, `map_label == "Safe House"`, and all 11 prompt strings are non-empty.
- A bad JSON string gives `Err` (`from_json("{")`).
- `SafeHouseState`: default is unclaimed, ladder up; `toggle_ladder` does nothing before claim (returns false); `claim` lowers the ladder; `toggle_ladder` flips up and down and returns the new value; after a toggle to up, a second `claim` changes nothing (the ladder stays up).
- Save round trip: `from_json(&s.to_json()) == s` for claimed+down, claimed+up, unclaimed. `from_json(&json!({}))` = default; `from_json(&json!("junk"))` = default.
- `travel_blocked(0) == false`, `travel_blocked(1) == true`.
- `count_chasing_near`: a chaser inside the radius counts; a chaser outside does not; a non-chaser inside does not; a chaser exactly on the radius counts (`<=`); a height difference does not matter (flat distance only).

## Done when
The files above exist, `data/safe_house.json` matches `R44-common.md`, the tests above exist and pass in the gate. Fill `## Worker notes` with:
- Files changed.
- The `pub mod` lines for `game_world/src/lib.rs` and `game_zombie/src/lib.rs`, and the `pub use` lines for `game/src/lib.rs` if you add any.
- Any name you had to change from the common file, and why.

## Worker notes
