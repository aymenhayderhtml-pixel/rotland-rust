# R44 common: New Halden rooftop life (safe house, fast travel, rooftop camp, trader, rescues)

Setup and lead loop: `tasks/LEAD.md` (worktree `../rotland-r44`, branch `r44`, shared `CARGO_TARGET_DIR`).
Plan: `docs/PLAN_1.1.md` (R44 row). Owner choices: `docs/DECISIONS.md` 2026-11-28 row (this round's Claude defaults). R43 is the style reference: `tasks/R43-common.md`.

## What is true today (checked by Claude 2026-11-28)
- **Fire-escape towers** (`world/src/metro/fire_escapes.rs`):
  - `fire_escape_towers(metro) -> Vec<usize>`: brick towers that have working fire escapes. It skips climb towers, replaced / burnt towers and reserved blocks.
  - `fire_escape_ladders(metro, idx) -> Vec<ClimbLadder>`: the drop ladder (street -> first landing) and the roof ladder (top landing -> roof deck).
  - So the safe house sits on a fire-escape tower, and the player reaches the roof by its fire escape.
- **Climb towers** (`world/src/metro/climb.rs`):
  - `climb_towers(metro) -> Vec<ClimbTower>`, with `ClimbTower { tower, kind: ClimbKind {Midtown, Core, Landmark}, stop_floors: Vec<u32>, stop_kinds: Vec<StopKind {Office, Flat, Camp}>, deck_y }`.
  - `climb_in_chunk(metro, chunk, height) -> ClimbChunk { pois, props, crates, ladders, boxes }`. Stop-floor POIs have ids `metro_floor_<tower>_<i>`; confirm with `grep -n metro_floor_ world/src/metro/climb.rs`.
  - `climb.rs:215` skips the `Reserved::SafeHouseTower` block when picking climb towers (`world/src/metro/layout.rs:45` defines that reserved kind).
- **Tower data** (`world/src/metro/towers.rs` `TowerSpot`): `pos: Vec2`, `half`, `yaw`, `size` (16/24/32), `floors`, `style`, `landmark`, `block`. `metro.layout.site.center` is the city centre. `metro.damage.tower_is_replaced(idx)`, `metro.damage.tower_is_burnt(block)`.
- **Roof height:** `roof_deck_y(t, style, ground_y)` (climb.rs). Tower-local frame as in R43-common (front = local -Z). Shaft frame: see R43-common.
- **Radio glue to copy** (R43):
  - `rotland/src/metro_radio.rs` (`MetroRadioRuntime`, `to_save`, `restore_save` on `systems.extra["metro_radio"]`).
  - `rotland/src/app/start_world.rs:192` (init), `:234-235` (restore from save).
  - `rotland/src/app/frame_world.rs` ~`:388-398` (interact tick, then `consumed_press`), `:214` (crate filter).
  - Save write: `rotland/src/app/save_load.rs` `save_now(session, world, actors, play)` at line 7.
- **Ladders:** `game::LadderRail::new(foot, deck, suffix)` (`game_player/src/ladders/`). Rails come from `ClimbLadder` through `Streamer::rebuild_ladders` (`rotland/src/streaming.rs`). A rope ladder that can be pulled up needs runtime add/remove of its rail (wiring task).
- **Rescue** (`game_camp/src/rescue.rs`, `rotland/src/rescue_play.rs`, `game_camp/src/rescue_place.rs`):
  - `RescueConfig` (from `data/rescue.json`): `chance_per_poi`, `max_survivors`, `min_dist_from_camp`, `clear_radius`, `talk_range`, `min_zombies_near`, `per_town`.
  - `RescueState`: `new`, `restore_rescued`, `active_spots`, `is_rescued`, `has_survivor_at_poi`, `is_area_clear`, `nearest_in_range`, `prompt_for`, `try_rescue_nearest` (joins the guild recruits), `joined_message`.
  - `RescueSpot` (`rescue_place.rs:35`): `id, poi_id, building_id, name, role: RecruitRole, level, daily_wage, max_health, position, yaw`.
  - `place_survivors_in_world(world_gen, cfg, recruits)` (`rescue_place.rs:327`) is the random POI pass. It does NOT yet skip `metro_floor_` POIs.
  - `rotland/src/rescue_play.rs`: `load_state(world_gen, save_file)` (line 26) builds the spots; `interact_tick` (line 65) rescues on E; `save_rescued_ids`.
- **Trader** (`game_camp/src/economy/trader.rs`):
  - `TraderStock::from_entries(&[StockEntry])`, `take`, `qty`, `fill_all`, `tick`, `to_save`/`from_save`. `StockEntry { id, qty }` (`economy/config.rs:10`).
  - Camp trader: `rotland/src/camp.rs:27` (`pub trader: TraderStock`). Prices from `data/economy.json` (`buyRate`, `sellRate`).
  - UI: `CampAction::Buy{item,count}` / `Sell{item,count}` (`rotland_ui/src/camp_ui/action.rs:10-13`), `draw_shop` (`rotland_ui/src/camp_ui/draw_shop.rs:13`), `CampScreen::Shop`. Reached through a camp NPC with role `trader` (`frame_world.rs:525`).
- **Bank** (the closest thing to a camp stash): `game_camp/src/economy/bank.rs` (`move_to_bank`, `move_to_bag`), `bankSlots: 40` in economy.json, `CampScreen::Bank` (role `banker`, `frame_world.rs:526`).
- **Clock:** `world::time::Clock` (`world/src/time.rs`): field `hour`, `set_time(hour)` (line 75), `advance_hours(hours) -> ClockChanges` (line 100). The play side mirrors it: `play.hour = play.clock.hour` (`frame_world.rs:607`).
- **Teleport pattern:** `rotland/src/app/test_flags/metro_tp.rs` (test flags move the player and set `play.hour`).
- **Stats** (`rotland/src/app/stats.rs`): `playerPosition` (54), `playerOnFloor` (209), `poiZombiesAwake` (71), `nearestZombie` (81), `zombies` (95). **There is no game-hour key and no safe-house key yet.**

## Premise checks (things the owner's brief assumes that are NOT in the code)
1. **There is no camp bed and no sleep code.** `camp.bed_in_front` (`rotland/src/camp.rs:214`) is a farm plot named "bed" (a crop bed). `grep -rni sleep` finds nothing in `rotland`, `rotland_ui`, `game_camp`, `game`, `data`. Claude default: the safe-house bed does a **save** (`save_now`) and moves the clock to `restHour`.
2. **There is no player-level "chased" flag.** The closest checks are per zombie: `ZombieState::Chase` and `Attack` (`game_zombie/src/zombie/state.rs:5`, used in `rotland/src/zombies/spawning.rs:276`). R44-01 adds `count_chasing_near`.
3. **There is no home-camp sign.** R44 adds one `travel_sign` at the camp (placed by the wiring task, see R44-05).
4. **Storage:** the shack chest opens the existing `Bank` screen (the camp stash). Claude default; if the owner wants a separate chest, that is a new system.

## Owner decisions this round follows (DECISIONS 2026-11-28, Claude defaults)
- **Safe house:** one fire-escape brick tower, the one nearest the city centre (`safe_house_tower`). Its roof has a tarp shack (bed, storage chest, travel sign) and a **rope ladder hanging down the BACK face** (the front face keeps the fire escape). Claim with [E] at the shack; claiming lowers the ladder.
- **Bed:** saves the game and sets the clock to `restHour`.
- **Chest:** opens the Bank screen.
- **Roof edge [E]:** pulls the rope ladder up or down. Down = a ladder rail from the street to the roof.
- **Fast travel:** once claimed, [E] at the home-camp sign ("Travel to safe house") and at the shack sign ("Travel to camp"). Free. Game time moves forward `travelHours`. Blocked while any zombie is chasing (`chaseRadiusM`).
- **Rooftop camp:** the Core climb tower farthest from the safe house. Two tents, a green smoke barrel (the radio's "look for the green smoke"), a rooftop trader stall, 3 survivors (`npc_survivor`) and the trader (`npc_trader`).
- **Trader:** the existing `TraderStock`, with a city stock list in `data/rooftop_trader.json`. Prices from economy.json.
- **Rescues:** 3 survivors on stop floors of climb towers (not the top floor, not Camp floors). They join through the existing rescue flow.
- Numbers are in `data/safe_house.json`, `data/rooftop_trader.json`, `data/rescue.json` (`floorSurvivors`). Source tags: `// data: safe_house.json key`, `// new: DECISIONS 2026-11-28`.

## Data file `data/safe_house.json` (R44-01 creates it; the other tasks read these keys)
```json
{
  "_why": "R44: rooftop safe house on one fire-escape tower; fast travel; rooftop camp. Claude defaults 2026-11-28.",
  "travelHours": 3.0,
  "restHour": 7.0,
  "chaseRadiusM": 40.0,
  "useRadiusM": 2.0,
  "mapLabel": "Safe House",
  "prompts": {
    "claim": "[E] Claim safe house",
    "bed": "[E] Rest and save",
    "chest": "[E] Open storage",
    "ladderUp": "[E] Pull the rope ladder up",
    "ladderDown": "[E] Lower the rope ladder",
    "travelToSafeHouse": "[E] Travel to safe house",
    "travelToCamp": "[E] Travel to camp",
    "travelBlocked": "Zombies are chasing you",
    "notClaimed": "Find the safe house first",
    "claimedToast": "Safe house claimed. The ladder is down.",
    "rested": "Saved. Morning."
  },
  "shack": { "centreLocal": [-4.0, -5.0], "bedLocal": [-4.6, -5.6], "chestLocal": [-3.2, -5.9], "signLocal": [-2.2, -4.0], "travelPointLocal": [-4.0, -3.0] },
  "ladder": { "footOutM": 4.0, "deckInsetM": 0.6, "centreX": 0.0, "segmentM": 1.0 },
  "rooftopCamp": {
    "mapLabel": "Rooftop Camp",
    "props": [
      { "model": "evac_tent", "local": [-5.0, 3.0], "yaw": 0.4 },
      { "model": "evac_tent", "local": [-5.0, -1.0], "yaw": -0.3 },
      { "model": "green_smoke_barrel", "local": [0.0, 2.0], "yaw": 0.0 },
      { "model": "rooftop_trader_stall", "local": [4.0, 0.0], "yaw": -1.5708 },
      { "model": "fire_barrel", "local": [0.5, -1.0], "yaw": 0.0 }
    ],
    "npcs": [
      { "id": "rooftop_survivor_1", "model": "npc_survivor", "local": [-3.0, 2.5], "yaw": 0.6, "sitting": true },
      { "id": "rooftop_survivor_2", "model": "npc_survivor", "local": [-1.0, -2.5], "yaw": 1.2, "sitting": false },
      { "id": "rooftop_survivor_3", "model": "npc_survivor", "local": [2.0, 3.0], "yaw": 3.0, "sitting": true },
      { "id": "rooftop_trader", "model": "npc_trader", "local": [2.8, 0.0], "yaw": -1.5708, "sitting": false, "role": "trader" }
    ]
  }
}
```
- All `local` values are tower-local metres on the roof deck (x, z), using the R43 tower frame. `y` = `roof_deck_y`. The `ladder` block is on the BACK face (local +Z). `footOutM` is metres beyond the back face, on the street.
- If `safe_house.json` fails to load, the safe house is off and a warning is logged (as `metro_radio` does). Nothing else falls back to a Rust default.

## Shared names (every task uses EXACTLY these)
- **`game_world/src/safe_house.rs`** (NEW, pure; R44-01):
  - `SafeHouseConfig` (serde, `from_repo()`, `from_json()`): `travel_hours`, `rest_hour`, `chase_radius_m`, `use_radius_m`, `map_label`, `prompts: SafeHousePrompts`. It ignores the geometry blocks (`shack`, `ladder`, `rooftopCamp`), which belong to `world`.
  - `SafeHouseState { pub claimed: bool, pub ladder_down: bool }`, default false/false.
  - `claim(&mut self)` (if not yet claimed: sets claimed and `ladder_down = true`; if already claimed: does nothing), `toggle_ladder(&mut self) -> bool` (no-op returning false unless claimed; returns the new `ladder_down`), `to_json(&self) -> serde_json::Value`, `from_json(&Value) -> Self` (missing = default).
  - `pub fn travel_blocked(chasing: usize) -> bool` (`chasing > 0`).
- **`game_zombie/src/chasing.rs`** (NEW, pure; R44-01): `pub fn count_chasing_near(zombies: impl IntoIterator<Item = (Vec3, bool)>, pos: Vec3, radius: f32) -> usize`. The bool is "is in Chase or Attack"; the caller maps the zombie list.
- **`world/src/metro/safehouse.rs`** (NEW, pure; R44-03):
  - `pub struct SafeHouseLayout { pub shack: ShackLayout, pub ladder: LadderLayout, pub rooftop_camp: RooftopCampLayout }` (serde, with `from_json(&str)`; the functions below take it as a parameter so they stay pure). `pub fn safe_house_layout() -> Option<&'static SafeHouseLayout>`: the one `OnceLock` loader that reads `data/safe_house.json` with `paths::repo_path` (`world` depends on `paths`, as `generator_contents.rs` does for `climb_floors.json`). R44-05 and R44-06 call it; nothing else reads the file. `ShackLayout { centre_local, bed_local, chest_local, sign_local, travel_point_local: [f32; 2] }`, `LadderLayout { foot_out_m, deck_inset_m, centre_x, segment_m: f32 }`, `RooftopCampLayout { map_label, props: Vec<PropSpec>, npcs: Vec<NpcSpec> }`, `PropSpec { model, local: [f32; 2], yaw: f32 }`, `NpcSpec { id, model, local: [f32; 2], yaw: f32, sitting: bool, role: Option<String> }`.
  - `safe_house_tower(metro) -> Option<usize>`: the nearest `fire_escape_towers` pick to `metro.layout.site.center` (ties: lowest index).
  - `rooftop_camp_tower(metro) -> Option<usize>`: the Core climb tower farthest from the safe house (if none is Core, the farthest climb tower).
  - `pub struct SafeHouseSpots { pub tower: usize, pub deck_y: f32, pub shack: Vec3, pub bed: Vec3, pub chest: Vec3, pub sign: Vec3, pub travel_point: Vec3, pub ladder_foot: Vec3, pub ladder_deck: Vec3 }` and `safe_house_spots(metro, layout: &SafeHouseLayout, height: &dyn Fn(f32, f32) -> f32) -> Option<SafeHouseSpots>`.
  - `safehouse_in_chunk(metro, layout, chunk, height, ladder_down: bool) -> ClimbChunk`: props `safehouse_shack`, `safehouse_bed`, `safehouse_chest`, `travel_sign` (the shack sign), plus (ladder down) the `rope_ladder_seg` props (`ladder.segmentM`) or (ladder up) one `rope_ladder_coil` on the roof edge. No ladder rails here: the rails are runtime (R44-05).
  - `pub struct RooftopNpc { pub id: String, pub model: String, pub position: Vec3, pub yaw: f32, pub sitting: bool, pub role: String }`.
  - `rooftop_camp_in_chunk(metro, layout, chunk, height) -> (ClimbChunk, Vec<RooftopNpc>)`: the props from `rooftopCamp.props`, the NPCs.
  - `pub struct ClimbRescueSeed { pub poi_id: String, pub building_id: String, pub position: Vec3, pub yaw: f32 }` and `rescue_floor_seeds(metro) -> Vec<ClimbRescueSeed>` (exactly 3, see R44-03). Pure, deterministic.
- **`game_camp/src/rescue_floor.rs`** also has `pub fn is_climb_floor_poi(poi_id: &str) -> bool` (`metro_floor_` prefix); `place_survivors_in_world` skips those POIs.
- **`game_camp/src/economy/rooftop_stock.rs`** (NEW; R44-04): `RooftopTraderConfig { pub map_label: String, pub stock: Vec<StockEntry> }` (`from_repo()` from `data/rooftop_trader.json`, `from_json()`), `pub fn rooftop_stock(&self) -> TraderStock`.
- **`game_camp/src/rescue_floor.rs`** (NEW; R44-04): `pub struct FloorRescueSeed { pub poi_id: String, pub building_id: String, pub position: Vec3, pub yaw: f32 }`, and `pub fn floor_rescue_spots(seeds: &[FloorRescueSeed], cfg: &RescueConfig, recruits: &[RecruitDef]) -> Vec<RescueSpot>`. Ids `rescue_floor_<n>`. Takes the first `cfg.floor_survivors` seeds, and names are taken from recruit templates not already used.
- **`rotland/src/safe_house_play.rs`** (NEW; R44-05): `SafeHousePlay { pub state: SafeHouseState, config, ... }` with `claim()`, `to_save()`, `restore_save(&Value)`, `interact_tick(...)` (prompts and E), `open_rooftop_shop(...)` (opens the Shop screen with the rooftop stock). R44-06 calls `claim()` and `open_rooftop_shop()` for the flags.
- **Save keys:** `systems.extra["metro_safehouse"]` = `SafeHouseState::to_json()`. Rescues stay in `systems.extra["guild"]` (`rescued_ids`).
- **Stats keys (R44-05 adds to `rotland/src/app/stats.rs`):** `gameHour` (f32), `safeHouseClaimed` (bool), `safeHouseLadderDown` (bool), `rescuedCount` (usize), `chasingZombies` (usize, the count used by travel).
- **Flags** (R44-06, `rotland/src/app/metro_flags_r44.rs`): `--metro=safehouse`, `safehouse_claimed`, `rooftop_camp`, `rooftop_trader`, `rescue_floor`.
- **Models** (R44-02 builds them): `safehouse_shack`, `safehouse_bed`, `safehouse_chest`, `travel_sign`, `rope_ladder_seg`, `rope_ladder_coil`, `survivor_tent`, `green_smoke_barrel`, `green_smoke_plume`, `rooftop_trader_stall`. Existing and reused: `evac_tent`, `fire_barrel`, `npc_survivor`, `npc_trader`, `camp_stash`, `rooftop_camp`, `roof_ladder_seg`, `roof_hatch`.
- **Lines for the lead:** each task writes the `mod` / `pub use` lines it needs into its Worker notes. The lead adds them to shared files (`game_world/src/lib.rs`, `game_zombie/src/lib.rs`, `game_camp/src/lib.rs`, `game/src/lib.rs`, `game/tests/all/main.rs`, `world/src/metro/mod.rs`, `rotland/src/main.rs`, `rotland/src/app/state.rs`, `rotland/src/app/mod.rs`), as in R43.

## Tests that must stay green
`world/tests/golden.rs` passes UNEDITED. R38-R43 metro tests still pass. Old saves load (no `metro_safehouse` = unclaimed). Towers that are not safe-house or rooftop-camp towers keep today's parts. Spawning outside New Halden is unchanged. Camp shop behaviour (the camp trader) is unchanged.
