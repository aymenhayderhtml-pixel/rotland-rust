# R31-05: world map picture tool, `--biome=` teleport, minimap colours, "Entering" banner

Owns: world/examples/worldmap.rs (new), world/Cargo.toml (dev-dependency only, if a PNG crate is needed: use one already in Cargo.lock, e.g. `image` or `png`), game_camp/src/map/tiles.rs, rotland/src/app/test_flags.rs, rotland_ui/src/hud/* (only a new `region_banner.rs` + its hook), rotland/src/region_banner.rs (new, if the logic fits better in rotland), tools/perf_scenes.sh, tools/README_TOOLS.md
Files: world/examples/worldmap.rs, world/Cargo.toml, game_camp/src/map/tiles.rs, rotland/src/app/test_flags.rs, rotland/src/region_banner.rs, rotland_ui/src/hud/region_banner.rs, tools/perf_scenes.sh, tools/README_TOOLS.md
Read: tasks/R31-round.md, tasks/LEAD.md, world/Cargo.toml, game_camp/src/map/tiles.rs, rotland/src/app/test_flags.rs, tools/perf_scenes.sh, tools/README_TOOLS.md, rotland_ui/src/hud/safe_zone.rs, world/src/generator.rs, rotland/src/app/start_world.rs
API: world/src/lib.rs, world/src/biome.rs, rotland/src/app/state.rs, rotland_ui/src/hud/mod.rs, rotland_ui/src/hud/state.rs
Rules: `tasks/LEAD.md` subagent rules. Use the "R31 shared names" in `tasks/R31-round.md` EXACTLY.

1. **World map tool** (`world/examples/worldmap.rs`): `cargo run --release -p world --example worldmap -- --seed=42 --size=12000 --px=1200 --out=/tmp/worldmap.png`.
   - Build the generator the same way the game does (camp + city + lake attached; find how `rotland` builds it and copy that order).
   - For each pixel, take `biome_sample` at the centre, use the biome's `map_color` mixed with the secondary by `blend`, and shade by height (a simple hillshade from neighbouring heights, so mountains show).
   - Draw the camp as a white dot, a 1 px circle at `core_radius`, and thin rings at each tier border. Print each biome's area share to stdout.
   - It must run in under ~60 s at 1200 px: one height sample per pixel, and use rayon over rows if `world` already depends on rayon (check; don't add it otherwise).
2. **`--biome=<key>` dev flag** (in `test_flags.rs`, next to `--tp-rescue`): at start, search outward from the camp along the sector centre angle of that biome for the first point at `core_radius + blend_band + 300 m`. Teleport the player there on the ground, facing outward. For `plains`/`forest`, use the nearest core point of that biome. Unit-test the search with a fake sample function if the code allows. Add it to `tools/README_TOOLS.md` with a screenshot example.
3. **Minimap / big-map tile colours** (`game_camp/src/map/tiles.rs`): today flat forest or plains green. Use the biome `map_color` from the registry (via the generator), mixed by `blend`, with the existing cliff/height shading kept. Core colours must stay as they are today.
4. **"Entering" banner:** when the player's primary biome or tier changes (check twice a second, with a 3 s hysteresis so borders don't flicker), show a short centre-top banner for 3 s, like the existing "IN CAMP"/"Safe zone" text: `Entering: Desert`, and for tier ≥ 2 a second line `Danger 3`. No banner inside the core except when entering it from outside ("Back in home lands"). Put the logic (change detection, timing) in a small testable struct, and only the drawing in the HUD. Report the lines the lead must add to shared files (state, mod.rs, the HUD draw call).
5. **`tools/perf_scenes.sh`:** add scenes `desert snow swamp mountains` using `--biome=<key> --autowalk`, and keep the old 5 as the default set, with `--set biomes` for the new 4 and `--set all` for all 9.

## Worker notes

## Neon notes

### What changed
- **`world/examples/worldmap.rs`**: New world map renderer using `WorldGenerator::from_data_dir` and `BiomeRegistry::load()`. Samples height once per pixel, blends `primary`/`secondary` `map_color` by `blend`, applies neighbour-height hillshade, draws the `core_radius` circle and tier borders from `outer`/`tier` transitions, marks the camp with an outlined white dot, writes a valid RGB8 PNG via a self-contained encoder (no new dev-dependency needed in `world/Cargo.toml`), and prints each biome's area share.
- **`game_camp/src/map/tiles.rs`**: Keeps exact core colours (`PLAINS_GRASS`/`FOREST_GRASS` when `sample.outer <= 0.0`) and cliff/height shading; outside the core, blends `primary`/`secondary` `map_color` from `generator.biome_registry()` by `sample.blend` and mixes with core by `sample.outer`.
- **`rotland/src/app/test_flags.rs`**: Added `--biome=<key>` (`TestFlags.biome`), `find_biome_point` (unit-tested with a fake `BiomeSample` function), and `apply_biome_tp` in `apply_startup`.
- **`rotland/src/region_banner.rs`**: New `RegionBannerTracker` with 0.5 s polling, 3 s hysteresis cooldown, 3 s display with fade-in/out, `"Back in home lands"` on returning to core, `"Entering: <Name>"` + `"Danger <tier>"` (tier >= 2) outside, and unit tests.
- **`rotland_ui/src/hud/region_banner.rs`**: New `RegionBannerInfo` and `draw_region_banner` drawing below the compass / safe-zone label.
- **`tools/perf_scenes.sh`**: Added `--set <default|biomes|all>` and scenes `desert`, `snow`, `swamp`, `mountains` using `--biome=<key> --autowalk --hour=12`.
- **`tools/README_TOOLS.md`**: Documented `worldmap`, `--biome=<key>` with a screenshot command, and `--set` for `perf_scenes.sh`.

### Shared-file lines for the lead to add
1. **`rotland_ui/src/hud/mod.rs`**:
   - Add `pub mod region_banner;`
   - In `Hud::draw(...)` (right next to `safe_zone::draw_safe_zone`):
     ```rust
     region_banner::draw_region_banner(&painter, &screen, state.region_banner.as_ref(), scale);
     ```
2. **`rotland_ui/src/hud/state.rs`**:
   - In `pub struct HudState`:
     ```rust
     pub region_banner: Option<super::region_banner::RegionBannerInfo>,
     ```
   - In `HudState::default()`:
     ```rust
     region_banner: None,
     ```
3. **`rotland/src/main.rs`** (or `rotland/src/lib.rs`, wherever top-level modules are declared):
   - Add `pub mod region_banner;`
4. **`rotland/src/app/state.rs`**:
   - In `pub struct PlaySide`:
     ```rust
     pub region_banner: crate::region_banner::RegionBannerTracker,
     ```
   - In `PlaySide::default()`:
     ```rust
     region_banner: crate::region_banner::RegionBannerTracker::default(),
     ```
5. **`rotland/src/app/start_world.rs`**:
   - Reset on world start:
     ```rust
     play.region_banner.reset();
     ```
6. **Per-frame play / HUD update (in `rotland/src/app/frame_*.rs`)**:
   - Update tracker (when `world.world_gen` is `Some(wg)` and in play scene):
     ```rust
     if let Some(wg) = world.world_gen.as_ref() {
         play.region_banner.update(
             dt,
             || wg.biome_sample(actors.player.x, actors.player.z),
             |id| wg.biome_registry().get(id).name.clone(),
         );
     }
     ```
   - Pass to `HudState`:
     ```rust
     region_banner: play.region_banner.active(),
     ```

### Known risks / cross-task checks
- **`WorldGenerator::biome_registry(&self) -> &BiomeRegistry`**: `game_camp/src/map/tiles.rs` (and the suggested HUD hook above) assumes R31-02 exposes `biome_registry(&self) -> &BiomeRegistry` on `WorldGenerator`. If R31-02 named the accessor differently (e.g. `biomes()`), update that one method name in `game_camp/src/map/tiles.rs`.
- **`rotland/Cargo.toml`**: `rotland/src/region_banner.rs` imports `rotland_ui::hud::region_banner::RegionBannerInfo`. If `rotland` aliases `rotland_ui` under a different crate name or re-exports it via `crate::hud`, adjust that import line.
