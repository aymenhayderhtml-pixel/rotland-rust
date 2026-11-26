# R27 common rules (6 Gemini lanes, 2026-11-24)

You work in your own git worktree on your own branch. Read `AGENTS.md`, then `docs/CODEMAP.md` once, then only the files your task needs.
- Edit ONLY files matching your task's `Files:` line (`dir/*` = any file in that folder, new files included). A change you need elsewhere: write it exactly under `## Worker notes` → "Wiring notes for Claude".
- **Animation / feel only where the task says so. Never change gameplay numbers.** Port, don't invent: numbers go in a `data/*.json` file; every number in Rust code gets a tag `// data: <file> <key>` or `// new: DECISIONS 2026-11-24`.
- Files under ~300 lines. Pure logic in `game/` or `humanoid/` with unit tests. `game` tests go in `game/tests/all/` with `mod <name>;` in `game/tests/all/main.rs`; `humanoid` tests sit next to the module like `humanoid/src/action_anim_tests.rs`.
- New clip tables use EXACTLY this format (copy it; a different format crashed the model build last time):
```python
CLIPS = {
    "clip_name": {
        "length": 0.6,
        "loop": False,
        "frames": {
            0.0: {"spine": (0, 0, 0), "chest": (0, 0, 0), "upper_arm_R": (10, 0, 0), "forearm_R": (40, 0, 0)},
            0.3: {"spine": (-8, 10, 0), "chest": (-6, 12, 0), "upper_arm_R": (80, 0, 20), "forearm_R": (20, 0, 0)},
            0.6: {"spine": (0, 0, 0), "chest": (0, 0, 0), "upper_arm_R": (10, 0, 0), "forearm_R": (40, 0, 0)},
        },
    },
}
```
  Signs: spine/chest/head X negative = bend forward; upper_arm X positive = arm forward/up; forearm/shin bend as in `tools/blender/characters/poses_combat.py` and `poses_locomotion.py`. Register new clip files in `tools/blender/characters/poses.py` (player-only clips in `PLAYER_ONLY_CLIPS`).
- After changing any `tools/blender/characters/*` file, rebuild once: `flock /tmp/rotland-blender.lock blender -b --python tools/blender/characters/build_characters.py > /tmp/<task>-blender.log 2>&1`, then check the log has NO `Traceback` and has a `CHARACTER human_player` line, and look at the new clip's contact sheet in `assets/models/_previews/human_player_<clip>.png`.
- Build/test: ONLY `tools/gate.sh --task tasks/<your task>.md`. Run it in the FOREGROUND and wait for it (it can take several minutes because it queues behind other lanes; that is normal: do not stop, do not end your turn while it runs). Loop until the last line is `GATE: PASS`. If the ONLY failing step is `scope` and it lists files you were told to touch, say so in Worker notes and continue.
- DO NOT touch `tools/blender/buildings/settlement.py`, `assets/models/buildings/settlement.glb`, `tools/blender/vehicles/car_parts.py`, `assets/models/vehicles/car_hatchback_*`.
- Never `pkill -f`. Never write to real save folders (always `--save-dir=/tmp/<task>-saves` if you run the game; normally you don't).
- Done = gate PASS, `## Worker notes` filled (what changed, how to see it in the game, anything unsure), ONE commit to your branch. Never push, never touch `main`.
