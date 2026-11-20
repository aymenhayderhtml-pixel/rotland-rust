# R12-03: F1 dev panel and --at=X,Z start flag

Read first: `AGENTS.md`, `docs/STATUS.md`, `tools/README_TOOLS.md`.

Files: rotland/src/dev_panel/mod.rs, rotland/src/dev_panel/draw.rs, rotland/src/dev_panel/actions.rs, game/src/dev_panel.rs, game/src/lib.rs, tasks/R12-03-dev-panel.md
Read: game/src/lib.rs, rotland/src/app/state.rs, rotland/src/camp_ui/draw.rs, rotland/src/app/frame_input.rs
API: rotland/src/zombies/mod.rs, rotland/src/zombies/spawning.rs, rotland/src/zombies/types.rs, rotland/src/zombies/combat_glue.rs, rotland/src/play.rs, game/src/items/inventory.rs, game/src/player.rs, game/src/health.rs, game/src/damage.rs, game/src/dir.rs, world/src/time.rs, world/src/generator.rs, world/src/poi.rs, physics/src/world.rs, engine/src/render/lit.rs, engine/src/input.rs, engine/src/cli.rs
Data: data/items.json, data/zombies.json
Test: dev_panel

Goal: Provide an in-game developer overlay toggled with F1 for testing, debugging, and screenshots without pausing the game, plus a `--at=X,Z` CLI start flag to position the player directly at specific coordinates.

## Features to Implement

1. **F1 Toggle & Mouse Freedom**:
   - Pressing `F1` toggles the egui "Dev" window open/closed during play.
   - When open, the game does NOT pause (simulation continues), but the mouse cursor is freed (`cursor_grab` returns false, camera look delta is suppressed, and melee click attack is prevented) so the user can interact with the panel.
   - Closing with `F1` or the window close `(X)` button re-captures the mouse on next click.
   - Key handling today: `engine::Input::pressed(KeyCode::F1)` using `winit::keyboard::KeyCode::F1` (or `ctx.input(|i| i.key_pressed(egui::Key::F1))`).

2. **Performance & Stats Section**:
   - Real-time display: current FPS, frame ms, triangle counts, shadow triangle counts, draw calls, and player position `(X, Y, Z)`.
   - FPS history graph of the last 300 frames:
     - Shows frame time in milliseconds.
     - Horizontal guide line at 16.7 ms (60 FPS target, green/yellow).
     - Horizontal guide line at 33.3 ms (30 FPS minimum, red/orange).
     - Drawn using egui's immediate-mode `painter` allocating a rect within the window.

3. **Teleport Section**:
   - **Camp spawn**: teleports player to settlement respawn point (`settlements[0].respawn_point(config)`).
   - **Custom X/Z**: input text fields for X and Z coordinates, with ground height sampled from the world (`physics.ground_height(x, z).unwrap_or(world.height(x, z)) + 0.1`).
   - **Nearest POI**: finds the nearest building from loaded streamer chunks / POI sites, and teleports the player adjacent to it.

4. **Time of Day & Freezing Section**:
   - Time of day slider from 0.0 to 24.0 hours, updating `play.clock.hour` and `play.hour`.
   - "Freeze time" checkbox toggling `play.clock_frozen`.

5. **Zombie Spawning & Clearance Section**:
   - Zombie spawn controls: select type from `data/zombies.json` (`walker`, `runner`), and button to spawn 8 m in front of the player (`p.position + yaw_forward(p.yaw) * 8.0`) at terrain ground height using `zombies.spawn_kind(kind, pos)`.
   - Kill all zombies: button to kill all active zombies within 50 m of the player using `zombies.hit(id, DamageInfo::new(9999.0).with_headshot(true), Vec3::ZERO)`.

6. **Item Spawning Section**:
   - Searchable list of all items from `data/items.json`.
   - Text search filter and quantity stepper/field (default 1).
   - "Give" button adds the item to the player's inventory via `inventory.add(id, count)` and updates hotbar cache.

7. **God Mode Toggle**:
   - Checkbox toggle for God Mode.
   - While active: player takes no damage, health remains full (`combat.health.heal(...)`), and survival needs stay full (`hunger.set(max)`, `thirst.set(max)`, `stamina.current = stamina.max`).

8. **CLI Flag `--at=X,Z`**:
   - When launching with `--at=X,Z` (e.g. `--at=150.0,-220.5`), the player spawns at that (X, Z) coordinate at world ground height at startup.
   - Parsed by pure helper `game::dev_panel::parse_at_flag`.

---

## File Architecture

Keep files small (~300 lines max), one topic per file, clippy-clean, no warnings.

### 1. `game/src/dev_panel.rs` (New pure logic in `game` crate, ~150 lines)
- `FpsHistory`:
  - Ring buffer storing the last 300 frame durations in milliseconds (`[f32; 300]`).
  - Methods: `new()`, `push(ms: f32)`, `len()`, `is_empty()`, `as_slice()`, `avg_ms()`, `max_ms()`.
- `parse_at_flag(s: &str) -> Option<(f32, f32)>`:
  - Parses comma-separated coordinates `"X,Z"`, handling optional spaces, positive and negative floats.
- `filter_items<'a>(items: &'a [&'a str], query: &str) -> Vec<&'a str>`:
  - Case-insensitive substring matching on item ids.
- Unit tests for all pure logic:
  - `#[cfg(test)] mod tests` testing `FpsHistory` ring buffer behavior and statistics, `parse_at_flag` valid and invalid inputs, and item filtering. Every test name must match `dev_panel`.

### 2. `game/src/lib.rs` (Existing file, ~80 lines)
- Expose `pub mod dev_panel;` and `pub use dev_panel::*;`. Neon provides the full updated file.

### 3. `rotland/src/dev_panel/mod.rs` (New file, ~120 lines)
- `pub mod actions;`
- `pub mod draw;`
- `pub use actions::{apply_dev_actions, DevAction};`
- `pub use draw::draw_dev_panel;`
- `pub struct DevPanelState`:
  - `pub open: bool`
  - `pub history: game::dev_panel::FpsHistory`
  - `pub custom_x: String`
  - `pub custom_z: String`
  - `pub selected_zombie_type: String`
  - `pub item_search: String`
  - `pub give_count: u32`
  - `pub god_mode: bool`
  - `pub status_msg: Option<String>`
- `pub struct DevPanelView<'a>`:
  - Read-only data gathered each frame: `player_pos: Vec3`, `player_yaw: f32`, `fps: f32`, `draws: u32`, `triangles: u64`, `shadow_draws: u32`, `shadow_triangles: u64`, `current_hour: f32`, `clock_frozen: bool`, `all_item_ids: &'a [String]`, `zombie_types: &'a [&'static str]`, `nearest_poi: Option<(String, Vec3)>`.
- `pub fn update_dev_panel(...)`: Convenience coordinator ticking frame time, checking F1, updating view, drawing window, applying actions, and maintaining god mode.

### 4. `rotland/src/dev_panel/draw.rs` (New file, ~250 lines)
- `pub fn draw_dev_panel(ctx: &egui::Context, state: &mut DevPanelState, view: &DevPanelView) -> Vec<DevAction>`
- Immediate mode UI using `egui::Window::new("Dev Panel").open(&mut state.open).resizable(true).default_width(340.0)`
- Renders:
  - Performance & Graph section (with custom painter lines for 16.7ms and 33.3ms)
  - Teleport controls (Camp, X/Z, Nearest POI)
  - Time of day slider & freeze checkbox
  - Zombie spawn picker & kill button
  - Item give search list and count stepper
  - God mode checkbox

### 5. `rotland/src/dev_panel/actions.rs` (New file, ~180 lines)
- `pub enum DevAction`:
  - `Teleport(Vec3)`
  - `SetTime(f32)`
  - `SetFreezeTime(bool)`
  - `SetGodMode(bool)`
  - `SpawnZombie(String)`
  - `KillZombiesNear`
  - `GiveItem { id: String, count: u32 }`
- `pub fn apply_dev_actions(actions: &[DevAction], state: &mut DevPanelState, world: &mut WorldState, actors: &mut ActorsState, play: &mut PlaySide, session: &mut SessionState)`
- `pub fn maintain_god_mode(god_mode: bool, actors: &mut ActorsState, play: &mut PlaySide)`: Restores health, stamina, hunger, and thirst to max every frame when enabled.

---

## Wiring Snippets in NOTES

Since existing files in `rotland/src/app/` are long (>200-400 lines), do NOT rewrite them as full files in Files:. Instead, provide exact copy-paste snippets in `=== NOTES ===`:
1. `rotland/src/main.rs`: Add `mod dev_panel;`.
2. `rotland/src/app/state.rs`: Add `pub dev_panel: crate::dev_panel::DevPanelState` to `UiState`, and include `self.dev_panel.open` in `UiState::screen_open()`. Add `pub start_at: Option<(f32, f32)>` to `SessionState`.
3. `rotland/src/app/frame_input.rs`: Add F1 toggle for `ui.dev_panel.open = !ui.dev_panel.open;`.
4. `rotland/src/app/frame_ui.rs`: Call `crate::dev_panel::update_dev_panel(...)` when `!on_main`.
5. `rotland/src/app/startup.rs`: Parse `--at=X,Z` into `session.start_at`.
6. `rotland/src/app/start_world.rs`: If `session.start_at` is set, spawn the player at that position.

## Neon notes


Summary of files written

  - game/src/dev_panel.rs: Pure logic (FpsHistory 300-frame chronological ring
    buffer, parse_at_flag, filter_items, dev_zombie_spawn_spot,
    dev_adjacent_poi_spot, item/zombie catalogues from data/items.json and
    data/zombies.json, and 9 unit tests matching dev_panel for cargo test
    dev_panel).
  - game/src/lib.rs: Exposed pub mod dev_panel; and pub use dev_panel::*;.
  - rotland/src/dev_panel/mod.rs: DevPanelState, DevPanelView, update_dev_panel
    coordinator, camp spawn lookup, and nearest POI lookup.
  - rotland/src/dev_panel/draw.rs: Immediate-mode egui 0.37 "Dev Panel" window
    with frame-time graph (16.7 ms & 33.3 ms guides), teleport controls,
    time-of-day slider/presets/freeze, zombie spawner & 50 m killer, searchable
    item giver with quantity stepper, and god mode toggle.
  - rotland/src/dev_panel/actions.rs: DevAction, apply_dev_actions, and
    maintain_god_mode.

Wiring snippets for existing files

1. rotland/src/main.rs

Add module declaration:

mod dev_panel;

(Also ensure rotland/src/app/mod.rs exposes pub mod state; or pub(crate) mod
state; so crate::app::state::{ActorsState, PlaySide, RenderState, SessionState,
WorldState} is accessible from crate::dev_panel.)

2. rotland/src/app/state.rs

In UiState:

    /// In-game F1 developer overlay state.
    pub dev_panel: crate::dev_panel::DevPanelState,

In UiState::screen_open(&self):

    pub fn screen_open(&self) -> bool {
        self.camp_ui.is_some() || self.hub.is_some() || self.dev_panel.open
    }

In SessionState struct and SessionState::default():

    pub start_at: Option<(f32, f32)>,

and in Default:

            start_at: None,

3. rotland/src/app/frame_input.rs

Right before the Esc menu check in process_input(...), add the F1 toggle:

    if input.pressed(KeyCode::F1) && !ui.menus.as_ref().is_some_and(|m| m.is_open()) {
        ui.dev_panel.open = !ui.dev_panel.open;
        ui.cursor_captured = false;
    }

And gate the left-click re-capture on !ui.screen_open() so clicking inside the
dev panel does not silently set cursor_captured = true while the window is open:

    if !ui.cursor_captured && !ui.screen_open() && input.mouse_pressed(MouseButton::Left) {
        ui.cursor_captured = true;
    }

4. rotland/src/app/frame_ui.rs

When !on_main (during gameplay):

    if !on_main
        && crate::dev_panel::update_dev_panel(
            ctx,
            &mut self.ui.dev_panel,
            &self.render,
            &mut self.world,
            &mut self.actors,
            &mut self.play,
            &mut self.session,
        )
    {
        self.ui.cursor_captured = false;
    }

5. rotland/src/app/startup.rs

When parsing CLI args (args: &engine::cli::EngineArgs):

    session.start_at = args.value("at").and_then(game::dev_panel::parse_at_flag);

6. rotland/src/app/start_world.rs

After initializing world and actors.player_ctrl (when not loading a saved
position, or overriding start position):

    if let Some((x, z)) = session.start_at {
        let y = world
            .physics
            .ground_height(x, z)
            .unwrap_or_else(|| world.height(x, z))
            + game::dev_panel::DEV_TELEPORT_GROUND_OFFSET;
        let pos = glam::Vec3::new(x, y, z);
        actors.player = pos;
        if let Some(p) = actors.player_ctrl.as_mut() {
            p.position = pos;
            p.velocity = glam::Vec3::ZERO;
        }
    }

Missing APIs & Known Risks

1.  world/src/settlement.rs (SettlementSite::respawn_point): Not included in the
    provided API signatures, so its return type (Vec2 vs Vec3) could not be
    verified. camp_spawn_pos in rotland/src/dev_panel/mod.rs therefore uses
    world.camps.first() (glam::Vec2 XZ on WorldState) sampled onto terrain
    ground height. If you prefer the exact settlement respawn point, swap
    camp_spawn_pos to call wgen.settlements().first().map(|s|
    s.respawn_point(wgen.config())).
2.  game/src/items/registry.rs (ItemRegistry): Iteration methods on ItemRegistry
    were not in the provided API signatures. DEV_ALL_ITEM_IDS in
    game/src/dev_panel.rs provides the sorted catalogue of all 43 items from
    data/items.json; if new items are added later, either update
    DEV_ALL_ITEM_IDS or read keys dynamically from play.play.items in
    all_item_ids().
3.  game/src/survival.rs (NeedStat) & God Mode damage prevention: NeedStat
    methods (set, max) were not in the provided API signatures, so
    maintain_god_mode uses PlayState::needs_respawn() (verified in
    rotland/src/play.rs:269), alongside Health::heal/revive and
    player.stamina.current = player.stamina.max. To also prevent hit-flinch or
    single-frame lethal spikes before frame_ui runs, skip applying incoming
    bite/fall damage when self.ui.dev_panel.god_mode is true in your combat
    update step.
4.  rotland/src/streaming (Streamer) & PoiZombieSite: Not included in the
    provided API signatures. resolve_nearest_poi checks loaded world.poi_markers
    first and falls back to a chunk-cached WorldGenerator::chunk_contents scan
    (±4 chunks), offsetting the teleport target outside the lot via
    PoiBuildingDef::reach() + PoiConfigData::lot_margin + 2.0.
5.  DamageInfo::with_headshot: Not listed in game/src/damage.rs, so
    kill_zombies_near constructs let mut info =
    DamageInfo::new(DEV_KILL_ZOMBIE_DAMAGE); info.headshot = true; directly via
    the public headshot field.
6.  Typing in panel text fields: Because simulation intentionally continues
    while the Dev Panel is open, WASD keys typed into the X/Z or item search
    text inputs will still move the player unless frame_input.rs suppresses
    movement when ctx.wants_keyboard_input() is true.

## Gate report
- date: 2026-11-20
- base: main
- files changed (6): game/src/dev_panel.rs, game/src/lib.rs, rotland/src/dev_panel/actions.rs, rotland/src/dev_panel/draw.rs, rotland/src/dev_panel/mod.rs, tasks/R12-03-dev-panel.md
- tests passed: 9
- clippy: PASS
- test: PASS
- checks: FAIL
- gate: FAIL (checks)
