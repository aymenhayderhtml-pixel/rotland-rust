# R35-06: furniture inside town buildings

Owns: tools/blender/props/{int_table,int_chair,int_bed,int_shelf,int_stove,int_fridge,int_cabinet,int_rug}.py (new), world/src/towns/interior.rs (new), data/towns.json (an `interiors` block only; R35-03 adds `clutter`: edit different lines)
Files: tools/blender/props/*, world/src/towns/interior.rs, data/towns.json
Read only: world/src/towns/mod.rs, world/src/generator_contents.rs (`append_town_contents`), tools/blender/buildings/house_small.py (inside size, floor height, door side), tools/blender/props/crate_loot.py, common.py, palette.py, and the 8 walk-in town building scripts' size lines (adobe_house, desert_store, log_cabin_snow, research_station, ranger_station, stilt_shack, bait_shop, motel). Never run blender.

- 8 furniture props, 40-300 tris, real sizes (table 0.75 m high, bed 2 m long...).
- `interior::place(lot, data, seed) -> Vec<PropInstance>`: per building model in data, 3-6 pieces placed against inner walls in building-local space, rotated with the lot yaw, at the building's floor height, never blocking the door (keep a 1.2 m clear zone in front of it).
- Report to the lead the one line in `append_town_contents` that appends them.
- Tests: every piece is inside the footprint shrunk by 0.3 m; none in the door zone; deterministic.

## Worker notes
- **Files created/edited:**
  - `tools/blender/props/int_table.py`, `int_chair.py`, `int_bed.py`, `int_shelf.py`, `int_stove.py`, `int_fridge.py`, `int_cabinet.py`, `int_rug.py` (each 40–300 tris, base at `Z = 0`, facing `+Y` in Blender)
  - `data/towns.json` (added `"interiors"` block with `door_clear_m: 1.2`, `footprint_margin_m: 0.3`, and per-building `floor_y`, `count: [3, 6]`, `doors`, and `spots` for all 8 walk-in buildings)
  - `world/src/towns/interior.rs` (`InteriorsConfig`, `InteriorBuilding`, `InteriorSpot`, `place(lot, data, seed) -> Vec<PropInstance>`, and unit tests)
- **Shared-file lines for the lead:**
  1. `world/src/towns/mod.rs`:
     - Add `pub mod interior;` next to `pub mod lots;` / `pub mod place;`
     - Add to `TownsData`:
       ```rust
       #[serde(default)]
       pub interiors: interior::InteriorsConfig,
       ```
  2. `world/src/generator.rs` (in `struct WorldGenerator`):
     - `pub(crate) towns_data: crate::towns::TownsData,`
  3. `world/src/generator_world.rs`:
     - In `WorldGenerator::new_at` struct literal: `towns_data: TownsData::load_embedded(),`
     - In `WorldGenerator::attach_outer_world`: `self.towns_data = towns_data.clone();`
  4. `world/src/generator_contents.rs` (in `append_town_contents`, inside the `else` branch after `pois.push(...)`):
     ```rust
     props.extend(crate::towns::interior::place(lot, &self.towns_data, self.seed).into_iter().map(|mut p| { p.position.y += y; p }));
     ```
- **How to see it:** Run `build_detail_props.py` in Blender, bake `models.pack`, then launch with `--town=dusthollow` or `--town=frostmere` and walk inside any of the 8 walk-in buildings (`adobe_house`, `desert_store`, `log_cabin_snow`, `research_station`, `ranger_station`, `stilt_shack`, `bait_shop`, `motel`).
- **Known/unsure:** Did not run `cargo`, `blender`, or `tools/gate.sh` per subagent rules; `world/src/towns/interior.rs` and the 8 Blender scripts are ready for the lead's build/bake/gate pass.
