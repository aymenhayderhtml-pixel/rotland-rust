# R18-07: fix the crawler zombie: it must crawl on its belly

Read first: `tasks/R17-common.md` (Blender rules), `tasks/R17-04-special-zombies.md` + its Worker notes, `tools/blender/characters/poses_zombie.py` (the `zombie_crawler_*` clips), `build_characters.py`, look at `assets/models/_previews/zombie_crawler_run.png` (today it looks like SITTING and waving the arms, not crawling).
Files: tools/blender/characters/poses_zombie.py, tools/blender/characters/build_characters.py, tools/blender/characters/zombie_details.py, assets/models/characters/zombie_crawler*, assets/models/_previews/zombie_crawler*, tasks/R18-07-crawler-fix.md

Make the crawler lie PRONE (chest near the ground, body horizontal, head up looking forward, stumps behind). Crawl/run clips: alternate arm pulls (reach forward, plant the hand, drag the body), small body sway; sprint = faster pulls; attack = lunge up with both arms; die = collapse flat. Check every contact sheet frame: hands touch the ground when planted, nothing goes under the ground, the body faces the direction of travel (same forward axis as the other zombies). Build only `-- --only=zombie_crawler`; other characters must stay byte-identical (`git status`).

## Worker notes

### Files changed
- `tools/blender/characters/build_characters.py`:
  - Updated `zombie_crawler` posture definition: lowered `hips` by -0.75m (`(0, -0.75, 0)`), rotated `spine` (-75°, 0, 0) and `chest` (-15°, 0, 0) making the spine/chest horizontal along the +Y forward travel axis close to the ground, lifted `head` (55°, 0, 0) to look forward, rotated `thigh` (-85°, 0, ±10°) so leg stumps trail behind along the ground, bent arms and planted hands on the ground.
  - Updated `clips` dictionary mappings for `zombie_crawler` to map `"zombie_crawl": "zombie_crawler_crawl"` (so `AnimClip::Crawl` uses the prone crawler animation instead of the standing zombie crawl) and `"zombie_eat": "zombie_crawler_eat"`.
  - Updated rest preview camera framing: `target_z = 0.24`, `cam_dist_mult = 0.85` for proper low-angle framing of the prone crawler.
- `tools/blender/characters/poses_zombie.py`:
  - Reworked all `zombie_crawler_*` clips for the prone horizontal body posture:
    - `zombie_crawler_idle`: Prone on belly with subtle breathing motion (spine pitch/yaw) and head scanning.
    - `zombie_crawler_crawl`: Alternate arm pulls (reach forward, plant hand on ground, drag body forward) with rhythmic spine yaw/roll sway and trailing stump shifts. Hands plant firmly at ground height without ground penetration.
    - `zombie_crawler_sprint`: Rapid alternating crawl pulls with aggressive reach, dynamic torso sway, and planted hand pulls.
    - `zombie_crawler_attack`: Lunge up onto planted hands, raising chest and head, followed by a double-handed forward rake down to the ground.
    - `zombie_crawler_hit`: Impact flinch, head recoils back, arms buckle, and body drops back prone.
    - `zombie_crawler_die`: Spine buckles, arms collapse outward/forward, head drops, collapsing flat on the ground.
    - `zombie_crawler_scream`: Rears head and upper chest up on straining arms, screaming upward before dropping back down.
    - `zombie_crawler_eat`: Prone on belly feeding with rhythmic shoulder and head bobbing and tearing hand motions.
- `assets/models/characters/zombie_crawler.glb`:
  - Rebuilt binary glTF model (2614 tris, within 5000 budget), containing all 14 animation actions exported with zero ground penetration (`min_z >= 0.005` across all sampled frames).
- Visual contact sheets in `assets/models/_previews/zombie_crawler*.png` inspected and verified.

### How to test
1. Rebuild crawler model:
   `flock /tmp/rotland-blender.lock blender -b --python tools/blender/characters/build_characters.py -- --only=zombie_crawler`
2. Inspect rendered rest preview and contact sheets in `assets/models/_previews/zombie_crawler*.png`.
3. Run gate checks:
   `python3 -I tools/gate_checks.py --base main --task tasks/R18-07-crawler-fix.md`

### Wiring notes for Claude
- Rebuilding `zombie_crawler.glb` automatically packages all 14 humanoid animation clips properly tailored to the crawler's prone belly posture.
- `build_characters.py` maps `"zombie_crawl": "zombie_crawler_crawl"` and `"zombie_eat": "zombie_crawler_eat"` in addition to standard `idle`, `walk`, `run`, `hit`, `die`, `zombie_attack`, etc., so all game engine clip requests (`AnimClip::Crawl`, `AnimClip::Walk`, `AnimClip::Run`, etc.) play prone crawler animations.
- No Rust game code changes were necessary; models bake normally via `cargo run --release -p baker`.

### Known issues
- None.

