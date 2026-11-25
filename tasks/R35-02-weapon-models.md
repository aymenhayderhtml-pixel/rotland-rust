# R35-02: models + icons for the new weapons

Owns: tools/blender/weapons/revolver.py, tools/blender/weapons/hunting_rifle.py, tools/blender/weapons/crossbow.py, tools/blender/weapons/crossbow_bolt.py, tools/blender/weapons/grenade.py, tools/blender/weapons/build_weapons.py, assets/models/weapons/*, tools/blender/icons/*, assets/icons/*
Rules: `tasks/R35-common.md`. Also use the `blender-art` skill rules if you have them. Copy the style of `tools/blender/weapons/pistol.py`, `rifle.py` and `bow.py`: same scale, same palette approach, the same origin and grip point convention (the grip at the origin, the barrel pointing the same way as `pistol.py`).

Make low-poly models (each under ~600 triangles):
- **revolver:** pistol-sized, round 6-shot cylinder, wooden grip, long barrel.
- **hunting_rifle:** wooden stock, bolt handle, a scope on top. Longer than `rifle`, about 1.15 m.
- **crossbow:** stock like a short rifle, a bow arm across the front (~0.6 m wide), and a string. About 0.75 m long.
- **crossbow_bolt:** a thin shaft with small fins (copy the `arrow` model approach, 0.4 m).
- **grenade:** round green body with a ridged look, a lever and a pin ring (~0.1 m).
Add them to `build_weapons.py`. Also add inventory icons the same way the other weapon icons are made (find the icon script in `tools/blender/icons/`). Icons are needed for `revolver`, `hunting_rifle`, `crossbow`, `crossbow_bolt`, `grenade` and `revolver_ammo`.

**Lead builds ONCE (subagent doesn't):**
```bash
flock /tmp/rotland-blender.lock blender -b --python tools/blender/weapons/build_weapons.py > /tmp/r35-weapons.log 2>&1; grep -c Traceback /tmp/r35-weapons.log   # 0
```
Then run the icon build (whatever command `tools/blender/icons/` uses), and look at `assets/models/_previews/` pictures of the 5 new models.
