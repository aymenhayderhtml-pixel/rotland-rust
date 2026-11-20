# Triangle and model budgets (R12-01)

Target: owner's laptop class, Intel UHD (CML GT2), 1080p, release build. Both bench runs and camp run were on AC power.

## Bench runs (`--bench=<path>`, camp spawn, no zombies)
Copies double every 3 s from 50. Median frame ms per step:

| Step | Copies | Tris | Run 1 ms | Run 2 ms |
|-----:|-------:|-----:|---------:|---------:|
| 0 | 50 | 148,178 | 8.23 | 9.99 |
| 1 | 100 | 193,246 | 8.82 | 12.68 |
| 2 | 200 | 296,420 | 10.49 | 14.83 |
| 3 | 400 | 489,832 | 13.63 | 19.16 |
| 4 | 800 | 886,356 | 19.47 | (not reached) |

- Run 1: trianglesAt60 = 698,211 (interpolated, steps 3-4).
- Run 2: trianglesAt60 = 379,849 (interpolated, steps 2-3).
- trianglesAt30 (extrapolated): run 1 = 1,828,613; run 2 = 1,122,750.
- The runs differ almost 2x. The lower run sets the budget.

## Camp stats (`--seconds=15 --stats=`)
- Avg FPS 99.1, p99 frame 27.7 ms, worst 1% FPS 26.0.
- Triangles 124,990, shadow triangles 98,434, draw calls 53.

## Formula
- frame.triangles = 60% of 379,849 = 227,909, rounded to 228,000.
- frame.shadowTriangles = 60% of the shadow triangles at the crossing (200,737), rounded to 121,000.
- frame.drawCalls = 120 (provisional, Claude 2026-11-20): the bench uses instancing, so draws stayed 58-62 and don't measure the draw limit. Set to about 2x the camp's 53 until a draw-call bench exists.
- Bench runs 1 and 2 differ ~2x (698k vs 380k at 60 FPS). Run 2 likely overlapped a release build by the other helper. The frame budget uses the lower run; re-run on a quiet, plugged-in laptop to confirm.
- Model cap = max(2 x biggest in folder, biggest + 2000), rounded up to the next 1000. All 9 current caps match.

## Reading `--report-budgets`
Run: `cargo run --release -p baker -- --report-budgets`
- Prints folder, model, triangles, cap and OK / OVER BUDGET for every model. Writes no pack.
- Normal baking prints `BUDGET: <model> <tris> > <cap>` and fails if a model is over its folder cap.
- Result now: all models within caps. Biggest per folder: settlement 5,614 (buildings cap 12,000), zombie_01 3,700 (characters 8,000), tree_oak_big 1,380 (nature 4,000), car_hatchback_* 2,172 (vehicles 5,000).
