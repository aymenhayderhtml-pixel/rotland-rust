# R41-09: Wiring, night-vision look, HUD, flags (after 01-08 and 10)

Owns:
- `rotland/src/play_metro.rs` (NEW: metro spawn profile + street herds glue; keeps `play.rs` from growing)
- `rotland/src/play.rs` (ONLY one-line calls into `play_metro.rs`)
- `rotland/src/combat_damage.rs` (armour in bites)
- the hotbar `UseItem` arms for the new items (where `flashlight` is handled today)
- `rotland/src/metro_life.rs` call sites, `rotland/src/app/frame_*.rs` (only the lines needed)
- `rotland/src/app/metro_flags.rs`, `rotland/src/app/test_flags.rs`
- `engine/src/render/lit.wgsl` + the scene uniform struct in Rust (night-vision fields only)
- `rotland_ui/src/hud/` (a new `boosts.rs` widget, small)
- save systems for the new state (`save/src/systems/`, one small file)
- `world/src/generator_contents.rs` (the life + metro loot lines)
- `docs/WORLD.md` (a short R41 section)
- `tools/README_TOOLS.md` (new flags)
- `tools/perf_scenes.sh` (only if the metro set needs a night scene; see Checks)

Read: tasks/R41-common.md, the Worker notes of R41-01 to R41-08 and R41-10 (their exact wiring lines), `rotland/src/play.rs` (`spawn_tick`), `rotland/src/zombies/` (spawn, hordes, notice events), `rotland/src/app/metro_flags.rs`, `rotland/src/wildlife.rs`, `rotland/src/temperature_play.rs`, `engine/src/render/lit.rs` + `lit.wgsl`.

## Wiring
1. **Metro zombies:** in `play_metro.rs`, `fn metro_profile_at(world_gen, player, night, cfg) -> Option<MetroSpawnProfile>` using `world_gen.metro_layout()?.district_at(p)`. `spawn_tick` puts it in `SpawnContext.metro` (one line). Outside the city it is `None` (unchanged behaviour).
2. **Street herds:**
   - Build `StreetHerds` once from `herd_routes(...)` when the world loads.
   - Each tick: spawn the orders as `walker`s, make them ONE horde (`create_horde`), and set the horde target to `leader_target(...)` while walking.
   - Members walk at the herd `speed` while not chasing. Use the zombies' existing slow-walk/wander speed setting; if there is none, add the smallest possible override in the zombie glue and state it.
   - Despawn on orders.
   - When a member notices the player, call `on_noticed`; when the herd chases, the horde target becomes the player.
   - Save `StreetHerds` state in the save `extra` map.
   - Herd zombies do NOT count against the normal spawner cap, but the global hard cap of live zombies still holds: if a spawn would break it, spawn fewer.
3. **Life:**
   - `life_props_in_chunk` goes into the per-chunk metro props in `generator_contents.rs` (static, with shadows for poles/billboards; no shadows for wires and faces under 0.05 m2, like R40's tiny-prop rule). Colliders for poles (standing + fallen) and billboard legs.
   - `metro_life.rs` is updated and drawn each frame.
   - A gunshot (from the gun-fire event) startles crows.
4. **Items:**
   - Hotbar use:
     - `night_vision`: toggle
     - `antibiotics` / `adrenaline_shot`: start the boost, and adrenaline revives while down (the use must work while downed: check the downed input path)
     - `surgery_kit`: a 6 s use like the existing timed actions, then full health
     - `gun_parts`: only with a gun in hand
     - `car_parts`: only near a car, reusing the car prompt distance
     - `army_ration`: is normal food
   - Armour: multiply each bite in `take_bites` / `take_damage_with_downed` by `1 - armor_reduction`.
   - Boosts: update each frame and add health. Free sprint: stamina doesn't drain while it runs.
   - Night vision drains each frame.
   - Save boosts + night vision.
5. **Night-vision look:** add to the scene uniform `nvg: vec4<f32>` = (strength, gain, noise, time) and the tint.
   - In `lit.wgsl`, after the filmic curve, when strength > 0:
     - `l = luminance(color) * gain`
     - `color = tint * clamp(l, 0, 1)`
     - plus a little per-pixel noise from the screen position and time
     - plus a soft dark vignette
   - Spot lights and the flashlight still add light.
   - Also lift the night ambient a little so dark interiors show shapes. Keep it cheap: no new pass.
   - Fog: keep it, tinted green.
6. **HUD:** a small boosts row near the vitals:
   - a heal-over-time icon with seconds left
   - an adrenaline sprint icon with seconds left
   - a night-vision battery bar when it is on

   Use the existing HUD style. The `army_helmet`/`body_armor` show nowhere else; the bag tooltip shows "Bite damage -35%".

## Flags (in `metro_flags.rs` / `test_flags.rs`)
- `--metro=herd`: the camera stands on a corner 25 m from an active herd walking toward it (hour 10).
- `--metro=crows`: a perch with a body and crows pecking, the camera 15 m away. With `--shot-frame=` later than the startle time, the crows are in the air (add `--walk-in` or reuse an existing flag that moves the player 5 m forward, if one exists; otherwise the flag `--metro=crows_up` places the player inside the startle radius).
- `--metro=army`: in front of an army cache in the Core, `--hold-e` style shows the opened loot.
- `--metro=night`: the Core at hour 23 (runners).
- `--nvg`: start with night vision on and 2 batteries in the bag (works with any scene: `--metro=night --nvg`).
- Each flag gets a line in `tools/README_TOOLS.md`.

## Done when
Everything is wired, the gate passes, the checks in R41-round.md are done. Fill `## Worker notes
- **Files created / changed:**
  - `rotland/src/play_metro.rs` (NEW): `metro_profile_at`, `MetroHerdRuntime` (`ensure_init`, `tick`, `to_save`, `from_save`), and `SurgeryUse` (6 s timed use).
  - `rotland/src/play.rs`: added `armor_cfg`, `boosts_cfg`, `boosts`, `night_vision`, `metro_herds`, `surgery` on `PlayState`; `HandChange::{ToggleNightVision, UseAdrenaline, StartSurgery, RepairCarParts}`; `use_item` / `bag_action` arms; `spawner_hard_cap()`; `spawn_tick` metro profile + non-herd alive filter.
  - `rotland/src/app/frame_input.rs`: `apply_hand` arms for night vision, adrenaline (revives when downed, free sprint when standing), surgery kit, and car parts; allows `adrenaline_shot` hotbar key while downed.
  - `rotland/src/app/frame_sim_player.rs`: applies `game::armor_reduction` and `game::apply_armor_damage` to zombie bites in `apply_player_bites`.
  - `rotland/src/app/frame_actors_move.rs`: skips sprint stamina drain while `boosts.free_sprint()` is true.
  - `rotland/src/app/frame_world.rs`: ticks `boosts` (heal over time), `night_vision` (battery drain), `surgery` (6 s full heal), and `metro_herds.tick(...)`.
  - `rotland/src/app/frame_sim.rs` & `rotland/src/app/render_frame.rs`: updates and poses `render.metro_life` (with gunshot startle from `noises_scratch`), and passes night vision parameters (`nvg`, `nvg_tint`, lifted `ambient_energy`) to `Lighting`.
  - `rotland/src/app/frame_ui.rs`: populates `state.boosts` for the HUD.
  - `rotland/src/session.rs`, `rotland/src/app/save_load.rs`, `rotland/src/app/start_world.rs`: saves/loads `boosts`, `night_vision`, and `street_herds` in `systems.extra`.
  - `game_zombie/src/zombie/brain.rs`, `rotland/src/zombies/{types.rs, spawning.rs, mod.rs}`: added `set_wander_speed` on `ZombieBrain`, `herd_route: Option<u32>` on `Zombie`, and slow `Horde` steering while idle/wandering.
  - `engine/src/render/{lit.rs, lit.wgsl}` & `rotland/src/light.rs`: added `nvg: [f32; 4]` and `nvg_tint: Vec3` to `Lighting` / `SceneUniforms` / `Scene` and `apply_nvg` post-filmic pass (luminance * gain, green tint, screen-space noise, vignette).
  - `rotland_ui/src/hud/{boosts.rs (NEW), state.rs, draw.rs}` & `rotland_ui/src/hub_ui/draw_bag_details.rs`: added `BoostsHudInfo` + `draw_boosts` (heal timer, adrenaline timer, NVG battery bar) and `"Bite damage -X%"` tooltip in bag details.
  - `rotland/src/app/{metro_flags.rs, test_flags.rs}`, `docs/WORLD.md`, `tools/README_TOOLS.md`: added `--metro=herd|crows|crows_up|army|night` and `--nvg`.
- **Shared-file lines for the Lead:**
  - `rotland/src/main.rs`:
    ```rust
    mod play_metro;
    ```
  - `rotland_ui/src/hud/mod.rs`:
    ```rust
    pub mod boosts;
    pub use boosts::BoostsHudInfo;
    ```
- **Not run:** Per subagent rules, did not run `cargo`, `tools/gate.sh`, `blender`, or the game.`.

## Worker notes
- **Files created / changed:**
  - `rotland/src/play_metro.rs` (NEW): `metro_profile_at`, `MetroHerdRuntime` (`ensure_init`, `tick`, `to_save`, `from_save`), and `SurgeryUse` (6 s timed use).
  - `rotland/src/play.rs`: added `armor_cfg`, `boosts_cfg`, `boosts`, `night_vision`, `metro_herds`, `surgery` on `PlayState`; `HandChange::{ToggleNightVision, UseAdrenaline, StartSurgery, RepairCarParts}`; `use_item` / `bag_action` arms; `spawner_hard_cap()`; `spawn_tick` metro profile + non-herd alive filter.
  - `rotland/src/app/frame_input.rs`: `apply_hand` arms for night vision, adrenaline (revives when downed, free sprint when standing), surgery kit, and car parts; allows `adrenaline_shot` hotbar key while downed.
  - `rotland/src/app/frame_sim_player.rs`: applies `game::armor_reduction` and `game::apply_armor_damage` to zombie bites in `apply_player_bites`.
  - `rotland/src/app/frame_actors_move.rs`: skips sprint stamina drain while `boosts.free_sprint()` is true.
  - `rotland/src/app/frame_world.rs`: ticks `boosts` (heal over time), `night_vision` (battery drain), `surgery` (6 s full heal), and `metro_herds.tick(...)`.
  - `rotland/src/app/frame_sim.rs` & `rotland/src/app/render_frame.rs`: updates and poses `render.metro_life` (with gunshot startle from `noises_scratch`), and passes night vision parameters (`nvg`, `nvg_tint`, lifted `ambient_energy`) to `Lighting`.
  - `rotland/src/app/frame_ui.rs`: populates `state.boosts` for the HUD.
  - `rotland/src/session.rs`, `rotland/src/app/save_load.rs`, `rotland/src/app/start_world.rs`: saves/loads `boosts`, `night_vision`, and `street_herds` in `systems.extra`.
  - `game_zombie/src/zombie/brain.rs`, `rotland/src/zombies/{types.rs, spawning.rs, mod.rs}`: added `set_wander_speed` on `ZombieBrain`, `herd_route: Option<u32>` on `Zombie`, and slow `Horde` steering while idle/wandering.
  - `engine/src/render/{lit.rs, lit.wgsl}` & `rotland/src/light.rs`: added `nvg: [f32; 4]` and `nvg_tint: Vec3` to `Lighting` / `SceneUniforms` / `Scene` and `apply_nvg` post-filmic pass (luminance * gain, green tint, screen-space noise, vignette).
  - `rotland_ui/src/hud/{boosts.rs (NEW), state.rs, draw.rs}` & `rotland_ui/src/hub_ui/draw_bag_details.rs`: added `BoostsHudInfo` + `draw_boosts` (heal timer, adrenaline timer, NVG battery bar) and `"Bite damage -X%"` tooltip in bag details.
  - `rotland/src/app/{metro_flags.rs, test_flags.rs}`, `docs/WORLD.md`, `tools/README_TOOLS.md`: added `--metro=herd|crows|crows_up|army|night` and `--nvg`.
- **Shared-file lines for the Lead:**
  - `rotland/src/main.rs`:
    ```rust
    mod play_metro;
    ```
  - `rotland_ui/src/hud/mod.rs`:
    ```rust
    pub mod boosts;
    pub use boosts::BoostsHudInfo;
    ```
- **Not run:** Per subagent rules, did not run `cargo`, `tools/gate.sh`, `blender`, or the game.
