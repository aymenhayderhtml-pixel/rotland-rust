# R9-01: First-person weapon in view

Read `tasks/R9-common.md` first. Reference: `scripts/Combat/WeaponMounts.cs` (FirstPersonMount), `scripts/Combat/WeaponHold.cs`, `data/weapons.json` (`fpHold` pos/rot per weapon).

In first person (V) the held weapon is invisible. Write `rust/rotland/src/fp_weapon.rs`:
- `pub fn fp_weapon_matrix(camera: &engine::render::Camera, def: &game::WeaponDef, sway: FpSway) -> glam::Mat4`: the weapon model's world matrix placed in front of the camera from `fpHold` (pos in camera space, rot in degrees, Godot order), so the normal 3D pass can draw it. Find where `WeaponDef` keeps fpHold (rust/game/src/weapons.rs); if it doesn't, add the field there (you own weapons.rs this round, serde `fpHold`, default zero).
- `FpSway`: a small bob while walking and a kick back on a swing/shot, ported from Godot if it has one (otherwise a simple documented bob: amplitude and speed as consts).
- Godot draws the view model so it does not clip into walls; here, pull the weapon toward the camera when a short physics ray (`PhysicsWorld::raycast`) from the camera hits within the weapon's reach. Document it.
- Read how main.rs draws the weapon in hand today (search `weapon_model_name`) and describe in your notes the exact change: in first person, draw the same model with `fp_weapon_matrix` instead of the hand bone matrix.
Tests (`#[cfg(test)]` in fp_weapon.rs): camera at origin looking -Z, bat fpHold (0, -0.05, -0.3) lands at that point; yaw 90 deg rotates it; the wall pull moves it closer.

## Worker notes

### Files changed
- `rust/rotland/src/fp_weapon.rs`: New module implementing first-person weapon matrix placement, sway (walking bob + firing/swing kick), obstacle wall-clip prevention via `PhysicsWorld::raycast`, and unit tests.
- `rust/game/src/weapons.rs`: Added `WeaponDef::fp_hold_or_default(&self) -> WeaponHold` helper. (Note: `WeaponDef.fp_hold: Option<WeaponHold>` was already present from earlier rounds).
- `tasks/R9-01-fp-weapon.md`: Documented public API, traced test numbers, and exact wiring steps for Claude.

### Public API (`fp_weapon.rs`)
- `pub const FIRST_PERSON_MOUNT: glam::Vec3 = glam::Vec3::new(0.28, -0.22, -0.45);`: Godot `WeaponMounts.FirstPersonMount`.
- `pub const FIRST_PERSON_REST_POSE_DEG: glam::Vec3 = glam::Vec3::new(-20.0, 0.0, 0.0);`: Godot `MeleeHandler.RestPose`.
- `pub const BOB_AMPLITUDE_X: f32 = 0.015;` / `pub const BOB_AMPLITUDE_Y: f32 = 0.010;` / `pub const BOB_SPEED: f32 = 8.0;`: Walking bob parameters.
- `pub const GUN_KICK_OFFSET: glam::Vec3 = glam::Vec3::new(0.0, 0.01, 0.04);`: Godot `GunEffects.ViewKick`.
- `pub const SWING_KICK_ROT_DEG: glam::Vec3 = glam::Vec3::new(20.0, -15.0, 0.0);`
- `pub const SWING_KICK_OFFSET: glam::Vec3 = glam::Vec3::new(-0.02, 0.02, 0.05);`
- `pub const MIN_CAMERA_DISTANCE: f32 = 0.06;`: Camera near plane clearance.
- `pub struct FpSway`:
  - `pub bob_phase: f32`
  - `pub move_speed: f32`
  - `pub kick_offset: glam::Vec3`
  - `pub kick_rot_deg: glam::Vec3`
  - `pub wall_pull: f32`
  - `pub fn new() -> Self`
  - `pub fn trigger_shot(&mut self)`
  - `pub fn trigger_swing(&mut self)`
  - `pub fn update(&mut self, dt: f32, horizontal_speed: f32)`
  - `pub fn bob_offset(&self) -> glam::Vec3`
- `pub fn weapon_reach(def: &game::WeaponDef) -> f32`: Forward extent of weapon model along camera view.
- `pub fn compute_wall_pull(camera: &engine::render::Camera, def: &game::WeaponDef, physics: &physics::PhysicsWorld) -> f32`: Casts ray from camera along forward axis; returns penetration depth `(reach - distance).max(0.0)`.
- `pub fn fp_weapon_matrix(camera: &engine::render::Camera, def: &game::WeaponDef, sway: FpSway) -> glam::Mat4`: World matrix placed in front of camera using `fpHold` (or `FIRST_PERSON_MOUNT`), rotated in Godot Euler order YXZ.
- `pub fn fp_weapon_matrix_with_physics(camera: &engine::render::Camera, def: &game::WeaponDef, sway: FpSway, physics: &physics::PhysicsWorld) -> glam::Mat4`: Convenience wrapper computing wall pull and weapon matrix.

### Traced test numbers
1. **Camera at origin (0, 0, 0) looking -Z (`yaw = 0.0`, `pitch = 0.0`)**:
   - `bat` hold: `fp_hold = [0.0, -0.05, -0.3]`, `rot = [0, 0, 0]`.
   - `cam_world = Mat4::IDENTITY`.
   - `local_pos = Vec3(0.0, -0.05, -0.3)`.
   - Result: `world_matrix.transform_point3(Vec3::ZERO) = Vec3(0.0, -0.05, -0.3)`.
   - Verified in test: `(pos - (0.0, -0.05, -0.3)).length() < 1e-5`.
2. **Yaw 90 degrees (`PI / 2` radians)**:
   - `cam_world = Mat4::from_rotation_y(PI / 2)`.
   - Point `(0.0, -0.05, -0.3)` rotated clockwise around +Y:
     `x' = -0.3 * sin(PI / 2) = -0.3`
     `y' = -0.05`
     `z' = -0.3 * cos(PI / 2) = 0.0`
   - Result: `pos = Vec3(-0.3, -0.05, 0.0)`.
   - Distance from origin is invariant: `sqrt((-0.3)^2 + (-0.05)^2) = sqrt(0.0925) ≈ 0.304138`.
   - Verified in test: `(pos - (-0.3, -0.05, 0.0)).length() < 1e-5`.
3. **Wall pull**:
   - With `sway.wall_pull = 0.15`:
     `z' = -0.3 + 0.15 = -0.15`.
     `pos = Vec3(0.0, -0.05, -0.15)`.
     Distance to camera: `sqrt((-0.05)^2 + (-0.15)^2) = sqrt(0.025) ≈ 0.158114 < 0.304138`.
   - With `PhysicsWorld` raycast:
     Wall collider at `z = -0.5` (half-thickness 0.1, front surface at `z = -0.4`).
     Bat reach = `0.3 + 0.85 = 1.15m`.
     Ray hits at `0.4m < 1.15m`, penetration = `1.15 - 0.4 = 0.75m`.
     `compute_wall_pull` returns `0.75m`.
     `pos_pulled` distance to camera is `0.0781m < 0.3041m`.

### Wiring for Claude (in `rust/rotland/src/main.rs`)
1. **Module declaration**:
   At the top of `main.rs`, declare `mod fp_weapon;`.
2. **State in `struct R0`**:
   Add field `fp_sway: fp_weapon::FpSway,` to `R0`. Initialize it to `fp_weapon::FpSway::default()` in `R0::new`.
3. **Tick / input hooks**:
   - In `R0::update` (each frame):
     `let speed = self.player_ctrl.as_ref().map_or(0.0, |p| p.horizontal_speed);`
     `self.fp_sway.update(dt, speed);`
   - In firing hook (when a shot is fired):
     `self.fp_sway.trigger_shot();`
   - In melee attack hook (when a swing starts):
     `self.fp_sway.trigger_swing();`
4. **Drawing weapon in first person**:
   In `R0::render` (lines 1716-1725 of `main.rs`):
   Currently:
   ```rust
   if !pose.first_person {
       main.push(engine::render::SkinnedBatch { mesh: body.mesh, instances: body.instances.clone() });
       if let Some(w) = &weapon {
           main.push(engine::render::SkinnedBatch { mesh: w.mesh, instances: w.instances.clone() });
       }
   }
   shadow.push(body);
   shadow.extend(weapon);
   ```
   Replace with:
   ```rust
   if !pose.first_person {
       main.push(engine::render::SkinnedBatch { mesh: body.mesh, instances: body.instances.clone() });
       if let Some(w) = &weapon {
           main.push(engine::render::SkinnedBatch { mesh: w.mesh, instances: w.instances.clone() });
       }
   } else if let Some(w) = &weapon
       && let Some(def) = self.play.as_ref().and_then(|p| p.gun.as_ref()).or_else(|| self.combat.as_ref().and_then(|c| c.current_weapon()))
   {
       let fp_mat = fp_weapon::fp_weapon_matrix_with_physics(
           &self.camera,
           def,
           self.fp_sway,
           &self.physics,
       );
       main.push(engine::render::SkinnedBatch {
           mesh: w.mesh,
           instances: vec![engine::render::SkinInstance::new(fp_mat, 0)],
       });
   }
   shadow.push(body);
   shadow.extend(weapon);
   ```
   Note: Joint index 0 uses `self.bones[0] = Mat4::IDENTITY` (already guaranteed at line 1703 of `main.rs`), so `SkinInstance::new(fp_mat, 0)` renders the static weapon mesh with `fp_mat` as its world transform.
