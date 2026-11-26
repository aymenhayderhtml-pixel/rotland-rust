# R39 common: New Halden towers (detailed kit, landmarks, lobbies)

Setup, subagent rules and the lead loop: `tasks/LEAD.md` (worktree `../rotland-r39`, branch `r39`, shared `CARGO_TARGET_DIR`).
Plan: `docs/PLAN_1.1.md`. Owner choices: `docs/DECISIONS.md` 2026-11-26 rows. What R38 built: `tasks/archive/R38-*.md` (Worker notes), code in `world/src/metro/`, `rotland/src/metro_render/`.

## Owner decisions this round follows
- **Detail first, minimum 20 FPS** in the slowest moments. Dense and detailed close up. The owner hated the old sparse look.
- New York feel: a packed downtown, mostly 10-30 floors (40-120 m), with **3-5 landmark skyscrapers of about 150 m** that you can see from far away.
- Dead city, about 10 years on: **broken windows** (many dark, some missing), rust streaks, grime, scorch marks, open or boarded doors. The big damage (fallen tower, leaning tower, burnt block) and the vines come in R40. Keep the shapes ready for vines (flat wall areas).
- **Every tower has a ground-floor lobby you can walk into.** Upper floors are closed (climbable towers come in R43). The lobby has a door facing the street.
- No sounds. Palette-texture flat style, but rich shapes: cornices, window frames, sills, AC units, fire escapes, water tanks, roof machinery, antennas.

## How towers are built (the kit idea, every task follows it)
A tower = **base** (lobby, 2 floors tall, walk-in) + **N floor modules** stacked (one model per floor, drawn instanced) + **crown** (roof, parapet, water tank or machinery). Landmarks also stack **setback sections** (narrower floors higher up) and end in a **landmark crown** (spire, mast, stepped top).
Floor modules are made at fixed footprint widths so windows line up: **16, 24 and 32 m** square (style data says which sizes a style has). The tower planner snaps each lot to the nearest size that fits.

## R39 shared names (every task uses EXACTLY these)
- **Styles** (R39-01, `world/src/metro/styles.rs`, data `data/metro.json` key `styles`): `TowerStyle { Brick, Stone, Glass, Slab, Deco }`.
  - Brick: brownstone/tenement, fire escapes. Edge and Midtown, 4-12 floors.
  - Stone: pre-war office, cornices. Midtown and Core.
  - Glass: curtain-wall office. Core.
  - Slab: 70s concrete, balconies. Midtown.
  - Deco: setbacks, ornament. Core; also the landmarks.
  - `StyleDef { key, sizes: Vec<u32> /*16|24|32*/, districts, floors: [u32;2], floor_height: f32, base_height: f32, crown_height: f32 }`.
- **Model ids** (Blender, `assets/models/buildings/`): `tw_<style>_<size>_base`, `tw_<style>_<size>_floor`, `tw_<style>_<size>_floor_b` (a second floor variant with a different broken-window pattern), `tw_<style>_<size>_crown`. Example: `tw_brick_16_floor`. Each model also gets an `_lod1` version (simple, about 1/10 of the triangles). Landmarks: `tw_landmark_spire`, `tw_landmark_mast` (the tallest tower; the radio goal in R43), `tw_landmark_stepped`, `tw_landmark_glass_top`. Lobby interior props: `lobby_desk`, `lobby_elevators`, `lobby_mailboxes`, `lobby_bench`, `lobby_rubble`, `lobby_plant_dead`.
  - Origin: the bottom centre of the footprint.
  - Floor modules are exactly `floor_height` tall, so stacking at `base_height + i * floor_height` has no gaps.
  - The street side is **+Y in Blender** (the front, with the lobby door).
- **Parts** (R39-01, `world/src/metro/tower_parts.rs`): `TowerPart { model: String, pos: Vec3, yaw: f32, kind: PartKind }`, `PartKind { Base, Floor, Crown, Setback, LandmarkCrown, Interior }`. `fn tower_parts(spot: &TowerSpot, style: &StyleDef, ground_y: f32, rng_seed: u64) -> Vec<TowerPart>`: pure, deterministic. It alternates `_floor` / `_floor_b` by seed, and puts landmark setbacks at data-given floor fractions.
  - `TowerSpot` gains `style: TowerStyle` and `size: u32` (the snapped module width). The footprint becomes size x size.
  - `front_yaw`: the lobby faces the nearest street.
- **Budgets per model** (R38 raised them, see `data/budgets.json`):
  - floor module LOD0: 16 m up to 3k, 24 m up to 4.5k, 32 m up to 6k triangles; LOD1 at most 1/10 of LOD0.
  - base (with lobby walls): up to 15k. Crown: up to 8k. Landmark crown: up to 25k. Lobby props: up to 6k each.
- **Drawing** (R39-06): tower parts are drawn through the existing instanced prop path (`rotland/src/props.rs`), per loaded chunk.
  - LOD0 out to `towers.lod0Distance` (data, start 90 m), then LOD1 to the load ring, then the R38 skyline beyond.
  - Only LOD0 parts cast sun shadows.
  - Colliders: the base model's own wall colliders (the lobby stays walkable) plus a solid box from the top of the base to the roof (reuse `rotland/src/metro_render/hulls.rs`).
  - The R38 placeholder box towers are removed from the near ring.
- **Skyline** (R39-07): far towers get dark window bands and style colours, and are tinted toward the haze colour.
- **Sidewalks** (R39-07): New Halden sidewalks become a **raised 15 cm curb mesh** along every street (smooth straight edges, kerb stones, corner ramps), built per chunk from `MetroLayout.streets`. This replaces the jagged painted sidewalk edge from R38. The painted asphalt stays.
- **Test flags** (R39-08):
  - `--metro=lobby`: inside the lobby of the nearest tower, looking at its door.
  - `--metro=landmark`: on the street, looking up at the tallest landmark.
  - `--tower-style=<key>`: with `--metro=core`, view the nearest tower of that style.
  - `--map`: opens the big map, for a screenshot.
- **Perf**: the `metro` perf scene must keep its 1% low at or above 20 FPS. Expect it to drop a lot from 150 (that is fine and intended).

## Agreed numbers (lead, 2026-11-26: code AND models use exactly these)
| Style | Sizes (m) | Districts | Floors | Crown height (m) |
|---|---|---|---|---|
| brick | 16, 24 | Edge, Midtown | 4-12 | 3.5 |
| stone | 24, 32 | Midtown, Core | 10-24 | 5.0 |
| glass | 24, 32 | Core | 18-30 | 4.5 |
| slab | 24, 32 | Midtown | 10-20 | 3.0 |
| deco | 24, 32 | Core + landmarks | 20-30 | 5.5 |
- Every style: `floor_height` 3.8 m, `base_height` 7.6 m (the lobby is 2 floors tall).
- Landmark crowns sit on a 24 m footprint: spire 18 m, mast 30 m (tallest landmark only), stepped 12 m, glass_top 10 m. Landmark setbacks reuse `tw_deco_24_floor`.
- R39-01 changes the `edge` footprint range in data to [16, 24], so Edge lots never snap below 16.
- Lobby front door: 2.8 m wide x 3.0 m tall (not `kit.DOOR_H`).
- **Palette:** the new colours (brick_red, limestone, glass_dark, soot, ...) are added by **R39-05**, together with its shared `tools/blender/towers/shell.py`. R39-04 owns `tools/blender/towers/roof_parts.py`. R39-02 and R39-03 may write their scripts at once, but must build only after 05 and 04 have landed (the lead starts their Blender runs in that order).
- Blender subagents (02-05) MAY run Blender under `flock /tmp/rotland-blender.lock` and must look at their previews. This overrides the LEAD.md "never blender" line for this round.

## Tests that must stay green
`world/tests/golden.rs` passes UNEDITED. R38 metro tests still pass (update the tower tests only where the new style/size fields change them). Old saves load.
