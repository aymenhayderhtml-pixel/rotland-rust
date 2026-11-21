# R18-07: fix the crawler zombie: it must crawl on its belly

Read first: `tasks/R17-common.md` (Blender rules), `tasks/R17-04-special-zombies.md` + its Worker notes, `tools/blender/characters/poses_zombie.py` (the `zombie_crawler_*` clips), `build_characters.py`, look at `assets/models/_previews/zombie_crawler_run.png` (today it looks like SITTING and waving the arms, not crawling).
Files: tools/blender/characters/poses_zombie.py, tools/blender/characters/build_characters.py, tools/blender/characters/zombie_details.py, assets/models/characters/zombie_crawler*, assets/models/_previews/zombie_crawler*, tasks/R18-07-crawler-fix.md

Make the crawler lie PRONE (chest near the ground, body horizontal, head up looking forward, stumps behind). Crawl/run clips: alternate arm pulls (reach forward, plant the hand, drag the body), small body sway; sprint = faster pulls; attack = lunge up with both arms; die = collapse flat. Check every contact sheet frame: hands touch the ground when planted, nothing goes under the ground, the body faces the direction of travel (same forward axis as the other zombies). Build only `-- --only=zombie_crawler`; other characters must stay byte-identical (`git status`).

## Worker notes

