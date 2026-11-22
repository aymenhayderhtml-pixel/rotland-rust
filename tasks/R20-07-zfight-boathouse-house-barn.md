# R20-07: stop the flicker in boathouse, house_small, barn (leftover from R19-08)

Read first: `tasks/R20-common.md`, `tasks/R19-08-zfight-camp-lake.md` (same job, same rules and steps: read it fully), `tools/blender/buildings/apartment.py` (fix style: details 2 cm out of the wall).
Files: tools/blender/buildings/boathouse*.py, assets/models/buildings/boathouse*, assets/models/_previews/boathouse*, tools/blender/buildings/house_small*.py, assets/models/buildings/house_small*, assets/models/_previews/house_small*, tools/blender/buildings/barn*.py, assets/models/buildings/barn*, assets/models/_previews/barn*, tasks/R20-07-zfight-boathouse-house-barn.md
Test: (Blender lane: no cargo)

Checker today (`python3 -I tools/find_coplanar.py <glb>`): boathouse 11.51 m2, house_small 8.28 m2, barn 2.96 m2. Goal: OK or under 0.05 m2 each. Work one model at a time: fix, rebuild, re-check, look at the preview, then the next. Don't change sizes, colors, colliders, markers or budgets. If a model gets worse, undo it (`git checkout -- <files>`). Write before/after areas in Worker notes. Commit after each model is done (small commits are fine), so nothing is lost if you run out of time.

## Worker notes
