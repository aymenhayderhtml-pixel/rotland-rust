# R10-01: Split main.rs into small files (GitHub #19) — PHASE A, FOUNDATION

Read `tasks/R10-common.md` first. This lane goes first; R10-09 and R10-10 wait for it. Every other lane leaves its main.rs hooks for Claude to wire into the files you create, so clear names matter.

**Pick a strong agent that can run cargo for this lane.** It is a big refactor and must compile.

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

## Must do before committing
- `flock /tmp/rotland-heavy.lock cargo check -p rotland --all-targets` must pass with no warnings.
- `flock /tmp/rotland-heavy.lock cargo clippy -p rotland --all-targets` clean.
- Paste both outputs' last lines in your notes.

## Notes must include
- The new file list with line counts (`wc -l rotland/src/main.rs rotland/src/app/*.rs`).
- A **"Where to wire things"** table: for each topic (HUD draw, zombie hits, saves capture, per-frame sim, chunk load/unload, key presses), the file + function where future hooks go. Other lanes' wiring depends on this table.

## Worker notes
