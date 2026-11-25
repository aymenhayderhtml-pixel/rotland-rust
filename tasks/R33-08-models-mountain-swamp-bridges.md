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
