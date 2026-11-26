# R29-02: rescue survivors in the world (they join your team)

Owns: game/src/rescue.rs, game/src/rescue_place.rs, data/rescue.json, game/tests/all/rescue.rs, rotland/src/rescue_play.rs, rotland/src/rescue_view.rs, rotland/src/app/frame_world.rs (only the E-prompt / interact lines for survivors), game/src/guild/save.rs (one new field with `#[serde(default)]`), game/src/guild/recruits.rs (only a small "add a rescued recruit for free" function, if one doesn't exist)
Rules: `tasks/R29-common.md`. Owner decision: `docs/DECISIONS.md` row 2026-11-25.

**Goal:** survivors are trapped in some houses out in the world, with zombies around. Clear the zombies, walk up, press E, and they join your team for free and follow you.
- **Placement (pure, seeded):** from the world seed, choose some house/shop POIs (look at how `game/src/poi_zombies` picks POIs and copies their rules). Chance per POI is 0.15, at most 6 survivors in the world, and none within 300 m of camp. The same seed must always give the same places. Each survivor gets a name and role from the same lists the guild recruits use (`game/src/guild/roller.rs`), plus a stable id like `rescue_<n>`.
- **Zombies:** each survivor spot has at least 3 zombies near it (use the existing POI zombie spawning; only raise the count there if needed).
- **Model:** the `npc_survivor` model, standing with the idle clip, drawn only within normal NPC view distance (copy how `rotland/src/camp_npcs.rs` draws NPCs).
- **Rescue:** when no living zombie is within 12 m of the survivor and the player is within 2.5 m, show "[E] Rescue <name>". If the team is full (the existing guild team-size limit), show "Team full" instead. On E the survivor becomes a hired `GuildRecruit` (no cost) with order Follow, uses the existing teammate code from then on, and disappears from the spot. Show a short message: "<name> joined your team".
- **Save:** save the list of rescued ids (new `#[serde(default)]` field in the guild save), so a rescued survivor never shows up again.

**Parts:**
- A (pure logic, start first): `rescue.rs` (state + can-rescue check), `rescue_place.rs` (seeded placement), `data/rescue.json` (chancePerPoi 0.15, maxSurvivors 6, minDistFromCamp 300, clearRadius 12, talkRange 2.5, minZombiesNear 3), and tests in `game/tests/all/rescue.rs` (same seed gives same places, max 6, camp distance, clear/not clear, team full, rescued never returns).
- B (wiring, after A): `rescue_play.rs` (tick, prompt, E), `rescue_view.rs` (the model), the prompt hook in `frame_world.rs`, the save field.

## Worker notes

### Part A (pure logic & placement)
- **Created:**
  - `data/rescue.json` (`chancePerPoi: 0.15`, `maxSurvivors: 6`, `minDistFromCamp: 300.0`, `clearRadius: 12.0`, `talkRange: 2.5`, `minZombiesNear: 3`).
  - `game/src/rescue_place.rs`: `RESCUE_BUILDINGS` (`house_small`, `cabin`, `shop`, `apartment`, `fish_shack`), `RescueSpot` (with `to_recruit()`), `is_rescue_building`, `parse_recruit_role`, `place_survivors`, `collect_candidate_pois`, `place_survivors_in_world`. Picks names/roles/stats deterministically from `GuildConfig::recruits` (`data/guild.json`; note `game/src/guild/roller.rs` only has `GuildTaskRoller`, so recruit lists live in `data/guild.json` / `GuildConfig.recruits`).
  - `game/src/rescue.rs`: `RescueConfig` (`from_repo`, `from_file`, `from_json`), `RescueLoadError`, `RescueBlocker`, `RescueState` (`new`, `restore_rescued`, `rescued_ids_sorted`, `active_spots`, `is_rescued`, `has_survivor_at_poi`, `is_area_clear`, `nearest_in_range`, `can_rescue`, `prompt_for`, `try_rescue`, `try_rescue_nearest`, `joined_message`).
  - `game/tests/all/rescue.rs`: tests for `data/rescue.json` loading/roundtrip, deterministic placement & max 6 cap, >= 300 m camp exclusion & house/shop filtering, `WorldGenerator` placement, clear vs not clear (12 m living zombie radius), talk range (2.5 m), team full (`"Team full"` / `TeamFull`), `add_rescued` free + Follow, and `restore_rescued` preventing rescued survivors from returning.
- **Edited:**
  - `game/src/guild/recruits.rs`: added `GuildRecruits::add_rescued(&mut self, mut recruit: GuildRecruit, max_slots: usize) -> Result<(), RecruitHireError>` (adds rescued survivor for free with `hire_cost = 0` and `RecruitOrder::Follow`).
- **Lines the lead must add to shared files:**
  - `game/src/lib.rs`:
    ```rust
    pub mod rescue;
    pub mod rescue_place;
    ```
  - `game/tests/all/main.rs`:
    ```rust
    mod rescue;
    ```
- **Finding on `minZombiesNear` (for Part B / lead):**
  - `data/poi.json` sets `"zombies": [1, 2]` on `house_small`, `cabin`, and `fish_shack`, and `[1, 3]` on `shop` and `apartment`.
  - `PoiZombies::update` (`game/src/poi_zombies/manager.rs:178-192`) rolls strictly within `[def.zombies[0], def.zombies[1]]` per `building_id` (and cycles `site.spawn_points[i % len]`), so survivor POIs will often spawn only 1–2 zombies unless raised.
  - Part B can check `rescue_state.has_survivor_at_poi(&spawn.poi_id)` when `PoiZombies::update` wakes a POI and top up any shortfall below `rescue_state.config.min_zombies_near` (3) around the POI's spawn points / survivor spot (calling `pz.on_zombie_spawned(&poi_id, id)` for each extra zombie), OR `PoiZombies` can be given a per-`poi_id` minimum override.
- **Note for Part B (`CompanionSquad` initial position):**
  - `CompanionSquad::sync` spawns newly hired members at camp (`camp + camp_slot_offset(slot)`). When rescuing a survivor in the world (> 300 m from camp), Part B may want to either let `CompanionSquad` teleport them near the player or set/place them near `spot.position` (note `CompanionSquad` has `pub(super) members` in `game/src/guild/companion/squad.rs`).
- **Build/test status:** Not compiled or run (per subagent rules: never run `cargo`, `tools/gate.sh`, or the game).

### Part B (wiring + view + save field)
- **Created:**
  - `rotland/src/rescue_play.rs`: `RescueInteract`, `load_state` (builds `RescueState` from `data/rescue.json` + `data/guild.json` and restores `rescued_ids` from `"guild"` save entry), `save_rescued_ids` (writes sorted `rescued_ids` into `"guild"` save entry), `interact_tick` (`"[E] Rescue <name>"` / `"Team full"`, rescues on E with `"<name> joined your team"` toast), and `top_up_poi_zombies` (raises waking survivor POI zombie spawns up to `minZombiesNear`).
  - `rotland/src/rescue_view.rs`: `RescueView` drawing active (unrescued) survivor spots with `npc_survivor` + `idle` clip within `data/camp_life.json` `farRange` (60 m), matching `camp_npcs.rs`.
- **Edited:**
  - `game/src/guild/save.rs`: added `#[serde(default)] pub rescued_ids: Vec<String>` to `GuildState`.
  - `rotland/src/app/frame_world.rs`: hooked `crate::rescue_play::interact_tick` into `update_world_interactions` (consuming `ui.e_used` and setting prompt before crate/farm/dog prompts, while keeping ladder distance priority) and `rv.update(dt)` for `actors.rescue_view`.
- **Lines the lead must add to non-owned / shared files:**
  - `game/src/guild/manager.rs` inside `GuildManager::to_state()` (`GuildState { ... }`, after `shop_items: self.shop.items.clone(),`):
    ```rust
                rescued_ids: Vec::new(),
    ```
  - `rotland/src/main.rs` (alphabetical module list):
    ```rust
    mod rescue_play;
    mod rescue_view;
    ```
  - `rotland/src/app/state.rs`:
    - Inside `pub struct ActorsState` (e.g. after `pub camp_npcs`):
      ```rust
      pub rescue_view: Option<crate::rescue_view::RescueView>,
      ```
    - Inside `pub struct PlaySide` (e.g. after `pub guild_panel`):
      ```rust
      pub rescue: Option<game::rescue::RescueState>,
      ```
    - Inside `impl Default for PlaySide` (after `guild_panel: ...`):
      ```rust
                  rescue: None,
      ```
  - `rotland/src/app/start_world.rs`:
    - Reset near `actors.camp_npcs = None;` / `super::frame_guild::reset(play);`:
      ```rust
      actors.rescue_view = None;
      play.rescue = None;
      ```
    - In the `if let (Some(pack), true) = (&render.pack, session.scene == "play")` block (next to `actors.camp_npcs = Some(...)`):
      ```rust
      actors.rescue_view = Some(crate::rescue_view::RescueView::new(&gpu.device, pack));
      ```
    - After the `if let Some(file) = &save_file { ... }` block (and BEFORE `session.loaded_save = save_file;` moves `save_file`):
      ```rust
      if session.scene == "play" {
          play.rescue = crate::rescue_play::load_state(&world_gen, save_file.as_ref());
      }
      ```
  - `rotland/src/app/save_load.rs` right after `let mut file = session::capture(session.loaded_save.as_ref(), &snap);`:
    ```rust
    crate::rescue_play::save_rescued_ids(play.rescue.as_ref(), &mut file);
    ```
  - `rotland/src/app/frame_sim.rs` replacing lines 229-233 (`for spawn in pz.update(...) { ... }`):
    ```rust
                let mut spawns = pz.update(actors.player, &world.poi_zombie_sites, alive_count);
                crate::rescue_play::top_up_poi_zombies(
                    &mut spawns,
                    pz,
                    &world.poi_zombie_sites,
                    play.rescue.as_ref(),
                    &world.physics,
                );
                for spawn in spawns {
                    if let Some(id) = z.spawn_kind(&spawn.kind, spawn.position) {
                        pz.on_zombie_spawned(&spawn.poi_id, id);
                    }
                }
    ```
  - `rotland/src/app/render_frame.rs` right after `main.extend(people);`:
    ```rust
        if let (Some(rv), Some(rescue)) = (actors.rescue_view.as_ref(), play.rescue.as_ref()) {
            let survivors = rv.pose(rescue.active_spots(), actors.player, &mut render.bones);
            shadow.extend(survivors.iter().map(|b| engine::render::SkinnedBatch {
                mesh: b.mesh,
                instances: b.instances.clone(),
            }));
            main.extend(survivors);
        }
    ```
- **How to see it in the game:**
  - Start a `play` game, travel > 300 m from camp to a house/cabin/shop POI that rolled a rescue survivor (up to 6 in the world). Approach within 60 m to see the `npc_survivor` idling outside; clear all zombies within 12 m (at least 3 spawn when the POI wakes), walk within 2.5 m to see `"[E] Rescue <name>"` (or `"Team full"` if at the guild's `max_recruits` cap), press E to see `"<name> joined your team"` and have them added for free with `Follow` order. Save and reload to verify rescued survivors do not reappear.
- **Notes / caveats:**
  - `CompanionSquad::sync` spawns newly hired teammates at camp, so a survivor rescued > 300 m away uses `CompanionSquad::tick`'s existing follow/teleport rule (snaps near the player once outside the forward view cone beyond `teleportDistance`).
  - Not compiled or run (per subagent rules: never run `cargo`, `tools/gate.sh`, or the game).
