# R31-02: terrain height + ground colour per biome; split generator.rs; core stays exact

Owns: world/src/generator.rs, world/src/generator/* (new, if you split into a folder), world/src/terrain_shape.rs (new), world/src/terrain_paint.rs (new), world/src/ground_paint.rs, world/tests/terrain_unit_tests.rs, world/tests/biome_terrain_tests.rs (new)
Files: world/src/generator.rs, world/src/terrain_shape.rs, world/src/terrain_paint.rs, world/src/ground_paint.rs, world/tests/terrain_unit_tests.rs, world/tests/biome_terrain_tests.rs
Read: tasks/R31-round.md, tasks/LEAD.md, world/src/generator.rs, world/src/ground_paint.rs, world/src/noise.rs, world/src/lib.rs, world/tests/terrain_unit_tests.rs, world/tests/golden.rs
API: world/src/biome.rs, world/src/color.rs, world/src/math.rs, world/src/city/mod.rs, world/src/lake/mod.rs, world/src/settlement.rs, world/src/scatter.rs, world/src/poi.rs
Rules: `tasks/LEAD.md` subagent rules. Use the "R31 shared names" in `tasks/R31-round.md` EXACTLY. R31-01 writes `world/src/biome/*` at the same time: code against the shared names and don't edit its files.

**Goal:** outside the core, the ground takes the shape and colour of its biome. Inside the core, every height and colour stays bit-for-bit the same as today (`world/tests/golden.rs` must pass untouched).

1. **Split `generator.rs` (420 lines) to under ~300 per file:** height/shape code → `terrain_shape.rs`, triangle colour code (`emit_triangle` and helpers) → `terrain_paint.rs`. Remove the duplicated height formula: `sample()` and `raw_height()` must share one function.
2. **`WorldGenerator` holds a `WorldLayout` and a `BiomeRegistry`.** The constructors that load data (`with_configs`, and whatever the tests and `rotland` call) load them too. If a constructor can't fail today, use `BiomeRegistry::load().expect(...)` only where the other data files already use expect; otherwise add a `Result` path. `reference_dir` = camp → Ashford when the city is attached, else +X; `origin` = the camp pad centre. Mind the order: the city is placed after the camp, so build the layout (or update its direction) after `attach_city_with_ground`.
3. **`biome_sample(x, z)` and `tier(x, z)`** from the shared names. In the core (`outer == 0`): primary = today's Plains/Forest rule, secondary = the same, blend 0, tier 0.
4. **Height:** `h = lerp(core_h, biome_h, outer)`, where `core_h` = today's full `sample()` height and `biome_h` = the primary shape blended with the secondary shape by `blend`.
   - **Biome shape** from `BiomeHeight`: `base + amplitude * fbm(...)` with its frequency and octaves (use `crate::noise`; new seed offsets that are unused, with `wrapping_add`).
   - `ridged: true` = `1 - |n|`, squared (sharp mountain ridges).
   - `dune > 0` adds stretched dunes: noise sampled with x scaled by 0.25 along one fixed direction, times `dune`.
   - City/lake/camp flattening still apply as today (they're all inside the core anyway).
   - Mountains get high, but keep the slopes walkable-ish: clamp the shape so a 2 m cell never rises more than ~3 m (smooth the ridged value) and say what you did.
5. **Colour:** in the core, unchanged. Outside, `lerp(core_colour, biome_colour, outer)`.
   - Biome colour: `base`/`alt` mixed by the existing hue noise, `slope` colour on steep triangles (same slope test `GroundPaint` uses for rock), and `peak` above `peak_height` with a soft 6 m fade.
   - Swamp: darker in low spots (below the local average height).
   - Snow: use `snow_shadow` on north-facing slopes (normal.z > 0.3), so snow isn't flat white.
   - Keep the brightness/hue jitter so it looks like today's art style.
6. **`forest_weight()`:** in the core, exactly as today. Outside, `lerp(core_fw, if primary == Forest {1} else {0}, outer)`.
7. **Speed:** `sample()` runs ~1,100 times per chunk. Don't add a noise call in the core path: only evaluate the layout/biome noise when the distance from the origin is beyond `core_radius` (a cheap distance check first).
8. **Tests** (`biome_terrain_tests.rs`):
   - core points: identical heights/colours to a generator built the old way (the golden test also covers this);
   - mountains at 5000 m reach > 60 m somewhere in a 1 km scan;
   - swamp stays under 4 m;
   - no seam: heights continuous across the core edge and sector borders (max step between 2 m neighbours < 3.5 m);
   - chunk-edge seams still match (see the existing test).

## Worker notes

## Neon notes

* **What was wired:**
* Split `world/src/generator.rs` into `world/src/terrain_shape.rs` (height/shape evaluation, deduplicating `raw_height()` and `sample()` via `TerrainShape::raw_core_height`) and `world/src/terrain_paint.rs` (triangle mesh emission and biome/core ground coloring), keeping each file well under 300 lines.
* Exposed `GroundPaint::STEEP_START`, `GroundPaint::STEEP_FULL`, and `GroundPaint::steepness(normal_y)` in `world/src/ground_paint.rs` so outer biome slope coloring uses the exact same slope test as core rock painting without altering `GroundPaint::paint`.
* Updated `WorldGenerator` to hold `BiomeRegistry`, `WorldLayoutData`, and `WorldLayout` (loaded in `from_data_dir`, `new`, `with_configs`, and `with_all_configs`), setting `origin` to the camp pad center and updating `reference_dir` to `camp -> Ashford` inside `attach_city_with_ground` (defaulting to `+X` when no city is attached).
* Added `WorldGenerator::biome_sample(x, z)` and `WorldGenerator::tier(x, z)`, updated `biome(x, z)` to return `biome_sample(x, z).primary`, and updated `forest_weight(x, z)` to interpolate `lerp(core_fw, if primary == BiomeId::Forest { 1.0 } else { 0.0 }, outer)` outside the core.
* Added a fast squared-distance check against `layout_data.core_radius` before querying `WorldLayout::sample` so points inside the core execute zero additional noise calls and remain bit-for-bit identical (`golden.rs` untouched).
* **Mountain ridge slope smoothing:** For `ridged: true`, smoothed the `1 - |n|` V-cusp at `n = 0` via `((1.0 - (n * n + 0.01).sqrt()) / (1.0 - 0.01_f32.sqrt())).clamp(0.0, 1.0)` before squaring, capped ridge octaves to `2` (`RIDGE_MAX_OCTAVES`) so macro peaks still reach full `base + amplitude` (> 60 m) without high-frequency cliff spikes, and clamped effective amplitude against `2.8 m / (2.0 * 2.6 * frequency * octaves * ridge_factor / weight_sum)` so a 2 m cell never rises more than ~2.8 m.
* Added unit tests in `world/tests/terrain_unit_tests.rs` and `world/tests/biome_terrain_tests.rs`.


* **Shared-file lines for the Lead (`world/src/lib.rs`):**
Add module declarations (and optional re-exports) in `world/src/lib.rs`:
```rust
pub mod terrain_paint;
pub mod terrain_shape;

```


(And if not already re-exported by R31-01, `pub use biome::layout::BiomeSample;` is convenient for callers.)
* **Cross-task interface assumptions to verify against R31-01 (`world/src/biome/*`):**
* `BiomeGround` fields `base`, `alt`, `slope` are assumed to be `[u8; 3]` and `peak` to be `Option<[u8; 3]>` (matching `"map_color": [222,196,140]` in `R31-round.md`). If R31-01 defined them as `[f32; 3]` or `Color`, adjust `TerrainPaint::rgb_color` accordingly.
* `WorldLayoutData` is assumed to derive `serde::Deserialize` and expose `pub core_radius: f32`.
