# STATUS (read this first; keep it under ~80 lines)

Updated: 2026-11-20 by Claude.

## Where we are
- Version 0.1 = Godot parity in Rust, done (R0-R10). History: `docs/archive/RUST_QUEUE.md`.
- Working on **0.2**. Plan and owner decisions: `docs/PLAN_0.2.md`. Locked choices: `docs/DECISIONS.md`.
- Open Godot gaps (short list): `docs/PARITY_OPEN.md`. Worker mistakes: `docs/JESTER.md`.

## Latest numbers (debug build, camp, seed 42, 1920x986, 2026-11-20)
avg 106 FPS, median 8.9 ms, p99 27 ms, worst 1% 29 FPS (below the 30 minimum: stutters to fix),
116k triangles + 88k shadow triangles, 51 + 28 draw calls. Build ~5 s, `cargo test` ~6 s.

## Round now running: R11 (0.2 week 1, day 1: tools)
- R11-01 gate + Neon tools (DeepSeek V4.1 Flash Free, worktree): `tools/gate.sh`, `gate_checks.py`, `neon_pack.py`, `neon_apply.py`.
- Docs cleanup (Claude): done. Old tasks in `tasks/archive/`, history in `docs/archive/`.

## Next (from PLAN_0.2.md)
1. Day 2: `--bench` triangle budget + `data/budgets.json` + baker caps; fix stutter spikes; F1 dev panel (Neon) + `--at=x,z`.
2. Day 3: city spec (owner checks). Days 4-6: city lanes (at most 4). Day 7: owner play test.

## Workers (who does what: PLAN_0.2.md "AI roles")
- DeepSeek: `opencode` / `opencode/deepseek-v4.1-flash-free` (main repo worker).
- Gemini 3.8 Flash: `antigravity` / `gemini-3.8-flash` (only work a script or render can verify).
- Neon: chat only. A subagent builds `neon/out/<task>.prompt.md` with `tools/neon_pack.py`, the owner pastes it into AI Studio and saves the answer as `neon/in/<task>.md`, then a worker runs `tools/neon_apply.py` + the gate.
- Once `tools/gate.sh` is merged, workers run it and loop until `GATE: PASS`. Until then, workers stay write-only.

## Open questions for the owner
- Final game name.
- City spec numbers (region size, distance from camp): ask on day 3.
