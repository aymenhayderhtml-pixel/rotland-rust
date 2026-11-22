# R20-05: bag shows item pictures; profile stats not cut off; "Unequip" for the item in hand (issues 005, 006, 007 part 1)

Read first: `tasks/R20-common.md`, `docs/ISSUES.md` rows 005, 006, 007 (+ pictures), `rotland/src/hub_ui/*` (`draw.rs` ~128, `draw_bag.rs`, `draw_bag_details.rs`, `draw_profile.rs`), `rotland/src/icons.rs` (the hotbar already draws icons with it).
Files: rotland/src/hub_ui/*, rotland/src/icons.rs, rotland/src/app/frame_ui.rs, tasks/R20-05-bag-profile-ui.md
Test: hub

1. Issue 006: pass the icon cache into the hub and draw each item's icon in its bag slot (colour square only as fallback when there is no icon). Show the big icon in the details panel too.
2. Issue 005: the Profile stats list is in a 480 px high scroll box, so it is cut off. Let it use the panel's full height (scroll only if it really doesn't fit at 1280x720). Make the stat stripes the same width (fixed value column width, values right- or left-aligned consistently).
3. Issue 007 part 1: in the bag details, the item that is in the hand says "Unequip" (not "Equip"), and its "in hand" border stays visible when the slot is also selected (draw both, e.g. yellow outer + white inner, or a small hand badge).
4. Check at 1920x986 and 1280x720 by layout math in tests (nothing cut off); write how Claude can screenshot it in Worker notes.

## Worker notes
