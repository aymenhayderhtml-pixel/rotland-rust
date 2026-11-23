# R22-N4 (Neon): zombies attack the walls the player builds

Files: game/src/building/siege.rs, game/tests/all/siege_tests.rs, data/siege.json, rotland/src/siege_view.rs, tasks/R22-N4-base-siege.md, game/src/building/mod.rs, game/tests/all/main.rs, rotland/src/main.rs, rotland/src/app/frame_siege.rs, rotland/src/app/mod.rs, rotland/src/app/state.rs, rotland/src/app/frame_sim.rs, rotland/src/app/frame_ui.rs, rotland/src/app/save_load.rs, rotland/src/app/frame_input.rs
Read: game/src/building/state.rs, game/src/building/def.rs, rotland/src/swim_view.rs, rotland/src/menus/theme.rs
API: game/src/building/plan.rs, game/src/building/mod.rs
Data: data/build_parts.json, data/zombies.json

Rotland is a calm-looking, low-poly zombie survival game. The player can build a base, but zombies ignore it, so a base has no danger and no purpose. Write the pure logic and the on-screen look. **A helper does the game-loop wiring afterwards, so do NOT edit or assume any file outside `Files:`** (do not edit `building/mod.rs`, `lib.rs` or `tests/all/main.rs`; the helper adds those lines). No sounds of any kind. No new crates. The target is a weak laptop, so this must stay cheap with many zombies.

## How it must play
1. A zombie that is chasing the player and is blocked by a built part hits that part. Zombies do not chew on walls that are not between them and the player.
2. Each zombie type hits at its own strength. Brutes are the wall breakers; a brute breaks a basic wall quickly. Doors take extra damage, so they are the weak point.
3. Hits happen every `hitInterval` seconds per zombie, not every frame.
4. When a part reaches 0 health it is removed. Anything that loses its support collapses using the existing `BaseState::collapse_unsupported`.
5. **Fair rule: the base is only attacked while the player is within `activeRange` of the attacker.** The base is never damaged while the player is far away or the game is closed.
6. The player gets a warning when the base is being hit, which stays up for `alertSeconds` after the last hit.

## Data: `data/siege.json`
Keys (camelCase): `activeRange` 60, `reach` 1.6, `hitInterval` 1.2, `doorMultiplier` 1.5, `alertSeconds` 5, `maxAttackers` 24, `damage`: `{ "walker": 6, "runner": 4, "crawler": 3, "screamer": 2, "brute": 45 }`, `defaultDamage` 4. Check these against the part health in `build_parts.json`: a basic wall should survive roughly 30 seconds against 3 walkers and roughly 10 against one brute. Adjust the walker and brute numbers if needed and say so in your notes.

## Logic: `game/src/building/siege.rs`
Must be pure (no rendering, no files except `from_repo`). Use these exact names:
```rust
pub struct SiegeConfig { /* fields above, serde camelCase */ }
impl SiegeConfig {
    pub fn from_json(text: &str) -> Result<Self, serde_json::Error>;
    pub fn from_repo() -> Result<Self, std::io::Error>; // paths::repo_path("data/siege.json")
    pub fn damage_for(&self, kind: &str) -> f32;
}
pub struct SiegeAttacker<'a> { pub id: u32, pub kind: &'a str, pub position: Vec3, pub chasing: bool }
pub enum SiegeEvent {
    Hit { attacker: u32, part_id: u32, amount: f32 },
    Broken { part_id: u32, position: Vec3 },
    Collapsed { part_id: u32 },
}
#[derive(Default)]
pub struct SiegeFrame { pub events: Vec<SiegeEvent>, pub geometry_changed: bool }
pub struct SiegeSystem { /* per-attacker timers, alert timer */ }
impl SiegeSystem {
    pub fn new(cfg: SiegeConfig) -> Self;
    pub fn tick(&mut self, base: &mut BaseState, build: &BuildConfig, attackers: &[SiegeAttacker], player: Vec3, dt: f32) -> SiegeFrame;
    pub fn under_attack(&self) -> bool;
    pub fn alert_fraction(&self) -> f32; // 1.0 just hit, fading to 0.0
    pub fn clear(&mut self); // on death or load
}
```
Rules for `tick`:
- Return at once, with no allocation, when `base` is empty; still count the alert timer down.
- Skip attackers not `chasing` or farther than `activeRange` from the player. Handle at most `maxAttackers` per tick, nearest to the player first.
- Only parts where `BaseState::blocks_zombies` is true can be hit. An open door does not block, so it cannot be hit.
- A part is a valid target only if it is within `reach` of the zombie and lies roughly between the zombie and the player (it is in front of the zombie toward the player). Pick the nearest valid part.
- Forget timers of attackers that are no longer in the list, so the timer map cannot grow forever.
- `geometry_changed` is true when any part was removed, so the caller can rebuild colliders and zombie paths once.
- If a part was already destroyed by an earlier attacker this tick, later attackers must not hit it.

## Look: `rotland/src/siege_view.rs` (egui 0.37, colors and style from `menus/theme.rs`, match `swim_view.rs`)
- `pub fn draw_base_alert(ctx: &egui::Context, fraction: f32)`: nothing at 0. A small calm red-edged banner near the top centre, below the compass: "Your base is under attack". It fades with `fraction`. No flashing.
- `pub fn draw_part_health(ctx: &egui::Context, name: &str, fraction: f32)`: small label and thin health bar just below the screen centre, for a damaged part the player looks at. Green to amber to red.
- Must look right from 1280x720 to 2560x1440 (scale by screen height), and must not overlap the compass, hotbar, or the fishing and swimming panels. Use stable ids with `egui::Id::unique`.

## Tests: `game/tests/all/siege_tests.rs`
Cover: walker damages a blocking wall on the hit interval, not every frame; brute hits harder; door multiplier; open door is not hit; part breaks and is removed exactly once with `Broken`; unsupported part reports `Collapsed`; zombie beyond `activeRange` does nothing; non-chasing zombie does nothing; a wall behind the zombie (not toward the player) is not hit; `maxAttackers` is respected; two attackers on a part that breaks do not double-report; timer map is cleaned; alert fades to 0; `clear`; `data/siege.json` loads and every zombie type in `data/zombies.json` has damage. Every new number in Rust code needs a source tag (`// data: siege.json key` or `// new: DECISIONS 2026-11-23`).

## Notes to write
In `=== NOTES ===` list the exact wiring steps, and any number you changed and why.

## Worker notes

## Neon notes

Data balance adjustments (data/siege.json)

Checked the prompt's draft numbers against wall.hp = 150.0 in
data/build_parts.json with hitInterval = 1.2:

  - With the draft walker = 6 (5.0 DPS each), 3 walkers dealt 15.0 DPS and broke
    a 150 HP wall in 10.0s (target: ~30s). Adjusted walker to 2.0 (3 * 2.0 / 1.2
    = 5.0 DPS -> 30.0s).
  - With the draft brute = 45 (37.5 DPS), 1 brute broke a 150 HP wall in 4.0s
    (target: ~10s). Adjusted brute to 18.0 (18.0 / 1.2 = 15.0 DPS -> 10.0s
    average / 10.8s across 9 hits).
  - Scaled the other zombie types proportionally so the intended ordering (brute
    > walker > runner >= defaultDamage > crawler > screamer) is preserved:
    runner = 1.5, crawler = 1.0, screamer = 0.8, defaultDamage = 1.5.
  - Against a closed wall_door (hp = 120.0, doorMultiplier = 1.5), 1 brute
    breaks it in 5 hits (6.0s) and 3 walkers in 14 intervals (16.8s), keeping
    doors the clear weak point.

Exact wiring steps for the helper

1.  game/src/building/mod.rs:
      - Add pub mod siege; (and optionally pub use siege::{SiegeAttacker,
        SiegeConfig, SiegeEvent, SiegeFrame, SiegeSystem}; if mod.rs re-exports
        sibling modules). Ensure def, state, and siege are pub mod so
        game/tests/all/siege_tests.rs can import them.
2.  game/tests/all/main.rs:
      - Add mod siege_tests;.
3.  rotland/src/main.rs (or rotland/src/lib.rs, matching where swim_view is
    declared):
      - Add mod siege_view;.
4.  Game startup / state:
      - Load SiegeConfig::from_repo().unwrap_or_default() and store
        SiegeSystem::new(siege_cfg) on the gameplay state.
      - Call siege.clear() whenever the player dies or a save is loaded/reset.
5.  Per-frame update loop:
      - Build a reusable Vec<SiegeAttacker<'_>> from living zombies (id, kind,
        position, chasing).
      - Call let frame = siege.tick(&mut base, &build_cfg, &attackers,
        player_pos, dt);.
      - If frame.geometry_changed is true, rebuild base colliders, visual
        instances, and zombie navigation/blockers once for that frame.
      - Optionally spawn break/debris visuals from SiegeEvent::Broken {
        position, .. } and SiegeEvent::Collapsed { .. }.
6.  HUD pass (egui):
      - Always call siege_view::draw_base_alert(ctx, siege.alert_fraction());
        (it returns immediately with no work when 0.0).
      - When the player is looking at a placed part (via
        base.raycast_part(cam_pos, cam_dir, interact_dist)), look up part and
        def = build_cfg.get(&part.part); if def.hp > 0.0 && part.hp < def.hp,
        call siege_view::draw_part_health(ctx, &def.name, part.hp / def.hp);.

Risks / notes

  - find_target_part uses point distance to part.position (with vertical span
    clamping [part.y, part.y + def.height]), matching BaseState::parts_in_radius
    and BaseState::raycast_part. Because reach is 1.6 and a grid cell is 2.0, a
    zombie standing up against any point of a 2m edge wall is within 1.28m
    <= 1.6m of its center.
  - SiegeSystem::tracked_attackers(&self) -> usize was added as a small public
    accessor so tests can directly assert timer-map pruning in addition to
    behavioral checks. === END ===

## Gate report
- date: 2026-11-23
- base: main
- files changed (31): assets/models/buildings/settlement.glb, assets/models/_previews/car_hatchback_blue.png, assets/models/_previews/car_hatchback_blue_rear.png, assets/models/_previews/car_hatchback_blue_side.png, assets/models/_previews/car_hatchback_olive.png, assets/models/_previews/car_hatchback_olive_rear.png, assets/models/_previews/car_hatchback_olive_side.png, assets/models/_previews/car_hatchback_red.png, assets/models/_previews/car_hatchback_red_rear.png, assets/models/_previews/car_hatchback_red_side.png, assets/models/vehicles/car_hatchback_blue.glb, assets/models/vehicles/car_hatchback_olive.glb, assets/models/vehicles/car_hatchback_red.glb, data/siege.json, game/src/building/mod.rs, game/src/building/siege.rs, game/tests/all/main.rs, game/tests/all/siege_tests.rs, rotland/src/app/frame_input.rs, rotland/src/app/frame_siege.rs, rotland/src/app/frame_sim.rs, rotland/src/app/frame_ui.rs, rotland/src/app/mod.rs, rotland/src/app/save_load.rs, rotland/src/app/state.rs, rotland/src/main.rs, rotland/src/siege_view.rs, tasks/R22-N4-base-siege.md, tasks/R23-N5-teammates.md, tools/blender/buildings/settlement.py, tools/blender/vehicles/car_parts.py
- tests passed: 800
- clippy: PASS
- test: PASS
- checks: FAIL
- gate: FAIL (checks)
