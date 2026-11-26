# R37-08: wiring in the game (after 01-04 report)

Owns: rotland/src/gunsmith_play.rs (new), rotland/src/attachment_view.rs (new), rotland/src/app/* (only the lines needed), rotland/src/shooting*.rs, rotland/src/combat*.rs, rotland/src/fp_weapon.rs, rotland/src/player_visual.rs, rotland/src/dev_panel/* (test flags), rotland_ui/src/hud/* (condition bar on the hotbar slot), game_player/src/gun_feel.rs (read kick_mult), tools/README_TOOLS.md (new flags)
Read: tasks/R37-common.md, the 01-04 worker notes, how the camp trader screen opens and talks to the logic (grep `draw_shop`, `TraderStock`, `npc_trader` in rotland/), rotland/src/shooting.rs, rotland/src/fp_weapon.rs, rotland/src/player_visual.rs, how test flags are parsed (grep `--weapon=`).

- Press E near `npc_gunsmith` or a town shopkeeper → open `GunsmithScreen` for that shop; actions call the R37-03 logic; close with Esc; saves include the shops.
- Fire/melee apply wear (R37-01); jams play the existing "click"/dry-fire path; broken guns can't fire and show "Broken - repair at a gunsmith".
- Effective stats (R37-02) used everywhere a weapon's spread/mag/reload/noise/scope/kick is read for the held gun.
- Attachments drawn on the gun in third and first person at `data/weapon_mounts.json` positions, following the gun.
- Hotbar: a thin condition bar under weapon slots (green → yellow → red).
- Test flags from R37-round.md (`--gunsmith=`, `--gunsmith-town=`, `--give=`, `--wear=`, `--mods=`), documented in tools/README_TOOLS.md.
- Tests where logic lives outside rotland; keep rotland files under ~300 lines (split if needed).

## Worker notes

### What changed
- **Wear & broken/jam gating (R37-01)**:
  - `rotland/src/session.rs`: `restore_play` and `capture` now copy `wear` and `mods` between `save::SavedSlot` and `game::items::SavedSlot`.
  - `rotland/src/shooting.rs`: added `jammed` / `broken` to `ShootEvents`, `sync_mag_cap`, and `Shooting::update_targets_with_wear` (broken guns refuse fire, jams consume shot timing cleanly, spread scales by `spread_mult`, `aim_fov` returns `scope.fov` even on scoped bows like `crossbow`). Added unit tests `broken_gun_refuses_to_fire_and_clicks` and `scoped_crossbow_returns_scope_aim_fov`.
  - `rotland/src/combat.rs`: added `Combat::update_with_wear` scaling melee damage by `melee_damage_mult`. Added unit test `melee_damage_scales_with_wear_multiplier`.
  - `rotland/src/play.rs`: fixed `HubAction::SplitStack` to preserve `stack` via `ItemStack::new`; added `wear_config`, `attachments`, `equipped_mods`, `equipped_condition`, `refresh_equipped_stats`, `apply_equipped_hit_wear`, `apply_equipped_shot_wear`; `shoot_tick` gates/toasts broken (`"Broken - repair at a gunsmith"`) and jammed (`"Weapon jammed!"`) shots and applies shot wear.
  - `rotland/src/app/frame_actors.rs` (`strike_trees`) and `rotland/src/app/frame_actors_combat.rs` (`step_melee_and_respawn`, `step_ranged_and_projectiles`): melee damage scaled by wear + hit wear applied on zombie/tree hits (skipping gun bash); crossbow & bow gated on `!is_broken` with `"Broken - repair at a gunsmith"` toast + `"dryFire"`, shot wear applied on fire, and crossbow right-click uses scoped FOV.
- **Attachments stats, recoil & rendering (R37-02 / R37-05 / R37-06)**:
  - `game_player/src/gun_feel.rs`: added `GunFeelState::on_shot_scaled(&mut self, def: &GunFeelDef, aiming: bool, kick_mult: f32)` and unit test `on_shot_scaled_reduces_kick_and_climb_proportionally`.
  - `rotland/src/gun_feel_view.rs`: added `set_kick_mult` and routed `on_shot` through `on_shot_scaled`.
  - `rotland/src/app/frame_world.rs` & `rotland/src/app/frame_actors_combat.rs`: feed `game::kick_mult(...)` into `render.gun_feel.set_kick_mult` before shots; play `"dryFire"` on `ev.jammed` / broken click.
  - `rotland/src/attachment_view.rs` (new, 196 lines): loads `data/weapon_mounts.json` and attachment `GpuMesh`es from `ModelPack`, resolves per-slot attachment transforms (`weapon_world * Mat4::from_translation(mount_local)`), and emits `SkinnedBatch` draws + 2 unit tests.
  - `rotland/src/app/render_frame.rs` & `rotland/src/app/start_world.rs`: initialize `render.attachment_view` and draw fitted attachments in both third-person (`main` + `shadow`) and first-person (`main`).
- **Gunsmith shop, bench & town shopkeepers (R37-03 / R37-04 / R37-07)**:
  - `rotland/src/camp.rs`: added `gunsmith_config` and `gunsmith` (`GunsmithStock`), morning restock (`tick_with_config`), save/load under `game::GUNSMITH_SAVE_ID` (plus `gunsmith_level` fallback in `sys.extra`), and `UpgradeLine::Gunsmith` handling in `levels`, `upgrade_name`, `load_upgrades_from_save`, `override_levels`.
  - `rotland/src/gunsmith_play.rs` (new, 547 lines): `GunsmithSession`, `GunsmithWorld` (spawns/collides camp `gunsmith_bench` and poses/interacts with town `shopkeeper_<key>` NPCs), `build_view`, and `apply_action` bridging `rotland_ui::gunsmith_ui` with `game::gunsmith::*` and refreshing held stats/hotbar.
  - `rotland/src/app/frame_world.rs`, `frame_camp.rs`, `frame_input.rs`, `frame_ui.rs`, `render_frame.rs`, `start_world.rs`: `[E]` interaction near camp `npc_gunsmith` (`role == "gunsmith"`) or town `shopkeeper_<key>` opens `GunsmithScreen`; walking beyond `gunsmith_config.interact_range` or `Esc` closes it; `bench_placed` and `pose_shopkeepers` render the bench and town shopkeepers.
- **Hotbar condition bar & test flags**:
  - `rotland_ui/src/hud/{state,draw,hotbar}.rs`: added `HudState::hotbar_conditions: Option<Vec<Option<f32>>>` and rendered the thin green -> yellow -> red condition bar along the bottom of weapon slots (plus `condition_color_transitions_green_yellow_red` unit test); populated in `rotland/src/play.rs` (`PlayState::hud`).
  - `rotland/src/app/test_flags.rs`, `startup.rs`, `start_world.rs`, `tools/README_TOOLS.md`: wired `--gunsmith=<tab>`, `--gunsmith-town=<key>`, `--wear=<weapon>:<0..1>`, `--mods=<weapon>:<id,id>`, synced hotbar on `--give=`, added parser unit test coverage, and documented the new flags in `tools/README_TOOLS.md`.

### Shared-file lines for the Lead
1. **`rotland/src/main.rs`**:
   - Insert in module declarations (alphabetical order around line 5 & 29):
     ```rust
     mod attachment_view;
     ```
     ```rust
     mod gunsmith_play;
     ```
   - Optional re-export on line 55 (if desired; `gunsmith_play.rs` uses `rotland_ui::gunsmith_ui` directly):
     ```rust
     pub use rotland_ui::{camp_ui, display, gunsmith_ui, hub_ui, hud, keys_ui, map_ui, menus, settings};
     ```

2. **`rotland/src/app/state.rs`**:
   - In `pub struct RenderState` (around line 115):
     ```rust
     /// Weapon attachment meshes and mount-point offsets (R37-08).
     pub attachment_view: Option<crate::attachment_view::AttachmentView>,
     ```
   - In `pub struct ActorsState` (around line 174):
     ```rust
     /// Camp gunsmith bench and outer-town gun shopkeepers (R37-08).
     pub gunsmith_world: crate::gunsmith_play::GunsmithWorld,
     ```
   - In `pub struct UiState` (around line 298):
     ```rust
     pub gunsmith: Option<crate::gunsmith_play::GunsmithSession>,
     ```
   - In `UiState::screen_open(&self)` (line 317):
     ```rust
     pub fn screen_open(&self) -> bool {
         self.camp_ui.is_some() || self.hub.is_some() || self.gunsmith.is_some() || self.dev_panel.open
     }
     ```

3. **Lead integration dependencies & follow-up notes**:
   - **R37-01..04 `lib.rs` re-exports**: `rotland` and `game_camp` / `game_core` / `rotland_ui` call sites expect the Lead's Step 3 wiring (`game_core::weapons` module + `game::` re-exports for `WearConfig`, `apply_shot_wear`, `apply_hit_wear`, `is_broken`, `jam_roll`, `spread_mult`, `melee_damage_mult`, `AttachSlot`, `AttachmentDef`, `AttachmentRegistry`, `effective_stats`, `kick_mult`; `game_camp::gunsmith` + `game::` re-exports for `GunsmithConfig`, `GunsmithError`, `GunsmithStock`, `GUNSMITH_SAVE_ID`, `ShopKind`, `buy`, `buy_check`, `sell_value`, `sell_slot`, `repair_cost`, `repair_slot`, `repair_all`, `fit_cost`, `fit_slot_at_shop`, `remove_mod_slot`; and `rotland_ui::pub mod gunsmith_ui`).
   - **`CampUpgradesView` / `UpgradeLine::Gunsmith`**: R37-07 added `UpgradeLine::Gunsmith`, while `rotland_ui/src/camp_ui/upgrades/{detail,colors}.rs` and `view.rs` (`[UpgradeLineView; 4]`) and `save::CampUpgradesSaveData` have not yet been expanded to 5 lines; `rotland/src/camp.rs` currently leaves `upgrades_view` at 4 lines and persists `gunsmith_level` in `sys.extra["gunsmith_level"]`.
   - **File size note**: `rotland/src/gunsmith_play.rs` is 547 lines (combines `GunsmithWorld` and shop view/actions so only one new module needed wiring in `main.rs`). Can easily be split into `gunsmith_world.rs` + `gunsmith_play.rs` later if desired.

### How to see / test it (after Lead wires shared files)
- `cargo test` (covers new unit tests in `game_player::gun_feel`, `rotland_ui::hud::hotbar`, `rotland::attachment_view`, `rotland::shooting`, `rotland::combat`, `rotland::app::test_flags`, `rotland::camp`)
- `flock /tmp/rotland-heavy.lock target/debug/rotland --save-dir=/tmp/scratch --shot=/tmp/gunsmith.png --gunsmith=buy --give=scrap_metal:60 --frames=10`
- `flock /tmp/rotland-heavy.lock target/debug/rotland --save-dir=/tmp/scratch --shot=/tmp/town_shop.png --gunsmith-town=dusthollow --frames=10`
- `flock /tmp/rotland-heavy.lock target/debug/rotland --save-dir=/tmp/scratch --shot=/tmp/rifle_mods.png --weapon=rifle --mods=rifle:scope_4x,suppressor --wear=rifle:0.25 --cam-yaw=90 --frames=15`

### Known issues / not checked
- Did NOT run `cargo`, `tools/gate.sh`, `blender`, or the game (forbidden for subagents), so compile/clippy/runtime rendering have not yet been executed in this worktree.
