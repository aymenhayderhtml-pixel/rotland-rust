# R17 common rules (0.2 push, day 1 afternoon: models + first wiring)

Read `tasks/R13-common.md` (Blender lanes and Rust lanes rules apply) and `docs/PLAN_0.2_PUSH.md`.
- Blender: only `flock /tmp/rotland-blender.lock blender -b --python <script> [-- --only=...]`. Look at every preview PNG you make and fix what looks wrong. Low-poly, flat palette colors (existing names in `tools/blender/palette.py` only), real-world sizes, origin on the ground (z=0) unless the task says otherwise.
- Don't edit shared build lists (`build_buildings.py`, `build_world_props.py`, `build_weapons.py`' list, `build_all.py`, `palette.py`, `common.py`, `kit.py`) unless your task's `Files:` names them. Claude bakes the model pack after the round.
- Rust lanes: `tools/gate.sh --task <task>` until `GATE: PASS`. The gate now rebuilds our own crates first, so expect ~1-2 min extra; wait for it. Never wrap it in `flock` yourself.
- Don't edit your task's `Files:` line. Commit once at the end.
