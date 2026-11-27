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
All models + icons exist within budget and look right. Fill `## Worker notes
- **Scripts created** (`tools/blender/gear/`):
  - `tools/blender/gear/items_r41.py` (`build_all()` + standalone `__main__`)
  - `tools/blender/gear/army_gear.py` (`army_rifle`, `body_armor`, `army_helmet`, `ammo_556`, `army_ration`, `night_vision`, `battery`)
  - `tools/blender/gear/meds_parts.py` (`antibiotics`, `surgery_kit`, `adrenaline_shot`, `gun_parts`, `car_parts`)
  - `tools/blender/gear/crates_r41.py` (`army_crate`, `metro_locker`, `med_cabinet`, `camp_stash`)
- **Generated assets**:
  - Weapon: `assets/models/weapons/army_rifle.glb` (704 tris, budget 3000/6000)
  - Items (11): `assets/models/gear/{body_armor,army_helmet,ammo_556,army_ration,night_vision,battery,antibiotics,surgery_kit,adrenaline_shot,gun_parts,car_parts}.glb` (96-362 tris each, budget 1500)
  - Crates (4): `assets/models/props/{army_crate,metro_locker,med_cabinet,camp_stash}.glb` (84-180 tris each, budget 3000; origin at bottom center, front `+Y`, includes `box` collider)
  - Previews: 16 PNGs in `assets/models/_previews/`
  - Icons: 12 PNGs in `assets/icons/{army_rifle,body_armor,army_helmet,ammo_556,army_ration,night_vision,battery,antibiotics,surgery_kit,adrenaline_shot,gun_parts,car_parts}.png`
- **`army_rifle` mount empties & `weapon_mounts.json` coordinates**:
  - `GRIP = (0.0, 0.0, 0.0)` (pistol grip origin, same convention as `ak47.py` / `smg.py`; `mag` exported as separate node for reload).
  - Blender build coordinates (`+Y` forward, `+Z` up):
    - `grip_L`: `(0.0, 0.27, 0.015)`
    - `mount_optic`: `(0.0, 0.09, 0.120)`
    - `mount_muzzle`: `(0.0, 0.560, 0.074)`
    - `mount_magazine`: `(0.0, 0.115, -0.100)`
    - `mount_grip`: `(0.0, 0.275, 0.036)`
    - `mount_stock`: `(0.0, -0.356, 0.042)`
  - Godot / `data/weapon_mounts.json` coordinates (`(x, z, -y)` matching `ak47` / `smg` entries):
    - `"optic": [0.0, 0.120, -0.09]`
    - `"muzzle": [0.0, 0.074, -0.560]`
    - `"magazine": [0.0, -0.100, -0.115]`
    - `"grip": [0.0, 0.036, -0.275]`
    - `"stock": [0.0, 0.042, 0.356]`
- **Scale notes**:
  - `army_rifle` is ~0.92 m overall length (buttpad `-0.356` to muzzle `+0.560`), matching `ak47` (~1.02 m) and `smg` (~0.60 m) in first-person and world scale.
  - `med_cabinet` is 1.40 m tall (0.70 x 0.30 x 1.40 m).
  - All 16 previews and 12 icons were visually inspected via `read_file`.` (the rifle mount empties, any scale notes).

## Worker notes
- **Scripts created** (`tools/blender/gear/`):
  - `tools/blender/gear/items_r41.py` (`build_all()` + standalone `__main__`)
  - `tools/blender/gear/army_gear.py` (`army_rifle`, `body_armor`, `army_helmet`, `ammo_556`, `army_ration`, `night_vision`, `battery`)
  - `tools/blender/gear/meds_parts.py` (`antibiotics`, `surgery_kit`, `adrenaline_shot`, `gun_parts`, `car_parts`)
  - `tools/blender/gear/crates_r41.py` (`army_crate`, `metro_locker`, `med_cabinet`, `camp_stash`)
- **Generated assets**:
  - Weapon: `assets/models/weapons/army_rifle.glb` (704 tris, budget 3000/6000)
  - Items (11): `assets/models/gear/{body_armor,army_helmet,ammo_556,army_ration,night_vision,battery,antibiotics,surgery_kit,adrenaline_shot,gun_parts,car_parts}.glb` (96-362 tris each, budget 1500)
  - Crates (4): `assets/models/props/{army_crate,metro_locker,med_cabinet,camp_stash}.glb` (84-180 tris each, budget 3000; origin at bottom center, front `+Y`, includes `box` collider)
  - Previews: 16 PNGs in `assets/models/_previews/`
  - Icons: 12 PNGs in `assets/icons/{army_rifle,body_armor,army_helmet,ammo_556,army_ration,night_vision,battery,antibiotics,surgery_kit,adrenaline_shot,gun_parts,car_parts}.png`
- **`army_rifle` mount empties & `weapon_mounts.json` coordinates**:
  - `GRIP = (0.0, 0.0, 0.0)` (pistol grip origin, same convention as `ak47.py` / `smg.py`; `mag` exported as separate node for reload).
  - Blender build coordinates (`+Y` forward, `+Z` up):
    - `grip_L`: `(0.0, 0.27, 0.015)`
    - `mount_optic`: `(0.0, 0.09, 0.120)`
    - `mount_muzzle`: `(0.0, 0.560, 0.074)`
    - `mount_magazine`: `(0.0, 0.115, -0.100)`
    - `mount_grip`: `(0.0, 0.275, 0.036)`
    - `mount_stock`: `(0.0, -0.356, 0.042)`
  - Godot / `data/weapon_mounts.json` coordinates (`(x, z, -y)` matching `ak47` / `smg` entries):
    - `"optic": [0.0, 0.120, -0.09]`
    - `"muzzle": [0.0, 0.074, -0.560]`
    - `"magazine": [0.0, -0.100, -0.115]`
    - `"grip": [0.0, 0.036, -0.275]`
    - `"stock": [0.0, 0.042, 0.356]`
- **Scale notes**:
  - `army_rifle` is ~0.92 m overall length (buttpad `-0.356` to muzzle `+0.560`), matching `ak47` (~1.02 m) and `smg` (~0.60 m) in first-person and world scale.
  - `med_cabinet` is 1.40 m tall (0.70 x 0.30 x 1.40 m).
  - All 16 previews and 12 icons were visually inspected via `read_file`.
