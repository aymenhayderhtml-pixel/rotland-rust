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
