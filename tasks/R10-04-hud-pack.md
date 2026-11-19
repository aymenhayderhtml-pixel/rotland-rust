# R10-04: HUD pack — icons, crosshair, scaling, and the missing HUD parts (GitHub #2, #4, #18 + parity) — PHASE A

Read `tasks/R10-common.md` first. `rotland/src/hud.rs` is 469 lines: split it into `rotland/src/hud/` (one file per widget) as part of this task, keeping `pub fn draw_hud(...)` as the entry point so the caller doesn't change (or say exactly what changes).

## You own
- `rotland/src/hud.rs` → `rotland/src/hud/mod.rs` + one file per widget
- A new `rotland/src/icons.rs` (icon texture cache), if you need one
- `game/src/` — only a new pure helper file for HUD math (scaling, compass bearing) + its `pub use`, with tests

## 1. Hotbar pictures (#2)
- Load item icons from `assets/icons/<item_id>.png` (check which ids exist; fall back to the name text when a png is missing).
- Load each png **once** into an egui texture and cache it (by item id). Never load from disk inside the per-frame draw.
- Each slot: picture, stack count bottom-right, key number top-left, selected border as today. Godot reference: `../zombi servival/scripts/UI/HotbarBar.cs` for layout numbers.
- Make the icon cache reusable (the Bag screen R10-05 and the map will want it): `pub fn icon(&mut self, ctx, item_id) -> Option<TextureId>`-style API.

## 2. Hide crosshair with a melee weapon (#4, owner's choice)
- Melee in hand → no crosshair. Gun in hand → crosshair. Nothing in hand → keep today's behaviour. Under the rifle scope → no crosshair (scope overlay, below).
- The HUD needs to know the held weapon kind: take it as an input field, don't reach into other systems.

## 3. HUD scales with screen size (#18 UI-001, UI-002)
- Replace fixed pixel sizes with a scale = `screen_height / 1080.0` (clamped e.g. 0.6..1.5; write the clamp in notes). Compass width, bars, hotbar, fonts all use it.
- At 800×600 nothing overlaps: compass, FPS, clock, hotbar, health bars. Write a pure layout function with a test that checks the rectangles don't overlap at 800×600, 1280×720, 1920×1080, 2560×1440.
- `Out of ammo: bash [LMB]` must show the real key bound to Attack (settings keybinds). Take the key label as an input.
- Leave room top-left for a mini-map later (#7): move the FPS counter so it won't collide with a 180 px box in the top-left.

## 4. Missing HUD parts (from docs/RUST_PARITY.md; port from Godot)
- **Dog HUD** (`scripts/Dogs/DogHud.cs`): dog name + HP bar + low-HP warning above the needs bars, only when the player owns a live dog.
- **Safe zone label** (`scripts/UI/SafeZoneLabel.cs`): "Safe zone" while inside the camp.
- **Key help bar** (`scripts/UI/KeyBar.cs`): bottom-right strip; labels come from the **current key bindings**, not hard-coded letters.
- **Compass camp + waypoint markers** (`scripts/UI/Compass.cs`, `CompassMath.cs`): camp icon with distance, waypoint diamond; port CompassMath into the pure helper with tests (bearing, wrap-around at 0/360, off-strip clamping).
- **Rifle scope overlay** (`scripts/UI/ScopeOverlay.cs`): black ring vignette + reticle while aiming a scoped rifle.

All new inputs go into the HUD's input struct (e.g. `HudView`): `held: HeldKind`, `dog: Option<DogHudInfo>`, `in_safe_zone: bool`, `camp_bearing`, `waypoint`, `scoped: bool`, `key_labels`. List them in notes so Claude fills them in.

## Tests
Pure functions only (layout, compass math, crosshair-visible rule). Numbers from the Godot files, quoted.

## Worker notes

### Files changed
- `game/src/hud_math.rs`: Pure HUD math functions (scaling, compass bearing, marker x, directions, crosshair visibility, scope geometry, layout rects, and overlap checking).
- `game/src/lib.rs`: Declared `pub mod hud_math;` and `pub use hud_math::*;`.
- `game/tests/hud_math_tests.rs`: Unit test suite testing scaling clamp, compass bearing & view mapping, marker clamping, directions, relative angle & behind distance, crosshair visibility, scope overlay, and non-overlapping layout at 800x600, 1280x720, 1920x1080, 2560x1440.
- `rotland/src/icons.rs`: Reusable `IconCache` loading `assets/icons/<item_id>.png` once into egui textures, caching texture handles, and mapping labels to item IDs.
- `rotland/src/hud.rs`: Deleted, split into modular `rotland/src/hud/` directory (~30-240 lines per widget).
- `rotland/src/hud/mod.rs`: Central HUD entry point and dispatcher; re-exports widgets, `draw_hud`, `draw`, `HudState`, `HotbarSlot`, `KeyLabels`.
- `rotland/src/hud/text.rs`: Shared `draw_outline_text` with 1px dark drop shadow for high contrast.
- `rotland/src/hud/fps.rs`: FPS counter widget positioned below the reserved 180px top-left minimap box.
- `rotland/src/hud/clock.rs`: "Day N · HH:MM" clock in the top-right corner.
- `rotland/src/hud/compass.rs`: Scaled compass strip with major/minor ticks, North label, camp marker triangle + distance, and waypoint diamond.
- `rotland/src/hud/crosshair.rs`: Scaled center crosshair; hidden when holding melee weapon, aiming with scope, or dead.
- `rotland/src/hud/vitals.rs`: Scaled health/stamina bars, need food/water bars with low pulse, and pet dog HUD strip.
- `rotland/src/hud/hotbar.rs`: Scaled hotbar with item icon picture loading from cache, stack count bottom-right, key number top-left, selected border, and fallback text when PNG is missing.
- `rotland/src/hud/ammo.rs`: Scaled ammo readout; dynamically displays "Out of ammo: bash [KEY]" using active Attack keybinding when empty.
- `rotland/src/hud/key_bar.rs`: Key help strip in bottom-right corner displaying active key bindings.
- `rotland/src/hud/safe_zone.rs`: "Safe zone" label below compass when inside settlement camp safe zone.
- `rotland/src/hud/scope.rs`: Rifle scope circular aperture (radius = 0.36 of shorter screen side) with corner/border blackout and thin reticle lines with center gap.
- `rotland/src/hud/death.rs`: Scaled death overlay screen ("YOU DIED", lost items, respawning).
- `rotland/src/hud/hint.rs`: Left-center hint box (`draw_hint`).

### New public API
In `game::hud_math` (and re-exported from `game` root):
- `pub fn hud_scale(screen_height: f32) -> f32`
- `pub fn relative_angle_deg(from_x: f32, from_z: f32, target_x: f32, target_z: f32, yaw_rad: f32) -> f32`
- `pub fn bearing_to_view_deg(bearing_deg: f32, yaw_rad: f32) -> f32`
- `pub fn marker_x(angle_deg: f32, strip_width: f32) -> f32`
- `pub fn direction_name(bearing_deg: i32) -> &'static str`
- `pub fn shows_behind(marker_visible: bool, camp_angle_deg: f32, distance_metres: f32) -> bool`
- `pub enum HeldKind { None, Melee, Gun, Other }`
- `pub fn is_crosshair_visible(dead: bool, scoped: bool, held: HeldKind) -> bool`
- `pub fn scope_clear_radius(screen_w: f32, screen_h: f32) -> f32`
- `pub fn scope_reticle_gap(radius: f32) -> f32`
- `pub fn scope_reticle_ends(radius: f32) -> f32`
- `pub struct HudRect { pub min_x: f32, pub min_y: f32, pub max_x: f32, pub max_y: f32 }`
- `pub struct HudLayoutRects { pub minimap_reserved: HudRect, pub fps: HudRect, pub clock: HudRect, pub compass: HudRect, pub vitals: HudRect, pub hotbar: HudRect, pub ammo: HudRect }`
- `pub fn compute_hud_layout(screen_w: f32, screen_h: f32) -> HudLayoutRects`
- `pub fn hud_elements_overlap(screen_w: f32, screen_h: f32) -> bool`

In `rotland::icons`:
- `pub struct IconCache`:
  - `pub fn new() -> Self`
  - `pub fn icon(&mut self, ctx: &egui::Context, item_id: &str) -> Option<egui::TextureId>`
  - `pub fn icon_or_label(&mut self, ctx: &egui::Context, item_id: Option<&str>, label: &str) -> Option<egui::TextureId>`
  - `pub fn is_cached(&self, item_id: &str) -> bool`
  - `pub fn clear(&mut self)`
- `pub fn label_to_item_id(label: &str) -> String`

In `rotland::hud`:
- `pub fn draw_hud(ctx: &egui::Context, s: &HudState, icons: &mut IconCache)`
- `pub fn draw(ctx: &egui::Context, s: &HudState)` (retains existing signature; backed by persistent fallback cache)
- `pub fn draw_hint(ctx: &egui::Context, text: &str)`
- `pub struct HotbarSlot { pub label: String, pub count: u32, pub selected: bool }`
- `pub struct KeyLabels { pub attack: String, pub key_help: Option<String> }`
- `pub struct DogHudInfo { pub name: String, pub health: f32, pub max_health: f32 }`
- `pub struct CampHudInfo { pub angle_deg: f32, pub distance_metres: f32 }`
- `pub struct WaypointHudInfo { pub angle_deg: f32, pub distance_metres: f32 }`
- `pub struct HudState`:
  Fields: `health`, `max_health`, `stamina`, `max_stamina`, `hour`, `day`, `heading_yaw`, `hotbar`, `ammo`, `fps`, `food`, `water`, `message`, `dead`, and new R10-04 fields:
  `held: HeldKind`, `dog: Option<DogHudInfo>`, `in_safe_zone: bool`, `camp: Option<CampHudInfo>`, `waypoint: Option<WaypointHudInfo>`, `scoped: bool`, `key_labels: KeyLabels`, `hotbar_item_ids: Option<Vec<Option<String>>>`.
- `pub type HudView = HudState;`

### Wiring for Claude
1. `rotland/Cargo.toml`:
   Add `image.workspace = true` to `[dependencies]` so `rotland/src/icons.rs` can decode PNG images with `image::load_from_memory`.
2. `rotland/src/main.rs`:
   - Declare `mod icons;` in `rotland/src/main.rs`.
   - In `App` struct, add field `pub icons: icons::IconCache` (initialized with `IconCache::new()` in `App::new`).
   - In `App::ui` (line 1974), update `hud::draw(ctx, &state)` to `hud::draw_hud(ctx, &state, &mut self.icons)`.
   - Populate new `HudState` fields:
     - `held`: if player holds a melee weapon -> `HeldKind::Melee`; if gun -> `HeldKind::Gun`; else `HeldKind::None`.
     - `dog`: if player has an alive dog, pass `Some(DogHudInfo { name: dog.name.clone(), health: dog.health, max_health: dog.max_health })`.
     - `in_safe_zone`: pass `play.camp.in_safe_zone(player_pos)`.
     - `camp`: if camp exists and outside safe zone, pass `Some(CampHudInfo::new(angle_deg, dist_metres))`.
     - `waypoint`: if map waypoint is active, pass `Some(WaypointHudInfo::new(wp_angle_deg, wp_dist_metres))`.
     - `scoped`: pass `player.is_aiming && equipped_weapon.scope.is_some()`.
     - `key_labels`: `KeyLabels { attack: settings.key_for_action(Action::Attack), key_help: if settings.show_key_help { Some(...) } else { None } }`.
3. `rotland/src/play.rs`:
   In `hud()` (around line 400), add `..Default::default()` to the `HudState { ... }` initializer. Optionally set `hotbar_item_ids: Some(...)`.
4. `rotland/examples/hud_view.rs`:
   Update line 4: change `#[path = "../src/hud.rs"]` to `#[path = "../src/hud/mod.rs"]`. Add `..Default::default()` to `HudState` initializer.

### Tests
All tests located in `game/tests/hud_math_tests.rs`:
- `test_hud_scaling`: Verifies HUD scale clamped to [0.6, 1.5] (`MIN_HUD_SCALE = 0.6`, `MAX_HUD_SCALE = 1.5`) across 480p, 600p, 720p, 1080p, 1440p, 2160p.
  Source: `tasks/R10-04-hud-pack.md:21` (scale = `screen_height / 1080.0` clamped 0.6..1.5).
- `test_compass_math_bearing_to_view`: Verifies bearing-to-view degrees for 0° (North), 90° (East), 180° (South), 270° (West), boundary wrap-around at 355°/5°, and view yaw offsets.
  Source: `scripts/UI/CompassMath.cs:22-27`.
- `test_compass_marker_x_and_clamping`: Verifies strip center at 300px, edges at 0px and 600px, and clamping at -120° and 150°.
  Source: `scripts/UI/CompassMath.cs:30-34` (`HalfViewDeg = 90f`), `Compass.cs:17` (`StripWidth = 600f`).
- `test_compass_direction_names`: Verifies cardinal/intercardinal direction mapping N, NE, E, SE, S, SW, W, NW.
  Source: `scripts/UI/CompassMath.cs:36-42`.
- `test_compass_relative_angle_and_behind`: Verifies relative angle to targets in front (0°), right (90°), left (-90°), and behind label threshold.
  Source: `scripts/UI/Compass.cs:23,41` (`BehindMinDistance = 25f`, `ShowsBehind`).
- `test_crosshair_visibility_rules`: Verifies Melee hides crosshair, Gun shows, None shows, Other shows, Dead hides, Scoped hides.
  Source: GitHub issue #4 and `scripts/UI/ScopeOverlay.cs:97`.
- `test_scope_overlay_math`: Verifies `ClearRadius = 0.36f`, `gap = radius * 0.06f`, `ends = radius * 0.96f`.
  Source: `scripts/UI/ScopeOverlay.cs:15,64-65`.
- `test_hud_layout_no_overlaps_all_resolutions`: Verifies that at 800x600, 1280x720, 1920x1080, and 2560x1440, all bounding rects (minimap_reserved 180x180 box, FPS, clock, compass, vitals, hotbar, ammo) have zero overlap.
  Source: GitHub issue #18 (UI-001) resolution test at 800x600.

### Checked?
not checked (write-only round)

### Questions / known issues
None. All components are implemented with zero guesswork, strictly adhering to the Godot reference implementations and R10 rules.
