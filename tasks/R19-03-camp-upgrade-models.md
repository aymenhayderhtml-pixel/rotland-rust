# R19-03: show camp upgrade models in the camp (walls, gates, towers, workshop, doctor, mechanic)

Read first: `tasks/R19-common.md`, `world/src/camp_layout/*` (`camp_upgrade_props(site, levels)` returns the prop instances), `rotland/src/camp.rs` (camp state; `upgrades`, `tick`), `rotland/src/props.rs` and how settlement/POI props are drawn and given colliders today (search `add_static` and `PropInstance` in `rotland/src`).
Files: rotland/src/camp_upgrade_view.rs, rotland/src/camp.rs, rotland/src/app/render_frame.rs, tasks/R19-03-camp-upgrade-models.md
Test: camp

Today upgrading the camp changes numbers but nothing appears in the camp.
1. New module `rotland/src/camp_upgrade_view.rs`: holds the current prop list for the camp's levels. When the levels change (an upgrade finishes, or a save loads), rebuild the list with `world::camp_upgrade_props` and replace the old colliders with new ones (walls block the player and zombies; gates leave the gate gap open).
2. Draw these props every frame with the same instanced model path the other props use (LOD and shadows like other buildings).
3. Add a test flag `--camp-levels=W,G,K,T` (walls, guards, workshop, traders) that sets the levels at start so Claude can screenshot each tier. If the args parser isn't in your files, write the parse line in Wiring notes and read the value through a small pub fn in your module.
4. Tests: level change produces a new list; same levels produce no rebuild; level 0 has no props.
`mod camp_upgrade_view;` in `main.rs`: put it in Wiring notes.

## Worker notes
