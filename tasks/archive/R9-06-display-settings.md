# R9-06: Apply the FOV, render scale and V-Sync settings

Read `tasks/R9-common.md` first. Reference: `scripts/Core/GraphicsApplier.cs` / `scripts/World/GraphicsApplier.cs`, `scripts/Core/GameSettings.cs`, `scripts/Player/CameraRig.cs` (FOV). The Rust settings are in `rust/rotland/src/settings.rs` (field_of_view, render_scale, vsync); the menu already edits them, but the game ignores them.

You own `rust/engine/src/app.rs`, `rust/engine/src/gpu.rs`, `rust/engine/src/lib.rs` (the `Game` trait) and a new `rust/rotland/src/display.rs`. Do NOT touch `engine/src/render/` (another lane edits the lighting there).
- Engine: let the game change render scale and V-Sync while running: e.g. a `Game` trait method with a default `fn display(&self) -> Option<DisplayRequest { render_scale: f32, vsync: bool }> { None }` that the app reads each frame and applies when it changed (reconfigure the surface present mode; resize the offscreen target the way `--scale` does at start, app.rs line ~252). Command-line `--scale` / `--vsync` must still win when given (tests and benchmarks use them).
- `rotland/src/display.rs`: `pub fn base_fov(settings: &Settings) -> f32` and `pub fn apply_fov(camera_fov_from_rig: f32, settings) -> f32` (how CameraRig uses the setting: replace or scale the default), plus `DisplayRequest` from `Settings`.
- Notes: the exact lines Claude adds in main.rs (camera FOV before the aim zoom; implementing `display()`).
Tests: `#[cfg(test)]` in display.rs (FOV 90 setting gives the Godot camera FOV; scale clamps 0.5..1.0), and in engine where practical (the "changed?" check).

## Worker notes

### Files changed
- `rust/engine/src/gpu.rs`: Added `supported_present_modes` to `Gpu`, added `Gpu::choose_present_mode(modes: &[wgpu::PresentMode], vsync: bool) -> wgpu::PresentMode`, and added `Gpu::set_vsync(&mut self, vsync: bool)` to reconfigure the surface present mode dynamically at runtime. Added unit tests for present mode selection.
- `rust/engine/src/app.rs`: Defined `DisplayRequest` and `DisplayChange` structs; added `check_display_change` to detect differences and enforce CLI override precedence (`--scale` and `--vsync`); added `display(&self) -> Option<DisplayRequest>` default method to `Game` trait; added dynamic display change polling and handling in `App::frame` (reconfiguring surface present mode via `Gpu::set_vsync` and resizing offscreen target via `rebuild_target`); added unit tests covering change detection, clamping, and CLI overrides.
- `rust/engine/src/lib.rs`: Re-exported `DisplayRequest`, `DisplayChange`, and `check_display_change`.
- `rust/rotland/src/display.rs`: Created new module implementing `base_fov`, `apply_fov`, `display_request`, `from_settings`, and `From<&Settings>` / `From<Settings>` trait implementations for `DisplayRequest`. Added unit tests verifying FOV 90 computation, clamping ranges, aim FOV scaling, render scale clamping, vsync reflection, and conversions.

### Public API
- `engine::DisplayRequest { pub render_scale: f32, pub vsync: bool }` (with `DisplayRequest::new(render_scale: f32, vsync: bool) -> Self`).
- `engine::DisplayChange { pub new_scale: Option<f32>, pub new_vsync: Option<bool> }` (with `has_change(&self) -> bool`).
- `engine::check_display_change(args: &EngineArgs, current_scale: f32, current_vsync: bool, req: Option<DisplayRequest>) -> Option<DisplayChange>`.
- `engine::Game::display(&self) -> Option<DisplayRequest>` (default: `None`).
- `engine::Gpu::supported_present_modes: Vec<wgpu::PresentMode>`.
- `engine::Gpu::choose_present_mode(modes: &[wgpu::PresentMode], vsync: bool) -> wgpu::PresentMode`.
- `engine::Gpu::set_vsync(&mut self, vsync: bool)`.
- `rotland::display::DEFAULT_CAMERA_FOV: f32 = 75.0`.
- `rotland::display::base_fov(settings: &Settings) -> f32`.
- `rotland::display::apply_fov(camera_fov_from_rig: f32, settings: &Settings) -> f32`.
- `rotland::display::display_request(settings: &Settings) -> DisplayRequest`.
- `rotland::display::from_settings(settings: &Settings) -> DisplayRequest`.
- `rotland::display::DisplayRequest` (re-exported from `engine`).

### Traced test numbers and sources
- `75.0`: Default camera FOV in degrees from Godot `CameraRig.cs` line 24 (`DefaultZoom` / camera FOV default) and `rust/game/src/camera_rig.rs` line 24 (`DEFAULT_FOV = 75.0`).
- `90.0`: User FOV setting tested in `fov_90_setting_gives_godot_camera_fov`; un-aimed FOV 75.0 becomes 90.0 (`apply_fov(75.0, settings)` = 90.0).
- `60.0`: Minimum FOV limit from `scripts/Core/GameSettings.cs` line 15 (`MinFov = 60f`) and `rust/rotland/src/settings.rs` line 9 (`MIN_FOV = 60.0`).
- `100.0`: Maximum FOV limit from `scripts/Core/GameSettings.cs` line 16 (`MaxFov = 100f`) and `rust/rotland/src/settings.rs` line 10 (`MAX_FOV = 100.0`).
- `55.0`: Third-person aim FOV from `scripts/Player/CameraRig.cs` line 30 (`ThirdPersonAimFov = 55f`) and `rust/game/src/camera_rig.rs` line 22 (`THIRD_PERSON_AIM_FOV = 55.0`). When scaled with FOV 90 setting (`90.0 / 75.0`), it scales to `66.0`.
- `0.5`: Minimum render scale limit from `scripts/World/GraphicsApplier.cs` line 47 (`Mathf.Clamp(preset.RenderScale, 0.5f, 1f)`) and `rust/rotland/src/settings.rs` line 11 (`MIN_RENDER_SCALE = 0.5`).
- `1.0`: Maximum render scale limit from `scripts/World/GraphicsApplier.cs` line 47 and `rust/rotland/src/settings.rs` line 12 (`MAX_RENDER_SCALE = 1.0`).
- CLI override precedence: `--scale=0.5` suppresses in-game render scale changes; `--vsync` suppresses in-game vsync toggle.

### Wiring for Claude (in `rust/rotland/src/main.rs`)
1. **Module declaration**: Add `mod display;` near top of `rust/rotland/src/main.rs` (around line 32 alongside other module declarations).
2. **FOV application in `update`**: In `R0::update` (lines ~1015-1033), after the camera is calculated from the rig and car chase camera, and immediately before the gun aim zoom:
```rust
                self.player = p.position;
                self.camera = rig.camera(&self.player, &self.physics);
                // Driving: a chase camera further back and higher than on foot (Godot CarCamera)
                if self.cars.as_ref().is_some_and(|c| c.driven_car_id.is_some()) {
                    self.camera.pitch = self.camera.pitch.clamp(-0.35, 0.15);
                    let f = self.camera.forward();
                    let mut at = self.player + Vec3::Y * 2.2 - f * 7.0;
                    // Never under a hill behind the car
                    if let Some(h) = self.physics.ground_height(at.x, at.z) {
                        at.y = at.y.max(h + 1.2);
                    }
                    self.camera.position = at;
                }
                // Apply player's FOV setting from settings menu (before gun aim zoom)
                if let Some(m) = &self.menus {
                    self.camera.fov_y_deg = display::apply_fov(self.camera.fov_y_deg, &m.settings);
                }
                // A gun's aim zoom (the rifle scope zooms much further, CameraRig/GunHandler)
                let aiming = self.cursor_captured && input.mouse_held(winit::event::MouseButton::Right);
```
3. **Display request in `Game` trait**: In `impl Game for R0` (around line 1960 alongside `cursor_grab` / `should_quit`):
```rust
    fn display(&self) -> Option<engine::DisplayRequest> {
        self.menus.as_ref().map(|m| display::display_request(&m.settings))
    }
```
