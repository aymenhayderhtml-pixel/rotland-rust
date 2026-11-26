# R37-02: attachments (data + stat effects)

Owns: game_core/src/weapons/attachments.rs (new), data/attachments.json (new), data/items.json (one item per attachment), game/tests/all/attachments_tests.rs (new)
Read: tasks/R37-common.md, game_core/src/weapons.rs (WeaponDef fields: spread, magazine, reload, noise, scope), data/weapons.json, data/items.json (item format), data/gun_feel.json (kick).

- Attachments (ids from shared names), which guns they fit, and effects (balanced, small): scopes set `scope_fov` (2x ≈ 40, 4x ≈ 20) and lower aim spread; red_dot a little less hip+aim spread, short-range only (pistols, smg, ak47, revolver); suppressor `noise_mult` 0.35 (pistol, smg, ak47, rifle, hunting_rifle); ext_mag +50% mag (pistol, smg, ak47), drum_mag +150% mag but reload ×1.3 (ak47, smg); foregrip kick ×0.7 (ak47, smg, shotgun); stock_pad kick ×0.8 (rifles, shotgun); bolt_quiver reload ×0.75 (crossbow).
- One slot each: a weapon has at most one mod per `AttachSlot`.
- Items in `data/items.json` (category `attachment`, weight, icon name = id, value).
- `effective_stats` is pure and tested: each effect, two mods stacking, wrong-weapon mods ignored, unknown ids ignored.
- `kick_mult` must reach gun feel: report to the lead how `gun_feel` should read it (don't edit game_player).

## Worker notes
- **Files created/modified**:
  - `game_core/src/weapons/attachments.rs` (new): defines `AttachSlot`, `AttachEffects`, `AttachmentDef`, `AttachmentLoadError`, `AttachmentRegistry` (`from_repo`, `load`, `from_json`, `get`, `contains`, `len`, `is_empty`, `iter`, `for_weapon`), `can_fit`, `kick_mult`, `effective_stats`.
  - `data/attachments.json` (new): defines the 9 attachments (`scope_2x`, `scope_4x`, `red_dot`, `suppressor`, `ext_mag`, `drum_mag`, `foregrip`, `stock_pad`, `bolt_quiver`) with `slot`, `fits`, `model`, `priceScrap`, and `effects`.
  - `data/items.json` (modified): added all 9 attachment items with `"category": "attachment"`, `shortName`, `weight`, `maxStack: 1`, `value`, `color`, and `description`. Note: `game/tests/all/data_integrity.rs::every_item_has_icon_png` expects `assets/icons/<id>.png` for each of the 9 items (created by R37-06).
  - `game/tests/all/attachments_tests.rs` (new): tests loading/validation of `data/attachments.json` + matching `data/items.json` & `data/weapons.json`, each effect (`scope_2x`, `scope_4x`, `red_dot`, `suppressor`, `ext_mag`, `drum_mag`, `foregrip`, `stock_pad`, `bolt_quiver`), stacking (`foregrip` + `stock_pad` kick `0.56`), wrong-weapon/unknown mods ignored, and one-mod-per-`AttachSlot` enforcement.
- **Exact shared-file lines needed (for Lead)**:
  - `game_core/src/weapons/mod.rs`:
    ```rust
    pub mod attachments;

    pub use attachments::{
        AttachEffects, AttachSlot, AttachmentDef, AttachmentLoadError, AttachmentRegistry,
        can_fit, effective_stats, kick_mult,
    };
    ```
  - `game_core/src/lib.rs` (replace lines 26-28):
    ```rust
    pub use weapons::{
        AttachEffects, AttachSlot, AttachmentDef, AttachmentLoadError, AttachmentRegistry,
        WeaponAnim, WeaponDef, WeaponHold, WeaponKind, WeaponLoadError, WeaponRegistry, WeaponScope,
        can_fit, effective_stats, kick_mult,
    };
    ```
  - `game/src/lib.rs` (replace lines 37-39):
    ```rust
    pub use game_core::weapons::{
        AttachEffects, AttachSlot, AttachmentDef, AttachmentLoadError, AttachmentRegistry,
        WeaponAnim, WeaponDef, WeaponHold, WeaponKind, WeaponLoadError, WeaponRegistry, WeaponScope,
        can_fit, effective_stats, kick_mult,
    };
    ```
  - `game/tests/all/main.rs`:
    ```rust
    mod attachments_tests;
    ```
- **How `gun_feel` should read `kick_mult` (for R37-08 / Lead)**:
  1. In `game_player/src/gun_feel.rs`, `GunFeelState::on_shot(&mut self, def: &GunFeelDef, aiming: bool)` currently sets `let mul = if aiming { def.aim_mul } else { 1.0 };` and scales `kick_deg`, `kick_yaw_deg`, `fp_back`, and `fp_tilt_deg` by `mul`.
  2. Add `pub fn on_shot_scaled(&mut self, def: &GunFeelDef, aiming: bool, kick_mult: f32)` (keeping `on_shot` calling `self.on_shot_scaled(def, aiming, 1.0)` so existing tests in `game/tests/all/gun_feel.rs` stay unchanged): multiply `mul` by `kick_mult.max(0.0)` before computing `kick`, `yaw_deg`, `fp_back`, and `fp_tilt_deg` (for climbing weapons like `ak47`/`smg`, optionally also scale `max_climb_deg * kick_mult.max(0.0)` if the cap should drop with a foregrip/stock pad).
  3. In `rotland/src/gun_feel_view.rs`, store `kick_mult: f32` (default `1.0`) on `GunFeelView`, pass it in `set_context(&mut self, weapon_id: Option<&str>, aiming: bool, kick_mult: f32)` (or a `set_kick_mult(f32)` setter), and call `self.state.on_shot_scaled(def, self.aiming, self.kick_mult)` inside `GunFeelView::on_shot`.
  4. In `rotland/src/app/frame_world.rs` (around line 133, right before `render.gun_feel.on_shot(weapon_id)`), compute `let km = game::weapons::kick_mult(&equipped_stack.mods, weapon_id, &attachments_reg);` and pass `km` to `render.gun_feel`.
- **Design notes / checks**:
  - `effective_stats` distinguishes magnified scopes (`scope_fov.is_some()`, e.g. `scope_2x`/`scope_4x`) from non-zoom optics (`red_dot`) by scaling only `spread_aim_deg` when `scope_fov` is `Some`, and scaling both `spread_hip_deg` and `spread_aim_deg` when `scope_fov` is `None`.
  - Not compiled or executed locally per subagent rule ("Never run `cargo`, `tools/gate.sh`, `blender` or the game"). All types/field names (`WeaponDef`, `WeaponScope`, `ItemCategory::Attachment`, `GunFeelState`) were verified directly against source files.
