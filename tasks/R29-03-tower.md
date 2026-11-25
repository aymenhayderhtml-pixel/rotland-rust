# R29-03: camp guard towers are solid (issue 003)

Owns: tools/blender/buildings/settlement_parts.py, assets/models/buildings/settlement.glb
Rules: `tasks/R29-common.md`. Issue: `docs/ISSUES.md` row 003.

**Goal:** the player walks straight through the camp guard tower legs and rails. Make them solid, but keep the ladder climbable.
- In `guard_platform()` the deck is already a `k.solid` (has a collider). The 4 legs and the rails are plain `k.box` (no collider). Make the legs and the rails solid (`k.solid(..., base="platform")` or `k.box` + `k.collider` with the same box).
- **Don't block the ladder.** The ladder sits on the camp side, 0.15 m in front of the back rail, and the back rail has a gap where the ladder comes up. Keep that gap open and put no collider on the ladder itself. Check that no new collider box overlaps the ladder column (x = mid ± 0.5, y from y0 − 0.4 to y0).
- Don't change how anything looks, and don't change `settlement.py` (the owner's work).

**Lead rebuilds the model ONCE (subagent doesn't):**
```bash
flock /tmp/rotland-blender.lock blender -b --python tools/blender/buildings/build_buildings.py > /tmp/r29-blender.log 2>&1
git status --short assets/models          # it rebuilds ALL buildings
git checkout -- $(git diff --name-only assets/models | grep -v 'buildings/settlement.glb')   # keep only settlement.glb
```
Check the log has no `Traceback`, and that the log line for settlement shows more colliders than before.
