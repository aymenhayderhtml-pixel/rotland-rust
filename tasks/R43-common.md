# R43 common: New Halden climbing (stairwells, stop-off floors, rooftops, fire escapes, the radio)

Setup, subagent rules and the lead loop: `tasks/LEAD.md` (worktree `../rotland-r43`, branch `r43`, shared `CARGO_TARGET_DIR`).
Plan: `docs/PLAN_1.1.md` (R43 row). Owner choices: `docs/DECISIONS.md` 2026-11-27 rows (the "New Halden climbing" row is this round). Read the R42 Lead notes + "Claude review fixes" in `tasks/R42-round.md` once.

## What is true today (checked by Claude 2026-11-27)
- **Towers** (`world/src/metro/towers.rs` `TowerSpot`):
  - `pos` (world XZ centre), `half`, `yaw`, `floors`, `size` (16/24/32), `style`, `kind`, `landmark`, `block`.
  - The tower local frame is the same as the lobby props in `world/src/metro/tower_parts.rs`: `wx = pos.x + lx*cos(yaw) + lz*sin(yaw)`, `wz = pos.y - lx*sin(yaw) + lz*cos(yaw)`. Front (the lobby door) = local -Z.
- **Stacking** (`tower_parts.rs`):
  - The base (lobby) spans `ground_y .. ground_y + baseHeight` (7.6 m = two 3.8 m storeys).
  - Floor `i` (0-based) starts at `ground_y + baseHeight + i*floorHeight`. **Every style has `floorHeight` 3.8** (`data/metro.json` `styles`).
  - The crown starts at `crown_y = ground_y + baseHeight + floors*3.8`.
  - The walkable roof deck of a normal tower is at `crown_y + roofDeckHeight` (per style: brick 2.4, stone 2.95, glass 3.33, slab 0.3, deco 1.0).
  - Landmarks have setbacks (at 45% / 75% of the floors the module size drops, never below 24 m) and a landmark crown. The tallest landmark wears `tw_landmark_mast`: a lattice mast with a 6x6 m platform 18 m above the crown base, and the `radio_hut` on it (`tools/blender/towers/landmarks.py`, "R43 goal").
- **Colliders today:**
  - Tower parts are draw-only. Each tower gets ONE solid box hull from `baseHeight` up to the roof deck (landmarks: the full height), plus the base model's own wall colliders, so the lobby stays walk-in (`rotland/src/metro_render/hulls.rs`, `rotland/src/streaming_metro.rs` `load_metro_chunk`).
  - Crowns, floors and lobby props have NO colliders.
  - Chunk props (`PropInstance`) and POI buildings DO get convex colliders from their models (`Streamer::upload`, `shapes_of`; tiny decals are skipped by `is_no_collider_prop`).
- **Lobby props** (`metro.json` `towers.lobbyProps`, size-relative): `lobby_desk` (0, 0), `lobby_elevators` (0, +0.25), `lobby_mailboxes` (-0.32, 0.05), `lobby_bench` (0.3, -0.05), `lobby_rubble`, `lobby_plant_dead`.
- **Ladders:**
  - `game::LadderRail::new(foot, deck, suffix)` (`game_player/src/ladders/`). The climber faces from foot toward deck (flat), climbs up and steps off at `deck`.
  - Today rails come only from `ladder_bottom*`/`ladder_top*` empties of POI building models (`Streamer::upload` -> `LoadedChunk.ladders` -> `Streamer::rebuild_ladders`).
- **Brick towers** already show fire escapes (front + one side, zig-zag stairs, a drop ladder on the base; `tools/blender/towers/brick.py`, `brick_parts.py` `add_fire_escape_floor`). They are draw-only today.
- **POIs:**
  - A chunk `PoiInstance` (building model, pos, yaw, loot) wakes when the player is within `approachDistance` (35 m) **flat** distance (`game_zombie/src/poi_zombies/manager.rs`).
  - Zombies spawn at the model's `spawn_*` empties at their real 3D height (`frame_sim_zombies.rs` -> `spawn_kind_look`, no ground snap).
  - Awake POI zombies are capped by `poi.json` `maxActiveZombies` (16).
- **Loot boxes** = `LootSpot` with the R41 `table`/`model` overrides.
- **The city spawner never puts street zombies inside tower footprints** (`metro_spawn_ground`).
- The player walks slopes up to 46 degrees (`physics/src/mover.rs`). The third-person camera pulls in on walls (spring-arm raycast, `game/src/camera_rig.rs`).
- Back faces are culled: from inside a tower the exterior floor modules are invisible, and you see whatever interior model is there (this is how the R39 lobbies work).

## Owner decisions this round follows (PLAN_1.1 R43 + Claude defaults, DECISIONS 2026-11-27; numbers live in data)
- **Climbable towers:**
  - 4 normal towers (2 Midtown, 2 Core) + the **tallest landmark**: 5 in all.
  - Not damaged / replaced / burnt, size at least 24, spread out (at least 3 blocks apart), never the reserved safe-house tower (that is R44).
  - Each is marked from the street: a painted "ROOF" arrow sign at the lobby door + the stair door open in the lobby.
- **Inside:**
  - A stairwell next to the lobby elevators, from the lobby floor to the top floor.
  - Most floor doors are jammed shut (debris, barricade).
  - **Stop-off floors** are open: 2 (Midtown), 3 (Core), 4 (landmark), plus the **top floor**, which is always open. Each is a dressed interior: an office floor, an apartment floor, or a survivors' last-stand camp. They have loot and zombies.
  - A few stairwell landings have 1-2 zombies (lying, a crawler).
  - The zombies of a floor wake only when you are on that floor.
- **Roof:**
  - The top floor has a ladder up through a roof hatch onto the roof deck.
  - From the roof, **plank bridges** (and ladders where the roofs differ in height) lead to 1-2 nearby roofs. That makes a small rooftop route; R44 builds on it.
- **Fire escapes:** 8 brick towers (Edge / Midtown) get **working** fire escapes:
  - the drop ladder from the street to the first landing
  - the zig-zag stairs up every floor
  - a short ladder over the parapet onto the roof
  They are low roofs with a view and the rooftop links.
- **The radio (the goal):**
  - On the tallest landmark, a ladder runs up the mast to the platform 18 m above the roof. The radio console stands at the radio hut.
  - Using it ([E]) once shows a radio message from `data/radio.json` (static, then a voice: survivors are holding out on the rooftops; this sets up R44). It opens one **top loot crate** (`metro_radio` table) on the platform and marks the radio as used in the save.
  - The big map shows "Radio Mast" on the tallest tower, and a small "Stairs" label on each climbable tower.
- No elevators, no rope swinging, no fall-damage changes. No sounds. Detail first; flat palette; generic words only ("ROOF", "EXIT", "STAIRS", floor numbers).
- **FPS:** `metro` scene only, average at least 20; the 1% low is for information.

## R43 shared names (every task uses EXACTLY these)
- **Data keys in `data/metro.json`:**
  - `climb` (R43-01)
  - `rooftops` and `fireEscapes` (R43-02)
- **Other data files:**
  - `data/climb_floors.json` (R43-04: crates per interior model)
  - `data/radio.json` (R43-07)
- **Shaft frame** (tower-local metres, front = local -Z):
  - Shaft centre at `(0, 0.25*size)`, right where `lobby_elevators` stands; climbable towers drop that prop.
  - Outer size 4.4 m (x) x 6.4 m (z), walls 0.2 m.
  - This fits inside every setback (the smallest landmark module is 24 m).
- **Stair module** (one per storey, 3.8 m tall; origin = shaft centre on the module's floor landing, y = 0 is the landing top):
  - **floor landing:** z -3.0..-1.6, full width
  - **flight A:** x -2.0..-0.1, rises z -1.6 -> +1.6, y 0 -> 1.9
  - **mid landing:** z +1.6..+3.0, y 1.9
  - **flight B:** x +0.1..+2.0, rises z +1.6 -> -1.6, y 1.9 -> 3.8 (it arrives on the next module's floor landing)
  - **door:** in the -Z wall, x -0.7..+0.7, 2.2 m high
  - Slopes are about 31 degrees: walkable. Each flight is ONE ramp collider (steps are visual only); every landing and wall has a collider.
- **Models (Blender):**
  - Stairs (R43-03):
    - `tw_stair_flight`: door jammed
    - `tw_stair_flight_b`: jammed door + debris, blood, a dropped bag
    - `tw_stair_flight_z`: as `_b` with `spawn_1`, `spawn_2` empties on the landings; a stairwell POI
    - `tw_stair_open`: open door, EXIT sign
    - `tw_stair_top`: the top floor: floor landing + open door + ceiling, no flights
  - Stair extras (R43-03):
    - `stair_digit_0`..`stair_digit_9`: painted floor numbers; tiny, no collider, no shadow
    - `stair_sign_roof`: the lobby "ROOF" arrow sign
    - `roof_ladder_seg`: one 1.0 m ladder segment
    - `roof_hatch`: an open hatch frame on the deck, with a dark opening
  - Interiors, one storey each (R43-04): `tw_interior_{office,flat,camp}_{24,32}`.
    - Origin = tower centre on the floor surface (y = 0). They fill the footprint minus the shaft rect, with the corridor reaching the shaft door.
    - They keep a 2 x 2 m clear spot at the **hatch**, local `(3.4, 0.25*size)` (east of the shaft).
    - Each has `spawn_1..spawn_N` empties (N 4-7) and inner walls + furniture colliders.
  - Rooftop kit (R43-05):
    - `roof_plank_4`, `roof_plank_8`, `roof_plank_12`: plank bridges 1.2 m wide, long along local Z, a rope rail on one side
    - `roof_step_crate`
    - `radio_console`
    - colliders + ladder empties for the brick fire escapes (in the brick tower scripts; visuals unchanged)
- **Pure world API** (`world/src/metro/`):
  - R43-01, `climb.rs`:
    - `pub struct ClimbTower { pub tower: usize, pub kind: ClimbKind, pub stop_floors: Vec<u32>, pub stop_kinds: Vec<StopKind>, pub deck_y: f32 }` with `ClimbKind { Midtown, Core, Landmark }`, `StopKind { Office, Flat, Camp }`
    - `pub fn climb_towers(metro: &MetroWorld) -> Vec<ClimbTower>` (memoised like `places::place_sites_cached`; do NOT add fields to `MetroWorld`, because tests build it literally)
    - `pub fn roof_deck_y(t: &TowerSpot, style: &StyleDef, ground_y: f32) -> f32`
    - `pub fn climb_hull_boxes(t: &TowerSpot, style: &StyleDef, plan: &ClimbTower) -> Vec<[Vec3; 2]>`: tower-local min/max boxes, same frame and origin as `hulls.rs` `box_points`
    - `pub fn climb_in_chunk(metro: &MetroWorld, chunk: (i32, i32), height: &dyn Fn(f32, f32) -> f32) -> ClimbChunk`
    - `pub struct ClimbChunk { pub pois: Vec<PoiInstance>, pub props: Vec<PropInstance>, pub crates: Vec<LootSpot>, pub ladders: Vec<ClimbLadder>, pub boxes: Vec<ClimbBox> }`
    - `pub struct ClimbLadder { pub foot: Vec3, pub deck: Vec3, pub id: String }`
    - `pub struct ClimbBox { pub centre: Vec3, pub yaw: f32, pub half: Vec3 }`: extra world colliders, e.g. the mast platform
  - R43-02, `rooftops.rs` / `fire_escapes.rs`:
    - `pub fn roof_links(metro) -> Vec<RoofLink>`
    - `pub fn rooftops_in_chunk(metro, chunk) -> ClimbChunk` (the same struct)
    - `pub fn fire_escape_towers(metro) -> Vec<usize>`
    - `pub fn fire_escape_ladders(metro, idx) -> Vec<ClimbLadder>`
  - R43-06: `PoiBuildingDef` gains `#[serde(default)] wake_height_m: f32` (`wakeHeightM`; 0 = off, as today).
  - R43-07: `game_world/src/radio.rs`: `RadioState { used: bool }`, `RadioConfig` (from `data/radio.json`), `pub fn radio_prompt(...)`, `pub fn use_radio(...) -> Option<RadioOutcome>`. Saved in `systems.extra["metro_radio"]`.
- **Flags** (R43-09):
  - `--metro=climb`: the lobby, facing the open stair door
  - `--metro=climb_stairs`: inside the shaft, mid-tower, looking up flight A
  - `--metro=climb_floor`: on a stop-off floor
  - `--metro=climb_roof`: on the roof of a climbable tower, looking over the city (THE rooftop view shot)
  - `--metro=roof_planks`: on a roof, looking along a plank bridge
  - `--metro=fire_escape`: the street in front of a working fire escape
  - `--metro=radio`: on the mast platform, at the radio

## Tests that must stay green
`world/tests/golden.rs` passes UNEDITED. R38-R42 metro tests still pass. Towers that are not climbable keep exactly today's parts and hull. Spawning outside New Halden is unchanged. Old saves load (no `metro_radio` = not used).
