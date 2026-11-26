# R29-01: knocked down before death (downed, crawl, revive)

Owns: game/src/downed.rs, data/downed.json, game/tests/all/downed.rs, rotland/src/combat.rs, rotland/src/downed_play.rs, rotland/src/app/frame_actors.rs, rotland/src/app/frame_team.rs, rotland/src/hud/* (only a new small file for the downed bar + the line that draws it)
Rules: `tasks/R29-common.md`. Owner decision: `docs/DECISIONS.md` row 2026-11-25.

**Goal:** when the player's health reaches 0, they are knocked down instead of dying at once.
- **Downed:** the player falls and plays the existing `downed_crawl` clip (made in R27, polished in R28). They can only crawl slowly. No weapons, no shooting, no blocking, no sprint, no jump, no car.
- **Bleed-out:** a timer (30 s) counts down. Each zombie bite while downed takes 4 s off it. At 0 the normal death runs (use the existing death path in `rotland/src/combat.rs` `take_bites`: `dead_for`, death animation, respawn).
- **Revive by a teammate:** a recruit with order Follow within 12 m walks to the player and kneels for 4 s (use the existing `use`/bandage-type clip on the teammate if one exists; otherwise just stand still). Then the player stands up with 25 health.
- **Self revive:** hold E for 6 s with at least 1 bandage in the bag. It uses up 1 bandage, and the player stands up with 25 health.
- **Downed again within 60 s of getting up = real death.** Drowning and falling into deep water still kill at once (no downed).
- **HUD:** while downed, show a centered bar with "DOWNED 23 s" and the hint "[Hold E] Bandage" (only if you have one) or "Waiting for a teammate…".
- Zombies keep attacking a downed player (that's what shortens the timer).

**Parts:**
- A (pure logic, start first): `game/src/downed.rs`: a `Downed` state machine (`Up` / `Down { bleed, revive_progress, by }` / `Dead`) with `on_lethal_hit(now)`, `bite()`, `tick(dt, reviving_teammate_in_range, self_revive_held, has_bandage) -> DownedEvent` (None / Revived / BledOut / BandageUsed), and the 60 s rule. Numbers in `data/downed.json` (bleedOutSeconds 30, biteSeconds 4, teammateRange 12, reviveSeconds 4, selfReviveSeconds 6, reviveHealth 25, crawlSpeedMul 0.35, reDownWindowSeconds 60). Tests in `game/tests/all/downed.rs` cover every event and the 60 s rule.
- B (wiring, after A): `rotland/src/downed_play.rs` glue. In `combat.rs` route the lethal hit through `Downed` (not for drowning). In `frame_actors.rs` block the inputs and slow the movement while downed, and play `downed_crawl` (full body). In `frame_team.rs` make one Follow teammate walk over and revive. Add the HUD bar.
- Save: a downed player who quits loads as up with 25 health. Don't add save fields.

## Worker notes

### Part A (pure logic)
- **Files created:**
  - `data/downed.json` (`bleedOutSeconds`: 30.0, `biteSeconds`: 4.0, `teammateRange`: 12.0, `reviveSeconds`: 4.0, `selfReviveSeconds`: 6.0, `reviveHealth`: 25.0, `crawlSpeedMul`: 0.35, `reDownWindowSeconds`: 60.0)
  - `game/src/downed.rs` (`BANDAGE_ITEM`, `DownedConfig`, `ReviveBy`, `DownedPhase`, `DownedEvent`, `Downed`)
  - `game/tests/all/downed.rs` (unit tests for config load, bleed-out, zombie bites, teammate revive, self-revive with/without bandage & interrupt, 60 s re-down window & `reset()`)
- **Lines for lead to add to shared files:**
  - `game/src/lib.rs`:
    ```rust
    pub mod downed;
    pub use downed::{BANDAGE_ITEM, Downed, DownedConfig, DownedEvent, DownedPhase, ReviveBy};
    ```
  - `game/tests/all/main.rs`:
    ```rust
    mod downed;
    ```
- **Notes for Part B:**
  - `Downed::on_lethal_hit(now: f64)` expects world/session time in seconds (`SessionState::play_time`). When `tick` completes a revive (`Revived` or `BandageUsed`), `last_revived_at` is recorded as `downed_at + down_elapsed`.
  - On `DownedEvent::BandageUsed`, Part B must consume 1 `"bandage"` (`BANDAGE_ITEM`) from inventory and restore health to `config().revive_health` (25.0). On `DownedEvent::Revived`, restore health to `config().revive_health` without consuming a bandage. `DownedEvent::is_revived()` covers both.
  - On `DownedEvent::BledOut` (or `on_lethal_hit` returning `DownedPhase::Dead`), trigger the normal death path; call `downed.reset()` on camp respawn so the re-down window clears.
  - Not built or run by this subagent (per R29-common rules).

### Part B (wiring)
- **Files created / modified:**
  - Created `rotland/src/downed_play.rs` (`DownedPlay`, `on_knocked_down`, `sync_downed_visual`, `tick`, `take_drowning`, `restore_health`, `hud_info`, plus unit tests).
  - Modified `rotland/src/combat.rs` (`CombatEvents::downed_now`, `Combat::cancel_actions`, `Combat::take_damage_with_downed`, plus unit tests).
  - Modified `rotland/src/app/frame_actors.rs` (tracks `self_revive_held` + `session.play_time`; exits car, blocks ladders/vault/sprint/jump/crouch/aim/attack/block/throwable/bow, slows movement by `crawl_speed_mul`, keeps `downed_crawl` playing while downed, resets downed on respawn or `--die-at`).
  - Modified `rotland/src/app/frame_team.rs` (added `player_downed` and `teammate_range` to `TeamFrame`, `reviving_player` to `TeamScratch`, and `step_player_revive` so one alive `Follow` teammate within 12 m walks to the downed player and stands beside them to revive).
  - Created `rotland/src/hud/downed.rs` (`DownedHudInfo`, `draw_downed_bar` showing `"DOWNED N s"`, bleed bar, revive progress bar, and `"[Hold E] Bandage"` or `"Waiting for a teammate…"`, plus unit test).
  - Modified `rotland/src/hud/mod.rs` (added `pub mod downed`, `DownedHudInfo` re-export, `HudState::downed`, hides crosshair/center prompt while downed, draws `draw_downed_bar`).
- **Lines for lead to add to shared / non-owned files:**
  - `rotland/src/main.rs`:
    1. Near module declarations (~line 22):
       ```rust
       mod downed_play;
       ```
    2. In `R0::update`, right after `app::frame_sim::update_sim(...)` (~line 180):
       ```rust
       crate::downed_play::tick(
           dt,
           &mut self.actors,
           &mut self.play,
           &self.session,
           &mut self.audio.audio,
       );
       ```
  - `rotland/src/app/state.rs` (in `ActorsState`, ~line 164):
    ```rust
    pub downed: crate::downed_play::DownedPlay,
    ```
  - `rotland/src/app/start_world.rs`:
    1. When resetting actors (~line 43, right after `actors.combat = None;`):
       ```rust
       actors.downed.reset();
       ```
    2. When restoring saved health (~line 193-195, replacing `c.health.set_current(r.health);`):
       ```rust
       if let Some(c) = actors.combat.as_mut() {
           crate::downed_play::restore_health(c, &mut actors.downed, r.health);
       }
       ```
  - `rotland/src/app/frame_input.rs`:
    1. In `apply_hand` (`HandChange::Health(amount)`, line 85, replacing `c.take_bites(&[-amount]);`):
       ```rust
       c.take_damage_with_downed(&[-amount], &mut actors.downed.machine, actors.downed.last_play_time(), false);
       ```
    2. In `process_input` (hotbar keys, line 350, adding `!actors.downed.is_down()`):
       ```rust
       if !dev_typing
           && !play.build_mode.active
           && !actors.downed.is_down()
           && let Some(change) = play.play.as_mut().and_then(|p| p.hotbar_keys(input, &keys.slots))
       ```
    3. In `process_input` (`attack`, line 355, adding `!actors.downed.is_down()`):
       ```rust
       let attack = !dead
           && !actors.downed.is_down()
           && !play.build_mode.active
           && !ui.screen_open()
           && (session.autoattack || (ui.cursor_captured && input.mouse_pressed(MouseButton::Left)));
       ```
  - `rotland/src/app/frame_world.rs`:
    1. In `update_needs` (line 24):
       ```rust
       let dead = c.is_dead() || actors.downed.is_down();
       ```
    2. In `update_needs` (line 35, replacing `if damage > 0.0 && c.take_bites(&[damage]).died {`):
       ```rust
       if damage > 0.0 {
           let ev = c.take_damage_with_downed(&[damage], &mut actors.downed.machine, actors.downed.last_play_time(), false);
           if ev.downed_now {
               log::info!("the player was downed by hunger or thirst");
           }
           if ev.died {
               if let Some(pv) = actors.player_visual.as_mut() {
                   pv.animator.play_die();
               }
               log::info!("the player died of hunger or thirst");
           }
       }
       ```
    3. In `update_world_interactions` (line 74):
       ```rust
       let alive_now = !actors.combat.as_ref().is_some_and(|c| c.is_dead()) && !actors.downed.is_down();
       ```
  - `rotland/src/app/frame_water.rs` (line 55):
    ```rust
    let dead = actors.combat.as_ref().is_some_and(|c| c.is_dead()) || actors.downed.is_down();
    ```
  - `rotland/src/app/frame_sim.rs`:
    1. In `TeamFrame` construction (~line 544):
       ```rust
       super::frame_team::TeamFrame {
           dt,
           player: actors.player,
           player_yaw: actors.player_ctrl.as_ref().map_or(0.0, |p| p.yaw),
           camp: super::frame_team::camp_point(world.world_gen.as_deref()),
           player_downed: actors.downed.is_down(),
           teammate_range: actors.downed.teammate_range(),
       },
       ```
    2. In bite handling (~line 561-585, replacing `let ev = c.take_bites(&bites);` and updating `else if ev.bitten`):
       ```rust
       if let Some(c) = actors.combat.as_mut() {
           let was_down = actors.downed.is_down();
           let ev = c.take_damage_with_downed(&bites, &mut actors.downed.machine, session.play_time, true);
           if ev.downed_now {
               crate::downed_play::on_knocked_down(
                   render,
                   play,
                   &mut actors.downed,
                   c,
                   actors.player_visual.as_mut(),
                   session.anim_shot.is_pose_frozen(),
                   &mut audio.audio,
               );
           }
           if ev.died {
               // ... existing death block unchanged ...
           } else if ev.bitten && !was_down && !ev.downed_now {
               // ... existing hit_react + playerHurt block unchanged ...
           } else if ev.bitten {
               if let Some(a) = audio.audio.as_mut() {
                   a.event("playerHurt", None);
               }
           }
       }
       ```
    3. In drowning handling (~line 645-648):
       ```rust
       let in_deep_water = water_res.zone == world::LakeZone::Deep && !on_dock;
       if (water_res.drowning_damage > 0.0 || (actors.downed.is_down() && in_deep_water)) && alive
           && let Some(c) = actors.combat.as_mut()
       {
           let ev = crate::downed_play::take_drowning(
               c,
               &mut actors.downed,
               water_res.drowning_damage,
               in_deep_water,
           );
       ```
  - `rotland/src/app/frame_ui.rs` (right before `hud::draw_hud(ctx, &state, &mut ui.icons);`, ~line 222):
    ```rust
    state.downed = crate::downed_play::hud_info(&actors.downed, play_state);
    ```
- **How to see it in the game:**
  - Start play mode (`--give=bandage:2` optional) and let zombies reduce your health to 0: you drop into `downed_crawl`, your weapon holsters, movement slows to 0.35x, and the centered `"DOWNED 30 s"` bar appears with `"[Hold E] Bandage"` (or `"Waiting for a teammate…"`).
  - Hold `E` for 6 s to consume 1 bandage and stand up at 25 HP, or have a `Follow` guild teammate within 12 m walk up and revive you in 4 s.
  - Let zombies bite while downed to shave 4 s per bite until bleed-out (`YOU DIED`), or take lethal damage again within 60 s of reviving for instant death.
- **Notes / caveats:**
  - Teammate NPC models (`npc_guard`, `npc_scout` in `tools/blender/characters/build_characters.py` / `rotland/src/companion_model.rs`) do not export a `use`/`bandage` clip (only `PLAYER_ONLY_CLIPS` has `bandage`), and `companion_model.rs` only maps `Idle`/`Walk`/`Run`/`Attack`/`Downed`, so the reviving teammate stands still (`CompanionAnim::Idle`) beside the downed player during the 4 s revive.
  - Not built or run by this subagent (per `tasks/R29-common.md` rules).
