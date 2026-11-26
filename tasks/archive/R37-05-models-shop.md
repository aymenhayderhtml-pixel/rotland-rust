# R37-05: gunsmith bench + town gun shop models

Owns: tools/blender/props/gunsmith_bench.py (new), tools/blender/buildings/gun_shop.py (new), tools/blender/build_r37_models.py (new: builds ONLY R37 models: these 2 + R37-06's 9 attachments + icons)
Read: tasks/R37-common.md, tools/blender/building_parts/ (base_workbench), tools/blender/buildings/house_small.py, desert_store.py, kit.py, tools/blender/props/crate_loot.py, tools/blender/common.py, palette.py, tools/blender/build_detail_props.py (copy its runner), tools/blender/icons/ (how icons are rendered). Never run blender.

- `gunsmith_bench`: a sturdy workbench (~2.2 × 0.9 m), vise, scattered tools, a wall rack with 3 rifles silhouettes, a hanging lamp, ammo boxes; 600-1500 tris.
- `gun_shop`: one-storey shop, walk-in exactly like house_small (door size, floor height, wall thickness), big crossed-guns sign on the front, barred window, counter inside, wall racks; neutral colours so it fits all 4 biomes; 1500-3000 tris.
- Add `gun_shop` footprint to the town data via the lead (report the line for `data/towns.json` footprints).

## Worker notes
### Files changed (all new, only in `/home/aymen/Documents/rotland-r37`)
- `tools/blender/props/gunsmith_bench.py` (162 lines, `CATEGORY = "props"`, `BUDGET = 1500`, estimated ~1,060 tris; Sheet A palette only):
  - 2.20 × 0.90 m timber & steel-lip workbench (`z = 0.0..0.88`), lower storage shelf, heavy front-left vise, rear wall rack (`z = 0.88..2.22`) with 3 horizontal rifle silhouettes (hunting, assault, shotgun), overhead hanging lamp (`pine_dark` shade + `brass` bulb), gun cleaning mat, disassembled receiver/barrel, oil can, ball-peen hammer, screwdrivers, wrenches, brass parts tray, and ammo boxes on tabletop and lower shelf.
  - Two box colliders (`bench`, `rack`); empties `interact_1` at `(0.0, 0.90, 0.0)` (`rot_z = pi`) and `lot_size` (`scale = (2.6, 1.8, 1.0)`).
- `tools/blender/buildings/gun_shop.py` (167 lines, `CATEGORY = "buildings"`, `BUDGET = 3000`, estimated ~1,950 tris; Sheet A neutral colors `rock_light`/`white`/`rock_mid`/`rock_dark`/`metal`/`gun_black`/`bark_dark`/`wood_crate` so it fits desert, snow, swamp, and mountains):
  - 8.0 × 5.8 m walk-in body (`x: -4.0..4.0`, `y: -3.5..2.3`) + 1.2 m front porch (`y: 2.31..3.5`), `F = 0.1` (`kit.FLOOR_TOP`), `DOOR_H = 2.2`, `0.15 m` walls (`kit.wall`), `TOP = 2.6`, gable roof + porch roof on 3 posts.
  - Doorway `1.2 m` wide (`x: -2.0..-0.8`, `y: 2.15..2.30`) with `door_1` empty and zero doorway colliders.
  - Barred front and left windows (steel vertical + horizontal bars offset in depth to avoid z-fighting), 2 cm proud door/window frames, and a bold crossed-guns sign on the front porch fascia (`y: 3.42..3.65`, two crossed rifles separated in Y to avoid z-fighting).
  - Built-in interior: counter (`x: -3.0..1.2`, `y: -1.80..-1.10`, top `z = 1.11`) with register & ammo box, back wall gun rack (4 horizontal rifles), right wall gun rack (3 horizontal rifles), left-back ammo shelf, and 2 crates.
- `tools/blender/build_r37_models.py` (108 lines):
  - Builds ONLY R37 models (`gunsmith_bench`, `gun_shop`, and R37-06's `tools/blender/weapons/attachments.py`), renders ONLY the 9 attachment icons (`scope_2x`, `scope_4x`, `red_dot`, `suppressor`, `ext_mag`, `drum_mag`, `foregrip`, `stock_pad`, `bolt_quiver`), and writes `assets/models/_previews/contact_sheet_r37.png`. Does not modify `palette.py` or overwrite sheets A/B/C.

### Exact lines / info for Lead & R37-07 (`data/towns.json`)
- Add to `data/towns.json` under `"footprints"`:
  ```json
  "gun_shop": [6.0, 5.0]
  ```
  (Building outer bounds are 8.0 × 7.0 m -> half-extents 4.0 × 3.5 m, plus ~1.5–2.0 m town lot margin matching `bait_shop` and `desert_store`.)
- `gun_shop` building-local coords (exported `.glb` origin is centered at `(0, 0, 0)` with ~0.02 m Y offset from the rear eave overhang; exact empties are baked into `gun_shop.glb`):
  - **Door (`door_1` empty)**: Blender `(-1.40, 2.225, 0.10)` / Godot `(x = -1.40, y = 0.10, z = -2.225)`, width `1.20 m` (`x: -2.00..-0.80`), floor height `0.10 m`.
  - **Shopkeeper behind counter (`npc_1` empty)**: Blender `(-0.90, -2.45, 0.10)` (`rot_z = 0.0`, facing `+Y` toward the counter/door) / Godot `(x = -0.90, y = 0.10, z = 2.45)` facing `-Z` (`yaw = 0.0`), behind counter (`x: -3.0..1.2`, Blender `y: -1.80..-1.10` / Godot `z: 1.10..1.80`).
  - Note: `gun_shop` already includes its counter, gun racks, ammo shelf, and crates in the `.glb`, so do NOT add `gun_shop` to `interiors.buildings` in `data/towns.json` (which would spawn random house furniture over the counter/aisle).

### How to build and see it
1. Once R37-06 (`tools/blender/weapons/attachments.py` + icon builders) is present, run:
   `blender -b --python tools/blender/build_r37_models.py`
   (Or run `blender -b --python tools/blender/props/gunsmith_bench.py` and `blender -b --python tools/blender/buildings/gun_shop.py` individually to inspect `assets/models/_previews/gunsmith_bench.png`, `gun_shop.png`, and `gun_shop_top.png`.)
2. Bake into `assets/models.pack`: `cargo run --release -p baker`.

### Unsure / notes
- `blender` was not run per subagent rules; only `python3 -m py_compile` was run on the 3 new Python files (passed with 0 errors). Exact triangle counts (~1,060 for `gunsmith_bench`, ~1,950 for `gun_shop`) are calculated from geometry and should be confirmed in Blender output.
- `build_r37_models.py` imports `attachments` from `tools/blender/weapons/attachments.py` and calls `attachments.main()`. For the 9 attachment icons, it checks `item_meshes.BUILDERS` first, then `items.json` (`weaponId`), then `attachments.BUILDERS` / `attachments.build_<id>` as a fallback.
