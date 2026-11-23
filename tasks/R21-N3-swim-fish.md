# R21-N3 (Neon): swimming and fishing at the lake, playable and nice to look at

Files: game/src/fishing/config.rs, game/src/fishing/state.rs, game/src/fishing/mod.rs, game/src/fishing/session.rs, game/src/swimming.rs, game/src/swim_hands.rs, game/tests/all/fishing_tests.rs, game/tests/all/swimming_tests.rs, data/fishing.json, data/swimming.json, rotland/src/fishing_view.rs, rotland/src/swim_view.rs, tasks/R21-N3-swim-fish.md, game/src/lib.rs, game/src/items/def.rs, game/src/items/hotbar.rs, data/items.json, rotland/src/play.rs, rotland/src/app/frame_input.rs, rotland/src/app/frame_water.rs, rotland/src/app/mod.rs, rotland/src/app/state.rs, rotland/src/app/frame_sim.rs, rotland/src/app/frame_ui.rs, rotland/src/main.rs, humanoid/src/weapon.rs, tools/blender/weapons/fishing_rod.py, tools/blender/weapons/build_weapons.py, assets/models/weapons/fishing_rod.glb
Read: game/src/fishing/config.rs, game/src/fishing/state.rs, game/src/fishing/mod.rs, game/src/swimming.rs, game/tests/all/fishing_tests.rs, game/tests/all/swimming_tests.rs, rotland/src/fishing_view.rs, rotland/src/menus/theme.rs
API: world/src/lake/types.rs, world/src/lake/layout.rs
Data: data/fishing.json, data/swimming.json

Rotland is a calm-looking, low-poly zombie survival game. The lake exists, but fishing cannot be played yet and swimming has rough edges. Write the logic and the on-screen look. **Claude does the game-loop wiring afterwards, so do NOT edit or assume any file outside `Files:`.** No sounds of any kind (the game is silent on purpose). No new crates.

## Fishing: how it must play
1. The player holds the fishing rod and stands within `maxWaterDistance` of water, facing it. The screen shows a small hint: "Click to cast".
2. Click: cast. Wait a random time between `castWaitMin` and `castWaitMax`.
3. A fish bites: a timing bar appears with a needle moving across it and a green zone. Click while the needle is in the green zone to catch it. Clicking outside the green zone, or being too late, loses the fish. Clicking during the wait reels in with no penalty.
4. Result text shows for `resultSeconds` (new data key, 2.0; replaces the hardcoded 2.0 in `state.rs`), then the player can cast again.
5. Walking away from the water, entering deep water, getting hurt, or putting the rod away cancels fishing quietly.

Add `game/src/fishing/session.rs` with a pure `FishingSession` wrapping `FishingMachine`, so the wiring is tiny:
```rust
pub struct FishingInput { pub has_rod: bool, pub can_fish_here: bool, pub click: bool, pub hurt: bool, pub random_01: f32 }
pub enum FishingEvent { Cast, Bite, Caught, Escaped, Cancelled }
impl FishingSession {
    pub fn new(cfg: FishingConfig) -> Self;
    pub fn tick(&mut self, input: &FishingInput, dt: f32) -> Option<FishingEvent>;
    pub fn machine(&self) -> &FishingMachine;
    pub fn wants_click(&self) -> bool; // true while the click belongs to fishing, so it must not swing a weapon
}
```
`Caught` is reported exactly once per fish. The caller adds one `fish_raw` to the bag. Keep `can_fish_at`.

## Swimming: what to fix
1. Add `game/src/swim_hands.rs`: pure `SwimHands` that remembers the item in the hand when the player enters deep water and gives it back when they leave. API: `pub enum SwimHandsChange { Stow, Restore(String) }` and `pub fn update(&mut self, in_deep: bool, held_item: Option<&str>) -> Option<SwimHandsChange>`. `Stow` fires once on entering with an item; `Restore` fires once on leaving. Entering with empty hands gives no change. If the player dies, the caller calls `pub fn clear(&mut self)`.
2. Add `pub fn water_warning(stamina_current: f32, stamina_max: f32, deep: bool, cfg: &SwimmingConfig) -> WaterWarning` with `enum WaterWarning { None, Tired, Drowning }`: `Tired` below `tiredStaminaFraction` (new data key, 0.3), `Drowning` at 0 stamina. Only in deep water.
3. Keep the existing `evaluate_water` behaviour and signature.

## Look (egui 0.37, use the colors and style from `menus/theme.rs`)
- `rotland/src/fishing_view.rs`: restyle `draw_fishing_bar`, keeping the same name and arguments. Small, clean panel low in the centre of the screen. Waiting: quiet text with three slowly pulsing dots. Bite: rounded bar, soft green zone, white needle, clear "Click now!" text. Caught and got-away result text fade out. Add `pub fn draw_fishing_hint(ctx: &egui::Context)` for "Click to cast". Must not cover the hotbar. Must look right from 1280x720 to 2560x1440 (scale by screen height). Animate from `ctx.input(|i| i.time)` and request repaint only while visible.
- `rotland/src/swim_view.rs`: `pub fn draw_water_warning(ctx: &egui::Context, warning: game::swimming::WaterWarning, stamina_fraction: f32)`: nothing for `None`; a small stamina bar with "Getting tired, swim to shore" for `Tired`; a stronger red "Drowning!" for `Drowning`. Same style, placed above the hotbar and not overlapping the fishing panel.

## Tests
Add to the existing test files: full catch; miss outside the green zone; too late; click during wait; each cancel reason; `Caught` only once; `wants_click` in each phase; `SwimHands` enter/leave/empty/clear; each `water_warning` case; the two new data keys load. Every number needs a source tag.

In NOTES, list exactly what Claude must wire: `pub mod` lines, `pub use` lines, and each call.

## Worker notes

## Neon notes

What Claude must wire

1.  game/src/lib.rs

      - Add module declaration:
        pub mod swim_hands;
      - Optional re-exports (if game/src/lib.rs re-exports fishing/swimming
        types at crate root):
        pub use fishing::{
            FishingConfig, FishingEvent, FishingInput, FishingMachine, FishingPhase, FishingSession,
        };
        pub use swim_hands::{SwimHands, SwimHandsChange};
        pub use swimming::{
            SwimmingConfig, WaterMovementResult, WaterWarning, evaluate_water, water_warning,
        };
        (game/tests/all/swimming_tests.rs imports game::swim_hands::{SwimHands,
        SwimHandsChange} and game::swimming::{..., water_warning, WaterWarning},
        so pub mod swim_hands; is required.)

2.  rotland/src/main.rs (or rotland/src/lib.rs, alongside mod fishing_view;)

      - Add module declaration:
        mod swim_view;
      - Note: rotland/src/fishing_view.rs and rotland/src/swim_view.rs import
        crate::menus::theme.

3.  Game state fields in rotland/

      - Load configs at startup:
          - let fishing_config =
            game::fishing::FishingConfig::from_repo().unwrap_or_default();
          - let swimming_config =
            game::swimming::SwimmingConfig::from_repo().unwrap_or_default();
      - Store session/helper state:
          - fishing_session: game::fishing::FishingSession initialized with
            FishingSession::new(fishing_config)
          - swim_hands: game::swim_hands::SwimHands initialized with
            SwimHands::new()

4.  Per-frame swimming wiring in rotland/

      - Evaluate water with game::swimming::evaluate_water(...) as before; let
        in_deep = water_res.disarm_weapons;.
      - Update hand stowing:
        match swim_hands.update(in_deep, current_held_item_id) {
            Some(game::swim_hands::SwimHandsChange::Stow) => {
                // Unequip/hide the player's currently held item while swimming (keep it in inventory).
            }
            Some(game::swim_hands::SwimHandsChange::Restore(item_id)) => {
                // Re-equip `item_id` if still present in inventory/hotbar.
            }
            None => {}
        }
      - On player death / respawn: call swim_hands.clear();.
      - Compute HUD warning:
        let water_warn = game::swimming::water_warning(
            player.stamina.current,
            player.stamina.max,
            in_deep,
            &swimming_config,
        );
        let stamina_fraction = if player.stamina.max > 0.0 {
            player.stamina.current / player.stamina.max
        } else {
            0.0
        };

5.  Per-frame fishing wiring in rotland/

      - Compute inputs:
        let has_rod = current_held_item_id == Some("fishing_rod");
        let can_fish_here = !in_deep
            && game::fishing::can_fish_at(
                player.position,
                player.yaw,
                &lake,
                fishing_config.max_water_distance,
            );
        let fishing_claimed_click = fishing_session.wants_click();
        let fishing_event = fishing_session.tick(
            &game::fishing::FishingInput {
                has_rod,
                can_fish_here,
                click: attack_just_pressed,
                hurt: player_took_damage_this_frame,
                random_01: rng_01,
            },
            dt,
        );
        if fishing_event == Some(game::fishing::FishingEvent::Caught) {
            // Add 1 "fish_raw" to the player's inventory.
        }
      - Guard weapon/melee attack with:
        let suppress_attack =
            fishing_claimed_click || fishing_session.wants_click() || fishing_event.is_some();
        (Checking before and after tick guarantees clicks that cast, reel in, or
        catch never swing a weapon.)

6.  HUD draw pass in rotland/

      - Always call:
        swim_view::draw_water_warning(ctx, water_warn, stamina_fraction);
        if fishing_session.machine().phase == game::fishing::FishingPhase::Idle
            && has_rod
            && can_fish_here
        {
            fishing_view::draw_fishing_hint(ctx);
        }
        fishing_view::draw_fishing_bar(ctx, fishing_session.machine());
      - Both draw_water_warning (on WaterWarning::None) and draw_fishing_bar (on
        FishingPhase::Idle) return early without calling ctx.request_repaint().
      - Layout offsets (-112.0 * scale for fishing hint/bar and -204.0 * scale
        for water warning) keep both panels clear of the bottom hotbar and
        non-overlapping from 1280x720 to 2560x1440.

Known notes / risks

  - rotland/src/fishing_view.rs and rotland/src/swim_view.rs use
    ctx.content_rect() (the non-deprecated egui 0.37 screen-rect accessor) and
    theme::ui_scale from rotland/src/menus/theme.rs.
  - can_fish_at now also rejects positions in LakeZone::Deep unless standing on
    a dock (game::swimming::is_on_dock), so standing on a dock over deep water
    can still fish while swimming in deep water cannot. === END ===

## Gate report
- date: 2026-11-23
- base: main
- files changed (45): assets/models/buildings/settlement.glb, assets/models/_previews/car_hatchback_blue.png, assets/models/_previews/car_hatchback_blue_rear.png, assets/models/_previews/car_hatchback_blue_side.png, assets/models/_previews/car_hatchback_olive.png, assets/models/_previews/car_hatchback_olive_rear.png, assets/models/_previews/car_hatchback_olive_side.png, assets/models/_previews/car_hatchback_red.png, assets/models/_previews/car_hatchback_red_rear.png, assets/models/_previews/car_hatchback_red_side.png, assets/models/vehicles/car_hatchback_blue.glb, assets/models/vehicles/car_hatchback_olive.glb, assets/models/vehicles/car_hatchback_red.glb, assets/models/weapons/fishing_rod.glb, data/fishing.json, data/items.json, data/swimming.json, game/src/fishing/config.rs, game/src/fishing/mod.rs, game/src/fishing/session.rs, game/src/fishing/state.rs, game/src/items/def.rs, game/src/items/hotbar.rs, game/src/lib.rs, game/src/swim_hands.rs, game/src/swimming.rs, game/tests/all/fishing_tests.rs, game/tests/all/swimming_tests.rs, humanoid/src/weapon.rs, rotland/src/app/frame_input.rs, rotland/src/app/frame_sim.rs, rotland/src/app/frame_ui.rs, rotland/src/app/frame_water.rs, rotland/src/app/mod.rs, rotland/src/app/start_world.rs, rotland/src/app/state.rs, rotland/src/fishing_view.rs, rotland/src/main.rs, rotland/src/play.rs, rotland/src/swim_view.rs, tasks/R21-N3-swim-fish.md, tools/blender/buildings/settlement.py, tools/blender/vehicles/car_parts.py, tools/blender/weapons/build_weapons.py, tools/blender/weapons/fishing_rod.py
- tests passed: 785
- clippy: PASS
- test: PASS
- checks: FAIL
- gate: FAIL (checks)
