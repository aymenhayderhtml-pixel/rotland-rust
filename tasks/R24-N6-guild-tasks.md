# R24-N6 (Neon): guild tasks you can finish in play

Files: game/src/guild/task_hooks.rs, game/src/guild/mod.rs, game/tests/all/guild_task_hook_tests.rs, game/tests/all/main.rs, rotland/src/guild_task_view.rs, rotland/src/main.rs, rotland/src/app/frame_guild.rs, rotland/src/app/mod.rs, rotland/src/app/state.rs, rotland/src/app/frame_sim.rs, rotland/src/app/frame_ui.rs, rotland/src/app/start_world.rs, rotland/src/camp.rs, rotland/src/camp_ui/action.rs, rotland/src/camp_ui/guild/mod.rs, rotland/src/camp_ui/guild/tasks_ui.rs, data/guild.json, tasks/R24-N6-guild-tasks.md
Read: game/src/guild/tasks.rs, game/src/guild/board.rs, rotland/src/swim_view.rs, rotland/src/menus/theme.rs
API: game/src/guild/manager.rs, game/src/guild/mod.rs
Data: data/guild.json

Guild board tasks come in four kinds. Kill tasks already count (`GuildManager::on_kill`, called from `camp.rs` `on_zombie_killed`). Fetch and explore tasks have no game hooks yet, clear tasks are not wired to the guild, and the player has no on-screen tracker. Write the pure hook logic and the on-screen tracker. **A helper does the game-loop wiring afterwards, so do NOT edit or assume any file outside `Files:`** (do not edit `guild/mod.rs`, `lib.rs`, `tests/all/main.rs`, `data/guild.json` or `rotland/src/main.rs`; the helper adds those lines and keys). No sounds of any kind. No new crates. The target is a weak laptop, so this must stay cheap: no per-frame scanning of the world, and the tracker draws at most 3 short rows.

## How it must play
1. **Deliver (fetch):** when the player is within `hand_in_radius` of the guild in camp, one action hands in the needed items from the bag. Partial hand-ins count. A hand-in never takes more than the task still needs, and never takes items that no active fetch task wants.
2. **Explore:** entering a named place within `place_radius` completes that objective once. The places come from the caller as a list of named positions, because `data/guild.json` has names but no positions (`Ashford core`, `the lake`, `watchtower`). Checks run every `hook_check_seconds`, not every frame.
3. **Clear:** a clear objective completes through the existing `GuildManager::on_poi_cleared`. The hooks only notice that it is done (see Ready below).
4. **Finished tasks** show "Return to the guild to claim" in the tracker. The claim itself already exists on the Guild camp screen (`GuildCampAction::ClaimTask`).
5. **Abandoned tasks** stop tracking at once. Nothing is kept for them.

## Data: add three keys to `data/guild.json` (helper adds them, Neon only reads them)
Top level, snake_case like the other keys (`hand_in_radius` 6.0, `place_radius` 12.0, `hook_check_seconds` 0.5). `hook_check_seconds` 0.5 is given by this task. The other two are new values, so tag them `// new: DECISIONS 2026-11-23` in the Rust that reads them. `GuildConfig` ignores unknown keys, so the new keys do not break it.

## Logic: `game/src/guild/task_hooks.rs`
Must be pure (no rendering, no files except `from_repo`). Use `glam::{Vec2, Vec3}` like `game/src/swimming.rs`. Distances are horizontal (x and z only). Use these exact names:
```rust
#[derive(Clone, Debug, PartialEq)]
pub struct GuildTaskHookConfig { pub hand_in_radius: f32, pub place_radius: f32, pub hook_check_seconds: f32 }
impl GuildTaskHookConfig {
    pub fn from_json(text: &str) -> Result<Self, serde_json::Error>; // reads the three top-level keys, ignores the rest
    pub fn from_repo() -> Result<Self, std::io::Error>; // paths::repo_path("data/guild.json")
}
pub struct NamedPlace<'a> { pub name: &'a str, pub position: Vec3 }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HandIn { pub task_id: String, pub item_id: String, pub count: u32 }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GuildTaskEvent {
    Progress { task_id: String, progress: u32, target: u32 },
    Ready { task_id: String },
}
#[derive(Clone, Debug, PartialEq)]
pub struct GuildTaskHooks { /* cfg, check timer, ids already announced as Ready */ }
impl GuildTaskHooks {
    pub fn new(cfg: GuildTaskHookConfig) -> Self;
    /// Death or load: forget the timer and the announced ids.
    pub fn clear(&mut self);
    /// Pure plan, nothing is changed. Empty when the player is farther than hand_in_radius from guild_pos.
    /// `count` returns how many of an item the bag holds. Walks the active tasks in order.
    pub fn plan_hand_in(&self, manager: &GuildManager, count: impl Fn(&str) -> u32, guild_pos: Vec3, player: Vec3) -> Vec<HandIn>;
    /// Call AFTER the caller removed each HandIn's items from the bag. Applies progress per task, returns events (Ready included).
    pub fn apply_hand_in(&mut self, manager: &mut GuildManager, plan: &[HandIn]) -> Vec<GuildTaskEvent>;
    /// Throttled: does nothing until hook_check_seconds has passed (then subtracts it). Checks explore objectives and Ready.
    pub fn tick(&mut self, manager: &mut GuildManager, places: &[NamedPlace], player: Vec3, dt: f32) -> Vec<GuildTaskEvent>;
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskRow { pub title: String, pub progress: u32, pub target: u32, pub ready: bool }
/// One row per active task, in board order, at most 3. Reads the board only, so abandoned tasks vanish by themselves.
pub fn tracker_rows(manager: &GuildManager) -> Vec<TaskRow>;
```
Rules:
- **Hand-in is per task, not per manager call.** `GuildManager::on_item_delivered` advances every active task for that item, so two fetch tasks for wood would both count the same wood. Instead loop over `manager.board_mut().active` and call `GuildTask::on_item_delivered(item, count)` on each one. `GuildTask::on_item_delivered` already clamps to `target`.
- **Plan amounts:** for each active fetch task, `need = target - progress`. Take `min(need, bag left for that item)`, where "bag left" is `count(item)` minus what this plan already took for that item. Skip zero amounts. Skip tasks that are done.
- **Explore once:** for each active explore task that is not done, find a place whose name matches `place_name` (case-insensitive, same as `GuildTask::on_place_reached`) and whose horizontal distance to the player is at most `place_radius`. Call `GuildTask::on_place_reached` on that task only. A done task is never touched again.
- **Ready once:** after any change, a task that `is_done()` and is not yet in the announced list gives one `Ready`, and its id goes into the list. This also catches kill and clear tasks that were finished outside the hooks. Remove announced ids that are no longer active, so the list stays small.
- **Throttle:** `tick` only does work when the timer reaches `hook_check_seconds`, then subtracts it. Large `dt` must not cause a burst of repeated checks.
- Progress events are emitted only when the progress really changed.
- No allocation in `tracker_rows` beyond its returned Vec.

## Look: `rotland/src/guild_task_view.rs` (egui 0.37; colors, radius and margin helpers from `menus/theme.rs`; the placement style matches `swim_view.rs`)
- `pub fn draw_task_tracker(ctx: &egui::Context, rows: &[TaskRow])`: nothing when `rows` is empty.
- A compact list anchored `Align2::RIGHT_TOP`, right-aligned with the clock (16 * scale in from the right, under the clock). Offset from the top: `TRACKER_TOP` = 46.0 * scale. `// new: DECISIONS 2026-11-23`.
- Panel uses `theme::PANEL_BG`, a thin `theme::BORDER`, and `theme::radius(theme::CORNER_CONTROL, scale)`. Max 3 rows, width about 260 * scale.
- Each row: title on the left (`theme::TEXT_PRIMARY`, size `FONT_BODY` * scale), and on the right the progress such as `3/5` (`TEXT_MUTED`). When `ready` is true, the second line reads "Return to the guild to claim" in `theme::SAFE`, and the progress on the right is replaced by "Done".
- Scale with `theme::ui_scale(ctx.content_rect().height())`. Must look right from 1280x720 to 2560x1440. No flashing, no animation, no sound.
- Must not overlap the minimap, compass, hotbar, key help line, or the fishing, swimming and siege panels. The helper checks this with screenshots and moves the offset if needed.
- Use `egui::Area::new(egui::Id::unique("rotland_guild_tracker"))` as `swim_view.rs` does. Stable ids with `egui::Id::unique`.

## Tests: `game/tests/all/guild_task_hook_tests.rs`
Build a `GuildManager::new(seed, GuildConfig::default())`, set the board with `board_mut().set_offered(vec![GuildTask::new(...)])`, then `accept_task`. Cover:
- partial hand-in: a fetch task needing 5 wood with a bag of 2 wood gives a plan of 2, progress 2, not done.
- over-delivery: a bag of 9 wood for a task needing 5 gives a plan of exactly 5.
- wrong item: a bag of rope does not change a wood task (empty plan).
- two fetch tasks for the same item share the bag correctly: no wood is counted twice.
- hand-in out of `hand_in_radius` gives an empty plan; inside it gives the plan.
- explore once: the player inside `place_radius` of "Ashford core" completes the explore task; a second tick inside the radius gives no event.
- explore out of radius: no progress.
- explore name matching is case-insensitive.
- throttle: `tick` with `dt` 0.1 five times does nothing before 0.5 s, then checks once.
- abandoned task: after `abandon_task`, `tick` gives no event and `tracker_rows` does not list it.
- ready event once: a finished task gives exactly one `Ready`, and a later tick gives none. A clear task finished via `manager.on_poi_cleared` also gives one `Ready`.
- `tracker_rows`: at most 3 rows, ready flag set when done, title and target copied.
- data tasks: load `GuildManager::from_repo(1)`, call `refresh_morning(1, 7.0, None)`, and check that the rolled board can hold a kill, a fetch, an explore and a clear task (use `GuildTaskRoller` output or set tasks by hand per kind). Each kind is handled by the hooks, not ignored.
- `GuildTaskHookConfig::from_repo()` loads and gives `hook_check_seconds` 0.5.

Every new number in Rust code needs a source tag on its line or the line above (`// data: guild.json hand_in_radius`, `// data: guild.json place_radius`, `// data: guild.json hook_check_seconds`, or `// new: DECISIONS 2026-11-23`). This includes numbers used only in tests.

## Notes to write
In `=== NOTES ===` list the exact wiring steps for the helper, and any number you changed and why. The wiring list must include at least these:
1. `game/src/guild/mod.rs`: `pub mod task_hooks;` and `pub use task_hooks::{...}` for the public names.
2. `game/tests/all/main.rs`: `mod guild_task_hook_tests;`.
3. `rotland/src/main.rs`: `mod guild_task_view;` (next to `mod swim_view;`).
4. `data/guild.json`: add `hand_in_radius` 6.0, `place_radius` 12.0, `hook_check_seconds` 0.5 at the top level.
5. Store `GuildTaskHooks` beside the guild (derives Clone, Debug, PartialEq, so it fits the derive on `GuildPlay`). Call `hooks.clear()` on death and on load.
6. Wire the clear path: `rotland/src/app/frame_sim.rs` (the loop over `cleared` POIs next to `bounties.on_poi_cleared`) must also call `play_state.camp.guild.on_poi_cleared(poi)`. Today the guild never hears cleared POIs. Check that the POI ids there match the `poi_id` values in data (`house_small`, `gas_station`, `supermarket`).
7. Per frame: `hooks.tick(play.camp.guild.manager_mut(), &places, player_pos, dt)`. `places` is a list of `NamedPlace` built from the world's named spots (the helper decides the positions; none exist in data).
8. Interact key near the guild: `let plan = hooks.plan_hand_in(manager, |id| bag.count(id), guild_pos, player);` then for each `HandIn` call `bag.remove(&h.item_id, h.count)`, then `hooks.apply_hand_in(manager, &plan)`. Show the events as short status text.
9. HUD: `guild_task_view::draw_task_tracker(ctx, &game::guild::task_hooks::tracker_rows(manager));` in the play HUD pass.
10. Screenshot check at 1280x720 and 2560x1440 (helper).

Risks / notes
- `guild_pos` (the guild location in camp) is not in data yet. The helper picks it and passes it in.
- Only the three named explore places in data have no positions. Until the helper supplies them, explore tasks cannot finish in play.
- The minimap position is not in this pack. The helper must check the tracker does not cover it.

## Worker notes

## Neon notes

Helper wiring steps

1.  game/src/guild/mod.rs: add:
    pub mod task_hooks;
    pub use task_hooks::{
        GuildTaskEvent, GuildTaskHookConfig, GuildTaskHooks, HandIn, NamedPlace, TaskRow,
        tracker_rows,
    };
2.  game/tests/all/main.rs: add mod guild_task_hook_tests;.
3.  rotland/src/main.rs: add mod guild_task_view; (next to mod swim_view;).
4.  data/guild.json: add at the top level:
    "hand_in_radius": 6.0,
    "place_radius": 12.0,
    "hook_check_seconds": 0.5,
5.  Store GuildTaskHooks (initialized from
    GuildTaskHookConfig::from_repo().unwrap_or_default()) beside the guild
    (GuildTaskHooks derives Clone, Debug, PartialEq, matching GuildPlay). Call
    hooks.clear() on player death and on save load.
6.  Wire the clear path in rotland/src/app/frame_sim.rs: in the loop over
    cleared POIs next to bounties.on_poi_cleared, also call
    play_state.camp.guild.on_poi_cleared(poi). Verify that the POI ids passed
    there match the poi_id values in data/guild.json (house_small, gas_station,
    supermarket).
7.  Per frame: call hooks.tick(play.camp.guild.manager_mut(), &places,
    player_pos, dt), where places is a slice of NamedPlace for the world's named
    locations (Ashford core, the lake, watchtower). Surface emitted
    GuildTaskEvents as short status messages.
8.  Interact key near the guild:
    let plan = hooks.plan_hand_in(manager, |id| bag.count(id), guild_pos, player_pos);
    for h in &plan {
        bag.remove(&h.item_id, h.count);
    }
    let events = hooks.apply_hand_in(manager, &plan);
    Show the returned events as short status text.
9.  HUD: call guild_task_view::draw_task_tracker(ctx,
    &game::guild::task_hooks::tracker_rows(manager)); in the play HUD pass.
10. Screenshot check at 1280x720 and 2560x1440 to verify TRACKER_RIGHT (16.0)
    and TRACKER_TOP (46.0) sit cleanly under the clock without overlapping the
    minimap, compass, hotbar, key help line, or fishing/swimming/siege panels.

Numbers added / notes

  - game/src/guild/task_hooks.rs: uses the task-specified hand_in_radius (6.0),
    place_radius (12.0), hook_check_seconds (0.5), and MAX_TRACKER_ROWS (3, from
    guild.json max_active). GuildTaskHookConfig uses #[serde(default = ...)] on
    those three keys so it ignores all other keys in guild.json and degrades
    safely if a partial JSON is parsed.
  - rotland/src/guild_task_view.rs: added layout constants TRACKER_RIGHT (16.0),
    TRACKER_TOP (46.0), PANEL_WIDTH (260.0), PANEL_MARGIN (10.0), ROW_GAP (4.0),
    BORDER_WIDTH (1.0). All scale via theme::ui_scale. Ready sub-line uses
    theme::FONT_HELP * scale in theme::SAFE so it stays compact inside the 260px
    panel without wrapping.
  - GuildTaskHooks::tick clamps leftover check_timer to 0.0 if a single frame
    hitch (dt >= 2 * hook_check_seconds) occurs, preventing a burst of
    consecutive checks on following frames while preserving exact remainder
    subtraction for normal dt.
  - collect_ready_events retains an announced id only while that task is still
    active and is_done(), so abandoned/claimed tasks (or a re-accepted task with
    the same id) are cleanly re-tracked.

Risks / caller dependencies

  - guild_pos and the world coordinates for the three explore places (Ashford
    core, the lake, watchtower) are not in data/guild.json yet; the helper
    supplies them when wiring steps 7 and 8.
  - The minimap rect was not in this pack; adjust TRACKER_TOP or TRACKER_RIGHT
    in rotland/src/guild_task_view.rs if the screenshot check shows overlap. ===
    END ===

## Gate report
- date: 2026-11-23
- base: main
- files changed (34): assets/models/buildings/settlement.glb, assets/models/_previews/car_hatchback_blue.png, assets/models/_previews/car_hatchback_blue_rear.png, assets/models/_previews/car_hatchback_blue_side.png, assets/models/_previews/car_hatchback_olive.png, assets/models/_previews/car_hatchback_olive_rear.png, assets/models/_previews/car_hatchback_olive_side.png, assets/models/_previews/car_hatchback_red.png, assets/models/_previews/car_hatchback_red_rear.png, assets/models/_previews/car_hatchback_red_side.png, assets/models/vehicles/car_hatchback_blue.glb, assets/models/vehicles/car_hatchback_olive.glb, assets/models/vehicles/car_hatchback_red.glb, data/guild.json, game/src/guild/mod.rs, game/src/guild/task_hooks.rs, game/tests/all/guild_task_hook_tests.rs, game/tests/all/main.rs, rotland/src/app/frame_guild.rs, rotland/src/app/frame_sim.rs, rotland/src/app/frame_ui.rs, rotland/src/app/mod.rs, rotland/src/app/start_world.rs, rotland/src/app/state.rs, rotland/src/camp.rs, rotland/src/camp_ui/action.rs, rotland/src/camp_ui/guild/mod.rs, rotland/src/camp_ui/guild/tasks_ui.rs, rotland/src/guild_task_view.rs, rotland/src/main.rs, tasks/R24-N6-guild-tasks.md, tasks/R25-N7-special-zombies.md, tools/blender/buildings/settlement.py, tools/blender/vehicles/car_parts.py
- tests passed: 829
- clippy: PASS
- test: PASS
- checks: FAIL
- gate: FAIL (checks)
