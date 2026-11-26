# R20-05: bag shows item pictures; profile stats not cut off; "Unequip" for the item in hand (issues 005, 006, 007 part 1)

Read first: `tasks/R20-common.md`, `docs/ISSUES.md` rows 005, 006, 007 (+ pictures), `rotland/src/hub_ui/*` (`draw.rs` ~128, `draw_bag.rs`, `draw_bag_details.rs`, `draw_profile.rs`), `rotland/src/icons.rs` (the hotbar already draws icons with it).
Files: rotland/src/hub_ui/*, rotland/src/icons.rs, rotland/src/app/frame_ui.rs, tasks/R20-05-bag-profile-ui.md
Test: hub

1. Issue 006: pass the icon cache into the hub and draw each item's icon in its bag slot (colour square only as fallback when there is no icon). Show the big icon in the details panel too.
2. Issue 005: the Profile stats list is in a 480 px high scroll box, so it is cut off. Let it use the panel's full height (scroll only if it really doesn't fit at 1280x720). Make the stat stripes the same width (fixed value column width, values right- or left-aligned consistently).
3. Issue 007 part 1: in the bag details, the item that is in the hand says "Unequip" (not "Equip"), and its "in hand" border stays visible when the slot is also selected (draw both, e.g. yellow outer + white inner, or a small hand badge).
4. Check at 1920x986 and 1280x720 by layout math in tests (nothing cut off); write how Claude can screenshot it in Worker notes.

## Worker notes

### Changes Made
1. **Issue 006 (Bag item pictures & details panel icon):**
   - Passed `&mut IconCache` from `frame_ui.rs` into `hub_ui::draw` and `draw_bag`.
   - In `draw_slot_grid`, rendered item icons at 44x34 px using `icons.icon_or_label(ui.ctx(), Some(&s.item_id), label)` with fallback to the colored swatch rectangle.
   - Looked up the selected slot's icon texture and passed it to `draw_details_panel`, showing the 96x96 icon image in the details pane.
2. **Issue 005 (Profile stats panel & uniform stripes):**
   - Replaced fixed `max_height(480.0)` in `draw_profile.rs` with `ui.available_height().max(520.0)` and grid row spacing `vec2(16.0, 4.0)` so all 12 lifetime stat rows and the difficulty controls fit within the panel without being cut off at 1280x720 and 1920x986.
   - Added `STAT_VALUE_WIDTH = 240.0` in `add_stat_row`, allocating a consistent sized label for value columns so alternating background stripes are uniform across all rows.
3. **Issue 007 part 1 ("Unequip" for held item & visible dual border):**
   - Added `equipped_weapon: Option<&'a str>` to `HubView` and implemented `view.is_in_hand(&item_id)`.
   - In `frame_ui.rs`, synchronized `actors.combat` weapon state with `play_state.hotbar` and passed `equipped_weapon` into `HubView`.
   - In `draw_bag_details.rs`, `in_hand` now checks `view.is_in_hand(&s.item_id)`, displaying "Unequip" when held.
   - In `draw_slot_grid`, when a slot is both equipped and selected, it renders both borders: yellow outer border (`Stroke::new(2.0, Color32::from_rgb(255, 217, 51))`) plus white inner border (`Stroke::new(2.0, Color32::WHITE)` inset by 2.0 px).

### Layout Verification
- Unit test `test_profile_layout_math_fits_1280x720_and_1920x986` validates:
  - Panel size: 900x620 px inside 1280x720 has 50 px vertical margin; inside 1920x986 has 183 px vertical margin.
  - Body available height is 484 px.
  - Profile content total height (title + 12 stat rows + separator + difficulty header + 4 difficulty rows) is 418 px, leaving 66 px headroom. All rows fit without triggering a scrollbar.

### How Claude Can Screenshot & Verify
Run with `--save-dir` (never use owner saves) and `--shot`:
```bash
flock /tmp/rotland-heavy.lock target/debug/rotland --save-dir=/tmp/scratch_r20_05 --shot=/tmp/r20-05-bag.png
```
Or launch interactive test to switch to Bag (Tab) and Profile (P) tabs:
```bash
flock /tmp/rotland-heavy.lock target/debug/rotland --save-dir=/tmp/scratch_r20_05
```

## Gate report
- date: 2026-11-22
- base: main
- files changed (8): rotland/src/app/frame_ui.rs, rotland/src/hub_ui/draw_bag_details.rs, rotland/src/hub_ui/draw_bag.rs, rotland/src/hub_ui/draw_profile.rs, rotland/src/hub_ui/draw.rs, rotland/src/hub_ui/tests.rs, rotland/src/hub_ui/view.rs, tasks/R20-05-bag-profile-ui.md
- tests passed: 25
- clippy: PASS
- test: PASS
- checks: PASS
- gate: PASS
