# R33 prompt (paste everything inside the box into Gemini in Antigravity)

```
You are the LEAD for round R33 of the Rotland game (Rust repo at /home/aymen/Documents/rotland-rust): TOWNS + ROADS + RIVERS.

First read: AGENTS.md, tasks/LEAD.md, docs/WORLD.md, then tasks/R33-round.md (the owner decisions and the "R33 shared names": follow them EXACTLY), then the ten task files tasks/R33-01..R33-10-*.md.

Where things stand:
- Tasks 01, 02 and 03 (roads, rivers, towns: pure world logic) are already written by Neon. The answers are in neon/in/R33-01.md, neon/in/R33-02.md, neon/in/R33-03.md.
- The main checkout may have a half-applied start of them (world/src/lib.rs edited, world/src/roads/, empty world/src/rivers/ and world/src/towns/). Treat that as unfinished; re-apply cleanly.
- Tasks 04 to 10 are NOT done. You and your subagents write them.

How we work (the laptop is slow, so the game builds only ONCE):
1. Work in the MAIN checkout on a new branch: `git switch -c r33` (keeps the half-applied files). No new worktree.
2. Apply the three Neon answers, in order 01, 02, 03:
   `python3 -I tools/neon_apply.py neon/in/R33-01.md --task tasks/R33-01-roads-core.md` (same for 02 and 03; use `--dry-run` first if unsure).
   Then make world/src/lib.rs match the shared names (mods roads, rivers, towns and their `pub use` lines). Do NOT drop existing exports such as `CityBlock`. Add the data files data/roads.json, data/rivers.json, data/towns.json if the answers did not create them.
3. Start subagents at the same time for tasks 04-10. Suggested split (8):
   a. R33-04 world integration (terrain height carving, paint, chunk contents, bridges, split world/src/generator.rs into small files, add towns()/roads()/rivers()/town_at()/indoors())
   b. R33-05 play wiring (river water drawn + swimmable via water_at, colliders, town banner, signposts, flags --town= --trailhead= --bridge= --river=)
   c. R33-06 maps (minimap + big map + world/examples/worldmap.rs)
   d. R33-07 desert + snow buildings and tools/blender/buildings/build_town_buildings.py
   e. R33-08 mountain + swamp buildings, bridges, signpost, boardwalk
   f. R33-09 town gameplay (loot tables, town zombie density, trapped survivors, guild explore tasks, hints)
   g. R33-10 R32 leftovers (warmth saved, cold slows stamina, indoors warms, short rain streaks near the camera)
   h. Rotland wiring the tasks hand back: `indoors: false` in rotland/src/app/frame_warmth.rs -> generator.indoors(x, z); stamina multiplier passed by its callers; towns passed to the guild in rotland/src/app/frame_guild.rs; the `in_town` spawn flag; `mod river_view;` in rotland main.rs.
   Subagent B, C, F, G, H depend on A's names: give every subagent the "R33 shared names" from R33-round.md word for word, the LEAD.md subagent rules, and its task file. Subagents write code only: no cargo, no gate, no blender, no game. Each writes "## Worker notes" in its task file.
4. When all have reported: add the shared-file lines yourself (game/tests/all/main.rs `mod towns_gameplay_tests;` and `mod warmth_save_tests;`, any lib.rs / main.rs mod lines). Then run `tools/gate.sh --task tasks/R33-round.md` in the foreground and wait. Fix errors and run it again until the last line is GATE: PASS.
   - If the gate fails ONLY on the scope check (it reads the Files: line from main), that is expected for a new round: note it and go on once clippy and tests pass.
   - world/tests/golden.rs must pass UNEDITED (the home land must not change).
5. Blender and bake (one at a time):
   `flock /tmp/rotland-blender.lock blender -b --python tools/blender/buildings/build_town_buildings.py > /tmp/r33-bld.log 2>&1` (0 Traceback)
   `flock /tmp/rotland-blender.lock blender -b --python tools/blender/build_biome_props.py > /tmp/r33-props.log 2>&1` (BUILD OK)
   Old models must not change (`git status --short assets/`: only new files). Then `flock /tmp/rotland-heavy.lock cargo run --release -p baker`.
6. Check pictures (always `--save-dir=/tmp/r33-saves`, put pictures in /tmp/r33/):
   - world map: `cargo run --release -p world --example worldmap -- --seed=42 --size=12000 --px=1200 --out=/tmp/r33/worldmap.png`. LOOK: 4 towns with names, roads from the core edge to each town plus a ring, rivers that never enter the home land.
   - `cargo build --profile play`, then with target/play/rotland take shots: `--town=dusthollow`, `--town=frostmere`, `--town=blackwater`, `--town=ridgeback_station`, `--bridge=0`, `--river=0`, `--trailhead=0`, and camp with no flags (must look like before).
   Look at every picture. Fix anything clearly wrong (buildings floating or buried, a bridge not over water, water through the road, black or missing models).
7. FPS: `tools/perf_scenes.sh --set all --seconds 20 --bin target/play/rotland`. R32 was about camp 135, forest 105. Report the numbers.
8. Fill "## Lead notes" in tasks/R33-round.md: each task done or not, the last 5 lines of the final gate, picture paths, FPS, and anything Claude must check. Then ONE commit on branch r33 (`git add` only the files you changed, never `-a`/`-A`). Never push, never touch main.
9. Tell me: "R33 done" plus a short summary.

Safety: never `pkill -f`; never write to real save folders; heavy commands under `flock /tmp/rotland-heavy.lock`; never add sounds.
```
