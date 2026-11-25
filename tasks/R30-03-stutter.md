# R30-03: find and fix the stutter (short freezes between frames)

Owns: engine/src/app.rs, engine/src/timing*.rs, engine/src/render/*.rs (only if the cause is there), physics/src/world.rs, rotland/src/app/render_frame.rs, rotland/src/streaming.rs, docs/PERF_R30.md (new, the measurements)
Rules: `tasks/R30-common.md`. **This is the only task where running the game is allowed** (release build, always under the lock, always with `--save-dir`).

**Goal:** the owner sees short hitches while playing. `docs/STATUS.md` "Next" item 1 suspects the frame pacing (present mode / vsync), the compositor, or the physics collider rebuild in `physics/src/world.rs` when chunks stream in.

**Steps (one subagent, in order):**
1. Read `--spikes`, `--bench` and `--stats` in `tools/README_TOOLS.md` and the code that writes them. Read code first and form a guess (look for work that happens on some frames only: chunk streaming, collider rebuilds, mesh uploads, minimap cache, saves).
2. The LEAD measures ONCE before changes, and the subagent never runs the game. Use the camp, walking out with `--autowalk`, for ~40 s, and a city spot:
   `flock /tmp/rotland-heavy.lock target/release/rotland --save-dir=/tmp/r30-saves --autowalk --seconds=40 --spikes=/tmp/r30-spikes-before.txt --stats`
   (If `--autowalk` with `--seconds` isn't allowed, use what the README says.) Write the worst frames and what they were doing in `docs/PERF_R30.md`.
3. Fix the biggest cause you can prove, for example spreading collider rebuilds or GPU uploads over several frames, or a budget per frame. Don't lower view distance or quality, and don't change how the game looks.
4. The lead measures again after the gate passes, with the same command into `/tmp/r30-spikes-after.txt`, and puts before/after (worst frame ms, number of frames over 33 ms) in `docs/PERF_R30.md`.
If no clear cause is found, don't guess-fix. Write what you measured and your best guess in `docs/PERF_R30.md`. That's still a good result.

## Worker notes
- **What changed:**
  - `physics/src/world.rs`: added `add_ground_chunk_deferred`, `add_static_deferred`, `add_static_tagged_deferred`, and `remove_deferred` (skipping per-collider `update_broad_phase()`), plus unit test `deferred_colliders_match_immediate_after_broad_phase`. Existing `add_ground_chunk`, `add_static`, `add_static_tagged`, and `remove` now delegate to the `*_deferred` variants and call `update_broad_phase()`.
  - `rotland/src/streaming.rs`: switched chunk unload (`remove_deferred`) and chunk upload (`add_ground_chunk_deferred`, `add_static_deferred`) to the deferred methods so `physics.update_broad_phase()` runs once at the end of `Streamer::update()` instead of 100–300+ times per frame; reduced `MAX_APPLIED_PER_FRAME` from `3` to `1` (`// new: DECISIONS 2026-11-25`) to spread chunk GPU + physics uploads across frames.
  - `docs/PERF_R30.md`: recorded the before-run metrics, spike breakdown from `/tmp/r30-spikes-before.txt`, root causes, and a placeholder section for the lead's after-run metrics.
- **Lines the lead must add to shared files:** None.
- **How to see it in the game:** Run the release build with `--save-dir=/tmp/r30-saves --autowalk --seconds=40 --spikes=/tmp/r30-spikes-after.txt --stats` and compare `world streaming` spikes and frames > 33 ms in `docs/PERF_R30.md`.
- **Unsure / notes for lead:**
  - 76 of the 148 spike rows (max 69.05 ms) are `surface acquire` (`gpu.surface.get_current_texture()`) under `Mailbox` + `ControlFlow::Poll` + `desired_maximum_frame_latency = 2` (`engine/src/gpu.rs`, not owned by this task).
  - Frame 462's `physics=39.23ms` spike is inside `rotland/src/app/frame_sim.rs` (`PhaseScope::new("physics")` around `update_sim`, not owned by this task).
  - Cargo/gate was not run by this subagent per R30 rules.
