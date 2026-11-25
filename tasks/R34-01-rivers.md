# R34-01: long rivers

Owns: world/src/rivers/path.rs, world/src/rivers/mod.rs, data/rivers.json
Files: world/src/rivers/path.rs, world/src/rivers/mod.rs, data/rivers.json
Read only: world/src/rivers/path.rs, world/src/rivers/mod.rs, data/rivers.json

Problem: on the world map the 3 rivers are short, curly loops (~300-600 m) instead of rivers several km long. `trace_river` follows downhill and gets stuck circling in dips.

Fix:
1. The river must make progress toward `target_pos` every step: blend the downhill choice with the target direction (raise `target_pull` in data, e.g. 0.6) and never let the heading turn more than `max_turn_deg` (new data key, e.g. 12) per step.
2. Stop when within `step_m * 2` of the target, or when it would enter `safe_radius` (then slide along the circle, as today).
3. Water level must never go UP along the river: keep `water_levels[i] = min(previous, local)`; carve the bed so it is always below the water level.
4. Tests (in mod.rs tests or world/tests): each river with seed 42 is at least 2500 m long; the end point is within 100 m of its target; the path never self-crosses (check segment pairs, skip neighbours); water levels never rise.
