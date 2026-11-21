# R17-03: new weapon models + icons (bow, arrow, spear, dagger, hammer, pipe, fire axe)

Read first: `tasks/R17-common.md`, `tools/blender/weapons/build_weapons.py`, `tools/blender/weapons/knife.py`, `axe.py`, `crowbar.py`, `parts.py`, `tools/blender/icons/render_icons.py` (`--only=` renders just some icons), `data/weapons.json` (the new weapons' `size`/grip fields if any).
Files: tools/blender/weapons/bow.py, tools/blender/weapons/arrow.py, tools/blender/weapons/spear.py, tools/blender/weapons/dagger.py, tools/blender/weapons/hammer.py, tools/blender/weapons/pipe.py, tools/blender/weapons/fire_axe.py, tools/blender/weapons/build_weapons.py, tools/blender/icons/item_meshes.py, assets/models/weapons/*, assets/models/_previews/*, assets/icons/bow.png, assets/icons/arrow.png, assets/icons/arrow_broken.png, assets/icons/spear.png, assets/icons/dagger.png, assets/icons/hammer.png, assets/icons/pipe.png, assets/icons/fire_axe.png, tasks/R17-03-weapon-models.md

- One script per weapon in the same style and with the same grip/orientation conventions as the existing ones (look at how `knife.py`/`axe.py` set the grip point and forward axis), budget 600 triangles each (bow 800). Add them to `WEAPONS` in `build_weapons.py` (you own that list this round).
- `bow`: recurve bow with a string; `arrow`: shaft, fletching, metal tip (also used as the flying/stuck arrow, so its origin is at the tip, pointing forward); `spear`: 2 m pole with a lashed blade; `dagger`; `hammer` (claw hammer); `pipe` (steel pipe with tape grip); `fire_axe` (red head, long handle).
- Icons: replace the placeholder PNGs for the 8 new item ids (`arrow_broken` = arrow snapped in two: add it to `item_meshes.py` if it isn't a weapon). Render only those with `-- --only=...`. Check that the icons look like the others (same size, angle, transparent background).
- Then run `python3 -I tools/gate_checks.py --base main --task tasks/R17-03-weapon-models.md` (scope must PASS).

## Worker notes
- **Files created**:
  - `tools/blender/weapons/bow.py`: recurve bow with central sculpted riser, leather grip, laminated recurve limbs, and taut white bowstring under tension (484 tris, budget 800).
  - `tools/blender/weapons/arrow.py`: metal triangular broadhead tip at origin (0, 0, 0) pointing forward (+Y), sinew binding, cedar shaft, 3-fletch white feathers at 120 degrees, and notched nock (272 tris, budget 600).
  - `tools/blender/weapons/spear.py`: 2 m ash pole with metal heel ferrule, main hand and off-hand leather grips, heavy cord lashing, and leaf-shaped spearhead (540 tris, budget 600).
  - `tools/blender/weapons/dagger.py`: symmetrical double-edged combat dagger with ribbed grip, round pommel, and crossguard (440 tris, budget 600).
  - `tools/blender/weapons/hammer.py`: claw hammer with contoured wooden handle, rubber grip sleeve, steel head eye, cylindrical striking poll with hardened face, and curved split claw with nail-pulling bevel (416 tris, budget 600).
  - `tools/blender/weapons/pipe.py`: heavy galvanized steel pipe with threaded ends, weighted coupling collar, and layered duct tape grip with peeling tape (436 tris, budget 600).
  - `tools/blender/weapons/fire_axe.py`: pick-head fire axe with long fawn-foot wooden haft, rubberized butt grip, fire-engine red head (`red_paint`), flared cutting bit with ground edge, and rear pick with hardened point (380 tris, budget 600).
- **Files updated**:
  - `tools/blender/weapons/build_weapons.py`: registered all 7 new weapons into `WEAPONS` list (total 18 weapons, all pass within budget).
  - `tools/blender/icons/item_meshes.py`: registered `arrow` and `arrow_broken` builders. `arrow` renders the full projectile; `arrow_broken` renders an arrow snapped in two with splintered wood fractures.
  - `assets/icons/*.png`: rendered updated transparent 128x128 icons for `bow`, `arrow`, `arrow_broken`, `spear`, `dagger`, `hammer`, `pipe`, `fire_axe`.
  - `assets/models/weapons/*.glb`: exported glTF models.
- **How to test**:
  - `flock /tmp/rotland-blender.lock blender -b --python tools/blender/weapons/build_weapons.py` (passes with all 18 weapons within budget).
  - `flock /tmp/rotland-blender.lock blender -b --python tools/blender/icons/render_icons.py -- --only=bow,arrow,arrow_broken,spear,dagger,hammer,pipe,fire_axe` (renders all 8 icons).
  - `python3 -I tools/gate_checks.py --base main --task tasks/R17-03-weapon-models.md` (passes scope check).
- **Wiring notes for Claude**:
  - All 7 new weapon glBs are in `assets/models/weapons/`. They will be baked into `assets/models.pack` during the pack bake step.
  - Coordinate conventions match: forward is +Y, vertical is Z.
  - Off-hand grip empties `grip_L` are placed on two-handed weapons: `bow` (0.0, -0.150, 0.0), `spear` (0.0, 0.30, 0.0), `hammer` (0.0, -0.250, 0.0), `fire_axe` (0.0, -0.170, 0.0).
  - `arrow.glb` origin is positioned exactly at the tip pointing forward (+Y) for accurate projectile collision registration.
- **Known issues**: None. All triangle budgets and icon visual checks passed.

