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
