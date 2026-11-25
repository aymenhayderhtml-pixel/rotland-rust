# R33-08: mountain and swamp buildings, bridges, signpost, boardwalk

Owns: tools/blender/buildings/{radio_tower,ranger_station,stilt_shack,bait_shop}.py (new), tools/blender/props/{bridge_wood,bridge_concrete,signpost,boardwalk_section}.py (new), tools/blender/build_biome_props.py (only adding the 4 props to MODELS)
Files: tools/blender/buildings/radio_tower.py, tools/blender/buildings/ranger_station.py, tools/blender/buildings/stilt_shack.py, tools/blender/buildings/bait_shop.py, tools/blender/props/bridge_wood.py, tools/blender/props/bridge_concrete.py, tools/blender/props/signpost.py, tools/blender/props/boardwalk_section.py, tools/blender/build_biome_props.py
Read: tasks/R33-round.md, tasks/LEAD.md, docs/WORLD.md, tools/blender/buildings/cabin.py, tools/blender/buildings/house_small.py, tools/blender/buildings/watchtower.py, tools/blender/buildings/kit.py, tools/blender/buildings/fish_shack.py, tools/blender/props/dock_section.py, tools/blender/props/road_sign.py, tools/blender/build_biome_props.py, tools/blender/common.py, tools/blender/palette.py
Overlap: `tools/blender/buildings/build_town_buildings.py` (R33-07) lists your 4 buildings by name. Do not write it: just keep your file names exactly `radio_tower`, `ranger_station`, `stilt_shack`, `bait_shop`.
Rules: `tasks/LEAD.md` subagent rules (never run blender). Use the "R33 shared names" in `tasks/R33-round.md` EXACTLY. Copy the style of `tools/blender/buildings/cabin.py`, `watchtower`, `kit.py`, `fish_shack.py` (swamp-like), and for props `tools/blender/props/dock_section.py`, `road_sign.py`, `common.py`. Palette sheets A/B/C. R33-07 writes `build_town_buildings.py`, which lists your 4 buildings: just make sure your file names match.

Buildings (walk-in ones copy the door/floor/wall sizes of `house_small`, so loot spots and colliders work):
- `radio_tower`: a 25 m lattice mast (4 legs narrowing upward, cross braces made of thin boxes), a red/white top section, a small equipment hut at the base. The mast is not enterable; the hut is.
- `ranger_station`: a 2-storey wooden lookout lodge with a wrap-around balcony on the upper floor, a stair, and a sign board. Walk-in on the ground floor.
- `stilt_shack`: a small wooden shack on 1.2 m stilts with steps, a tin roof (rust/metal), and a porch. Walk-in.
- `bait_shop`: a shack with a big sign board, a porch, crates and a fridge box out front. Walk-in.

Props:
- `bridge_wood` (≤ 400 tris): a 12 m × 5 m plank deck (top at local y = 0.5), with 2 log railings and 4 posts per side. Origin at the deck centre at ground level, deck along local +Z/-Z (say which axis in your notes).
- `bridge_concrete` (≤ 300): a 16 m × 9 m slab deck (top at y = 0.6), with low concrete side walls and 2 piers underneath.
- `signpost` (≤ 120): a wooden post with 2-3 arrow boards at different angles, 2.5 m tall.
- `boardwalk_section` (≤ 120): a 4 m × 2 m plank walkway on short posts, top at y = 0.4.

## Worker notes
- Added 4 buildings in `tools/blender/buildings/`:
  - `radio_tower.py`: 25 m lattice mast (5 bays narrowing from 3.0 m to 0.7 m, red/white top bays + antenna spire) + walk-in metal equipment hut at `+Y` (`lot_size` 6 × 7 m, `door_1`, `loot_1..2`, `spawn_1`).
  - `ranger_station.py`: 2-storey wooden lookout lodge with exterior stair along `+X`, wrap-around balcony, sign board, upper lookout room, and walk-in ground floor (`lot_size` 11 × 8 m, `door_1`, `loot_1..4`, `spawn_1..2`).
  - `stilt_shack.py`: wooden shack on 1.2 m stilts with 5 front steps, porch, and rust/metal tin roof (`lot_size` 6 × 7.5 m, `door_1`, `loot_1..2`, `spawn_1..2`).
  - `bait_shop.py`: walk-in swamp bait shop with porch, big front sign board, crates, and ice/fridge box out front (`lot_size` 7 × 6.5 m, `door_1`, `loot_1..3`, `spawn_1..2`).
- Added 4 props in `tools/blender/props/` and registered them in `tools/blender/build_biome_props.py` (`MODELS`):
  - `bridge_wood.py` (budget 400): 12 m × 5 m plank deck (`GRIP=(0,0,0)`), top at Blender `z = 0.5` (glTF/Godot `y = 0.5`).
  - `bridge_concrete.py` (budget 300): 16 m × 9 m slab deck (`GRIP=(0,0,0)`), top at Blender `z = 0.6` (glTF/Godot `y = 0.6`), side walls, and 2 piers extending to `z = -2.0`.
  - `signpost.py` (budget 120): 2.5 m wooden post with 3 angled arrow boards.
  - `boardwalk_section.py` (budget 120): 4 m × 2 m plank walkway on 4 short posts (`GRIP=(0,0,0)`), top at Blender `z = 0.4` (glTF/Godot `y = 0.4`).
- **Bridge/boardwalk deck axis:** Decks run along Blender `Y` (which exports with `export_yup=True` as **local `+Z / -Z`** in glTF/Godot/Rotland), with width along `X` and height along `Y` (Blender `Z`).
- Not run: `blender`, `cargo`, `tools/gate.sh` (per subagent rules). Note that `radio_tower.py` uses `bmesh`/`mathutils` directly inside `_strut` and has not yet been executed in Blender.
