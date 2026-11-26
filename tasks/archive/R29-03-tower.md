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

## Worker notes

- **Files changed:** `tools/blender/buildings/settlement_parts.py` (`guard_platform()`, lines 58-72). No shared Rust files or `settlement.py` / `settlement.glb` touched.
- **Exact colliders added per `guard_platform(k, x0, x1)`** (`y0 = 16.4, y1 = 19.1, top = 2.7, mid = (x0 + x1) / 2`):
  Replaced `k.box(...)` with `k.solid(..., base="platform")` (exact same visual coordinates/colors + matching `-convcolonly` collider):
  1. 4 corner legs (`z: 0.0..2.5`): `(lx, ly, 0.0)` to `(lx + 0.2, ly + 0.2, 2.5)` for `lx in (x0, x1 - 0.2)`, `ly in (y0, y1 - 0.2)`.
  2. West rail (`z: 2.7..3.6`): `(x0, y0, top)` to `(x0 + 0.08, y1, top + 0.9)`.
  3. East rail (`z: 2.7..3.6`): `(x1 - 0.08, y0, top)` to `(x1, y1, top + 0.9)`.
  4. Back rail left (`z: 2.7..3.6`): `(x0, y0, top)` to `(mid - 0.5, y0 + 0.08, top + 0.9)`.
  5. Back rail right (`z: 2.7..3.6`): `(mid + 0.5, y0, top)` to `(x1, y0 + 0.08, top + 0.9)`.
  Total added: 8 colliders per platform × 2 platforms (`(-8.6, -6.0)` and `(6.0, 8.6)` in `settlement.py`) = **16 new colliders** (18 total for the two towers including the 2 existing deck colliders).
- **Ladder column clearance math** (ladder column `x in [mid - 0.5, mid + 0.5]`, `y in [y0 - 0.4, y0] = [16.0, 16.4]`):
  - Platform 1 (`x0 = -8.6, x1 = -6.0, mid = -7.3`): ladder column `x in [-7.8, -6.8]`, `y in [16.0, 16.4]`.
  - Platform 2 (`x0 = 6.0, x1 = 8.6, mid = 7.3`): ladder column `x in [6.8, 7.8]`, `y in [16.0, 16.4]`.
  - **Legs:** all have `y >= 16.4` (`[16.4, 16.6]` and `[18.9, 19.1]`) and `x` in `[x0, x0 + 0.2]` or `[x1 - 0.2, x1]` (`[-8.6, -8.4]` / `[-6.2, -6.0]` and `[6.0, 6.2]` / `[8.4, 8.6]`), leaving 0.6 m lateral clearance from `mid ± 0.5` and zero Y overlap with `[16.0, 16.4)`.
  - **West/East rails:** `y >= 16.4`, `z >= 2.7`, `x` in `[x0, x0 + 0.08]` or `[x1 - 0.08, x1]` (0.72 m clear of `mid ± 0.5`).
  - **Back rail segments:** `y in [16.4, 16.48]`, `z in [2.7, 3.6]`, `x <= mid - 0.5` or `x >= mid + 0.5` — the 1.0 m ladder gap `[mid - 0.5, mid + 0.5]` remains completely open and clear of colliders.
  - **Ladder:** `k.ladder(mid, y0 - 0.15, top)` stays visual-only (no collider).
- **Shared files for lead:** none. Lead needs to rebuild `assets/models/buildings/settlement.glb` per the instructions above.
- **How to verify in game:** In the survivor camp (`settlement`), walk into the guard tower legs and up the ladder onto the deck (`[E] Climb ladder`); the legs and upper railings block walking through, while the ladder at the back rail gap remains climbable onto the deck.
- **Not run here:** `blender`, `cargo`, `tools/gate.sh`, and the game were not run per R29 subagent rules.
