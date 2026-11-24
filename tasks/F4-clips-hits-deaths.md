# F4 clips: hit reactions and deaths (for jobs A14, A16)

Files: tools/blender/characters/poses_hurt.py, tools/blender/characters/poses.py, tasks/F4-clips-hits-deaths.md

Read `tasks/F-clips-common.md` first. The old `hit` and `die` clips stay. Deaths end lying ON the ground:
check the last frame in the strip; no body part below the ground, nothing floating.
- `hit_front`, `hit_back`, `hit_left`, `hit_right` (no loop, 0.35 s each): body jolts away from the hit side, recovers.
- `hit_head` (no loop, 0.4 s): head snaps back, hand goes up toward the face.
- `stagger_back` (no loop, 0.9 s): two stumbling steps backward, catch balance.
- `knockdown` (no loop, 1.0 s): knocked onto the back, ends lying.
- `get_up` (no loop, 1.4 s): from lying on the back to standing.
- `death_forward` (no loop, 1.3 s): knees buckle, falls face down, arms limp.
- `death_back` (no loop, 1.3 s): falls backward onto the back.
- `death_left`, `death_right` (no loop, 1.3 s): twist and fall to that side.
- `downed_idle` (loop, 2.0 s): lying half on the side, one arm propped up, breathing.
- `downed_crawl` (loop, 1.4 s): crawling on the stomach with arms pulling.
- `revive_get_up` (no loop, 1.6 s): from downed to standing, hurt and slow.

## Worker notes
