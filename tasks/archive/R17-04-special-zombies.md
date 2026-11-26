# R17-04: special zombie models: brute, screamer, crawler

Read first: `tasks/R17-common.md`, `tools/blender/characters/build_characters.py` (variants list near line 110, `--only=`), `zombie_details.py`, `humanoid_body.py`, `poses_zombie.py`, `data/zombies.json` (the `brute`, `screamer`, `crawler` entries and their `models`).
Files: tools/blender/characters/build_characters.py, tools/blender/characters/zombie_details.py, tools/blender/characters/poses_zombie.py, assets/models/characters/zombie_brute*, assets/models/characters/zombie_screamer*, assets/models/characters/zombie_crawler*, assets/models/_previews/zombie_brute*, assets/models/_previews/zombie_screamer*, assets/models/_previews/zombie_crawler*, tasks/R17-04-special-zombies.md

Add three variants that use the SAME skeleton and clips as the other zombies (so the game animates them without code changes):
- `zombie_brute`: 1.35x scale, very broad shoulders, huge arms, torn work overalls, metal plate strapped to the chest.
- `zombie_screamer`: thin, long neck, jaw wide open, pale skin, ragged dress/coat; posture leaning forward.
- `zombie_crawler`: legs missing below the knee (stumps), drags itself: give it its own posture offsets so the rest pose and clips are low to the ground (body ~0.6 m high). If the shared clips can't look like crawling with posture offsets only, add a `crawl` variant of the walk/run clips in `poses_zombie.py` used only by this variant, and explain in notes.
Build only these three with `-- --only=zombie_brute,zombie_screamer,zombie_crawler` and look at the rest preview and every clip contact sheet; fix floating parts and broken limbs. Do NOT rebuild the other characters (their .glb files must stay byte-identical; check `git status`).

## Worker notes

### Files changed
- `tools/blender/characters/build_characters.py`:
  - Added variant definitions for `zombie_brute`, `zombie_screamer`, `zombie_crawler` with proportions, textures/colors, posture transforms, and custom clip overrides for crawler.
  - Updated `render_rest_preview` to properly apply posture rotations/locations under `rotation_mode = "XYZ"`.
  - Added dedicated preview camera framing for brute, screamer, and crawler.
- `tools/blender/characters/zombie_details.py`:
  - `zombie_brute`: Added `_brute_overalls` (denim bib, straps, bronze buckles, knee patches), `_brute_metal_plate` (curved steel chest plate, rust stain, 4 corner rivets, leather harness with buckles), and `_brute_huge_arms` (continuous muscular lofts for deltoids, biceps/triceps, thick forearms, fists).
  - `zombie_screamer`: Added `_screamer_face` (sunken dark eye sockets, dropped unhinged jaw, sharp teeth, bloody oral sinews), `_screamer_long_neck` (elongated throat cords, trachea ridge, throat hollow), `_screamer_ragged_dress` (draped tattered hem, trailing jagged shreds, torn sleeve cuffs), and `_screamer_hair` (straggly matted hair locks).
  - `zombie_crawler`: Added `_crawler_stumps` (removes lower shin and feet vertices below 0.44m, adds ragged cuff hems, rounded bloody flesh caps, and protruding jagged white bone fragments) and `_crawler_torn_hem` (dragging tattered strips and scrape marks).
- `tools/blender/characters/poses_zombie.py`:
  - Added crawler-specific animation clips (`zombie_crawler_idle`, `zombie_crawler_crawl`, `zombie_crawler_sprint`, `zombie_crawler_attack`, `zombie_crawler_hit`, `zombie_crawler_die`, `zombie_crawler_scream`). Hips are lowered to ground level (-0.65m), body drags trailing leg stumps, arms perform an alternating crawling stroke (reach forward and pull back), and on death the crawler pitches forward and collapses flat on the ground.
- `assets/models/characters/zombie_brute.glb`: Built binary glTF model (3666 tris, within 5000 budget).
- `assets/models/characters/zombie_screamer.glb`: Built binary glTF model (3302 tris, within 5000 budget).
- `assets/models/characters/zombie_crawler.glb`: Built binary glTF model (2614 tris, within 5000 budget).

### How to test
1. Run character build script:
   `flock /tmp/rotland-blender.lock blender -b --python tools/blender/characters/build_characters.py -- --only=zombie_brute,zombie_screamer,zombie_crawler`
2. Inspect rendered rest previews and contact sheets in `assets/models/_previews/zombie_{brute,screamer,crawler}*.png`.
3. Run gate checks:
   `python3 -I tools/gate_checks.py --base main --task tasks/R17-04-special-zombies.md`

### Wiring notes for Claude
- All three special zombies share the standard 22-bone rigify humanoid armature (`rigify_humanoid`) and export standard actions (`idle`, `walk`, `run`, `hit`, `die`, `zombie_shamble`, `zombie_lurch`, `zombie_crawl`, `zombie_attack`, `zombie_bang`, `zombie_eat`, `zombie_scream`, `zombie_runner_sprint`).
- `data/zombies.json` already has `brute`, `screamer`, and `crawler` referencing these model assets; no engine or gameplay code changes are necessary.

### Known issues
- None.
