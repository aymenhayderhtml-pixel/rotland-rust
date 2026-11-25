# R33-05: rivers drawn and swimmable, colliders, town banner, dev teleports

Owns: rotland/src/water_view.rs, rotland/src/river_view.rs (new), rotland/src/region_banner.rs, rotland/src/app/test_flags.rs, rotland/src/streaming.rs (only bridge/boardwalk colliders, if they need special handling), tools/README_TOOLS.md, tools/perf_scenes.sh
Files: rotland/src/water_view.rs, rotland/src/river_view.rs, rotland/src/region_banner.rs, rotland/src/app/test_flags.rs, rotland/src/streaming.rs, tools/README_TOOLS.md, tools/perf_scenes.sh
Read: tasks/R33-round.md, tasks/LEAD.md, docs/WORLD.md, rotland/src/water_view.rs, rotland/src/region_banner.rs, rotland/src/streaming.rs, rotland/src/app/test_flags.rs, tools/README_TOOLS.md, tools/perf_scenes.sh, engine/src/render/water.rs, rotland/src/app/frame_water.rs, game_player/src/swimming.rs
API: rotland/src/app/state.rs, world/src/generator.rs
Data: data/swimming.json
Overlap: wiring in files you do not own (`rotland/src/main.rs` `mod river_view;`, frame calls in `rotland/src/app/*`, the `water_at` hook in `game_player/src/swimming.rs`, and the `WaterRenderer` call) is NOT in your Files. Report those exact lines to the lead. `world/src/generator.rs` is R33-04's: read it, do not write it.
Rules: `tasks/LEAD.md` subagent rules. Use the "R33 shared names" in `tasks/R33-round.md` EXACTLY. Report any shared-file lines (app state, mod lines, frame calls) to the lead.

1. **River water** (`river_view.rs`):
   - Every ~1 s, or when the player has moved 50 m, rebuild the river triangle list with `generator.rivers().water_mesh_near(player_xz, 300.0)`. Cache it in an `Arc<Vec<Vec3>>`, the same way `water_view.rs` caches the lake mesh.
   - Draw it with the existing `WaterRenderer` the way the lake is drawn (read `water_view::draw`; add a second draw call or merge the lists).
   - Rebuild the mesh off the main thread if it takes > 2 ms. Measure with `std::time::Instant` in a debug log and say what you saw.
2. **Swimming and drinking in rivers:** find how the lake is treated as water for the player (grep `water_level`, `in_water`, `swim`) and make `generator.rivers().water_at(x, z)` count the same way. Report the exact lines if the code lives in files you don't own.
3. **Bridges and boardwalks you can walk on:** check how prop colliders are made in `streaming.rs` (the compound collider). A bridge deck must be walkable (the box collider from the model bounds is fine if the model is a flat deck plus railings; otherwise add a thin deck box). Test by screenshot: the player stands on a bridge (`--bridge=0`).
4. **Town banner:** in `region_banner.rs` (R31), entering a town radius shows `Entering: Dusthollow` (with the Danger line as today). Leaving shows nothing.
5. **Dev flags** (`test_flags.rs`), each teleporting to a ground point 25 m away from the target, facing it:
   - `--town=<key>`: the town centre.
   - `--trailhead=<n>`: trailhead n, facing outward along the road.
   - `--bridge=<n>`: the n-th crossing, standing on the deck.
   - `--river=<n>`: a bank point of river n, facing the water.
   Add each to `tools/README_TOOLS.md`, and add `town_desert` / `town_snow` scenes to `tools/perf_scenes.sh` under `--set all` (`--town=dusthollow --autowalk`, `--town=frostmere --autowalk`).

## Worker notes

### Files changed
- `rotland/src/river_view.rs` (new): off-thread (`rayon::spawn`) river mesh cache around the player (`water_mesh_near(player_xz, 300.0)`), rebuilding every ~1 s or when moved 50 m, with `std::time::Instant` `log::debug!` timing.
- `rotland/src/water_view.rs`: draws cached river water mesh via `WaterRenderer::draw_all` in `water_view::draw`.
- `rotland/src/streaming.rs`: `shapes_of` adds a thin walkable deck box fallback (`bridge_wood`, `bridge_concrete`, `boardwalk_section`) when a deck model has no colliders or only a single tall hull (> 0.6 m).
- `rotland/src/region_banner.rs`: `RegionBannerTracker::update` now accepts `town_name: impl FnMut() -> Option<String>`, shows `Entering: <town>` (with `Danger <tier>` for tier >= 2) on entering a town radius, and shows nothing when leaving a town.
- `rotland/src/app/test_flags.rs`: added `--town=<key>`, `--trailhead=<n>`, `--bridge=<n>`, `--river=<n>` flags and startup teleports (25 m away facing target, or on deck for `--bridge`).
- `tools/README_TOOLS.md`: documented `--town`, `--trailhead`, `--bridge`, `--river`.
- `tools/perf_scenes.sh`: added `town_desert` (`--town=dusthollow --autowalk`) and `town_snow` (`--town=frostmere --autowalk`) under `--set all`.

### Shared-file lines for the lead (not edited by this worker)
1. `rotland/src/main.rs` (around line 30):
```rust
mod river_view;
```
2. `rotland/src/app/frame_ui.rs` (lines 143-149, add the 4th closure argument to `play.region_banner.update`):
```rust
        if let Some(wg) = world.world_gen.as_ref() {
            play.region_banner.update(
                session.last_dt,
                || wg.biome_sample(p.position.x, p.position.z),
                |id| wg.biome_registry().get(id).name.clone(),
                || wg.town_at(p.position.x, p.position.z).map(|t| t.name.clone()),
            );
        }
```
3. `game_player/src/swimming.rs` (add `river_zone_at` helper):
```rust
/// Classifies a river point as `Land`, `Shallow`, or `Deep` from water level, riverbed height, and player Y.
pub fn river_zone_at(
    player_y: f32,
    ground_y: f32,
    river_water_level: Option<f32>,
    cfg: &SwimmingConfig,
) -> LakeZone {
    let Some(water_y) = river_water_level else {
        return LakeZone::Land;
    };
    if player_y >= water_y {
        return LakeZone::Land;
    }
    let depth = water_y - ground_y;
    if depth >= cfg.chest_height {
        LakeZone::Deep
    } else if depth > 0.0 {
        LakeZone::Shallow
    } else {
        LakeZone::Land
    }
}
```
(Re-export `river_zone_at` in `game_player/src/lib.rs` and `game/src/lib.rs` alongside `evaluate_water`.)
4. `rotland/src/app/frame_sim_player.rs` (`step_water`, lines 106-127, fall back to `wg.rivers().water_at` when outside the lake):
```rust
    if let (Some(wg), Some(p)) = (world.world_gen.as_ref(), actors.player_ctrl.as_mut()) {
        let p_pos2 = glam::Vec2::new(p.position.x, p.position.z);
        let swim_cfg = play.water.swim_cfg;
        let lake_opt = wg.lake();
        let lake_zone = lake_opt.map_or(world::LakeZone::Land, |l| l.zone_at(p_pos2));
        let on_dock = lake_opt.is_some_and(|l| game::swimming::is_on_dock(l, p_pos2));
        let (zone, water_level, on_platform) = if lake_zone != world::LakeZone::Land {
            (lake_zone, lake_opt.map_or(0.0, |l| l.water_level), on_dock)
        } else {
            let river_y = wg.rivers().water_at(p.position.x, p.position.z);
            let ground_y = wg.height(p.position.x, p.position.z);
            let r_zone = game::swimming::river_zone_at(p.position.y, ground_y, river_y, &swim_cfg);
            (r_zone, river_y.unwrap_or(0.0), false)
        };
        let water_res = game::swimming::evaluate_water(
            p.position,
            zone,
            water_level,
            p.stamina.current,
            on_platform,
            &swim_cfg,
            dt,
        );
        p.apply_water_movement(&water_res);
        p.water_depth = match water_res.zone {
            world::LakeZone::Shallow | world::LakeZone::Deep if !on_platform => {
                (water_level - p.position.y).max(0.0)
            }
            _ => 0.0,
        };
```
(And update `in_deep_water` on line 131 to use `!on_platform` instead of `!on_dock`.)

### How to see it
- Run with `--river=0`, `--bridge=0`, `--trailhead=0`, `--town=dusthollow` (once R33-04 + shared-file lines are wired and `gate.sh` passes).
- Walk into a river to swim/drown and into Dusthollow to see `Entering: Dusthollow`.

### Known / unsure
- Did not run `cargo` / `gate.sh` / game per subagent rules; off-thread river mesh rebuild timing (`Instant` log) should be observed during the lead's screenshot run.
- `frame_water.rs` (`can_fish`) currently only checks `lake`; left untouched unless river fishing is desired.
