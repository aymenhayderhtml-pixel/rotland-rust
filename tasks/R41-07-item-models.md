# R41-07: City item models, crate models and icons

Owns:
- `tools/blender/gear/items_r41.py` (NEW, `build_all()`: models + icons), helper files next to it if needed (`army_gear.py`, `meds_parts.py`, `crates_r41.py`)
- the new .glb files (weapons in `assets/models/weapons/`, items and crates where the existing items/crates live: check `crate_loot` and `medkit`)
- their previews in `assets/models/_previews/`
- the 12 new icons where the existing item icons live

Not owned: `tools/blender/build_r41_models.py` (R41-04 makes it and calls your `items_r41.build_all`), every other script and model, `data/*`.
Read: tasks/R41-common.md ("Item models + icons"), `tools/blender/build_r37_models.py` (how R37 built weapons + icons in one run), `tools/blender/weapons/` (the `ak47`/`rifle` scripts: grip, mount point empties, FP scale), `tools/blender/icons/render_icons.py`, `tools/blender/gear/` (style of carried items), `data/weapon_mounts.json` (mount names).

## Models
- Budgets:
  - items 1500
  - `army_rifle` 6000 (lod1 600)
  - crates 3000 (lod1 300)

  Palette materials only. Same origin/grip rules as the existing items and guns.
- `body_armor`: plate carrier vest with pouches, olive, straps.
- `army_helmet`: rounded combat helmet with a strap and a goggle band.
- `army_rifle`: a generic modern service rifle with a flat-top rail, adjustable stock, 30-round curved magazine and a short foregrip area. It needs the SAME mount empties as the R37 guns (scope rail, muzzle, under-barrel, side) at the right places; list their names and positions in Worker notes so R41-06 writes `weapon_mounts.json` to match. Olive and black. No real brand marks.
- `ammo_556`: a small cardboard box with a few loose rounds.
- `army_ration`: a brown pouch (MRE style) with a printed label block.
- `night_vision`: a head mount with twin tubes and a battery box.
- `battery`: a chunky square battery with a terminal.
- `antibiotics`: a pill bottle + blister pack.
- `surgery_kit`: an orange hard case with a red cross, half open showing tools.
- `adrenaline_shot`: an auto-injector pen, yellow and grey.
- `gun_parts`: a small oily cloth bundle with springs, a bolt and a barrel piece.
- `car_parts`: a box with a spark plug set, a belt and a filter.
- **Crates** (loot containers; origin at the bottom centre, front +Y, a lid or door that reads at a glance):
  - `army_crate`: a long olive footlocker with stencils and latches.
  - `metro_locker`: a dented grey metal locker, door ajar.
  - `med_cabinet`: a white wall cabinet with a red cross. It sits on the floor against a wall, 1.4 m tall.
  - `camp_stash`: a tarp-covered pile of boxes and a cooler.

## Icons
One icon per item (12, every id in R41-06's table), rendered exactly like the existing icons (same size, angle, background, file names = item id).

## Blender run and checks
- Run (you MAY run Blender this way): `flock /tmp/rotland-blender.lock blender -b --python tools/blender/build_r41_models.py`. If R41-04 has not made that file yet, run `items_r41.py` on its own with the same guard pattern R37 used.
- No `Traceback` or `FAILED`.
- Look at every preview and every icon:
  - each item reads at icon size
  - the rifle reads as a rifle from the side and in first person
  - the crates read as containers

## Done when
All models + icons exist within budget and look right. Fill `## Worker notes` (the rifle mount empties, any scale notes).

## Worker notes
