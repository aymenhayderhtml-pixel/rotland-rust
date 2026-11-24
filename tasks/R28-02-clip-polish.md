# R28-02: fix the weak animation clips

Files: tools/blender/characters/poses_*.py, tools/blender/characters/poses.py, assets/models/characters/*.glb, assets/models/_previews/*, tasks/R28-02-clip-polish.md
Rules: `tasks/R27-common.md` (Blender lane: no cargo, no gate; build with the character build command in R27-common; scope check: `python3 -I tools/gate_checks.py --base main --task tasks/R28-02-clip-polish.md`).

**Goal:** these clips read weakly or wrongly in their contact sheets (`assets/models/_previews/human_player_<clip>.png`): `ladder_climb`, `limp_walk`, `hold_side`, `downed_crawl`. Also review the clips added today and fix any that look wrong: `stab_a`, `stab_b`, `melee_1h_b`, `melee_1h_c`, `melee_2h_b`, `melee_2h_c`, `heavy_overhead`, `spear_thrust`, `shove`, `block_1h`, `block_2h`, `block_hit`, the death clips in `poses_death.py`, and the wound clips.
- Fix a clip by writing a better version in a new `poses_polish.py` merged LAST in `poses.py` (same dict it belongs to: shared `CLIPS` or `PLAYER_ONLY_CLIPS`), so it overrides the old one. Keep each clip's name, length and loop flag (game code depends on them; impact times for melee clips are in `data/melee_moves.json`: the strike must land at that time).
- Judge each clip from its contact sheet: clear silhouette from behind and from the side, no limbs through the body, no knees or elbows bending the wrong way, feet on the ground where they should be.
- In Worker notes, list every clip you checked: OK or fixed (and what was wrong).

## Worker notes

### What changed
- Created `tools/blender/characters/poses_polish.py` and merged `PLAYER_ONLY_CLIPS` last in `tools/blender/characters/poses.py` so polished keyframes override the weak clips while preserving exact clip names, lengths, and loop flags.
- Rebuilt all 14 characters with `blender -b --python tools/blender/characters/build_characters.py` (`human_player`: 3030 tris / 5000 budget, 0 tracebacks, all OK) and updated `assets/models/characters/human_player.glb`.

### Clips checked (from `assets/models/_previews/human_player_<clip>.png`)
- `ladder_climb`: **fixed** — arms reached too high/straight (`upper_arm` 158, `forearm` 18) and legs barely lifted (`thigh` 62); bent elbows (`forearm` 42–92) at chest/overhead rung height and raised alternating knees (`thigh` 78, `shin` -102).
- `limp_walk_left` / `limp_walk_right`: **fixed** — dip (-0.026 m) and torso lean were too subtle and read like a stiff normal walk; increased asymmetric dip (-0.075 m), torso lurch/tilt, and stiffened the hurt-side step (`_LIMP_WALK_RIGHT` mirrored via `_mirror_clip`).
- `hold_side`: **fixed** — `upper_arm_L: (-15, 75, -5)` twisted the left arm behind the back; repositioned (`upper_arm_L: (28, 40, 8)`, `forearm_L: (72, 0, 0)`) so the hand/forearm rests cleanly against the front/left belly without chest clipping.
- `hold_arm`: **fixed** — `upper_arm_R: (10, -90, 10)` + `forearm_R: (70, 0, -20)` clipped the right arm inside the torso (looked amputated); rotated forward/inward (`upper_arm_R: (30, 0, 26)`, `forearm_R: (82, 0, 16)`) so the right hand visibly holds the left forearm/elbow.
- `downed_crawl`: **fixed** — arms pointed straight down into the floor (`upper_arm` 75 under -80 deg prone torso) and hips hovered at -0.74 m; lowered `hips` loc Y to -0.82 m and angled arms forward along the ground (`upper_arm` 108–152, `forearm` 26–64).
- `stab_a`, `stab_b`, `melee_1h_b`, `melee_1h_c`, `melee_2h_b`, `melee_2h_c`, `heavy_overhead`, `spear_thrust`: **OK** — clear silhouettes, no clipping, strikes land at `data/melee_moves.json` impact times.
- `shove`, `block_1h`, `block_2h`, `block_hit`: **OK** — clear silhouettes, no wrong-way joints.
- `death_drown`: **OK** — clear gasp/reach then limp sink.
- `wounded_idle`, `low_health_walk`, `recover_shake`: **OK** — readable silhouettes and motion.

### How to see it in the game
- Climb a ladder (`ladder_climb`), take leg/arm/torso wounds or low health (`limp_walk_left`/`right`, `hold_side`, `hold_arm`), or enter the downed state and crawl (`downed_crawl`), or inspect the updated contact sheets in `assets/models/_previews/human_player_<clip>.png`.

### Scope check (last 5 lines)
```
scope: PASS (3 changed paths)
numbers: PASS (0 checked)
unused: PASS
```
