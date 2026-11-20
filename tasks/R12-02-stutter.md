# R12-02: find and fix the frame-time spikes (stutters)

Read first: `AGENTS.md`, `docs/STATUS.md`, `tools/README_TOOLS.md`.
Files: engine/src/timing.rs, engine/src/stats.rs, engine/src/app.rs, engine/src/cli.rs, engine/src/render/*, rotland/src/app/frame_*.rs, rotland/src/app/spikes.rs, rotland/src/app/mod.rs, rotland/src/app/startup.rs, rotland/src/app/state.rs, world/src/*, game/src/*, docs/STUTTER.md, tasks/R12-02-stutter.md
Test: spike

Problem (debug build, camp, 2026-11-20): median 8.9 ms but p99 27 ms, worst 1% = 29 FPS. The hard rule is never under 30 FPS. Target after the fix, **release build**: p99 under 20 ms and worst 1% over 50 FPS, both standing at camp and with `--autowalk` (walking through new chunks).

## 1. Measure first (no guessing)
- Add `--spikes=<path>`: for every frame over 20 ms, write one JSON line with the frame number, total ms, and the ms of each main phase of the frame (input, world streaming, physics, zombies/actors, camp, ui/egui, render submit, GPU wait/present; time the existing phase functions in `rotland/src/app/frame_*.rs` and the engine render with `std::time::Instant`). Also count first-time events per frame (new chunk built, new pipeline/shader created, new mesh uploaded, save written).
- Build release under `flock /tmp/rotland-heavy.lock cargo build --release`. Run (one window at a time, always under the flock):
  - `target/release/rotland --save-dir=/tmp/r12s --seconds=30 --stats=/tmp/r12s/camp.json --spikes=/tmp/r12s/camp.jsonl`
  - same with `--autowalk` (walk.json / walk.jsonl)
- Group spikes by which phase was largest. Write the before numbers and the top causes in `docs/STUTTER.md`.

## 2. Fix the top causes
Typical causes to check (confirm with the measurements, don't assume): chunk mesh/collider building on the main thread, GPU buffer uploads all in one frame, pipelines made on first use, the minimap cache rebuild, save/autosave writes, spawning many things in one frame. Fixes: spread work over frames (a per-frame time budget), move pure work to a worker thread, create pipelines at startup.
- Behaviour and gameplay numbers must not change. A refactor keeps every line of logic (JESTER #37).
- Pure logic (e.g. a per-frame work budget queue) gets unit tests whose names contain `spike`.

## 3. Prove it
Re-run both release runs; put before/after avg FPS, p99 ms, worst 1% FPS and spike counts in `docs/STUTTER.md`. Note battery or plugged in (`cat /sys/class/power_supply/*/online`).

## Rules
- Another worker (R12-01) is adding a `--bench` mode in `rotland/src/app/bench.rs` and touching `engine/src/cli.rs`, `startup.rs`, `state.rs`, `mod.rs`: add your lines without reordering theirs; small merge conflicts are fine.
- Run `tools/gate.sh --task tasks/R12-02-stutter.md` and loop until `GATE: PASS`. Then commit once to your branch.
- Ground stays green.

## Worker notes
