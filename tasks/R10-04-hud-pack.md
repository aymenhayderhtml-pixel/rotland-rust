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
