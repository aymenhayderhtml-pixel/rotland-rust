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
- Implemented all 15 animation clips in `tools/blender/characters/poses_hurt.py`:
  - `hit_front`: 0.35 s, loop=False (snappy backward flinch and recovery)
  - `hit_back`: 0.35 s, loop=False (forward torso hunch flinch and recovery)
  - `hit_left`: 0.35 s, loop=False (rightward body jolt and recovery)
  - `hit_right`: 0.35 s, loop=False (leftward body jolt and recovery, mirrored)
  - `hit_head`: 0.40 s, loop=False (head snaps back, right hand clutches face, dazed recovery)
  - `stagger_back`: 0.90 s, loop=False (two stumble steps backward, low balance catch crouch, standing recovery)
  - `knockdown`: 1.00 s, loop=False (knocked onto the back, lands and ends lying flat on floor)
  - `get_up`: 1.40 s, loop=False (from lying on the back, rolls and props on hand, feet tuck under into squat, rises to standing)
  - `death_forward`: 1.30 s, loop=False (knees buckle, drops forward, lands and ends lying flat on stomach, arms limp, cheek on floor)
  - `death_back`: 1.30 s, loop=False (falls backward, lands and ends lying flat on back on floor)
  - `death_left`: 1.30 s, loop=False (twist and side fall, lands and ends lying flat on left side on floor)
  - `death_right`: 1.30 s, loop=False (twist and side fall, lands and ends lying flat on right side on floor, mirrored)
  - `downed_idle`: 2.00 s, loop=True (lying half on side, propped on left arm, breathing cycle with chest rise/fall, seamless loop)
  - `downed_crawl`: 1.40 s, loop=True (crawling on stomach dragging legs, alternating arm reaching and pulling, seamless loop)
  - `revive_get_up`: 1.60 s, loop=False (from downed propped pose, rolls to all-fours, crouches, stands slowly clutching ribs, upright recovery)
- Built `human_player` in Blender and inspected all 15 preview contact sheet strips (`assets/models/_previews/human_player_<clip>.png`):
  - Ground contact verified: all 4 death clips (`death_forward`, `death_back`, `death_left`, `death_right`) and `knockdown` end lying flat ON the ground with no limbs penetrating or floating.
  - Joint angles verified: knee flexion is negative (no joints bent backwards), elbows natural, loops match at 0.0 and end.
- `tools/gate_checks.py`: PASS (scope, numbers, unused).

