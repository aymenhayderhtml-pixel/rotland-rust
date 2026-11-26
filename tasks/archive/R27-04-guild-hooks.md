# R27-04: guild "deliver" and "explore" tasks actually work

Files: game/src/guild/*, game/tests/all/guild*.rs, game/tests/all/main.rs, data/guild*.json, rotland/src/app/frame_guild.rs, rotland/src/camp_ui/*, tasks/R27-04-guild-hooks.md
Rules: `tasks/R27-common.md`.

**Goal:** guild tasks of kind "deliver" (bring items) and "explore" (reach a place) can be completed in the game. Today they have no hooks (see `docs/STATUS.md`, `tasks/R24-N6-guild-tasks.md` and its Worker notes, `rotland/src/app/frame_guild.rs` which already hooks clear/explore checks).
- Find exactly what is missing. Deliver: when the player hands in at the guild tab, check the inventory has the items, remove them, mark the task done (reuse the existing hand-in click if there is one). Explore: mark done when the player reaches the target spot (radius from data).
- **Do not invent rewards or rules.** Use what the guild data and code already define. If a rule is missing and would change how the game feels, write it as a question in Worker notes and do the rest.
- Tests for the pure parts (inventory check + removal, reach check).

## Worker notes
- **What changed:**
  - `game/src/guild/task_hooks.rs`: added `GuildTaskHooks::is_in_hand_in_range` and `GuildTaskHooks::plan_hand_in_for` so clicking "Hand in items" on a specific fetch task plans directly for that task (fixes a bug where a second fetch task for the same item got starved by the first task before `retain`).
  - `rotland/src/app/frame_guild.rs`: updated `hand_in` to use `is_in_hand_in_range` + `plan_hand_in_for` with distinct status messages for out-of-range vs missing items vs ready-to-claim; updated `build_places` for `"the lake"` to include all `lake.docks` and `lake.shore` points (instead of only `lake.docks.first()`).
  - `rotland/src/camp_ui/guild/mod.rs` and `rotland/src/camp_ui/guild/tasks_ui.rs`: passed `view.inventory` into `draw_tasks_column` so the "Hand in items" button shows `(N in bag)` and disables when `N == 0`.
  - `game/tests/all/guild_task_hook_tests.rs`: added unit tests `plan_hand_in_for_targets_requested_task_even_with_duplicate_item` and `plan_hand_in_for_returns_none_when_out_of_range_or_empty`.
- **How to see it in the game:**
  - **Deliver (fetch):** Stand near the camp center (within 6 m of `camp_point`), open the Camp Board -> Guild tab, accept a fetch task, and click `Hand in items (N in bag)` when you have the item in your inventory. Items are removed and progress advances (or marks the task ready to claim).
  - **Explore:** Accept an explore task (`Ashford core`, `the lake`, or `watchtower`) and walk within `place_radius` (12 m) of the city center intersection, any lake dock/shore point, or any loaded watchtower. The tracker marks it `Done` and shows "Task ready: return to the guild".
- **Questions / Wiring notes for Claude (outside `Files:`):**
  - **Hand-in radius vs Trader/Banker position (`rotland/src/app/frame_ui.rs` & `data/guild.json`):** `frame_ui.rs:331` sets `guild_pos = camp_point(...)` (`site.center`), and `data/guild.json` sets `hand_in_radius: 6.0`. However, `SettlementNpc::default_settlement_npcs` has no `"guild"` NPC, and the Trader (where `[B] Camp Board` is prompted) is at `(-8.2, 0, -7.7)` (~11.25 m from `site.center`), and the Banker is at `(7.2, 0, -7.5)` (~10.40 m from `site.center`). Should `hand_in_radius` in `data/guild.json` be increased (e.g., to `20.0` to cover the camp safe zone), or should `frame_ui.rs` allow hand-in anywhere inside the settlement safe zone / add a Guild NPC within 6 m of `site.center`?

## Gate report
- date: 2026-11-24
- base: main
- files changed (6): game/src/guild/task_hooks.rs, game/tests/all/guild_task_hook_tests.rs, rotland/src/app/frame_guild.rs, rotland/src/camp_ui/guild/mod.rs, rotland/src/camp_ui/guild/tasks_ui.rs, tasks/R27-04-guild-hooks.md
- tests passed: 1017
- clippy: PASS
- test: PASS
- checks: PASS
- gate: PASS
