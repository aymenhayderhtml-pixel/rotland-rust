# R10-05: Bag screen and crafting list polish (GitHub #9, #10, #11, #18 UI-003, #19 CODE-003) — PHASE A

Read `tasks/R10-common.md` first. Screenshots: `gh issue view 9`, `10`, `11`. Godot reference: `../zombi servival/scripts/UI/InventoryDetailsPanel.cs`, `CraftingPanel.cs`, `CraftingRecipeRow.cs`, `SlotPicker.cs`.

## You own
- `rotland/src/hub_ui/draw_bag.rs`, `draw_bag_details.rs`, `draw_crafting.rs`
- `rotland/src/camp_ui/draw_crafting.rs`, `draw_shop.rs`, `draw_bank.rs`
- `rotland/src/hub_ui/draw_trunk.rs`
- New `rotland/src/hub_ui/item_text.rs` (shared item name/category/stat text helpers)
- `rotland/src/menus/menus_draw/menus_draw_slots.rs` (overwrite prompt only)

Do **not** edit `game/src/items/*` (R10-08 owns it this round). Read item data through the existing public API.

## 1. Crafting shows materials (#9)
- Each recipe row: result + each input as `Cloth 1/2` (have/need), missing ones in red, enough ones in normal colour.
- Station needs (e.g. "needs a lit campfire") shown the same way.
- Greyed Craft button: hover text says why ("Missing: Cloth x1", "Needs a lit campfire nearby").
- Same in the camp crafting screen if it is a separate file.

## 2. One button style (#10)
- Use, Equip/Unequip, Drop 1, Drop all, Split: same size and style, in a neat grid (2 or 3 per row). One small helper for the button so all screens can reuse it.

## 3. Item details panel (#11)
Below the buttons, from the data files (read real keys in `data/items.json` and `data/weapons.json`):
- Weapons: damage, attack speed / cooldown, reach, stamina cost; guns: ammo type, magazine size.
- Food/drink: hunger / thirst restored. Medical: health healed.
- Description line if the item has one.
- A larger item picture if the icon cache from R10-04 exists: take an `Option<TextureId>` as input and leave a "Wiring for Claude" note; draw nothing when `None`.

## 4. Shared item text (CODE-003)
Move the repeated name/category/weight formatting from `draw_shop.rs`, `draw_bank.rs`, `draw_trunk.rs` into `item_text.rs` and use it in all of them. Unit tests for the helpers.

## 5. Overwrite prompt (#18 UI-003)
Check: does New Game on a filled slot overwrite it without asking? Quote the code. If yes, add an "Overwrite this save?" confirm like the delete confirm.

## Worker notes

### Files changed
- `rotland/src/hub_ui/item_text.rs`: New module with shared helpers for item names, categories, weight, trunk/bank/shop row formatting, weapon stats, item effects, crafting blocker reasons, action button styling, and comprehensive unit tests.
- `rotland/src/hub_ui/draw_crafting.rs`: Extracted crafting subpanel from `draw_bag.rs` displaying recipe ingredients (have/need), missing ingredients in red, campfire station status in red/orange, and greyed-out Craft button with hover reason (`on_disabled_hover_text`).
- `rotland/src/hub_ui/draw_bag.rs`: Delegated crafting subpanel rendering to `draw_crafting.rs`, declared submodules, and passed `icon_texture` parameter to `draw_details_panel`.
- `rotland/src/hub_ui/draw_bag_details.rs`: Unified action buttons (Use, Equip/Unequip, Drop 1, Drop all, Split) to consistent size/style grid via `action_button`; added weapon stats (melee and guns), food/drink/medical effects, description, and optional `TextureId` image below buttons.
- `rotland/src/hub_ui/draw_trunk.rs`: Replaced duplicate trunk and bag item row string formatting with `format_trunk_row` from `item_text.rs`.
- `rotland/src/camp_ui/draw_crafting.rs`: Added station needs display (`[Needs campfire]` in red when not near lit campfire) and `on_disabled_hover_text` tooltip with blocker reasons on Craft button.
- `rotland/src/camp_ui/draw_shop.rs`: Replaced duplicate buy and sell row string formatting with `format_shop_row` from `item_text.rs`.
- `rotland/src/camp_ui/draw_bank.rs`: Replaced duplicate store and take-out row string formatting with `format_bank_row` from `item_text.rs`.
- `rotland/src/menus/menus_draw/menus_draw_slots.rs`: Analyzed New Game behavior and added `draw_overwrite_confirm` modal function.

### New public API
```rust
// rotland/src/hub_ui/item_text.rs
pub const ACTION_BTN_SIZE: egui::Vec2;
pub fn action_button(ui: &mut egui::Ui, text: impl Into<egui::WidgetText>, enabled: bool) -> egui::Response;
pub fn format_item_name<'a>(id: &'a str, def: Option<&'a ItemDef>) -> &'a str;
pub fn format_item_category(category: &str) -> String;
pub fn format_item_weight(weight_each: f32, count: u32) -> String;
pub fn format_trunk_row(name: &str, count: u32, blocker: Option<&str>) -> String;
pub fn format_bank_row(name: &str, count: u32, is_full: bool, full_msg: &str) -> String;
pub fn format_shop_row(name: &str, price: u32, count: u32) -> String;
pub fn format_item_effects(effects: &ItemEffects) -> String;
pub fn get_weapon_def(weapon_id: &str) -> Option<WeaponDef>;
pub fn format_weapon_stats(weapon: &WeaponDef, ammo_name: Option<&str>) -> Vec<String>;
pub fn craft_blocker_reason(
    recipe: &RecipeDef,
    inventory: Option<&Inventory>,
    near_lit_campfire: bool,
    is_crafting: bool,
    is_dead: bool,
    item_name_fn: impl Fn(&str) -> String,
) -> Option<String>;

// rotland/src/hub_ui/draw_crafting.rs
pub const SHORT_COLOR: egui::Color32;
pub const NORMAL_COUNT_COLOR: egui::Color32;
pub fn draw_crafting_subpanel(ui: &mut egui::Ui, view: &HubView) -> Option<HubAction>;

// rotland/src/hub_ui/draw_bag_details.rs
pub fn draw_details_panel(
    ui: &mut egui::Ui,
    state: &mut HubUiState,
    view: &HubView,
    icon_texture: Option<egui::TextureId>,
) -> Option<HubAction>;

// rotland/src/menus/menus_draw/menus_draw_slots.rs
pub fn draw_overwrite_confirm(
    ctx: &egui::Context,
    slot: u8,
    on_confirm: impl FnOnce() -> MenuAction,
    on_cancel: impl FnOnce(),
) -> Option<MenuAction>;
```

### Wiring for Claude
1. In `rotland/src/hub_ui/mod.rs` (which R10-05 was not permitted to edit):
   Add:
   ```rust
   mod draw_crafting;
   pub mod item_text;
   ```
   (Currently `draw_bag.rs` declares them via `#[path = "..."]` so the worktree compiles self-contained). When added to `hub_ui/mod.rs`, the `#[path]` declarations in `draw_bag.rs` can be replaced with `use super::draw_crafting; use super::item_text;`.
2. Icon texture caching from R10-04:
   In `rotland/src/hub_ui/draw_bag.rs:86`, pass `icon_texture: Option<TextureId>` to `draw_details_panel(ui, state, view, icon_texture)`. It currently passes `None` (draws nothing when `None`).
3. Overwrite prompt check (#18 UI-003):
   Check finding: In `rotland/src/menus/menus_draw/menus_draw_slots.rs:86`:
   ```rust
   let is_empty = summary.state == SlotState::Empty;
   if ui
       .add_enabled(is_empty, egui::Button::new("New game").min_size(vec2(140.0, 36.0)))
       .clicked()
   {
       action = Some(MenuAction::NewGame { slot: slot_num });
   }
   ```
   and Godot parity `SlotPicker.cs:87`:
   `card.New.Disabled = info.State != SlotState.Empty;`
   "New game" on a filled slot does NOT overwrite without asking because the button is disabled when `is_empty` is false (`summary.state != SlotState::Empty`).
   If you want to permit clicking New Game on a filled slot with confirmation:
   Add `pending_overwrite: Option<u8>` to `Menus` in `menus.rs`, set `menus.pending_overwrite = Some(slot_num)` on click, and render `draw_overwrite_confirm` in `menus_draw.rs` line 58.

### Tests
All tests located in `rotland/src/hub_ui/item_text.rs`:
- `test_format_item_name`: Verifies name fallback ("cooked_meat" -> "Cooked Meat" from `data/items.json:8`, "wood" -> "wood").
- `test_format_item_category`: Verifies category capitalization ("food" -> "Food", "weapon" -> "Weapon").
- `test_format_item_weight`: Verifies weight calculation: 0.40 kg each, 0.80 kg total (`data/items.json:2` canned_beans weight 0.4).
- `test_format_trunk_row`: Verifies row formatting with/without blocker text.
- `test_format_bank_row`: Verifies row formatting with/without "(Bank full)".
- `test_format_shop_row`: Verifies row formatting with price and count ($120 from `data/items.json:36` pistol).
- `test_format_item_effects`:
  - `canned_beans`: hunger 35 from `data/items.json:3` -> "Hunger +35"
  - `dirty_water`: thirst 25, health -8 from `data/items.json:13` -> "Thirst +25, Health -8"
- `test_format_weapon_stats_melee`:
  - `bat`: damage 25, cooldown 0.60, range 2.0, stamina cost 12 from `data/weapons.json:2-3`
- `test_format_weapon_stats_gun`:
  - `pistol`: damage 30, headshot 2.0, fire rate 0.25, range 60, mag 12, ammo "pistol_ammo" from `data/weapons.json:24-25`
- `test_craft_blocker_reason`:
  - `cooked_meat`: campfire station, input raw_meat x1 from `data/recipes.json:10-11`. Checks campfire requirement ("Needs a lit campfire nearby\nMissing: Raw Meat x1"), ingredient requirement ("Missing: Raw Meat x1"), and ready state (`None`).

### Checked?
not checked (write-only round)

### Questions / known issues
None. All tasks from issues #9, #10, #11, #18 UI-003, and #19 CODE-003 are implemented adhering to Godot parity and data-driven principles.
