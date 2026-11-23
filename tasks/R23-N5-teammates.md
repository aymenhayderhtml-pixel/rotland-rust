# R23-N5 (Neon): hired teammates walk with the player and fight

Files: game/src/guild/companion/mod.rs, game/src/guild/companion/config.rs, game/src/guild/companion/types.rs, game/src/guild/companion/squad.rs, game/src/guild/companion/tick.rs, game/src/guild/companion/targeting.rs, game/src/guild/companion/geometry.rs, game/src/guild/companion/tuning.rs, game/src/guild/mod.rs, game/tests/all/main.rs, game/tests/all/companion_tests.rs, data/companions.json, rotland/src/companion_view.rs, rotland/src/companion_model.rs, rotland/src/main.rs, rotland/src/app/mod.rs, rotland/src/app/frame_team.rs, rotland/src/app/state.rs, rotland/src/app/frame_sim.rs, rotland/src/app/frame_ui.rs, rotland/src/app/frame_camp.rs, rotland/src/app/start_world.rs, rotland/src/app/render_frame.rs, rotland/src/app/save_load.rs, rotland/src/app/frame_actors.rs, rotland/src/camp.rs, tasks/R23-N5-teammates.md
Read: game/src/guild/recruits.rs, rotland/src/swim_view.rs, rotland/src/menus/theme.rs
API: game/src/guild/mod.rs, game/src/guild/manager.rs
Data: data/guild.json

Rotland is a calm-looking, low-poly zombie survival game. The player can hire teammates on the guild screen and give them orders (Stay, Follow, Guard), but they only exist on that menu. Write the pure logic that makes them act in the world, and a small on-screen party panel. **A helper does the game-loop wiring, 3D models and animation afterwards, so do NOT edit or assume any file outside `Files:`** (do not edit `guild/mod.rs`, `lib.rs` or `tests/all/main.rs`; the helper adds those lines). No sounds of any kind. No new crates. The target is a weak laptop: at most 4 teammates, and it must stay cheap with many zombies.

## How it must play
1. **Follow:** the teammate walks behind the player at a comfortable spot, each teammate in its own spot so they do not stack. It runs to catch up when far. If it is more than `teleportDistance` away and out of the player's view direction, it is moved near the player so it never gets lost.
2. **Guard:** stays near the camp point, patrols slowly inside `guardRadius`, and fights zombies that come inside that radius. It never chases beyond `guardLeash`.
3. **Stay:** stands at its camp spot. It defends itself only if a zombie is within attack range.
4. **Roles:** Fighter attacks zombies within `fightRange` of the player or itself. Guard is tougher and holds position. Scavenger only fights when cornered, and otherwise backs away toward the player.
5. **No permanent death (owner-friendly rule):** at 0 health a teammate is *downed*, not dead. It stops acting and zombies ignore it. It recovers at the next morning at its camp spot with `recoverHealthFraction` health. Never set `RecruitState::Dead` in this code.
6. A teammate out of combat slowly heals at `healPerSecond`.

## Data: `data/companions.json`
Keys (camelCase): `walkSpeed` 2.4, `runSpeed` 4.6, `runDistance` 7, `followDistance` 2.6, `followSpread` 1.8, `teleportDistance` 45, `fightRange` 11, `attackRange` 1.7, `attackCooldown` 1.3, `guardRadius` 14, `guardLeash` 20, `patrolPauseSeconds` 4, `retargetSeconds` 0.4, `healPerSecond` 1.5, `recoverHealthFraction` 0.5, `scavengerFleeRange` 4, `damage`: `{ "fighter": 22, "guard": 16, "scavenger": 8 }`, `damagePerLevel` 3, `damageTakenMultiplier`: `{ "fighter": 1.0, "guard": 0.7, "scavenger": 1.1 }`.

## Logic: `game/src/guild/companion.rs`
Must be pure. Use these exact names:
```rust
pub struct CompanionConfig { /* fields above, serde camelCase */ }
impl CompanionConfig {
    pub fn from_json(text: &str) -> Result<Self, serde_json::Error>;
    pub fn from_repo() -> Result<Self, std::io::Error>; // paths::repo_path("data/companions.json")
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompanionAnim { Idle, Walk, Run, Attack, Downed }
pub struct CompanionTarget { pub id: u32, pub position: Vec3 }
pub struct CompanionWorld<'a> {
    pub player: Vec3,
    pub player_forward: Vec3,
    pub camp: Vec3,
    pub zombies: &'a [CompanionTarget],
}
pub enum CompanionEvent {
    Hit { companion: String, zombie_id: u32, damage: f32 },
    Downed { companion: String },
    Recovered { companion: String },
}
pub struct CompanionDraw { pub id: String, pub name: String, pub position: Vec3, pub yaw: f32, pub anim: CompanionAnim, pub health_fraction: f32 }
pub struct CompanionSquad { /* one runtime body per hired teammate */ }
impl CompanionSquad {
    pub fn new(cfg: CompanionConfig) -> Self;
    /// Adds bodies for newly hired teammates, removes dismissed ones, applies new orders. Call when the guild changes.
    pub fn sync(&mut self, hired: &[GuildRecruit], camp: Vec3);
    /// `move_body(from, wanted_step)` returns the position after the caller's ground and wall checks.
    pub fn tick(&mut self, world: &CompanionWorld, move_body: &mut dyn FnMut(Vec3, Vec3) -> Vec3, dt: f32, events: &mut Vec<CompanionEvent>);
    pub fn damage(&mut self, id: &str, amount: f32, events: &mut Vec<CompanionEvent>);
    /// Living bodies zombies may target (not downed), written into a reused Vec.
    pub fn targets(&self, out: &mut Vec<(String, Vec3)>);
    pub fn on_morning(&mut self, camp: Vec3, events: &mut Vec<CompanionEvent>);
    pub fn draws(&self) -> impl Iterator<Item = CompanionDraw> + '_;
    /// Health to store back on the recruit, always at least 1 so it never becomes Dead.
    pub fn saved_health(&self, id: &str) -> Option<u32>;
    pub fn set_health(&mut self, id: &str, health: u32);
}
```
Rules:
- Yaw follows the project convention: forward = `(-sin(yaw), 0, -cos(yaw))`.
- Target search runs every `retargetSeconds` per teammate, with teammates staggered so they do not all search on the same frame. No allocation in the per-frame path except pushing events.
- Do not move through the player: keep a small separation from the player and from other teammates.
- The first hit on a target happens only after arriving in `attackRange`, then one hit per `attackCooldown`.
- Handle a target that has vanished from `zombies` safely.
- Tick must be stable for dt from 0.001 to 0.1 and must not produce NaN when positions are identical.

## Look: `rotland/src/companion_view.rs` (egui 0.37, colors and style from `menus/theme.rs`, match `swim_view.rs`)
- `pub fn draw_party_panel(ctx: &egui::Context, members: &[PartyMember])` with `pub struct PartyMember<'a> { pub name: &'a str, pub health_fraction: f32, pub downed: bool, pub order_label: &'a str }`. Small compact list on the left side, above the food and water bars, one thin row per teammate: name, thin health bar, order. Downed shows a muted "Down until morning". Nothing when empty.
- `pub fn draw_name_tag(ctx: &egui::Context, screen_pos: egui::Pos2, name: &str, health_fraction: f32)`: tiny name and thin bar over a teammate's head. Use a stable id that includes `name`.
- Must look right from 1280x720 to 2560x1440 (scale by screen height), and must not overlap the left hint box, the status bars or the hotbar.

## Tests: `game/tests/all/companion_tests.rs`
Cover: follow reaches its spot and stops; two followers get different spots; run when far; teleport when very far; guard stays within leash; stay does not wander; fighter attacks on cooldown; scavenger backs away; downed stops acting and is not in `targets`; `on_morning` recovers with the right health; `saved_health` is never 0; `sync` adds, removes and changes orders; vanished target is safe; identical positions give no NaN; dt range stays finite; `data/companions.json` loads. Every new number in Rust code needs a source tag (`// data: companions.json key` or `// new: DECISIONS 2026-11-23`).

## Notes to write
In `=== NOTES ===` list the exact wiring steps for the game loop, zombies targeting teammates, saving health, and the HUD.

## Worker notes

## Neon notes

Exact wiring steps for the helper

1.  Module declarations

      - game/src/guild/mod.rs: add pub mod companion; (and re-export types if
        guild/mod.rs re-exports sibling modules). Ensure recruits and companion
        are pub mod.
      - game/tests/all/main.rs: add mod companion_tests;.
      - rotland/src/main.rs (or rotland/src/lib.rs, matching swim_view): add mod
        companion_view;.

2.  Startup & guild synchronization

      - Load CompanionConfig::from_repo().unwrap_or_default() and hold a
        CompanionSquad::new(cfg) plus reusable buffers (zombie_targets:
        Vec<CompanionTarget>, companion_events: Vec<CompanionEvent>,
        companion_targets: Vec<(String, Vec3)>) on the gameplay state.
      - Call squad.sync(guild.recruits().hired(), camp_pos) after loading a
        save, after hiring/dismissing a recruit, after changing a recruit's
        order, and after morning wage departures (pay_recruit_wages).

3.  Game loop tick

      - Fill zombie_targets from living zombies (id, position), build
        CompanionWorld { player, player_forward, camp, zombies: &zombie_targets
        }, clear companion_events, and call: squad.tick(&world, &mut |from,
        step| resolve_ground_and_walls(from, step), dt, &mut companion_events);
      - For CompanionEvent::Hit { zombie_id, damage, .. }, apply damage to that
        zombie (and if the zombie dies, notify guild.on_kill(...) if desired).
      - For CompanionEvent::Downed / CompanionEvent::Recovered, optionally
        surface a brief HUD toast.
      - Render each CompanionDraw from squad.draws() at (draw.position,
        draw.yaw) with the animation matching draw.anim.

4.  Zombies targeting teammates

      - Each frame (or on the zombie retarget cadence), call squad.targets(&mut
        companion_targets) to get living (non-downed) (id, position) pairs.
      - Let zombies choose between the player and nearby entries in
        companion_targets; when a zombie lands a melee hit on companion id, call
        squad.damage(&id, zombie_damage, &mut companion_events).

5.  Morning recovery & saving health

      - At morning rollover (alongside guild.refresh_morning(...)), call
        squad.on_morning(camp_pos, &mut companion_events) so any downed
        teammates recover at camp with recoverHealthFraction health.
      - Before serializing (guild.to_state()), write each hired recruit's health
        back using if let Some(hp) = squad.saved_health(&r.id) { r.health = hp;
        } — saved_health always returns at least 1, so hired recruits are never
        marked RecruitState::Dead. On load, call
        squad.sync(guild.recruits().hired(), camp_pos) and
        squad.set_health(&r.id, r.health) if needed.

6.  HUD rendering (rotland/src/companion_view.rs)

      - Build a small stack slice/vec of PartyMember from squad.draws() paired
        with each recruit's order label ("Follow", "Guard", "Stay") and downed =
        draw.anim == CompanionAnim::Downed, then call
        companion_view::draw_party_panel(ctx, &party_members);.
      - For each living teammate in front of the camera within tag distance,
        project draw.position + Vec3::Y * head_height to screen coordinates and
        call companion_view::draw_name_tag(ctx, screen_pos, &draw.name,
        draw.health_fraction);.

Risks / notes

  - CompanionSquad::tick does zero heap allocation on the steady-state frame
    path (using positions_scratch), allocating only when pushing
    CompanionEvents.
  - data/companions.json uses the exact values from the task specification
    (RecruitRole deserializes "fighter", "guard", "scavenger" directly via its
    existing #[serde(rename_all = "lowercase")]).
  - rotland/src/companion_view.rs uses
    egui::Id::unique(("rotland_companion_tag", name)) (egui 0.37 API, verified
    against swim_view.rs and egui::Id::unique docs). If two recruits ever shared
    the exact same display name, passing draw.id or combining id into the tag
    would differentiate them; current data/guild.json names are all unique. ===
    END ===

## Gate report
- date: 2026-11-23
- base: main
- files changed (43): assets/models/buildings/settlement.glb, assets/models/_previews/car_hatchback_blue.png, assets/models/_previews/car_hatchback_blue_rear.png, assets/models/_previews/car_hatchback_blue_side.png, assets/models/_previews/car_hatchback_olive.png, assets/models/_previews/car_hatchback_olive_rear.png, assets/models/_previews/car_hatchback_olive_side.png, assets/models/_previews/car_hatchback_red.png, assets/models/_previews/car_hatchback_red_rear.png, assets/models/_previews/car_hatchback_red_side.png, assets/models/vehicles/car_hatchback_blue.glb, assets/models/vehicles/car_hatchback_olive.glb, assets/models/vehicles/car_hatchback_red.glb, data/companions.json, game/src/guild/companion/config.rs, game/src/guild/companion/geometry.rs, game/src/guild/companion/mod.rs, game/src/guild/companion/squad.rs, game/src/guild/companion/targeting.rs, game/src/guild/companion/tick.rs, game/src/guild/companion/tuning.rs, game/src/guild/companion/types.rs, game/src/guild/mod.rs, game/tests/all/companion_tests.rs, game/tests/all/main.rs, rotland/src/app/frame_actors.rs, rotland/src/app/frame_camp.rs, rotland/src/app/frame_sim.rs, rotland/src/app/frame_team.rs, rotland/src/app/frame_ui.rs, rotland/src/app/mod.rs, rotland/src/app/render_frame.rs, rotland/src/app/save_load.rs, rotland/src/app/start_world.rs, rotland/src/app/state.rs, rotland/src/camp.rs, rotland/src/companion_model.rs, rotland/src/companion_view.rs, rotland/src/main.rs, tasks/R23-N5-teammates.md, tasks/R24-N6-guild-tasks.md, tools/blender/buildings/settlement.py, tools/blender/vehicles/car_parts.py
- tests passed: 813
- clippy: PASS
- test: PASS
- checks: FAIL
- gate: FAIL (checks)
