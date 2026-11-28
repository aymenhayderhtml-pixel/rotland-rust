# R45 round: New Halden wild park + stadium horde. Lead file for the gate

Files: world/*, game_zombie/*, game_camp/*, game_world/*, game/*, rotland/*, rotland_ui/*, data/*, tools/blender/*, tools/README_TOOLS.md, assets/models/*, assets/models.pack, docs/WORLD.md, game/tests/all/*, tasks/R45-*
Rules: `tasks/LEAD.md` + `tasks/R45-common.md` (what is true today, decisions, "Shared names": follow them EXACTLY).

## Phase 0 (lead)
- Bake the current models once in the worktree: `cargo run --release -p baker` (under the flock). `assets/models.pack` is not in git.
- FPS before (metro ONLY): `tools/perf_scenes.sh --set metro --seconds 20 --bin target/play/rotland --out /tmp/r45/perf_before.txt`.
- Shots before (optional, hour 12): `--metro=core` -> `/tmp/r45/before_core.png`.

## Tasks (6)
| Task | What | Run |
|---|---|---|
| `R45-01-park-stadium-models.md` | Blender: 12 park and stadium models; `build_r45_models.py` ends "R45 BUILD OK" | 1st wave, at once |
| `R45-02-park-placement.md` | `world/src/metro/park.rs`, `data/park.json`, park loot tables `metro_park` and `metro_stadium` in `data/loot_tables.json`, tests | 1st wave, at once |
| `R45-03-stadium-placement.md` | `world/src/metro/stadium.rs`, `data/stadium.json` (stands, gates, pitch, relief camp, crates), tests | 1st wave, at once |
| `R45-04-horde-and-park-calm.md` | `game_zombie/src/stadium_horde.rs` (timetable, split counts, gates), `data/stadium_horde.json`, `park_calm` in `metro_spawn.rs` and `spawning.json`, tests | 1st wave, at once |
| `R45-05-wiring.md` | chunk contents, danger for park and stadium, the horde's real zombies (`stadium_horde_play.rs`), props rules, stats keys | 2nd wave, after 01-04 merged |
| `R45-06-flags-map-docs-checks.md` | flags `park`, `park_pond`, `stadium`, `stadium_in`, `stadium_night`; map labels "Park" and "Stadium"; docs; then the checks (run by the lead) | 3rd wave, after 05 merged |

Start 01-04 at once (4 subagents, the T3 limit). Each one writes its `mod` / `pub use` lines into Worker notes; the lead adds them to shared files (see `R45-common.md`, "Lines for the lead"). 05 and 06 use names from 02-04, so they start only after those are merged.

## After each merge (lead)
- Read each task's Worker notes. Check the premises in `R45-common.md` still hold (the park is 3x3 reserved blocks, the horde takes the hour not the day, `park_calm` is new).
- Run ONE `cargo build`, `cargo test`, `cargo clippy --all-targets` under the flock after the last wave, not after each task.
- `flock /tmp/rotland-heavy.lock` for every game run. Build the owner binary: `cargo build --profile play`.
- Stale build after merging helper worktrees: `touch` the changed file and rebuild (R43.5 lesson).

## Checks after the gate (`target/play/rotland`, `--save-dir=/tmp/r45-saves`, pictures to `/tmp/r45/`)
These are written in full in `R45-06-flags-map-docs-checks.md` (section "Checks after the gate"). Summary:
- **Pictures, one per flag:** `park`, `park_pond`, `stadium`, `stadium_in`, `stadium_night`, plus `--map` with the "Park" / "Stadium" labels. Look at every one.
- **Horde by day and night (stats):** hour 12 = 30 inside / 0 outside; hour 23 = 0 inside / 30 outside with real zombies near the player; hour 6 = returning (between). Zombies never above `hardCap` (60).
- **Park danger lower than the street (stats):** `metroDensity` in the park (`--metro=park`) is lower than on the avenue beside the stadium (`--metro=stadium`); `inPark` true / false.
- **FPS (metro only):** `tools/perf_scenes.sh --set metro --seconds 20 --bin target/play/rotland --out /tmp/r45/perf_after.txt`. Pass = **average FPS at least 20**. The 1% low is for information only (do not report it as a pass or fail).
- **Tests:** `cargo test` passes; the count is more than R44's 1579.
- Nothing floats, sinks or z-fights; the ground stays green; the horde does not exceed the zombie cap.

## Owner questions (Claude defaults, owner may change: write them in the summary, do not ask mid-round)
- The park covers the whole reserved 3x3 area (about 192 m), not 2x2 as the brief said. A 2x2 park would change `metro.json` `reserved` and so the city layout.
- The horde is 30 members, leaves 20:00-22:00, walks back 05:00-07:00 (`stadium_horde.json`).
- The park is calm (0.4 x the zombie density, specials -0.05).

## Lead notes
(The lead fills this after the checks: each task done or not, the last 5 lines of the final gate, before/after FPS (average only), picture paths, the stats results, and anything Claude must check.)
