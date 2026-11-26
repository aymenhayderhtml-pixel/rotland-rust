# R18-01: camp upgrades become playable (camp board + state + save + effects hookup)

FIRST run: `git merge --no-edit origin/neon/camp-board || git merge --no-edit neon/camp-board` (Neon's camp board screen; it does not compile until you wire it).
Read first: `tasks/R18-common.md`, `game/src/camp_upgrades/` (logic: `can_start`, `start`, `tick`, `cancel`, `CampEffects`), `rotland/src/camp_ui/` (the new `draw_upgrades.rs`, `view.rs` `CampUpgradesView`, actions `StartUpgrade`/`CancelUpgrade`), `rotland/src/camp.rs`, `rotland/src/app/frame_ui.rs` (how camp screens get their view and actions), `rotland/src/app/frame_world.rs` (~line 350: which camp NPC opens which screen), `rotland/src/app/startup.rs` (`--camp-screen=`), `save/src/systems/settlements.rs` + `save/src/systems/mod.rs` (how camp state is saved), `rotland/src/app/save_load.rs`.
Files: rotland/src/camp_ui/*, rotland/src/camp.rs, rotland/src/app/frame_ui.rs, rotland/src/app/frame_world.rs, rotland/src/app/frame_camp.rs, rotland/src/app/startup.rs, rotland/src/app/state.rs, rotland/src/app/save_load.rs, rotland/src/menus/menus_draw.rs, rotland/src/menus.rs, save/src/*, save/src/systems/*, save/tests/*, game/src/camp_upgrades/*, game/tests/all/camp_upgrades_tests.rs, tasks/R18-01-camp-board-wiring.md
Test: camp

1. Fix the merge so it compiles (`crate::menus::theme` must be reachable: make the theme module visible to the crate from `menus_draw.rs`/`menus.rs`).
2. Camp upgrade state lives with the camp/play state, is saved and loaded (add it to the save with `#[serde(default)]` so old saves load at level 0; add a save round-trip test in the save crate's tests).
3. Opening: a new camp NPC role or interact spot "board" opens `CampScreen::Upgrades` (reuse the trader-style NPC interaction; if the camp has no free NPC, the trader shows a second option or the board is a prop with an E interact: pick the simplest working way and explain). Also `--camp-screen=upgrades` for screenshots.
4. Fill `CampUpgradesView` from state + config + inventory + wallet every frame the screen is open. Handle `StartUpgrade` (check `can_start`, remove cash and items, `start`) and `CancelUpgrade` (refund).
5. Time: `tick` with in-game hours elapsed every frame; when a level finishes, `ui.hud.notify("Upgrade finished: <name>", Info)`.
6. Effects hookup (the parts that are just numbers): safe-zone size += `safe_zone_half_bonus`; guard count/damage/range/fire interval from `CampEffects` (where camp guards are configured today: find where `guardDamage` etc. from settlements.json are read and add the bonus there); crafting recipes with a `tier` above `recipe_tier` are hidden/locked (add an optional `tier` to recipes if missing, default 0); doctor (heal to full for cash) and mechanic (repair car for cash) as two new buttons on the camp board when unlocked. Placing the new 3D walls/towers in the camp is another lane (R18-02); just expose `camp_effects()` for it.

## Worker notes

## Gate report
- date: 2026-11-21
- base: main
- files changed (22): rotland/src/app/frame_camp.rs, rotland/src/app/frame_ui.rs, rotland/src/app/frame_world.rs, rotland/src/app/save_load.rs, rotland/src/app/startup.rs, rotland/src/app/state.rs, rotland/src/camp.rs, rotland/src/camp_ui/action.rs, rotland/src/camp_ui/draw_crafting.rs, rotland/src/camp_ui/draw.rs, rotland/src/camp_ui/draw_upgrades.rs, rotland/src/camp_ui/mod.rs, rotland/src/camp_ui/screen.rs, rotland/src/camp_ui/view.rs, rotland/src/menus/menus_draw.rs, rotland/src/menus.rs, save/src/lib.rs, save/src/systems/mod.rs, save/src/systems/settlements.rs, save/tests/saves.rs, tasks/R17-N2-camp-board.md, tasks/R18-01-camp-board-wiring.md
- tests passed: 66
- clippy: PASS
- test: PASS
- checks: FAIL
- gate: FAIL (checks)
