# R38-07: New Halden haze, big map, minimap streets, entry banner

Owns: rotland_ui/src/metro_ui/ (new folder: mod.rs, haze.rs, outline.rs, draw_map.rs, minimap.rs, tests.rs; each file under ~300 lines), rotland/src/metro_haze.rs (new, applies the haze to the frame's lighting), rotland/src/region_banner.rs (adds the New Halden state), data/sky.json (adds the `metro_haze` key only), data/map.json (adds the `metro_*` keys only).
Read: tasks/R38-common.md ("Haze", "Map", "Test flags"), rotland/src/light.rs (`at_hour_weather`, `WeatherLight`, `darken_tint_to_base`, `BASE_FOG_BEGIN`/`END`, `MIN_FOG_*`), rotland/src/sky_play.rs (`apply_biome_fog`, how `biome_fog_tint` is read from sky.json with `parse_biome_tints`), rotland/src/app/render_frame.rs around line 228-240 (the order: weather lighting, then sky biome fog), rotland/src/weather_play.rs (`light_overrides`), data/sky.json, engine/src/render/lit.rs (`Lighting`), rotland/src/region_banner.rs (the `RegionState` enum, `update`, its in-file tests), rotland/src/app/frame_ui.rs around line 144 (the one call site), rotland_ui/src/hud/region_banner.rs, rotland_ui/src/map_ui/draw_city.rs (`MapStreetsConfig`, `is_city_center_revealed`, the "Ashford" label drawing), rotland_ui/src/map_ui/draw.rs (order of drawing), rotland_ui/src/map_ui/view.rs (`MapView`), rotland_ui/src/map_ui/state.rs (`MapUiState::scale`, `centre`), rotland/src/app/minimap_draw.rs (`draw_streets_and_road`, the culling loop), game_camp/src/map/explore.rs (`MapExplore::cell_of`, `is_revealed`), game_camp/src/map/minimap.rs (`world_to_minimap_offset`), game_camp/src/map/city.rs (`is_city_center_revealed`, how explored cells are checked), data/map.json, data/minimap.json, world/src/metro/ (R38-01 types: `MetroSite`, `MetroLayout`, `MetroStreet`; read them once R38-01 is merged; do not edit).

## 1. Haze (sky and fog)
- `data/sky.json` gets `"metro_haze": { "color": [r, g, b], "fog_start": m, "fog_end": m, "blend_m": m, "skyline_fog_start": m, "skyline_fog_end": m }`. `color` is 0-255 like `biome_fog_tint`. Starting values (owner tunes): color `[150, 168, 150]`, fog_start 70, fog_end 190, blend_m 250 (light haze: same end as the normal fog 190 m, it only starts sooner and turns grey-green; do NOT make it thicker, the owner chose "light"), skyline_fog_start 3000, skyline_fog_end 6000 (= `skylineDistance` in data/metro.json).
- `rotland_ui/src/metro_ui/haze.rs` (pure, no GPU): `MetroHazeConfig` with `load(json)` / `load_default()` (paths::repo_path("data/sky.json"), reads only `metro_haze`); `fn city_signed_distance(p: Vec2, site: &MetroSite) -> f32` (negative inside the city square of `half_size` rotated by `yaw`, same rotation as `MetroLayout::contains`); `fn haze_weight(signed_dist: f32, blend_m: f32) -> f32` (1.0 inside or on the edge, smooth falloff to 0.0 at `blend_m` outside, smoothstep); `fn blend_fog(base_color: Vec3, base_begin: f32, base_end: f32, cfg, weight) -> (Vec3, f32, f32)`: fog colour lerps toward the haze colour by `weight`, fog begin and end lerp toward `fog_start`/`fog_end` by `weight`. The haze colour is capped to the base colour's luminance exactly like `darken_tint_to_base` in light.rs, so night stays dark (copy the cap, do not edit light.rs). `fn skyline_fog(cfg, base_color) -> (Vec3, f32, f32)`: the colour for the far skyline (capped haze colour) and its begin/end, so the skyline fades into haze but stays visible from the highway.
- `rotland/src/metro_haze.rs`: `pub fn apply(lighting: &mut engine::render::lit::Lighting (or the rotland light type used by render_frame), player_pos, site: Option<&MetroSite>, cfg)`: computes the weight from `city_signed_distance` and `haze_weight`, then calls `blend_fog`. Returns early when the player is far (weight 0). No new lights at night.
- Do NOT change `light.rs` or `sky_play.rs`; the lead inserts the call after `sky.apply_biome_fog` in `render_frame.rs` (report the line).
- Tests (in rotland_ui tests): inside weight 1.0; at the edge 1.0; `blend_m` outside 0.0; midway monotonic; rotated square signed distance correct; night base colour (luminance low) stays not brighter after the blend; config parses from a JSON string; missing key gives defaults and no panic.

## 2. Big map: New Halden
- `rotland_ui/src/metro_ui/outline.rs` (pure): `fn site_corners(site: &MetroSite) -> [Vec2; 4]` (rotated square); `fn discovered(site: &MetroSite, explore: &game::MapExplore) -> bool` (true if any of a 5 x 5 grid of points over the square is revealed, using `MapExplore::cell_of` and `is_revealed`); `fn streets_on_map(layout: &MetroLayout, explore: &game::MapExplore) -> Vec<MapStreet>` (only streets with a revealed midpoint cell).
- `rotland_ui/src/metro_ui/draw_map.rs`: `pub fn draw_metro_map(painter, state: &MapUiState, explore, site, layout, rect, view_size)`: when discovered, draws the city outline (`data/map.json` `metro_outline_color`, 2 px line), the street lines (`game::CITY_STREET_COLOR`-like colour from `metro_street_color`, width scaled like `draw_city.rs`), and the label "New Halden" at the centre (`metro_label_size` 16.0, same shadow style as "Ashford" in draw_city.rs). Streets use the same zoom rule as the Ashford city streets in `draw_city.rs` (read it); the outline and label show at any zoom once discovered. Nothing is drawn before discovery (no hint).
- R38-08 calls `draw_metro_map` from `rotland_ui/src/map_ui/draw.rs`. Do NOT edit map_ui files.

## 3. Minimap streets inside the city
- `rotland_ui/src/metro_ui/minimap.rs` (pure): `pub struct MinimapStreet { pub a: Vec2, pub b: Vec2, pub width_px: f32, pub avenue: bool }` and `pub fn minimap_streets(layout, player_xz: Vec2, radius_m: f32, radius_px: f32) -> Vec<MinimapStreet>` using `game::world_to_minimap_offset` (same call as minimap_draw.rs), dropping streets whose both ends are outside the disc. Width: `street_min_px` floor from `data/minimap.json`, avenues wider (same rule as minimap_draw.rs lines 78-81).
- Drawing is done by R38-08 in `rotland/src/app/minimap_draw.rs` with the existing `street_color`. Do not edit minimap_draw.rs.
- Tests: a street 100 m away is dropped, a street under the player is kept, avenue wider than street.

## 4. Entry banner "Entering New Halden"
- `rotland/src/region_banner.rs`: add `RegionState::Metro { name, biome, tier }` (highest priority: Metro > Town > Outer > Core) and a new last argument to `update`: `mut metro_name: impl FnMut() -> Option<String>`. Title `Entering: {name}`, subtitle `Danger {tier}` for tier 2+ (same rule as Town). Leaving the city into another state shows no banner, same as leaving a town (`leaving_town` rule extended to metro).
- Update the existing in-file tests to the new argument (`|| None`) and add: entering the city shows "Entering: New Halden"; leaving shows nothing; hysteresis still applies.
- Tests stay in `rotland/src/region_banner.rs` (the tracker lives there, not in rotland_ui); the haze and map tests are in rotland_ui.
- Report to the lead: `rotland/src/app/frame_ui.rs` line ~144 needs the fifth argument (`|| wg.metro_name_at(...)` or the New Halden name when inside `MetroSite`; R38-08 supplies it).

## 5. Data keys
- `data/map.json`: `metro_outline_color`, `metro_street_color`, `metro_label_size`, `metro_label_color` (hex strings like the existing keys). Nothing else in that file.
- `data/sky.json`: only `metro_haze`.
- No numbers in Rust except `// new: DECISIONS 2026-11-26` on structural constants (the 5 x 5 discovery grid) and `// data: sky.json metro_haze` / `// data: map.json metro_*` on values read from data.

## Done when
- The code and tests are written; the lead runs `cargo test` once and they pass (workers never run cargo).
- Haze math, outline, discovery, minimap and banner tests exist; `world/tests/golden.rs` untouched.
- No file outside Owns is changed. Shared-file lines for the lead are written under Worker notes.
- Night: the haze never brightens dark fog; grass outside the city stays green (haze weight is 0 outside `blend_m`).

## Worker notes
**Done (all owned files):** `rotland_ui/src/metro_ui/{mod,haze,outline,draw_map,minimap,tests}.rs`, `rotland/src/metro_haze.rs`, `rotland/src/region_banner.rs` (Metro state + 2 new tests + the old tests updated), `data/sky.json` (`metro_haze` only), `data/map.json` (4 `metro_*` keys only).
**Verified by me:** `cargo test metro_ui` 15 pass, `cargo test region` 6 pass, clippy shows no warnings in my files (only `metro_haze.rs apply is never used`, gone once wired). To get the banner tests to compile I temporarily added the fifth argument to `frame_ui.rs`; that edit is REVERTED (file is unchanged in this worktree).

**Lines for the lead (shared / other files):**
1. `rotland/src/app/frame_ui.rs` line ~144, the `play.region_banner.update(` call: add a fifth argument after the town closure, for example `|| wg.metro_name_at(p.position.x, p.position.z)` or, with the site in hand, `|| metro_site.as_ref().filter(|s| s.contains_square(Vec2::new(p.position.x, p.position.z))).map(|s| s.name.clone()),`. Without it the rotland crate does not compile.
2. `rotland/src/app/render_frame.rs`, right after `sky.apply_biome_fog(&mut lighting, actors.player, generator);` (inside the `if let (Some(sky), Some(generator))` block, or just after it):
   `crate::metro_haze::apply(&mut lighting, actors.player, metro_site.as_ref(), &metro_haze_cfg);`
   Load `metro_haze_cfg = rotland_ui::metro_ui::MetroHazeConfig::load_default();` ONCE in state (not per frame). Skyline fog for the far batch (R38-06/08): `rotland_ui::metro_ui::skyline_fog(&metro_haze_cfg, lighting.fog_color)` returns (colour, begin, end).
3. `rotland/src/main.rs` already has `mod metro_haze;`. `rotland_ui/src/lib.rs` already has `pub mod metro_ui;`. Nothing else needed.
4. Big map (R38-08, `rotland_ui/src/map_ui/draw.rs`): call once the city layout exists:
   `crate::metro_ui::draw_metro_map(painter, state, &view.map_state.explore, layout, &metro_map_cfg, rect, view_size);`
   with `metro_map_cfg = rotland_ui::metro_ui::MetroMapConfig::load_default();` cached once (not per frame). Here `layout` is `&world::metro::MetroLayout` (the site is `layout.site`). Note the function is `rotland_ui::metro_ui::draw_metro_map` (the module path is `metro_ui`, it is not inside map_ui).
5. Minimap (R38-08, `rotland/src/app/minimap_draw.rs`): `rotland_ui::metro_ui::minimap_streets(layout, player_xz, radius_metres, radius_px, cfg.street_min_px)` returns `MinimapStreet { a, b, width_px, avenue }` where a/b are pixel offsets from the minimap centre. Draw: clip with `game::clip_segment_to_circle(a, b, disk_r)`, stroke width `width_px * scale` in `cfg.street_color`, the same as the Ashford streets. Only call it when the player is near the city (`layout.near(player_xz)`) for speed.

**Deviations from the brief (on purpose):**
- `outline::streets_on_map` returns `Vec<&MetroStreet>` (there is no `MapStreet` type; `MetroStreet` from world).
- `draw_metro_map` takes `layout` (the site comes from `layout.site`) and a `&MetroMapConfig` argument, so it has 7 arguments (clippy limit).
- `minimap_streets` has an extra `street_min_px: f32` argument (from `data/minimap.json`); the rotland side owns that config, and rotland_ui cannot read it from the minimap cache.
- Minimap keep rule: a street is kept when its centreline comes within `radius + width/2` of the player (the same rule as `minimap_draw.rs`). The brief's "both ends outside the disc" would hide long streets that cross the disc, so I did not use it. The test cases in the brief still hold.
- Discovery uses a 5 x 5 sample grid over the square (`DISCOVERY_GRID`), so a very small explored spot far from the sample points may not count yet.
- `MetroMapConfig` has its own small `#rrggbb` parser because `draw_city.rs` keeps `parse_color` private. The lead may merge the two later.
- The Ashford-style zoom rule: streets draw at any zoom once a street midpoint is explored; the outline and label show at every zoom once discovered.

**Questions / unsure:** none blocking. Owner may want to tune `metro_haze` (sky.json) and the `metro_*` colours (map.json) by eye after the first screenshot.
