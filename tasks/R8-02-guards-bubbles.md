# R8-02: Camp guards shoot zombies + chat bubbles

Read `tasks/R8-common.md` first.

Part A, logic: port `scripts/Settlement/SettlementGuard.cs` (the shooting part: senses, target choice, aim, fire rate, damage, range, ammo if any) and `GuardRifle.cs` to `rust/game/src/camp_guards/`. Data from the same data file the C# reads (look at `data/settlement*.json` / `data/camp_life*.json`).
- API: `CampGuards::update(dt, guards: &[GuardPost {id, position, yaw}], targets: &[GuardTarget {id, position, alive}]) -> GuardFrame { shots: Vec<GuardShot {guard_id, target_id, from, to, damage: DamageInfo}>, yaws: Vec<(guard id, yaw)>, sounds }`. Only zombies near the camp count; guards never shoot the player. Open `rust/game/src/damage.rs` for `DamageInfo`.
- Line of sight: take a `&dyn Fn(Vec3, Vec3) -> bool` "clear" check so Claude can pass a physics raycast.

Part B, drawing: `rust/rotland/src/bubbles.rs`, port of `CampChatBubble.cs`: draws a speech bubble with text above a world point using egui's painter over the 3D view. API `draw_bubbles(ctx: &egui::Context, view_proj: glam::Mat4, screen: (f32, f32), bubbles: &[(Vec3, &str, f32 alpha)])`. Skip bubbles behind the camera or off screen; fade by alpha; cap the width and wrap text. The chat lines themselves already come from `rust/game/src/camp/chat.rs` (read it; say in your report which call gives the line and position for each speaking resident).
Tests (`rust/game/tests/camp_guards_tests.rs`): a zombie in range is shot at the fire rate traced from the data; out of range or blocked is not; dead targets ignored; nearest target picked. Add a small `#[cfg(test)]` test in bubbles.rs for the world-to-screen projection (point in front lands in the screen, behind is skipped).

## Worker notes
- Files added:
  - `rust/game/src/camp_guards/mod.rs`: module entry point re-exporting config, guards, and types
  - `rust/game/src/camp_guards/types.rs`: `GuardPost`, `GuardTarget`, `GuardShot`, `GuardFrame`, and `GuardRifle` constants
  - `rust/game/src/camp_guards/config.rs`: `CampGuardConfig` (deserializes `data/settlements.json` with Godot keys: `guardRange`, `guardSightInterval`, `guardFireInterval`, `guardDamage`, `guardMissChance`, `guardEyeHeight`)
  - `rust/game/src/camp_guards/guards.rs`: `CampGuards` manager, state machine per guard, LOS targeting, yaw calculation, firing, miss chance, sound triggers
  - `rust/rotland/src/bubbles.rs`: `draw_bubbles` painter over 3D view and `world_to_screen` projection with unit test in `#[cfg(test)]`
  - `rust/game/tests/camp_guards_tests.rs`: integration tests for fire rate, range, blocked line of sight, dead target filtering, nearest target selection, and aim yaw calculation
- Files modified:
  - `rust/game/src/lib.rs`: added `pub mod camp_guards;` and `pub use camp_guards::*;`
  - `tasks/R8-02-guards-bubbles.md`: filled in Worker notes

- Public API:
  - `CampGuards`:
    - `CampGuards::new(config: CampGuardConfig) -> Self`
    - `CampGuards::from_repo() -> Result<Self, String>`
    - `CampGuards::with_camp_center(self, center: Vec3) -> Self`
    - `CampGuards::with_seed(self, seed: i32) -> Self`
    - `CampGuards::with_miss_chance(self, miss_chance: f32) -> Self`
    - `CampGuards::set_camp_center(&mut self, center: Vec3)`
    - `CampGuards::set_miss_chance(&mut self, miss_chance: f32)`
    - `CampGuards::shots_fired(&self, guard_id: &str) -> u32`
    - `CampGuards::total_shots_fired(&self) -> u32`
    - `CampGuards::current_target(&self, guard_id: &str) -> Option<u64>`
    - `CampGuards::update(&mut self, dt: f32, guards: &[GuardPost], targets: &[GuardTarget], los: &dyn Fn(Vec3, Vec3) -> bool) -> GuardFrame`
    - `CampGuards::update_clear(&mut self, dt: f32, guards: &[GuardPost], targets: &[GuardTarget]) -> GuardFrame`
  - `GuardPost { pub id: String, pub position: Vec3, pub yaw: f32 }`
  - `GuardTarget { pub id: u64, pub position: Vec3, pub alive: bool }`
  - `GuardShot { pub guard_id: String, pub target_id: u64, pub from: Vec3, pub to: Vec3, pub damage: DamageInfo }`
  - `GuardFrame { pub shots: Vec<GuardShot>, pub yaws: Vec<(String, f32)>, pub sounds: Vec<(String, Vec3)> }`
  - `CampGuardConfig { pub guard_range: f32, pub guard_sight_interval: f32, pub guard_fire_interval: f32, pub guard_damage: f32, pub guard_miss_chance: f32, pub guard_eye_height: f32 }`
  - `GuardRifle`: constants `WEAPON_ID`, `READY_POSE`, `FIRE_ACTION`, `SHOT_SOUND`, `DAMAGE_TYPE`, `CHEST_HEIGHT`
  - `bubbles`:
    - `draw_bubbles(ctx: &egui::Context, view_proj: glam::Mat4, screen: (f32, f32), bubbles: &[(Vec3, &str, f32)])`
    - `world_to_screen(world_pos: Vec3, view_proj: Mat4, screen: (f32, f32)) -> Option<Pos2>`

- Traced test numbers from data/settlements.json:
  - `guardFireInterval = 1.2` s: verified first shot on frame 1 (t = 0.1s) and second shot after fire interval (t = 1.3s)
  - `guardSightInterval = 0.5` s: target scan interval
  - `guardDamage = 60.0`: bullet damage per hit
  - `guardRange = 40.0` m: max flat distance from camp centre (45m out of range ignored, 10m shot)
  - `guardEyeHeight = 1.6` m: eye height for head calculation = post position + (0, 1.6, 0)
  - `chestHeight = 1.2` m: target chest position = zombie position + (0, 1.2, 0)
  - `guardMissChance = 0.15`
  - Aim yaw calculation: target at +X (East) gives `-FRAC_PI_2` (-PI/2), target at -X (West) gives `FRAC_PI_2` (+PI/2), target at -Z (North) gives `0.0`, target at +Z (South) gives `PI`

- Speaking resident chat line call:
  - In `rust/game/src/camp/resident.rs`: `resident.bubble.is_saying()`, `resident.bubble.text()`, and `resident.bubble.alpha()` on each resident from `director.residents()`.
  - Position is `resident.position + Vec3::new(0.0, 1.8, 0.0)`.
  - Alternatively, `director.resident_outputs(player_pos)` gives `ResidentOutput` with `chat_line: Option<String>` and `position: Vec3`.

- Exact wiring steps for the orchestrator (Claude):
  1. Add `mod bubbles;` in `rust/rotland/src/main.rs`.
  2. Instantiate `CampGuards::from_repo()?.with_camp_center(camp_site.center)` in `Camp` or `PlayState`.
  3. In each physics/simulation frame:
     - Collect guards: `let guards: Vec<GuardPost> = npcs.iter().filter(|n| n.role == "guard").map(|n| GuardPost::new(&n.id, n.position, n.yaw)).collect();`
     - Collect targets: `let targets: Vec<GuardTarget> = zombies.list.iter().map(|z| GuardTarget::new(z.id, z.position, !z.health.is_dead())).collect();`
     - Update guards with physics raycast:
       `let frame = camp_guards.update(dt, &guards, &targets, &|from, to| physics.raycast(from, (to - from).normalize(), (to - from).length()).is_none());`
     - Apply hits: For each `shot in frame.shots`, find zombie by `shot.target_id` and call `zombie.health.take_damage(&shot.damage)`.
     - Play sounds: For each `(snd, pos) in frame.sounds`, call `audio.play_at(&snd, pos)`.
     - Update yaws: For each `(id, yaw) in frame.yaws`, update the guard's aiming direction in its animator/figure.
  4. In UI frame (inside egui render pass):
     - Collect active bubbles:
       ```rust
       let bubbles: Vec<(Vec3, &str, f32)> = director.residents().iter()
           .filter(|r| r.visible && r.bubble.is_saying())
           .map(|r| (r.position + Vec3::new(0.0, 1.8, 0.0), r.bubble.text.as_str(), r.bubble.alpha()))
           .collect();
       ```
     - Call `bubbles::draw_bubbles(ctx, camera.view_proj(aspect), (screen_w, screen_h), &bubbles);`.

- Known issues: None. All public names checked against crate, no name clashes, files under 300 lines, clippy-clean style with explicit types and standard constants.
