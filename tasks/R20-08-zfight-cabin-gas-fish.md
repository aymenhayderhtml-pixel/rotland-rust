# R20-08: stop the flicker in cabin, gas_station, fish_shack (leftover from R19-08)

Read first: `tasks/R20-common.md`, `tasks/R19-08-zfight-camp-lake.md` (same job, same rules and steps: read it fully), `tools/blender/buildings/apartment.py` (fix style: details 2 cm out of the wall).
Files: tools/blender/buildings/cabin*.py, assets/models/buildings/cabin*, assets/models/_previews/cabin*, tools/blender/buildings/gas_station*.py, assets/models/buildings/gas_station*, assets/models/_previews/gas_station*, tools/blender/buildings/fish_shack*.py, assets/models/buildings/fish_shack*, assets/models/_previews/fish_shack*, tasks/R20-08-zfight-cabin-gas-fish.md
Test: (Blender lane: no cargo)

Checker today (`python3 -I tools/find_coplanar.py <glb>`): cabin 2.08 m2, gas_station 2.66 m2, fish_shack 1.17 m2 (last time fish_shack got WORSE: be careful). Goal: OK or under 0.05 m2 each. Work one model at a time: fix, rebuild, re-check, look at the preview, then the next. Don't change sizes, colors, colliders, markers or budgets. If a model gets worse, undo it (`git checkout -- <files>`). Write before/after areas in Worker notes. Commit after each model is done (small commits are fine), so nothing is lost if you run out of time.

## Worker notes
