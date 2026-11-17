# R9-07: Car headlights light the road

Read `tasks/R9-common.md` first. Reference: `scripts/Vehicles/CarLights.cs`, `data/vehicles.json` (light range/angle/colour), the car markers `headlight_L` / `headlight_R`.

The renderer has exactly one spot light (the flashlight): `rust/engine/src/render/lit.rs` (`SpotLight`, `spot: Option<SpotLight>`, uniform fields `spot_pos/spot_dir/spot_color`) and `lit.wgsl`. You own `rust/engine/src/render/lit.rs`, `lit.wgsl`, `skinned.rs` / `pipelines.rs` only if the uniform layout is shared there, and a new `rust/rotland/src/car_lights.rs`. Do NOT touch engine/src/app.rs or gpu.rs (another lane).
- Engine: support up to 3 spot lights (`spots: [Option<SpotLight>; 3]` or a small Vec capped at 3) with the same falloff as today's one. Keep the uniform 16-byte aligned; update every place that fills it (search the whole rust/ tree for `spot:` and `SpotLight`). The flashlight must look exactly as before. Keep it cheap: a fixed loop of 3 in the shader.
- `rotland/src/car_lights.rs`: `pub fn headlight_spots(world: Mat4, left_marker: Vec3, right_marker: Vec3, lights_on: bool, cfg: &game::VehicleConfigData) -> [Option<SpotLight>; 2]` pointing forward and a little down, numbers from CarLights.cs / vehicles.json.
- Notes: the main.rs change (where the flashlight spot is set, add the driven or nearest lit car's two headlights).
Tests: `#[cfg(test)]` in car_lights.rs (positions follow the car matrix; off gives None), and a size/alignment test of the light uniform in lit.rs.

## Worker notes

### Files changed
- `rust/engine/src/render/lit.rs`: Updated `Lighting` struct field to `pub spots: [Option<SpotLight>; 3]`, added `SpotLightUniform` (48 bytes, 16-byte aligned), updated `SceneUniforms` (528 bytes, 16-byte aligned, `spots` array at offset 384), updated `write_scene` to pack up to 3 spots into the uniform buffer, and added `#[cfg(test)]` tests checking size, alignment, and struct offsets. Added `PartialEq` derive to `SpotLight`.
- `rust/engine/src/render/lit.wgsl`: Added `struct SpotLightUniform`, updated `struct Scene` with `spots: array<SpotLightUniform, 3>`, updated `spot_lights` to iterate over the fixed 3 spot lights, and updated `mesh_fs` to call `spot_lights(i.world, n)`.
- `rust/rotland/src/car_lights.rs`: New module implementing `headlight_spots` and `default_headlight_spots` with contract marker coordinates and forward-down beam direction (~5.7° pitch down), with full `#[cfg(test)]` test coverage.

### Public API
- `engine::render::SpotLightUniform`:
  - `pub struct SpotLightUniform { pub pos: [f32; 4], pub dir: [f32; 4], pub color: [f32; 4] }`
  - `impl SpotLightUniform { pub const ZERO: Self; }`
- `engine::render::Lighting`:
  - `pub spots: [Option<SpotLight>; 3]` (replaces `pub spot: Option<SpotLight>`)
- `engine::render::SpotLight`:
  - Added `PartialEq`
- `rotland::car_lights`:
  - `pub const HEADLIGHT_RANGE: f32 = 25.0;`
  - `pub const HEADLIGHT_CONE_DEG: f32 = 30.0;`
  - `pub const HEADLIGHT_ENERGY: f32 = 2.5;`
  - `pub const HEADLIGHT_COLOR: glam::Vec3 = glam::Vec3::new(1.0, 0.98, 0.90);`
  - `pub const DEFAULT_HEADLIGHT_L: glam::Vec3 = glam::Vec3::new(-0.55, 0.66, -1.93);`
  - `pub const DEFAULT_HEADLIGHT_R: glam::Vec3 = glam::Vec3::new(0.55, 0.66, -1.93);`
  - `pub fn headlight_spots(world: glam::Mat4, left_marker: glam::Vec3, right_marker: glam::Vec3, lights_on: bool, cfg: &game::VehicleConfigData) -> [Option<SpotLight>; 2]`
  - `pub fn default_headlight_spots(world: glam::Mat4, lights_on: bool, cfg: &game::VehicleConfigData) -> [Option<SpotLight>; 2]`

### Traced test numbers and sources
- `SceneUniforms` size: `528` bytes (3 x mat4 = 192 bytes, 12 x vec4 = 192 bytes, 3 x SpotLightUniform = 144 bytes; 192 + 192 + 144 = 528). 16-byte alignment: `528 % 16 == 0`.
- `SpotLightUniform` size: `48` bytes (3 x vec4). 16-byte alignment: `48 % 16 == 0`.
- `SceneUniforms.spots` offset: `384` bytes (`offset_of!(SceneUniforms, spots) == 384`).
- Headlight parameters from `scripts/Vehicles/CarLights.cs`:
  - Range = `25.0` m (`Range = 25f`)
  - ConeDegrees = `30.0` degrees (`ConeDegrees = 30f`)
  - Energy = `2.5` (`Energy = 2.5f`)
- Headlight model markers from `docs/VEHICLE_CONTRACT.md`:
  - `headlight_L` = `(-0.55, 0.66, -1.93)`
  - `headlight_R` = `(0.55, 0.66, -1.93)`
- Direction vector: `(0.0, -0.1, -1.0).normalize()`
  - Length = `sqrt(0^2 + (-0.1)^2 + (-1.0)^2) = sqrt(1.01) ~= 1.0049876`
  - Normalized Y = `-0.1 / sqrt(1.01) ~= -0.09950372`
  - Normalized Z = `-1.0 / sqrt(1.01) ~= -0.9950372`
- Translation test: offset `(10.0, 5.0, -20.0)`
  - Left pos: `(-0.55 + 10.0, 0.66 + 5.0, -1.93 - 20.0) = (9.45, 5.66, -21.93)`
  - Right pos: `(0.55 + 10.0, 0.66 + 5.0, -1.93 - 20.0) = (10.55, 5.66, -21.93)`
- Yaw test: 180° rotation (PI around Y axis):
  - Left pos: `(0.55, 0.66, 1.93)`
  - Right pos: `(-0.55, 0.66, 1.93)`
  - Direction: `(0.0, -0.1, 1.0).normalize()`

### Exact wiring steps for the orchestrator (Claude)
1. In `rust/rotland/src/light.rs`:
   Update `at_hour` to take `spots: [Option<SpotLight>; 3]`:
   ```rust
   pub fn at_hour(hour: f32, spots: [Option<SpotLight>; 3]) -> Lighting {
       let look = DayNightLook::at(hour);
       Lighting {
           sun_toward: Vec3::from(look.sun_direction),
           sun_energy: look.sun_energy,
           sun_color: Vec3::from(look.sun_color),
           sky_top: Vec3::from(look.sky_top),
           sky_horizon: Vec3::from(look.sky_horizon),
           ground_horizon: Vec3::from(look.ground_horizon),
           ground_bottom: Vec3::from(look.ground_bottom),
           ambient_energy: look.ambient_energy,
           fog_begin: 120.0,
           fog_end: 190.0,
           fog_density: 1.0,
           fog_color: Vec3::from(look.fog_color),
           moon_toward: Vec3::from(look.moon_direction),
           moon_energy: look.moon_energy,
           moon_color: Vec3::from(look.moon_color),
           spots,
       }
   }
   ```
2. In `rust/rotland/src/main.rs`:
   - Declare the module:
     ```rust
     mod car_lights;
     ```
   - In `render` (around line 1763 where `light::at_hour` is called):
     Compute spots for the flashlight and the driven car (if headlights on) or nearest lit car:
     ```rust
     let flashlight_spot = self.flashlight_on.then(|| {
         light::flashlight(self.camera.position, self.camera.forward())
     });
     let mut car_spots = [None, None];
     if let Some(cars) = self.cars.as_ref() {
         let target_car = cars.driven_car().filter(|c| c.lights_on).or_else(|| {
             cars.live_cars()
                 .filter(|c| c.lights_on)
                 .min_by(|a, b| {
                     let da = a.draw_state().world_matrix.w_axis.truncate().distance_squared(self.camera.position);
                     let db = b.draw_state().world_matrix.w_axis.truncate().distance_squared(self.camera.position);
                     da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
                 })
         });
         if let Some(car) = target_car {
             let d = car.draw_state();
             car_spots = car_lights::default_headlight_spots(d.world_matrix, true, &cars.config);
         }
     }
     let lighting = light::at_hour(self.hour, [flashlight_spot, car_spots[0], car_spots[1]]);
     ```
     And pass `&lighting` into `renderer.draw(...)`.

