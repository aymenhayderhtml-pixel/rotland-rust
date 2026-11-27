# R43-07: The radio at the top (the goal)

Owns:
- `game_world/src/radio.rs` (NEW, pure) + its tests, the `pub mod radio;` line in `game_world/src/lib.rs` (+ the re-export in `game/src/lib.rs` if the facade re-exports game_world modules: check, and write it in Worker notes if it is not yours to add)
- `data/radio.json` (NEW)
- `rotland/src/metro_radio.rs` (NEW: the play-side glue: prompt, [E], message, crate unlock, save/load of `systems.extra["metro_radio"]`)

Lines for the lead (R43-08 puts them in): the `mod metro_radio;` line, the call in the frame/interaction loop, save/load hooks. Write them exactly in Worker notes.
Read: tasks/R43-common.md, how an existing [E] interaction shows a prompt and acts (`rotland/src/rescue_play.rs` or the crate open path in `rotland/src/crates.rs`), how a HUD message/toast is shown (`rotland_ui` hint/notice), how `systems.extra[...]` is saved and loaded (`rotland/src/play_metro.rs` `street_herds` export/restore), `world/src/metro/climb.rs` (R43-01: the radio console position comes from the tallest landmark's plan; until it exists, take the position as an argument).

## `data/radio.json`
```json
{
  "_why": "R43: the radio on the tallest tower. Text is shown in order, one line every few seconds.",
  "useRadiusM": 2.0,
  "prompt": "[E] Use the radio",
  "usedPrompt": "Only static now.",
  "lineSeconds": 4.0,
  "lines": [
    "...static...",
    "\"...anyone? Anyone on this channel?\"",
    "\"We're on the rooftops. The streets are theirs, the roofs are ours.\"",
    "\"Look for the green smoke. Come up, don't come down.\"",
    "...static..."
  ],
  "crateId": "metro_radio_crate",
  "mapLabel": "Radio Mast"
}
```
The words are Claude defaults. Keep them generic, no real names.

## Logic (`game_world/src/radio.rs`, pure)
- `RadioConfig` (serde, the file above).
- `RadioState { used: bool }`, `Default` = not used.
- `radio_prompt(state, player_pos, console_pos, cfg) -> Option<&str>`: within `useRadiusM` 3D, it is `prompt`, or `usedPrompt` once used.
- `use_radio(&mut state, player_pos, console_pos, cfg) -> Option<RadioOutcome>`: `RadioOutcome { lines: Vec<String>, unlock_crate: String }`.
  - Only the first use returns it; later uses return `None`.
- Save: `to_json`/`from_json` of the state; missing or bad = not used.
- Tests:
  - The prompt range in 3D: standing under the mast on the roof is out of range.
  - The first use gives the lines + the crate, the second gives nothing.
  - A save round trip; an empty save = not used.

## Play glue (`rotland/src/metro_radio.rs`)
- Each frame, near the console: show the prompt like other [E] prompts. On [E]:
  - Show the lines one after another (`lineSeconds` each) in the existing notice/hint box.
  - Unlock the crate `crateId`. It stays closed until then: either the crate spawns only after use, or it is locked. Pick the simpler path in `crates.rs` and say which.
  - Set `used`.
- Save/load in `systems.extra["metro_radio"]` the same way the herds do.
- Nothing happens outside New Halden.

## Done when
Code + data + tests written. Fill `## Worker notes`.

## Worker notes

### Files changed
- `data/radio.json` (NEW): config for `useRadiusM`, `prompt`, `usedPrompt`, `lineSeconds`, `lines`, `crateId` (`"metro_radio_crate"`), `mapLabel` (`"Radio Mast"`).
- `game_world/src/radio.rs` (NEW): `RadioConfig`, `RadioState` (`to_json`/`from_json`), `RadioOutcome`, `radio_prompt`, `use_radio`, plus unit tests (`prompt_range_is_3d`, `first_use_returns_lines_and_crate_second_returns_none`, `save_round_trip_and_missing_defaults_unused`).
- `game_world/src/lib.rs`: added `pub mod radio;` and `pub use radio::*;`.
- `rotland/src/metro_radio.rs` (NEW): `MetroRadioRuntime` and `RadioInteract` (`interact_tick`, `advance_playback`, `can_open_crate`, `to_save`, `restore_save`).

### Crate lock choice
- `MetroRadioRuntime::can_open_crate(&self, crate_id)` returns `false` for `metro_radio_crate` until `state.used` is `true`. Filtering `world.near_crates_scratch` in `frame_world.rs` keeps the crate closed/uninteractable without modifying `rotland/src/crates.rs`.

### Exact lines for the lead / R43-08

1. **`game/src/lib.rs`** (re-export `game_world::radio`):
```rust
// In the module re-export line (~line 26):
pub use game_world::{animals, crows, dogs, grenade, guns_fire, melee, melee_guard, projectiles, radio};

// After `pub use game_world::projectiles::*;` (~line 114):
pub use game_world::radio::*;
```

2. **`rotland/src/main.rs`** (note: modules live in `main.rs`, not `app/mod.rs`):
```rust
mod metro_radio;
```

3. **`rotland/src/app/state.rs`** (on `PlaySubState`, next to `pub rescue`):
```rust
    pub metro_radio: crate::metro_radio::MetroRadioRuntime,
// and in PlaySubState::new / Default init:
            metro_radio: crate::metro_radio::MetroRadioRuntime::default(),
```

4. **`rotland/src/app/frame_world.rs`**:
   - Right after populating `world.near_crates_scratch` (~line 211):
```rust
        world.near_crates_scratch.retain(|s| play.metro_radio.can_open_crate(&s.crate_id));
```
   - Around `rescue_interact` (~line 378), passing `console_pos: Option<Vec3>` from the tallest landmark's plan in `world::metro::climb` (or `None` outside New Halden):
```rust
        let radio_interact = play.metro_radio.interact_tick(
            dt,
            p.position,
            console_pos,
            play_state,
            pressed && free_world,
        );
        if radio_interact.as_ref().is_some_and(|r| r.consumed_press) {
            ui.e_used = true;
        }
        let free_world = free_world && radio_interact.is_none();
```
   - And when applying prompts (~line 425, right after `if let Some(r) = &rescue_interact`):
```rust
        if let Some(r) = &radio_interact {
            if let Some(text) = &r.prompt {
                play_state.set_prompt(text.clone());
            }
            other_dist = other_dist.min(r.distance);
        }
```

5. **`rotland/src/app/save_load.rs`** (next to `street_herds`, ~line 54):
```rust
    if let Some(v) = play.metro_radio.to_save() {
        file.systems.extra.insert("metro_radio".into(), v);
    }
```

6. **`rotland/src/app/start_world.rs`** (next to `street_herds`, ~line 232):
```rust
        play.metro_radio = crate::metro_radio::MetroRadioRuntime::default();
        if let Some(v) = file.systems.extra.get("metro_radio") {
            play.metro_radio.restore_save(v);
        }
```

### How to test
- `cargo test` (runs `game_world::radio::tests::*`).
- In-game (once wired in R43-08): `--metro=radio`, press `[E]` at the radio console, verify lines play in order every 4s in the HUD toast, `"Only static now."` shows afterward, `metro_radio_crate` becomes openable, and save/reload preserves `used`.

### Known issues / notes
- `cargo` / `tools/gate.sh` were not run in this worker pass (per subagent rules).
- `console_pos` is passed as `Option<Vec3>` to `MetroRadioRuntime::interact_tick` because `world/src/metro/climb.rs` (R43-01) is not yet in this tree.
