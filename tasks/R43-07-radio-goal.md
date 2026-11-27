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
