# R36-02: models + icons for the new weapons

Owns: tools/blender/weapons/revolver.py, tools/blender/weapons/hunting_rifle.py, tools/blender/weapons/crossbow.py, tools/blender/weapons/crossbow_bolt.py, tools/blender/weapons/grenade.py, tools/blender/weapons/build_weapons.py, assets/models/weapons/*, tools/blender/icons/*, assets/icons/*
Rules: `tasks/R36-common.md`. Also use the `blender-art` skill rules if you have them. Copy the style of `tools/blender/weapons/pistol.py`, `rifle.py` and `bow.py`: same scale, same palette approach, the same origin and grip point convention (the grip at the origin, the barrel pointing the same way as `pistol.py`).

Make low-poly models (each under ~600 triangles):
- **revolver:** pistol-sized, round 6-shot cylinder, wooden grip, long barrel.
- **hunting_rifle:** wooden stock, bolt handle, a scope on top. Longer than `rifle`, about 1.15 m.
- **crossbow:** stock like a short rifle, a bow arm across the front (~0.6 m wide), and a string. About 0.75 m long.
- **crossbow_bolt:** a thin shaft with small fins (copy the `arrow` model approach, 0.4 m).
- **grenade:** round green body with a ridged look, a lever and a pin ring (~0.1 m).
Add them to `build_weapons.py`. Also add inventory icons the same way the other weapon icons are made (find the icon script in `tools/blender/icons/`). Icons are needed for `revolver`, `hunting_rifle`, `crossbow`, `crossbow_bolt`, `grenade` and `revolver_ammo`.

**Lead builds ONCE (subagent doesn't):**
```bash
flock /tmp/rotland-blender.lock blender -b --python tools/blender/weapons/build_weapons.py > /tmp/r36-weapons.log 2>&1; grep -c Traceback /tmp/r36-weapons.log   # 0
```
Then run the icon build (whatever command `tools/blender/icons/` uses), and look at `assets/models/_previews/` pictures of the 5 new models.

## Worker notes
- **Files added/changed:**
  - `tools/blender/weapons/revolver.py` — 6-shot revolver (wooden grip, round fluted cylinder, long barrel with underlug & top rib, trigger guard, hammer, brass front sight; `GRIP = (0, 0, 0)`, budget 600, ~392 tris).
  - `tools/blender/weapons/hunting_rifle.py` — ~1.15 m bolt-action hunting rifle (wooden stock & forend, bolt handle + knob, long tapered barrel, scope with blue lenses, brass turrets & rings, `grip_L` empty; `GRIP = (0, -0.14, -0.05)`, budget 600, ~524 tris).
  - `tools/blender/weapons/crossbow.py` — ~0.75 m crossbow with short rifle stock, ~0.60 m wide swept bow limbs, braced string, front foot stirrup, `grip_L` empty (`GRIP = (0, -0.12, -0.05)`, budget 600, ~328 tris).
  - `tools/blender/weapons/crossbow_bolt.py` — ~0.40 m bolt with tip at `(0, 0, 0)` pointing `+Y`, metal tip, dark shaft, 3 red fins, brass nock (`BUDGET = 600`, ~150 tris).
  - `tools/blender/weapons/grenade.py` — ~0.11 m green frag grenade with ridged body, fuse neck, spoon lever, and brass pull pin + ring (`GRIP = (0, 0, 0)`, budget 600, ~396 tris).
  - `tools/blender/weapons/build_weapons.py` — registered `revolver`, `hunting_rifle`, `crossbow`, `crossbow_bolt`, `grenade` in `WEAPONS`.
  - `tools/blender/icons/item_meshes.py` — added `revolver_ammo`, `crossbow_bolt`, and `grenade` (fallback if `grenade` lacks `weaponId` in `data/items.json`) to `BUILDERS`. `revolver`, `hunting_rifle`, `crossbow` (and `grenade` if it has `weaponId`) are rendered directly from their weapon scripts by `render_icons.py`.
- **Blender commands for the lead to run:**
  ```bash
  flock /tmp/rotland-blender.lock blender -b --python tools/blender/weapons/build_weapons.py > /tmp/r36-weapons.log 2>&1; grep -c Traceback /tmp/r36-weapons.log
  flock /tmp/rotland-blender.lock blender -b --python tools/blender/icons/render_icons.py -- --only=revolver,hunting_rifle,crossbow,crossbow_bolt,grenade,revolver_ammo > /tmp/r36-icons.log 2>&1; grep -c Traceback /tmp/r36-icons.log
  ```
  (Or omit `-- --only=...` to rebuild all icons and regenerate `assets/icons/_sheet.png`.)
- **Not run:** Per R36 subagent rules, Blender, `cargo`, and `tools/gate.sh` were not executed. Python syntax/imports should be verified when the lead runs the Blender commands above.
