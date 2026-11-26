# R37-01: weapon condition (wear)

Owns: game_core/src/items/stack.rs (the 2 new fields + helpers only), game_core/src/weapons/wear.rs (new), data/weapon_wear.json (new), game/tests/all/weapon_wear_tests.rs (new)
Read: tasks/R37-common.md, game_core/src/items/stack.rs, game_core/src/items/inventory*.rs (how stacks merge/split: stacks with wear or mods must never merge), game_core/src/weapons.rs, data/weapons.json, the save code that writes the inventory (grep `ItemStack` in save/).

- Add `wear`/`mods` to `ItemStack` exactly as in the shared names, with serde defaults so old saves load. Fix every place that builds an `ItemStack` literal (grep) to use `..` or `ItemStack::new`.
- `data/weapon_wear.json`: per weapon (all guns, bows, melee): `wearPerShot` / `wearPerHit`. Aim: a pistol lasts ~400 shots, an AK ~600, the bat ~300 hits, the crossbow ~250 shots. Globals: `jamBelow` 0.3, `jamChanceMax` 0.15, `spreadBelow` 0.5, `spreadMultMax` 1.6, `meleeDamageMultAtZero` 0.5 (melee keeps working but weaker; guns can't fire at 0).
- Pure fns from the shared names, with tests: wear goes down per shot, never below 0; jams only below jamBelow; spread grows smoothly; a fresh stack (`None`) counts as 1.0; old save JSON without the fields loads; stacks with different wear don't merge.
- Report the call sites rotland needs (where a shot / melee hit happens) to the lead. Don't edit rotland.


## Worker notes

### Files changed
- `game_core/src/items/stack.rs` — added `pub wear: Option<f32>` and `pub mods: Vec<String>` (with `#[serde(default, skip_serializing_if = ...)]`) to `ItemStack`, dropped `Eq` derive (kept `PartialEq` since `f32` is not `Eq`), updated `ItemStack::new`, and added `condition(&self) -> f32`, `has_mod(&self, id: &str) -> bool`, and `is_plain(&self) -> bool`.
- `game_core/src/items/inventory.rs` — updated `can_add_with`, `fill_stacks`, and `swap_slots` so stacks with `wear.is_some()` or `!mods.is_empty()` (`!stack.is_plain()`) never merge with other stacks, and `swap_slots` preserves `wear` & `mods`.
- `game_core/src/items/hotbar.rs` — updated the two `ItemStack { ... }` literals in `fixup_equipped_from_save` to `ItemStack::new(item, 1)`.
- `game_core/src/items/save.rs` — added `wear` and `mods` (with serde defaults + `skip_serializing_if`) to `SavedSlot`, dropped `Eq` derive on `SavedSlot` and `InventorySaveData` (kept `PartialEq`), updated `write_from_slots` and `read_into_inventory` (clamping finite `wear` to `0.0..=1.0`).
- `save/src/systems/inventory.rs` — added `wear` and `mods` (with serde defaults + `skip_serializing_if`) to `SavedSlot`, kept `SavedSlot::new(slot, id, count)` signature unchanged, dropped `Eq` derive on `SavedSlot` and `InventorySaveData` (`save/src/file.rs` and `save/tests/saves.rs` only require `PartialEq`).
- `game/tests/all/items_tests.rs` — updated the two `ItemStack { ... }` literals to `ItemStack::new(...)`.
- `data/weapon_wear.json` (new) — globals (`jamBelow: 0.3`, `jamChanceMax: 0.15`, `spreadBelow: 0.5`, `spreadMultMax: 1.6`, `meleeDamageMultAtZero: 0.5`) and per-weapon `wearPerShot` / `wearPerHit` for all 20 guns, bows, and melee weapons in `data/weapons.json`.
- `game_core/src/weapons/wear.rs` (new) — `WearConfig`, `WeaponWearEntry`, `WearLoadError`, `apply_shot_wear`, `apply_hit_wear`, `is_broken`, `jam_chance`, `jam_roll`, `spread_mult`, `melee_damage_mult`.
- `game/tests/all/weapon_wear_tests.rs` (new) — unit/integration tests covering `weapon_wear.json` completeness, shot/hit wear & clamping at 0, jam thresholds & rolls, spread/melee scaling, `has_mod`/`condition`, old save JSON loading, round-trip serialization, and non-merging in `Inventory`.

### Shared-file lines needed from Lead
1. `game_core/src/weapons/mod.rs`:
   ```rust
   pub mod wear;
   pub use wear::{
       apply_hit_wear, apply_shot_wear, is_broken, jam_chance, jam_roll, melee_damage_mult,
       spread_mult, WeaponWearEntry, WearConfig, WearLoadError,
   };
   ```
2. `game_core/src/lib.rs` (replace `pub use weapons::{ ... };` at lines 26–28):
   ```rust
   pub use weapons::{
       apply_hit_wear, apply_shot_wear, is_broken, jam_chance, jam_roll, melee_damage_mult,
       spread_mult, WeaponAnim, WeaponDef, WeaponHold, WeaponKind, WeaponLoadError,
       WeaponRegistry, WeaponScope, WeaponWearEntry, WearConfig, WearLoadError,
   };
   ```
3. `game/src/lib.rs` (replace `pub use game_core::weapons::{ ... };` at lines 37–39):
   ```rust
   pub use game_core::weapons::{
       apply_hit_wear, apply_shot_wear, is_broken, jam_chance, jam_roll, melee_damage_mult,
       spread_mult, WeaponAnim, WeaponDef, WeaponHold, WeaponKind, WeaponLoadError,
       WeaponRegistry, WeaponScope, WeaponWearEntry, WearConfig, WearLoadError,
   };
   ```
4. `game/tests/all/main.rs`:
   ```rust
   mod weapon_wear_tests;
   ```

### Shared-file lines & call sites needed in `rotland` (Lead / R37-08)
**Compile fixes for new `ItemStack` / `SavedSlot` fields:**
- `rotland/src/play.rs:337` (`HubAction::SplitStack`): replace `game::items::ItemStack { item_id: stack.item_id, count: n }` with `game::items::ItemStack::new(stack.item_id, n)`.
- `rotland/src/session.rs:120` (`restore_play`): update `SavedSlot` mapping to `game::items::SavedSlot { slot: s.slot, id: s.id.clone(), count: s.count, wear: s.wear, mods: s.mods.clone() }`.
- `rotland/src/session.rs:202` (`capture`): update `SavedSlot` mapping to `save::SavedSlot { slot: e.slot, id: e.id, count: e.count, wear: e.wear, mods: e.mods }`.

**Wear wiring call sites for R37-08:**
- **Gun firing** (`rotland/src/shooting.rs` `Shooting::update_targets` lines 196–229, invoked by `PlayState::shoot_tick` at `rotland/src/play.rs:659` and `update_world_interactions` at `rotland/src/app/frame_world.rs:105`):
  - Before firing: check `is_broken(cond)` (if `cond <= 0.0`, refuse to fire / show broken dry-fire feedback) and `jam_roll(cond, &wear_cfg, || self.rng.next_f32())` (if jammed, consume shot cooldown/click and skip `fire_rays`).
  - Scale `weapon.spread_hip_deg` and `weapon.spread_aim_deg` by `spread_mult(cond, &wear_cfg)` before `fire_rays(...)`.
  - When `events.shots_fired > 0`: call `apply_shot_wear(equipped_stack, &wear_cfg)`.
- **Crossbow / Bow firing** (`rotland/src/app/frame_actors_combat.rs`):
  - Crossbow (`step.fired` at line 269): gate on `!is_broken(cond)` and call `apply_shot_wear(equipped_stack, &wear_cfg)`.
  - Bow (`if let Some(draw) = released` at line 321): gate on `!is_broken(cond)` and call `apply_shot_wear(equipped_stack, &wear_cfg)`.
- **Melee hits** (`rotland/src/combat.rs` `Combat::update` lines 144–183, invoked by `step_melee_and_respawn` at `rotland/src/app/frame_actors_combat.rs:37`):
  - Scale `mv.weapon.damage *= melee_damage_mult(cond, &wear_cfg)` before `self.swing.try_start(&mv.weapon, &mut player.stamina)` at line 149 (melee stays usable at 0.0 condition at `meleeDamageMultAtZero` = 0.5x damage).
  - When melee hits land (`for hit in self.swing.update(...)` at line 166 or tree strike at `rotland/src/app/frame_actors.rs:60`): call `apply_hit_wear(equipped_stack, &wear_cfg)`.

### How to verify
- Run `cargo test weapon_wear` and `cargo test items` and `cargo test --test saves` (after Lead adds the shared-file `mod` / `pub use` lines and `rotland` struct literal fixes).

### Notes / unsure
- `jam_roll(cond, cfg, mut rng: impl FnMut() -> f32) -> bool` takes a closure returning `[0.0, 1.0)` so `game_core` needs no dependency on `game_world::guns_fire::GunRng`; `rotland/src/shooting.rs` can pass `|| self.rng.next_f32()`.
- Did not run `cargo` or `tools/gate.sh` per subagent rules.
