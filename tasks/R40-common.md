# R40 common: New Halden, nature takes it back + survivor traces

Setup, subagent rules and the lead loop: `tasks/LEAD.md` (worktree `../rotland-r40`, branch `r40`, shared `CARGO_TARGET_DIR`).
Plan: `docs/PLAN_1.1.md`. Owner choices: `docs/DECISIONS.md` 2026-11-26 rows. What exists: `world/src/metro/` (layout, towers, styles, tower_parts, jam, curbs, paint), `rotland/src/metro_render/`, `tools/blender/towers/`, `tools/blender/metro/`. Read the R38/R39 Worker notes in `tasks/archive/R38-*.md` and `R39-*.md` once.

## Owner decisions this round follows
- **About 10 years after the fall, heavily overgrown**, like the dead city in a famous zombie TV show. Grass and weeds burst through cracked roads, vines climb tower walls, small trees grow in the streets and on roofs, moss covers cars, and lots and parks have turned into wild scrub. No names or characters from any show.
- **Damage:** mostly standing broken towers, plus three big moments:
  1. **one fallen tower** lying across a street (rubble field, a broken top section, its floors exposed)
  2. **one leaning tower** resting on its neighbour
  3. **one burnt-out block** (charred facades, soot streaks, burnt cars, ash)
- **Survivor traces:**
  - streets walled off with **stacked cars** (planks and sheet metal, a gap or a ladder to climb)
  - **spray-painted warnings** on doors and walls (generic words only: "DEAD INSIDE" style warnings, X marks, arrows, "TURN BACK", "NO FOOD", "WE WENT NORTH")
  - **missing-person posters** near lobby doors
  - **old rooftop camps** (tent, tarp, mattress, barrels, washing line, fire barrel)
  - **bodies under sheets**
- **Show-level gore:** bodies under sheets, dark stains. Nothing extreme.
- **Detail first.** **FPS:** measure the `metro` scene only (`tools/perf_scenes.sh --set metro`), before and after. Pass = **average FPS at least 20**. The 1% low is noted for information only.
- No sounds. Palette flat style, dense and rich.

## R39 leftovers this round also fixes
1. **Lobbies are bare grey halls.** Add columns, hanging ceiling panels, broken light fixtures, posters, rugs, an overturned chair, debris, and a dead body under a sheet in some lobbies.
2. **Towers look too clean.** Add a damaged floor variant `tw_<style>_<size>_floor_c` (more shattered and dark panes, soot or rust streaks, a missing window frame) plus grime/rust streak overlay pieces.
3. **Roofs are plain.** Use bigger water tanks on legs (NY style), rooftop machinery, AC boxes, antennas and a stair hut. Change the crowns or add roof props.
4. **`--metro=landmark` doesn't look up at a landmark.** Stand about 60 m from the tallest landmark, camera pitched up.

## R40 shared names (every task uses EXACTLY these)
- **Data:** `data/metro.json` gets a new top-level `overgrowth`, `damage` and `traces` objects. Each task owns its own object; never rewrite the file.
- **Overgrowth placement** (R40-01, `world/src/metro/overgrowth.rs`, pure): `fn overgrowth_props_in_chunk(metro: &MetroWorld, chunk: (i32, i32), height: &dyn Fn(f32, f32) -> f32) -> Vec<PropInstance>`. Density is set per district in data (Edge greenest, Core least).
  - **Street level:** crack weeds along curbs and street joints, broken-asphalt street trees (the tree model comes with its own lifted slab), saplings, moss patches.
  - **Lots:** wild shrubs, young trees, tall grass.
  - **Walls:** vines hugging the lower floors of some towers, placed flat against a facade at a base or floor position from `tower_parts`.
  - **Roofs:** a bush or small tree on some low roofs, at the roof height from `tower_parts`.
- **Overgrowth models** (R40-02, Blender `tools/blender/overgrowth/`, category `nature`):
  - `vine_wall_s`, `vine_wall_m`, `vine_wall_l`: flat ivy sheets 0.3 m deep, 4 / 8 / 14 m tall, with hanging strands.
  - `vine_cornice`: ivy draped over a ledge.
  - `weeds_crack`: a line of weeds, 3 m long.
  - `tree_asphalt`: a young tree breaking a raised asphalt slab.
  - `sapling_city`, `shrub_wild`, `grass_tall_patch`, `moss_patch`.
  - `roof_bush`, `roof_tree`.
  - `leaves_litter`: dead leaves and paper.
  - Budgets up to 15k (nature). Vines must not z-fight the wall: put them 2-5 cm off the face.
- **Damage set pieces** (R40-03 placement in `world/src/metro/damage.rs`, R40-04 models in `tools/blender/damage/`):
  - Models:
    - `tw_fallen_section`: a long broken tower section lying on its side, floors exposed, 60-80 m long.
    - `rubble_field_l`, `rubble_field_m`: big and medium rubble fields with rebar, slabs and pipes.
    - `tw_leaning_tower`: a full Stone or Deco tower baked leaning about 8 degrees, with a cracked base and rubble.
    - `burnt_floor_overlay`: soot and char sheets for a facade.
    - `car_burnt_pile`.
    - `ash_patch`.
  - Placement:
    - `fn damage_sites(metro: &MetroWorld, cfg) -> DamageSites { fallen: Site, leaning: Site, burnt_block: (i32, i32) }` with `Site { block: (i32, i32), pos: Vec2, yaw: f32 }`. Deterministic per seed, never on reserved blocks, never on the highway entrance avenue.
    - The fallen tower's block has its own tower removed and the section lies across the next street. That street segment is blocked; the rubble is climbable on foot.
    - The leaning tower replaces one tower's normal parts.
    - In the burnt block, every tower uses `_floor_c` plus `burnt_floor_overlay`, and the cars there are burnt.
  - `fn damage_props_in_chunk(...) -> Vec<PropInstance>`, plus `fn tower_is_replaced(tower_index) -> bool` so `tower_parts` skips the fallen and leaning towers.
- **Survivor trace models** (R40-05, Blender `tools/blender/traces/`, category `props`):
  - `car_wall_stack`: 2-3 wrecks stacked with planks and sheet metal, 10-12 m wide, with a climbable side (ladder).
  - `graffiti_*` decals, flat planes 2 cm off a wall: `graffiti_dead_inside`, `graffiti_turn_back`, `graffiti_x`, `graffiti_arrow`, `graffiti_no_food`, `graffiti_went_north`. Paint letters with the palette, like the R38 gantry font.
  - `poster_wall`: a cluster of missing-person posters.
  - `rooftop_camp`: tent, tarp, mattress, barrels, washing line, fire barrel. One model 6x6 m.
  - `boarded_door`, `sandbags_line`.
  - `body_sheet_b`: a second pose.
  - Lobby dressing (R39 leftover 1): `lobby_column`, `lobby_ceiling_hang`, `lobby_light_broken`, `lobby_rug`, `lobby_chair_tipped`, `lobby_debris`.
- **Survivor trace placement** (R40-06, `world/src/metro/traces.rs`, pure):
  - Car walls across a few street ends: each blocks one street, with a gap or a ladder.
  - Graffiti on lobby fronts and street walls, at 1-3 m height, flat on the facade.
  - Posters beside some lobby doors.
  - Rooftop camps on a few low roofs (Brick, 4-8 floors).
  - Bodies on sidewalks and in lobbies.
  - Lobby dressing added to the `tower_parts` interior list.
  - `fn traces_props_in_chunk(...) -> Vec<PropInstance>`.
- **Tower variety** (R40-07, Blender `tools/blender/towers/` + `world/src/metro/tower_parts.rs`, R39 leftovers 2 + 3): the `_floor_c` damaged variant for every style and size (plus `_lod1`), mixed in by data share (`towers.damagedShare`, e.g. 0.25), and better crowns and roof props (`roof_water_tank_ny`, `roof_machinery`, `roof_stair_hut`, `roof_antenna`).
- **Wiring + flags** (R40-08):
  - Chunk props from 01/03/06 go into the existing per-chunk prop path, with colliders for the car walls, rubble (walkable slopes), the fallen section and the leaning tower.
  - Tiny props (weeds, moss, leaves, graffiti, posters) cast no shadow.
  - New flags: `--metro=fallen|leaning|burnt|carwall|camp`, and the `--metro=landmark` fix (R39 leftover 4).
  - README lines.

## Lead answers to writer questions (2026-11-26)
- Blender subagents (02, 04, 05, 07) MAY run Blender under `flock /tmp/rotland-blender.lock` and must look at their previews (overrides the LEAD.md "never blender" line for this round). They never run cargo.
- Roof props are standalone glbs (for placement) AND the crowns use the same `roof_parts` functions. Both are fine.
- The skip of replaced towers (`tower_is_replaced`) and the burnt-block call `tower_parts_with_damage(.., DamageMode::All)` are wired by R40-08 at the tower loop call site.
- The leaning tower is ONE baked model: an 18-floor Deco tower, about 80 m, leaning 8 degrees. Budget 80k (lod1 8k).

- `damage_sites(layout, towers, cfg)` (not `&MetroWorld`); `tower_is_replaced` / `tower_is_burnt` are methods on `DamageSites` (R40-03). Fine.
- Lobby dressing goes in `traces_props_in_chunk` with a `y` offset (R40-06), not in `tower_parts`. Fine.
- **The fallen tower lies ACROSS the street (at right angles to it)**: its base stump stays in its own block, the long section spans the street, and the top lands in the opposite block. Any tower its footprint touches in the opposite block is removed too (also counted by `tower_is_replaced`). The street is blocked there; the rubble is climbable.
- The `--metro=landmark` fix picks the TALLEST landmark, stands 60 m away and pitches the camera at the crown.

## Tests that must stay green
`world/tests/golden.rs` passes UNEDITED (home land, Ashford, camp, towns unchanged). R38/R39 metro tests still pass. Old saves load.
