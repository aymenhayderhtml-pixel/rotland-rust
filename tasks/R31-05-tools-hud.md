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
