# R12-01: triangle budget bench + model budgets in the baker

Read first: `AGENTS.md`, `docs/STATUS.md`, `tools/README_TOOLS.md`.
Files: engine/src/cli.rs, engine/src/stats.rs, rotland/src/app/bench.rs, rotland/src/app/mod.rs, rotland/src/app/startup.rs, rotland/src/app/state.rs, rotland/src/app/render_frame.rs, rotland/src/app/stats.rs, baker/src/*, data/budgets.json, docs/BUDGETS.md, tasks/R12-01-bench-budgets.md
Test: bench

Goal: find how many triangles / draw calls the owner's laptop (Intel UHD CML GT2, 1080p) can draw before the frame takes 16.7 ms, and stop the baker from accepting models that blow the budget.

## 1. `--bench=<path>` mode (new file `rotland/src/app/bench.rs`, small hooks elsewhere)
- Starts the play scene at the camp spawn (seed 42), camera fixed, no zombies (`--zombies=0` behaviour), muted, player frozen.
- In front of the camera, places instanced copies of a mix of real models from the pack: tree models, one building, the player character model, a car body (use what the pack already has; look up model names in the code, don't guess). Spread them on a grid 10-150 m ahead so they're inside the frustum.
- Step: every 3 s, double the copies (start ~50). For each step, record median and p95 frame ms (skip the first 0.5 s of each step), triangles, shadow triangles, draw calls, copies.
- Stop when the median goes over 16.7 ms, or after 12 steps. Write JSON to `<path>`: steps list + `trianglesAt60` (interpolated triangle count where the median crosses 16.7 ms) + `trianglesAt30` (extrapolate from the last two steps if never reached, and mark `"extrapolated": true`).
- Keep the logic that decides "next step / stop / interpolate" in a pure function with unit tests (test names contain `bench`).

## 2. Run it for real (allowed for this task, one window at a time)
- `cargo build --release` under `flock /tmp/rotland-heavy.lock` (first release build may take ~7 min; wait for it).
- `flock /tmp/rotland-heavy.lock target/release/rotland --save-dir=/tmp/r12bench --bench=/tmp/r12bench/bench.json`. Run it twice. Note if the laptop is on battery (`cat /sys/class/power_supply/*/online`).
- Also run `target/release/rotland --save-dir=/tmp/r12bench --seconds=15 --stats=/tmp/r12bench/camp.json` and record avg FPS, p99 ms, triangles at the camp.

## 3. Budgets
- `data/budgets.json`: `{ "frame": { "triangles": N, "shadowTriangles": N, "drawCalls": N }, "models": { "<folder under assets/models>": maxTrianglesPerModel, ... } }`. Fill `frame` with 60% of `trianglesAt60` (headroom for CPU, effects). For `models`, report the current biggest model per folder (from the baker) and set each cap to max(2x current biggest, current biggest + 2000), rounded up to the next 1000. Claude will adjust the numbers; just make them consistent.
- Baker: after baking, check every model's triangle count against its folder's cap. Over the cap = print `BUDGET: <model> <tris> > <cap>` and exit with an error. Add `--report-budgets` that prints a table (folder, model, tris, cap) and exits. Keep the check logic pure + unit tested (test names contain `bench` or `budget`... use `budget_` prefix and add `Test: bench|budget`? The gate takes one filter: name all new tests starting with `bench_` to match the Test line).
- `docs/BUDGETS.md` (under 40 lines): the measured numbers (both bench runs, camp stats, power state), the formula, and how to read `--report-budgets`.

## Rules
- Run `tools/gate.sh --task tasks/R12-01-bench-budgets.md` and loop until `GATE: PASS`. Then commit once to your branch.
- Ground stays green. Don't change rendering itself.

## Worker notes

## Gate report
- date: 2026-11-20
- base: main
- files changed (13): baker/src/budget.rs, baker/src/lib.rs, baker/src/main.rs, data/budgets.json, docs/BUDGETS.md, engine/src/cli.rs, rotland/src/app/bench.rs, rotland/src/app/mod.rs, rotland/src/app/render_frame.rs, rotland/src/app/startup.rs, rotland/src/app/state.rs, rotland/src/app/stats.rs, tasks/R12-01-bench-budgets.md
- tests passed: 10
- clippy: PASS
- test: PASS
- checks: PASS
- gate: PASS
