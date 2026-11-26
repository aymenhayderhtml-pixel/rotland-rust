# R37 prompt (paste everything inside the box into Hydrogen in Antigravity)

```
You are the LEAD for round R37 of Rotland (/home/aymen/Documents/rotland-rust): THE WEAPON SHOP (gunsmith). QUALITY FIRST: the owner wants this to look and feel great.

First read: AGENTS.md, tasks/LEAD.md, tasks/R37-common.md (owner decisions + "R37 shared names": follow them EXACTLY), tasks/R37-round.md and its 8 task files. Read docs/CODEMAP.md once, and look at how the camp trader works today (game_camp/src/economy/, rotland_ui/src/camp_ui/draw_shop.rs, grep npc_trader) and how weapons are held and fired (rotland/src/shooting*.rs, fp_weapon.rs, player_visual.rs) before you hand out work, so your instructions are precise.

1. Setup from LEAD.md (ONE worktree ../rotland-r37, branch r37, shared CARGO_TARGET_DIR). Only you build; subagents never run cargo, the gate, blender or the game.
2. Start subagents for R37-01 to R37-07 at once. Start R37-08 when 01-04 have reported (give it their Worker notes). Give each the LEAD.md subagent rules, the whole R37-common.md, its task file, and the exact existing files to copy the style of. Ask each for tests and full "## Worker notes".
3. REVIEW each diff yourself: shared names exact, numbers in data files, old saves still load, no half-done stubs, files under ~300 lines. Send work back with clear notes if it is not good enough.
4. Add the shared-file lines. Blender ONCE under `flock /tmp/rotland-blender.lock`: `tools/blender/build_r37_models.py` (0 Traceback), then rebuild the guns whose scripts got mount nodes and check their old geometry did not change. Bake.
5. `tools/gate.sh --task tasks/R37-round.md` in the foreground until GATE: PASS (a scope-only failure about the Files: line is OK; note it). world/tests/golden.rs must pass UNEDITED.
6. `cargo build --profile play`, then every check in "Checks after the gate" in R37-round.md. Look carefully at every picture; fix and re-shoot until right (shop screen readable at 1920x986 and 1280x720, attachments sitting on the guns in both views, the bench and shop look good, shopkeeper behind the counter).
7. Fill "## Lead notes" (each task done or not, what you fixed in review, last 5 gate lines, every picture path, FPS, anything Claude must check). ONE commit on r37 (git add only changed files). Never push, never touch main.
8. Tell me "R37 done" plus a short summary.

Safety: never `pkill -f`; never write to real save folders; heavy commands under `flock /tmp/rotland-heavy.lock`; never add sounds.
```
