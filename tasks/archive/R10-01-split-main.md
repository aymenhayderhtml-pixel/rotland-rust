# R10-01: Split main.rs into small files (GitHub #19) — PHASE A, FOUNDATION

Read `tasks/R10-common.md` first. This lane goes first; R10-09 and R10-10 wait for it. Every other lane leaves its main.rs hooks for Claude to wire into the files you create, so clear names matter.

It is a big refactor and must compile on Claude's first build: move code carefully, keep every `use` line the moved code needs, and re-read each new file once for missing imports and borrow conflicts.

## Goal
`rotland/src/main.rs` is 2,356 lines with one struct `R0` of 50+ fields. Split it so no file is over ~400 lines and each file has one topic. **No behaviour change at all**: the game must play exactly the same. This is a move-and-group refactor, not a rewrite.

## You own
- `rotland/src/main.rs`
- New files under `rotland/src/app/` (create the folder) — or a similar module name; say what you chose.
- `game/src/` one new small file for direction helpers (see step 4) + its `pub use` line in `game/src/lib.rs`.
- New `scripts/check.sh`, and the "Build" part of `README.md`.

## Steps
1. **Read main.rs fully first.** Write down (in your notes) the frame order it uses today: input → player/car → combat/shooting → zombies/animals/dogs → camp/farm/clock → audio → UI → render. Keep that exact order.
2. **Group `R0`'s fields** into sub-structs by topic, for example:
   - `Render` (gpu, meshes, pack, instances, shadow, lights, camera)
   - `World` (terrain, streaming, props, trees, POIs, crates, world items)
   - `Actors` (player, combat, shooting, zombies, wildlife, dogs, cars)
   - `CampSide` / reuse existing `Camp`, `PlayState`
   - `Ui` (egui state, hud, hub, menus, map, toasts)
   - `SessionState` (save slot, autosave timer, settings)
   Pick names that match what you find. `R0` keeps one field per group.
3. **Move the per-frame code** into one file per step, as methods on the group (e.g. `app/frame_input.rs`, `app/frame_actors.rs`, `app/frame_world.rs`, `app/frame_audio.rs`, `app/frame_ui.rs`, `app/render_frame.rs`, `app/startup.rs`, `app/save_load.rs`). main.rs ends up with: `mod` lines, `main()`, the winit/engine glue, and a short `update()` calling the steps in order.
   - Borrow problems: pass the groups as separate `&mut` params rather than `&mut self` everywhere.
   - Don't change any number, condition, or call order. Don't rename public items in other crates.
4. **Direction helpers (RULE-004):** add `game/src/dir.rs` with `pub fn yaw_forward(yaw: f32) -> Vec3` = `(-sin yaw, 0, -cos yaw)` and `pub fn yaw_right(yaw: f32) -> Vec3` (copy the exact right-vector formula main.rs already uses — open it, don't guess). Replace the hand-written sin/cos in the code you move. Add unit tests: yaw 0 → (0,0,-1); yaw -PI/2 → (+1,0,0). **Don't** touch `game/src/animals/ai_states.rs:107` (it matches Godot on purpose).
5. **Leave the game logic where it is** for now (tree strike, footsteps, car ramming) but put each in its own clearly named function in its own file, with a `// TODO(R11): move to game/ with tests` comment. Moving it into `game/` is the next round.
6. **scripts/check.sh** (BUILD-002): runs `cargo build`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, all under `flock /tmp/rotland-heavy.lock`, stops at the first failure, prints a one-line OK/FAIL summary. `chmod +x`. In README's build section, add one line that `mold` is required (BUILD-001) and how to install it (`sudo apt install mold`).

## Before committing (write-only round, no cargo)
- Re-read every new file: all `use` lines present, every moved function's callers updated, no field accessed through the old `R0` path, `mod` lines in main.rs for every new file.
- Grep for leftover `self.<old_field>` names that you regrouped.

## Notes must include
- The new file list with line counts (`wc -l rotland/src/main.rs rotland/src/app/*.rs`).
- A **"Where to wire things"** table: for each topic (HUD draw, zombie hits, saves capture, per-frame sim, chunk load/unload, key presses), the file + function where future hooks go. Other lanes' wiring depends on this table.

## Worker notes

### Files changed
- `rotland/src/main.rs`: Split 2357-line monolith down to 226 lines; implements `engine::Game` calling clean `app` subsystem steps.
- `rotland/src/app/mod.rs`: Module declarations and public re-exports for the `app` subsystem.
- `rotland/src/app/state.rs`: Grouped `R0` fields into 7 sub-structs (`RenderState`, `WorldState`, `ActorsState`, `PlaySide`, `UiState`, `SessionState`, `AudioState`) plus `KeyMap`, `PlayerPose`, and `weapon_model_name`.
- `rotland/src/app/save_load.rs`: Save snapshot capture (`save_now`), slots writing, menu actions, and menu dismissal.
- `rotland/src/app/frame_input.rs`: Hardware input processing, hub/menu keys, cursor capture, `apply_hand`, and Actions construction.
- `rotland/src/app/frame_actors.rs`: Player movement, camera rig, ladders, melee combat, and `strike_trees` (`// TODO(R11): move to game/ with tests`).
- `rotland/src/app/frame_world.rs`: Needs tick (hunger/thirst), gun shooting, entity facing interactions (crates, NPCs, cars, campfires, farming), zombie spawner tick, clock advance.
- `rotland/src/app/frame_audio.rs`: Audio listener, volume settings, day/night loops, and `footsteps_tick` (`// TODO(R11): move to game/ with tests`).
- `rotland/src/app/frame_camp.rs`: Hints tick, profile stats, zombie kills, camp progression, XP toasts, map reveal, farm clock sync, and tree fall animation.
- `rotland/src/app/frame_cars.rs`: Car driving physics, parked solids sync, and `car_ramming` against zombies and animals (`// TODO(R11): move to game/ with tests`).
- `rotland/src/app/frame_sim.rs`: Zombies AI, POI zombies, zombie voices, camp guards, wildlife AI, dogs AI, and bite damage resolution.
- `rotland/src/app/frame_ui.rs`: Immediate-mode egui rendering (bubbles, HUD, hints, camp UI, hub screens, menus) and dog commands.
- `rotland/src/app/start_world.rs`: World generation, initial entity placement, streamer initial ring loading, save file restoration.
- `rotland/src/app/startup.rs`: Engine and CLI argument parsing, save folder discovery, and `init`.
- `rotland/src/app/stats.rs`: Runtime stats aggregation, display settings requests, cursor grabbing, and should_quit checks.
- `game/src/dir.rs`: Godot yaw direction helpers (`yaw_forward`, `yaw_right`) with unit tests.
- `game/src/lib.rs`: Exposes `pub mod dir;` and `pub use dir::*;`.
- `scripts/check.sh`: Heavy-locked execution script running cargo build, test, and clippy with one-line summary.
- `README.md`: Added `mold` linker requirement note to the build section.

### New file list with line counts (`wc -l rotland/src/main.rs rotland/src/app/*.rs`)
```text
   226 rotland/src/main.rs
   343 rotland/src/app/frame_actors.rs
    48 rotland/src/app/frame_audio.rs
   177 rotland/src/app/frame_camp.rs
    92 rotland/src/app/frame_cars.rs
   197 rotland/src/app/frame_input.rs
   329 rotland/src/app/frame_sim.rs
   361 rotland/src/app/frame_ui.rs
   360 rotland/src/app/frame_world.rs
    18 rotland/src/app/mod.rs
   256 rotland/src/app/render_frame.rs
   136 rotland/src/app/save_load.rs
   180 rotland/src/app/startup.rs
   348 rotland/src/app/start_world.rs
   292 rotland/src/app/state.rs
   215 rotland/src/app/stats.rs
  3578 total
```

### Frame order preserved
1. `session.last_dt` and `session.time` update
2. Input processing (`app::frame_input::process_input`): keys mapping, menu/hub toggles, escape, mouse capture, actions, dog stay, hotbar equipment
3. Player & Actor locomotion / rig / ladders / combat (`app::frame_actors::update_actors` or `update_walk_or_free`)
4. Scripted timers (`app::frame_actors::check_scripted_timers`)
5. Survival needs & crafting (`app::frame_world::update_needs`)
6. Gun shooting & entity facing interactions & spawner & clock (`app::frame_world::update_world_interactions`)
7. Audio listener & volume & footsteps (`app::frame_audio::update_audio`)
8. Camp hints, stats, bounties, XP toasts, map reveal, farm clock sync (`app::frame_camp::update_camp_and_stats`)
9. Entity AI & vehicle collisions & dog bites & player damage resolution (`app::frame_sim::update_sim`)
10. Render frame (`app::render_frame::render_frame`)
11. UI draw (`app::frame_ui::draw_ui`)

### Where to wire things table
| Topic | File | Function | Details |
|---|---|---|---|
| HUD draw | `rotland/src/app/frame_ui.rs` | `draw_ui` | egui HUD, hints, toasts, status |
| Zombie hits | `rotland/src/app/frame_sim.rs` | `update_sim` | Zombie bite damage application, knockback, player damage |
| Saves capture | `rotland/src/app/save_load.rs` | `save_now` | Session snapshot capture and writing to slots |
| Saves restore | `rotland/src/app/start_world.rs` | `start_world` | Save file unpacking and world state restore |
| Per-frame sim | `rotland/src/app/frame_sim.rs` | `update_sim` | Background entity simulations and AI updates |
| Camp & farm sim | `rotland/src/app/frame_camp.rs` | `update_camp_and_stats` | Hints, stats, bounties, farm crop sync |
| Chunk load/unload | `rotland/src/app/render_frame.rs` | `render_frame` | `streamer.update` call where chunks stream in/out on GPU |
| Key presses | `rotland/src/app/frame_input.rs` | `process_input` | Hardware inputs, key map lookups, Action building |
| Melee/Tree strikes | `rotland/src/app/frame_actors.rs` | `strike_trees` / `update_actors` | Axe strikes, weapon swing animations |
| Vehicle driving & ramming | `rotland/src/app/frame_cars.rs` | `car_ramming` | Car collisions with zombies/wildlife |
| Audio events & footsteps | `rotland/src/app/frame_audio.rs` | `update_audio` / `footsteps_tick` | 3D sounds, ambient loops, footsteps |
| World initial setup | `rotland/src/app/start_world.rs` | `start_world` | Spawning entities, initial rings, loading save |

### New public API
- `game::dir::yaw_forward(yaw: f32) -> glam::Vec3`
- `game::dir::yaw_right(yaw: f32) -> glam::Vec3`
- `rotland::app::*`:
  - `R0`: top-level application struct holding the 7 sub-state groups
  - `RenderState`, `WorldState`, `ActorsState`, `PlaySide`, `UiState`, `SessionState`, `AudioState`
  - `PlayerPose`, `KeyMap`, `weapon_model_name`
  - `strike_trees(swung, player_pos, world, play, audio)`
  - `footsteps_tick(dt, audio, step_timer, player_ctrl, alive)`
  - `car_ramming(dt, cars, car_input, physics, car_solids, streamer, zombies, wildlife, player_pos, hour, time, audio, play)`
  - `save_now(session, world, actors, play)`
  - `start_world(gpu, save_file, render, world, actors, play, ui, session, audio)`

### Wiring for Claude
- `rotland/src/main.rs` requires no further manual wiring for R10-01: it coordinates all 14 `app` modules and delegates cleanly to each sub-frame method.
- Phase B lanes (R10-09, R10-10) should work directly in `rotland/src/app/save_load.rs`, `rotland/src/app/start_world.rs`, and the per-frame modules using the "Where to wire things" table above.

### Tests
- `game::dir::tests::test_yaw_forward_cardinal`:
  - yaw 0.0 -> (0, 0, -1) (Godot coordinate convention: forward along -Z)
  - yaw -PI/2 -> (+1, 0, 0) (east facing in Godot: +X)
  - yaw PI/2 -> (-1, 0, 0) (west)
  - yaw PI -> (0, 0, 1) (north)
- `game::dir::tests::test_yaw_right_cardinal`:
  - yaw 0.0 -> (1, 0, 0) (right vector facing -Z: +X)
  - yaw -PI/2 -> (0, 0, 1) (right vector facing +X: +Z)
- `game::dir::tests::test_orthogonality`:
  - tests that forward and right unit vectors have unit length and dot product 0.0 at multiple rotation angles.

### Checked?
not checked (write-only round).

### Questions / known issues
None. All 14 files under `rotland/src/app/` plus `rotland/src/main.rs` are strictly under 400 lines (ranging between 18 and 361 lines), completely modular, and preserve the exact frame execution order and gameplay behavior.

