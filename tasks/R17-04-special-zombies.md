# R17-04: special zombie models: brute, screamer, crawler

Read first: `tasks/R17-common.md`, `tools/blender/characters/build_characters.py` (variants list near line 110, `--only=`), `zombie_details.py`, `humanoid_body.py`, `poses_zombie.py`, `data/zombies.json` (the `brute`, `screamer`, `crawler` entries and their `models`).
Files: tools/blender/characters/build_characters.py, tools/blender/characters/zombie_details.py, tools/blender/characters/poses_zombie.py, assets/models/characters/zombie_brute*, assets/models/characters/zombie_screamer*, assets/models/characters/zombie_crawler*, assets/models/_previews/zombie_brute*, assets/models/_previews/zombie_screamer*, assets/models/_previews/zombie_crawler*, tasks/R17-04-special-zombies.md

Add three variants that use the SAME skeleton and clips as the other zombies (so the game animates them without code changes):
- `zombie_brute`: 1.35x scale, very broad shoulders, huge arms, torn work overalls, metal plate strapped to the chest.
- `zombie_screamer`: thin, long neck, jaw wide open, pale skin, ragged dress/coat; posture leaning forward.
- `zombie_crawler`: legs missing below the knee (stumps), drags itself: give it its own posture offsets so the rest pose and clips are low to the ground (body ~0.6 m high). If the shared clips can't look like crawling with posture offsets only, add a `crawl` variant of the walk/run clips in `poses_zombie.py` used only by this variant, and explain in notes.
Build only these three with `-- --only=zombie_brute,zombie_screamer,zombie_crawler` and look at the rest preview and every clip contact sheet; fix floating parts and broken limbs. Do NOT rebuild the other characters (their .glb files must stay byte-identical; check `git status`).

## Worker notes

