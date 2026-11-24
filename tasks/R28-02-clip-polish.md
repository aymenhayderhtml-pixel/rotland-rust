# R28-02: fix the weak animation clips

Files: tools/blender/characters/poses_*.py, tools/blender/characters/poses.py, assets/models/characters/*.glb, assets/models/_previews/*, tasks/R28-02-clip-polish.md
Rules: `tasks/R27-common.md` (Blender lane: no cargo, no gate; build with the character build command in R27-common; scope check: `python3 -I tools/gate_checks.py --base main --task tasks/R28-02-clip-polish.md`).

**Goal:** these clips read weakly or wrongly in their contact sheets (`assets/models/_previews/human_player_<clip>.png`): `ladder_climb`, `limp_walk`, `hold_side`, `downed_crawl`. Also review the clips added today and fix any that look wrong: `stab_a`, `stab_b`, `melee_1h_b`, `melee_1h_c`, `melee_2h_b`, `melee_2h_c`, `heavy_overhead`, `spear_thrust`, `shove`, `block_1h`, `block_2h`, `block_hit`, the death clips in `poses_death.py`, and the wound clips.
- Fix a clip by writing a better version in a new `poses_polish.py` merged LAST in `poses.py` (same dict it belongs to: shared `CLIPS` or `PLAYER_ONLY_CLIPS`), so it overrides the old one. Keep each clip's name, length and loop flag (game code depends on them; impact times for melee clips are in `data/melee_moves.json`: the strike must land at that time).
- Judge each clip from its contact sheet: clear silhouette from behind and from the side, no limbs through the body, no knees or elbows bending the wrong way, feet on the ground where they should be.
- In Worker notes, list every clip you checked: OK or fixed (and what was wrong).
