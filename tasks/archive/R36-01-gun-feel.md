# R36-01: guns kick and flash

Owns: game_player/src/gun_feel.rs, data/gun_feel.json, game/tests/all/gun_feel.rs, rotland/src/gun_feel_view.rs, rotland/src/shooting.rs, rotland/src/fp_weapon.rs, game/src/camera_rig.rs, rotland/src/light.rs
Rules: `tasks/R36-common.md`. Owner decision: `docs/DECISIONS.md` row 2026-11-25 (weapons).

**Goal:** shooting feels weak today. `recoil_climb` is parsed in `game_core/src/weapons.rs`, but nothing uses it, and there is no muzzle flash. Add:
1. **View kick:** each shot pitches the camera up by `kickDeg` plus a small random sideways `kickYawDeg`. It eases back down at `recoverDegPerSec`. Automatic guns (ak47, smg) climb while you hold the trigger, up to `maxClimbDeg`. When aiming, the kick is `aimMul` (0.6) of normal. Works in first and third person.
2. **First-person gun kick:** the FP gun model jumps back `fpBack` m and tilts up `fpTiltDeg`, then springs back in ~0.12 s (`fp_weapon.rs`).
3. **Muzzle flash:** for 0.05 s, a small bright star-shaped quad (2 crossed quads, additive or unlit yellow-white) at the barrel tip, plus a short warm point light (radius `flashLight` m) using the existing light code. Bow and crossbow: no flash, small kick only.
4. **Data:** `data/gun_feel.json`, per weapon id, with these starting values:
   pistol kick 2.5 / revolver 5 / rifle 6 / hunting_rifle 7 / shotgun 8 / ak47 1.4 (climb, max 9) / smg 0.9 (climb, max 7) / bow 0.6 / crossbow 1.5. recoverDegPerSec 12, fpBack 0.06, fpTiltDeg 4, flashLight 4, aimMul 0.6.
- Pure math (kick state, ease back, climb clamp, aim multiplier) goes in `game_player/src/gun_feel.rs` with tests. `rotland/src/gun_feel_view.rs` holds `GunFeelView` with `on_shot(&mut self, weapon_id: &str)` plus a per-frame update that applies the offsets. Call `on_shot` from the place in `shooting.rs` where a shot is fired (and from the bow/crossbow fire).
- The kick must not change where bullets go after the shot has left. Only the view moves.

## Worker notes (view & wiring half)
### Files changed
- `rotland/src/gun_feel_view.rs` (new): `GunFeelView` with `on_shot(&mut self, weapon_id: &str)`, `set_muzzle`, `set_context`, `update`, `view_kick_deg`, `fp_kick`, `flash_light`, `draw_flash`, and `init` (2 crossed double-sided star-shaped quads, self-lit yellow-white `[255, 240, 140, 0]`).
- `game/src/camera_rig.rs`: added `kick_pitch_deg: f32`, `kick_yaw_deg: f32`, `apply_kick(&self, cam: &mut Camera)`, and `camera_with_kick(&self, ...)` + unit test. `CameraRig::camera()` stays unkicked so shooting uses the true aim direction.
- `rotland/src/fp_weapon.rs`: added `fp_kick: (f32, f32)` to `FpSway` and applied `fp_kick.0` (+Z back) and `fp_kick.1` (pitch up) in `fp_weapon_matrix` + unit test.
- `rotland/src/shooting.rs`: added `pub fired_weapon: Option<String>` to `ShootEvents`, set when `gun.try_fire` succeeds + unit test assertion.
- `rotland/src/light.rs`: added `muzzle_flash(position, direction, range)` and `add_flash_light(&mut Lighting, SpotLight)` + unit test.

### Shared-file lines for the lead
1. `rotland/src/main.rs` (near `mod ground_detail;`):
   ```rust
   mod gun_feel_view;
   ```
2. `rotland/src/app/state.rs` (in `RenderState`):
   ```rust
   pub gun_feel: crate::gun_feel_view::GunFeelView,
   ```
3. `rotland/src/app/frame_world.rs` (around shooting, after `shoot_tick`):
   ```rust
   let active_id = play_state.gun.as_ref().map(|g| g.id.as_str());
   render.gun_feel.set_context(active_id, shoot.aiming);
   if let Some(weapon_id) = ev.fired_weapon.as_deref() {
       let fp = actors.camera_rig.as_ref().is_some_and(|r| r.is_first_person);
       let dir = render.camera.forward();
       let muzzle_pos = if fp {
           render.camera.position + dir * 0.75 - glam::Vec3::Y * 0.1
       } else {
           p.position + glam::Vec3::Y * 1.35 + dir * 0.7
       };
       render.gun_feel.set_muzzle(muzzle_pos, dir);
       for _ in 0..ev.shots_fired {
           render.gun_feel.on_shot(weapon_id);
       }
   }
   ```
   And in `rotland/src/app/frame_actors_combat.rs` (when bow/crossbow fires in `if let Some(draw) = released`):
   ```rust
   render.gun_feel.set_context(Some(&weapon.id), c.bow.draw > 0.0);
   render.gun_feel.on_shot(&weapon.id);
   ```
   And once per frame (e.g. in `frame_world.rs`):
   ```rust
   render.gun_feel.update(dt);
   actors.fp_sway.fp_kick = render.gun_feel.fp_kick();
   if let Some(rig) = actors.camera_rig.as_mut() {
       let (pitch_deg, yaw_deg) = render.gun_feel.view_kick_deg();
       rig.kick_pitch_deg = pitch_deg;
       rig.kick_yaw_deg = yaw_deg;
   }
   ```
4. `rotland/src/app/render_frame.rs`:
   - Near the top of `render_frame` (after aim/shoot has already run, before building `sun_frustum` / `fp_weapon_matrix_with_physics` / `renderer.draw`):
     ```rust
     if let Some(rig) = actors.camera_rig.as_ref() {
         rig.apply_kick(&mut render.camera);
     }
     ```
   - After car headlights in `lighting.spots`:
     ```rust
     if let Some(flash) = render.gun_feel.flash_light() {
         light::add_flash_light(&mut lighting, flash);
     }
     ```
   - Before `renderer.draw(...)`:
     ```rust
     render.gun_feel.init(&gpu.device);
     render.gun_feel.draw_flash(&mut main);
     ```

### Not checked
- Did not run `cargo`, `tools/gate.sh`, or the game (per subagent rules). Relies on `game::gun_feel::{GunFeelConfig, GunFeelState}` from the sibling subagent matching the spec in the prompt.
