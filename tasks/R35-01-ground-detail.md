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
