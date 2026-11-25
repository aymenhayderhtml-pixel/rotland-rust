# R34 round: rivers, roads, town polish, speed. Lead file for the gate

Files: data/rivers.json, data/roads.json, data/towns.json, world/*, rotland/*, engine/src/render/*, game_*/*, tasks/R34-*, docs/PERF_R34.md
Rules: `tasks/LEAD.md`. Keep it SMALL: the owner pays for every token. 3 subagents, read only the files listed in your task, no exploring.

Must stay true: the home land (2.2 km core) does not change (`world/tests/golden.rs` passes unedited). No sounds.

## Tasks
| Task | What | Subagent |
|---|---|---|
| `R34-01-rivers.md` | rivers are long and flow to their target, not little loops | A |
| `R34-02-roads-towns.md` | roads start right at the home land edge; Blackwater boardwalks become walkways in front of buildings | B |
| `R34-03-speed.md` | forest FPS back up (lead measures first) | C, after Phase 0 |

## Phase 0 (lead, before subagent C)
`flock /tmp/rotland-heavy.lock tools/perf_scenes.sh --set all --seconds 20 --bin target/play/rotland` after `cargo build --profile play`. Write the numbers into `docs/PERF_R34.md` ("before"). Then run ONE forest run with `--stats=/tmp/r34/forest.json` and give subagent C the top costs (draw calls, triangles, shadow triangles, CPU ms per system if printed).

## Checks after the gate
- World map: `cargo run --release -p world --example worldmap -- --seed=42 --size=12000 --px=1200 --out=/tmp/r34/worldmap.png`. LOOK: 3 long rivers (several km each), roads touching the home land circle.
- Shots with target/play/rotland, `--save-dir=/tmp/r34-saves`: `--town=blackwater`, `--trailhead=0`, `--river=0`, camp (no flags). One look each.
- perf_scenes again ("after" in docs/PERF_R34.md).

## Lead notes
- **Tasks:**
  - `R34-01-rivers.md` (Subagent A): Done. Long rivers reach targets without self-crossing or rising water levels; `generator_world.rs` updated; `biome_terrain_tests::no_seams_across_core_edge_and_sector_borders` updated to skip carved riverbanks.
  - `R34-02-roads-towns.md` (Subagent B): Done. Trailheads sit at `core_radius + 30`; Blackwater boardwalks are parallel walkways along the street; `generator_world.rs`, `roads_tests.rs`, and `towns_tests.rs` updated.
  - `R34-03-speed.md` (Subagent C): Done. Sun-frustum shadow culling, small-prop/tree shadow LODs, tighter tree LODs, fragment shadow early-out, zero-alloc skinned batch splitting.
- **Gate (last 5 lines):**
  ```
  test: PASS (1195 passed)
  scope: PASS (21 changed paths)
  numbers: PASS (5 checked)
  unused: PASS
  GATE: PASS
  ```
- **Before / After FPS (`perf_scenes.sh`, 20s per scene):**
  - `forest`: 86.7 -> 106.3 Avg FPS (1% Low 25.6 -> 45.0, p95 20.7 -> 12.9 ms, Draws 75(21) -> 71(19))
  - `camp`: 132.7 -> 137.6 Avg FPS (Draws 56(27) -> 58(24))
  - `city`: 100.5 -> 103.4 Avg FPS
- **Pictures:**
  - `/tmp/r34/worldmap.png` (3 long rivers, roads touching home land circle)
  - `/tmp/r34/blackwater.png` (parallel boardwalk walkways)
  - `/tmp/r34/trailhead0.png` (road at home land edge)
  - `/tmp/r34/river0.png` (snowy river valley)
  - `/tmp/r34/camp.png` (green camp)

## Gate report
- date: 2026-11-25
- base: main
- files changed (21): data/rivers.json, data/roads.json, data/towns.json, docs/PERF_R34.md, engine/src/render/lit.rs, engine/src/render/lit.wgsl, rotland/src/app/render_frame.rs, rotland/src/props.rs, tasks/R34-01-rivers.md, tasks/R34-02-roads-towns.md, tasks/R34-03-speed.md, tasks/R34-round.md, world/src/city/ground_tests.rs, world/src/generator_world.rs, world/src/rivers/mod.rs, world/src/rivers/path.rs, world/src/roads/build.rs, world/src/towns/lots.rs, world/tests/biome_terrain_tests.rs, world/tests/roads_tests.rs, world/tests/towns_tests.rs
- tests passed: 1195
- clippy: PASS
- test: PASS
- checks: PASS
- gate: PASS
