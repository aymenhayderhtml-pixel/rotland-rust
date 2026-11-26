# R35-01: ground detail layer (dense, short range)

Owns: rotland/src/ground_detail.rs (new), world/src/ground_detail.rs (new), data/ground_detail.json (new)
Files: rotland/src/ground_detail.rs, world/src/ground_detail.rs, data/ground_detail.json
Read only: rotland/src/props.rs (how instanced props are drawn since R34), world/src/scatter.rs (seeded placement), data/biomes.json (biome keys), world/src/biome/mod.rs (BiomeId).

- World (pure): `ground_detail::place(cell: IVec2, cell_m, biome_at, height_at, seed) -> Vec<DetailInstance>`: deterministic, jittered grid per 8 m cell, density from data, skip slopes > data max, skip roads/rivers/town lots (take a `blocked: &dyn Fn(Vec2)->bool`).
- Rotland: keep a ring of cells within `radius_m` (data, start 45 m) of the player, rebuild only cells that enter/leave, draw through the existing instanced prop path, no shadows, fade/shrink near `fade_m`. Budget: ≤ 25k instances total (data).
- Biome lists: plains/forest `grass_tuft_a`, `grass_tuft_b`, `flowers_white`, `flowers_yellow`, `fern`; desert `desert_shrub`, `tumbleweed`, `bleached_bones`; snow `snow_bush_small`, `ice_shards`; swamp `cattails`, `reeds`, `swamp_moss_clump`, `lily_mud_patch`; mountains `scree_patch`, `alpine_flowers`. (Some come from R35-02; missing models must be skipped quietly.)
- No camp detail inside the camp walls area (use the settlement clear radius).
- Tests: same seed → same instances; density roughly matches data; blocked points get nothing.
- Report the `mod` lines and the one call site in the frame loop to the lead.

## Worker notes
### Files changed
- `data/ground_detail.json` (new): global `cell_m` (8.0), `radius_m` (45.0), `fade_m` (36.0), `max_instances` (25000), `max_slope` (0.7), `ground_sink` (0.03), and per-biome `biomes` rules (`model`, `density_per_100m2`, `scale`, `radius_m`) for all 6 biomes.
- `world/src/ground_detail.rs` (new): `DetailRule`, `GroundDetailConfigData`, `GroundDetailConfig`, `DetailInstance`, and pure `place(cell, config, biome_at, height_at, blocked, seed)` with deterministic jittered grid per 8 m cell, slope filtering, and unit tests (`same_seed_produces_same_instances`, `density_roughly_matches_data`, `blocked_points_get_nothing`, `steep_slopes_are_skipped`, `loads_data_ground_detail_json`).
- `rotland/src/ground_detail.rs` (new): `GroundDetailLayer` managing an 8 m cell ring within `radius_m` of the player, rebuilding only cells that enter/leave, skipping missing models quietly, capping at `max_instances`, shrinking instances in the `[fade_m, radius_m]` band, and blocking camp clear zone (`SettlementGround::in_clear_zone`), roads, rivers, town lots/streets, lake water, and non-park city areas.

### Shared-file / wiring lines for lead
1. `world/src/lib.rs`:
   ```rust
   pub mod ground_detail;
   ```
2. `rotland/src/main.rs`:
   ```rust
   mod ground_detail;
   ```
3. `rotland/src/app/state.rs` (inside `pub struct RenderState`):
   ```rust
   pub ground_detail: crate::ground_detail::GroundDetailLayer,
   ```
4. `rotland/src/app/render_frame.rs` (right before `let placed = streamer...` around line 192):
   ```rust
   if let (Some(pack), Some(world_gen)) = (render.pack.as_ref(), world.world_gen.as_ref()) {
       render.ground_detail.update(
           &gpu.device,
           pack,
           &mut render.library,
           world_gen,
           actors.player,
       );
   }
   ```
   And chain `.chain(render.ground_detail.placed().iter())` onto `let placed = streamer...` (line 198).
5. `rotland/src/props.rs` (optional/recommended for no-shadow on new R35-02 detail models): add `"tumbleweed"`, `"bleached_bones"`, `"snow_bush_small"`, `"ice_shards"`, `"cattails"`, `"swamp_moss_clump"`, `"scree_patch"`, `"alpine_flowers"` to `is_ground(id)` so `PropLibrary::batches` skips shadows for them too.

### How to see it / known issues
- Walk in any biome outside the camp clear radius (`32 m`); dense grass, flowers, ferns, and biome ground detail appear within `45 m` and shrink out smoothly past `36 m`.
- Did not run `cargo` or `tools/gate.sh` per subagent rules (uncompiled/untested until lead runs the gate).
