# STATUS (read this first; keep it under ~80 lines)

Updated: 2026-11-20 by Claude.

## Where we are
- Version 0.1 = Godot parity in Rust, done (R0-R10). History: `docs/archive/RUST_QUEUE.md`.
- Working on **0.2**. Plan and owner decisions: `docs/PLAN_0.2.md`. Locked choices: `docs/DECISIONS.md`.
- Open Godot gaps (short list): `docs/PARITY_OPEN.md`. Worker mistakes: `docs/JESTER.md`.

## Latest numbers (release, camp, plugged in, 2026-11-20; noisy, other jobs were running)
avg 95-126 FPS, p99 16-29 ms, worst 1% 26-36 FPS (stutters still below the 30 minimum).
60 FPS triangle limit (bench): 380k-700k; frame budget 228k tris / 121k shadow / 120 draws (`data/budgets.json`, `docs/BUDGETS.md`).

## Last round: R12 (0.2 week 1, day 2) DONE
- R12-01 `--bench` + `data/budgets.json` + baker caps (`--report-budgets`). Gemini stalled, Haiku finished.
- R12-02 `--spikes` logger: no fix yet. Slow frames are mostly time outside the timed phases (between frames); see `docs/STUTTER.md`. Gemini stalled, Haiku finished.
- R12-03 F1 dev panel (Neon, repaired by Haiku) + `--at=X,Z`, `--dev-panel`.
- Crash on quit fixed (egui-winit clipboard off).

## Next (from PLAN_0.2.md)
1. Stutter: re-measure on a quiet laptop (nothing else running), then find the between-frames stall (present mode / vsync, compositor, physics collider rebuild in `physics/src/world.rs`). Re-run `--bench` quietly too.
2. Day 3: city spec (owner checks). Days 4-6: city lanes (at most 4). Day 7: owner play test.

## Workers (who does what: PLAN_0.2.md "AI roles")
- DeepSeek: `opencode` / `opencode/deepseek-v4.1-flash-free`: free usage ran out on 2026-11-20.
- Gemini 3.8 Flash: `antigravity` / `gemini-3.8-flash`: SHORT jobs only; it can't wait on long builds or runs (stops with "I'll wait").
- Haiku (Agent tool): long build/run jobs; build+run inside ONE flock (shared release binary gets overwritten).
- Neon: chat only. A subagent builds `neon/out/<task>.prompt.md` with `tools/neon_pack.py`, the owner pastes it into AI Studio and saves the answer as `neon/in/<task>.md`, then a worker runs `tools/neon_apply.py` + the gate.
- Workers run `tools/gate.sh --task <file>` and loop until `GATE: PASS` (usage: `tools/README_TOOLS.md`).

## Open questions for the owner
- Final game name.
- City spec numbers (region size, distance from camp): ask on day 3.
