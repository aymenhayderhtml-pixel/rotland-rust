# R17-03: new weapon models + icons (bow, arrow, spear, dagger, hammer, pipe, fire axe)

Read first: `tasks/R17-common.md`, `tools/blender/weapons/build_weapons.py`, `tools/blender/weapons/knife.py`, `axe.py`, `crowbar.py`, `parts.py`, `tools/blender/icons/render_icons.py` (`--only=` renders just some icons), `data/weapons.json` (the new weapons' `size`/grip fields if any).
Files: tools/blender/weapons/bow.py, tools/blender/weapons/arrow.py, tools/blender/weapons/spear.py, tools/blender/weapons/dagger.py, tools/blender/weapons/hammer.py, tools/blender/weapons/pipe.py, tools/blender/weapons/fire_axe.py, tools/blender/weapons/build_weapons.py, tools/blender/icons/item_meshes.py, assets/models/weapons/*, assets/models/_previews/*, assets/icons/bow.png, assets/icons/arrow.png, assets/icons/arrow_broken.png, assets/icons/spear.png, assets/icons/dagger.png, assets/icons/hammer.png, assets/icons/pipe.png, assets/icons/fire_axe.png, tasks/R17-03-weapon-models.md

- One script per weapon in the same style and with the same grip/orientation conventions as the existing ones (look at how `knife.py`/`axe.py` set the grip point and forward axis), budget 600 triangles each (bow 800). Add them to `WEAPONS` in `build_weapons.py` (you own that list this round).
- `bow`: recurve bow with a string; `arrow`: shaft, fletching, metal tip (also used as the flying/stuck arrow, so its origin is at the tip, pointing forward); `spear`: 2 m pole with a lashed blade; `dagger`; `hammer` (claw hammer); `pipe` (steel pipe with tape grip); `fire_axe` (red head, long handle).
- Icons: replace the placeholder PNGs for the 8 new item ids (`arrow_broken` = arrow snapped in two: add it to `item_meshes.py` if it isn't a weapon). Render only those with `-- --only=...`. Check that the icons look like the others (same size, angle, transparent background).
- Then run `python3 -I tools/gate_checks.py --base main --task tasks/R17-03-weapon-models.md` (scope must PASS).

## Worker notes

