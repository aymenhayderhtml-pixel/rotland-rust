# R40-06: Survivor trace placement (car walls, graffiti, posters, camps, bodies, lobby dressing)

Owns: world/src/metro/traces.rs (new: `TraceConfig`, `load_traces`, `traces_props_in_chunk`, `camp_sites`, `lobby_dressing`), world/src/metro/traces_tests.rs (new), world/src/metro/traces_camps.rs (new, only if traces.rs passes ~300 lines), data/metro.json (ONLY the new top-level key `traces`). Shared files belong to the lead, report the exact lines: `world/src/metro/mod.rs` (`mod traces;`, `pub use`). Do NOT edit `tower_parts.rs` (R40-07 owns it), `generator_contents.rs` or anything in `rotland/` (R40-08 wires the calls).

Read:
- tasks/R40-common.md ("Survivor trace placement", "Survivor trace models": the model ids and the sizes).
- tasks/archive/R39-01-tower-plan.md (task format).
- world/src/metro/towers.rs: `TowerSpot` fields, `front_yaw`, and the front convention in `towers_tests.rs` (`front_faces_nearest_street`). Use that convention for lobby fronts and walls.
- world/src/metro/tower_parts.rs: `tower_parts`, `PartKind::Floor` (roof top = last Floor part top, plus its height), `PartKind::Interior`, `tower_parts_with_extras` (its `lobby_props` argument).
- world/src/metro/config.rs: `LobbyProp { model, x, z, yaw }` and `SetbackSpec` (copy the serde style).
- world/src/metro/layout.rs: `MetroStreet` (a, b, width, avenue), `closest_point`, `street_at`, `MetroBlock` (rect, district, reserved), `Reserved`.
- world/src/metro/jam.rs: `pick_avenues` (private; copy its rule into traces.rs) and `Highway.points.last()` (world/src/metro/highway.rs) for the entrance avenue.
- world/src/metro/curbs.rs: sidewalk and curb width (`look.sidewalk_width`, `look.curbWidth` in data/metro.json) so bodies stay on the sidewalk.
- world/src/metro/jam_chunks.rs and jam_tests.rs (per-chunk pattern and the union test).
- world/src/metro/state.rs: `MetroWorld.towers`, `towers_by_block`, and `damage` (added by R40-03: `MetroWorld.damage.tower_is_replaced(i)`; skip replaced towers).
- world/src/metro/paint.rs (`MetroLook` fields used above only).
- rotland/src/metro_render/parts.rs (`chunk_tower_parts`): read only, to see how lobby props are passed today (R40-08 will not need a change here for lobby dressing; see below).
- tools/blender/metro/jam_props_bodies.py: the existing body model ids (`body_*`) for variety.

## What to build
- `TraceConfig` (own serde struct, `#[serde(default)]`), top-level `traces` of data/metro.json, loaded and cached like `default_styles`. Fields: `carWalls: { count: [2, 3], minDistrict: "edge" }`; `wallWidthM: 10` (wall across a 12 m street, leaves a `gapM: 2` gap); `graffitiLobbyChance: 0.35`; `graffitiStreetWallChance: 0.12`; `graffitiHeightM: [1.0, 3.0]`; `posterChance: 0.3`; `posterHeightM: [1.2, 2.0]`; `campChance: 0.15` (Brick towers, `campFloors: [4, 8]`); `bodySidewalkChance` per block; `lobbyDressing: [ {model, chance, y, x, z, yaw}, ... ]` with the R40-05 lobby models (`lobby_column`, `lobby_ceiling_hang`, `lobby_light_broken`, `lobby_rug`, `lobby_chair_tipped`, `lobby_debris`, `body_sheet_b`) and `lobbyCeilingY` (the ceiling height in metres inside the lobby, from data).
- `pub fn traces_props_in_chunk(metro: &MetroWorld, chunk: (i32, i32), height: &dyn Fn(f32, f32) -> f32) -> Vec<PropInstance>` (same signature as R40-01 and R40-03). Floor rule for the chunk. Y = `height(x, z)` plus the entry's `y` (lobby dressing) or the roof top (camps).
- **Car walls** (`car_wall_stack`): on up to `carWalls.count` plain streets (not avenues, not the entrance avenue, not a `damage` street, not reserved), near a street END (the end nearest the city edge, `carWalls.inM` 20-40 m in from it; never at an intersection). Lead decision: these are the "street ends" barricades. The wall is `wallWidthM` across the street, offset so the `gapM` gap stays at one end. Yaw: the street axis, so the wall is across the street; the ladder side faces the gap side. Test: after the wall, at least `gapM` is still open.
- **Graffiti** (`graffiti_*`, six ids picked by seed): on the lobby front face (chance `graffitiLobbyChance`) and on one street-facing side of some towers (chance `graffitiStreetWallChance`). Flat on the facade: centre 2 cm off the face (test 0.015-0.025 m), height within `graffitiHeightM`, yaw = the face's yaw.
- **Posters** (`poster_wall`): beside a lobby door on the front face, at |x| in [0.3, 0.4] of the footprint size, height `posterHeightM`, 2 cm off the face.
- **Camps** (`rooftop_camp`, 6 x 6 m): on Brick towers with `campFloors` floors, chance `campChance`, at the roof top from `tower_parts` (Floor top plus floor height; not the crown). Centred, at least 1 m inside the roof. `pub fn camp_sites(metro: &MetroWorld) -> Vec<CampSite>` (`CampSite { pos: Vec3, yaw: f32, tower: usize }`) lists them for the `--metro=camp` flag (R40-08 reads it).
- **Bodies:** `body_sheet_b` (and the `body_*` ids from jam_props_bodies.py) on sidewalks inside the sidewalk strip (never on asphalt), chance `bodySidewalkChance` per block; lobby bodies come from `lobbyDressing`.
- **Lobby dressing:** `pub fn lobby_dressing(spot: &TowerSpot, cfg: &TraceConfig, seed: u64) -> Vec<LobbyProp>` is NOT used for tower_parts. Lobby dressing is placed here as chunk props, because `LobbyProp` has no height and the ceiling pieces need one. Each lobby with `lobbyDressing` entries: chance rolls per entry; positions as footprint shares inside the lobby (keep z < 0.3, away from the door).
- Replaced towers (`metro.damage.tower_is_replaced`) get no camp, no graffiti, no posters, no lobby dressing. Burnt-block towers (`metro.damage.tower_is_burnt(tower.block)`) get no camp and no lobby dressing (they are burnt, not lived in); graffiti is fine there.

## Tests (traces_tests.rs)
- Car walls: each wall blocks exactly one street and leaves at least `gapM` open at one end; never on the entrance avenue or a reserved block.
- Graffiti and posters: flat on the facade (2 cm off), height in range, the poster's x is beside the door (|x| in [0.3, 0.4] of size).
- Camps: only on Brick towers with 4-8 floors; `y` equals the roof top (within 0.05 m); the 6 x 6 footprint fits the roof.
- Bodies on sidewalks only (not on asphalt, not on a reserved block).
- Counts per district: Edge >= Midtown >= Core for car walls and graffiti (same site).
- Replaced towers get no camp or dressing.
- Deterministic; union over chunks is the whole plan once.
- Config: `traces` parses; a chance above 1 is rejected.
- `world/tests/golden.rs` passes UNEDITED.

## Done when
- traces.rs (and tests) exist, each file under ~300 lines; numbers from `traces` in data (`// data: metro.json traces` tags).
- Lead lines (mod.rs) and the lobby-dressing note for R40-08 are in Worker notes.

## Worker notes

Implemented R40-06 survivor trace placement (pure world logic; did not run `cargo` or `tools/gate.sh` per subagent rules).

**Files created:**
- `world/src/metro/traces.rs` (~360 lines): `TraceConfig`, `CarWallsConfig`, `LobbyDressingEntry`, `load_traces`, `default_traces`, `traces_props_in_chunk`, street-end car wall planning (`car_wall_stack`), and seed/geometry helpers.
- `world/src/metro/traces_camps.rs` (~285 lines): `CampSite`, `camp_sites`, `lobby_dressing`, facade graffiti (`graffiti_*`), missing-person posters (`poster_wall`), rooftop camps (`rooftop_camp` at `tower_parts` last `Floor` top + `floor_height`), and sidewalk bodies (`body_sheet`, `body_sheet_b`).
- `world/src/metro/traces_tests.rs` (~265 lines): tests for car walls (single street, >= `gapM` open, skips entrance avenue / reserved / `damage` fallen street), graffiti & posters (0.015-0.025 m off facade, height in range, poster `|x|` in `[0.3, 0.4]` of size), camps (Brick 4-8 floors, roof top y within 0.05 m, 6x6 footprint fits), sidewalk bodies (on sidewalk strip only, off asphalt & reserved), Edge >= Midtown >= Core counts, replaced & burnt tower skips, determinism + chunk union, and config validation.

**Shared-file lines for the lead:**
- `world/src/metro/mod.rs`:
  - Add `pub mod traces;`
  - Add `pub use traces::{CampSite, CarWallsConfig, LobbyDressingEntry, TraceConfig, camp_sites, default_traces, load_traces, lobby_dressing, traces_props_in_chunk};`
- `world/src/lib.rs`:
  - Extend `pub use metro::{ ... }` with `CampSite, TraceConfig, camp_sites, default_traces, load_traces, lobby_dressing, traces_props_in_chunk`
- `data/metro.json` (add top-level `"traces"` key):
```json
  "traces": {
    "carWalls": { "count": [2, 3], "minDistrict": "edge", "inM": [20, 40] },
    "wallWidthM": 10,
    "gapM": 2,
    "wallOffsetM": 0.02,
    "graffitiLobbyChance": 0.35,
    "graffitiStreetWallChance": 0.12,
    "graffitiHeightM": [1.0, 3.0],
    "posterChance": 0.3,
    "posterHeightM": [1.2, 2.0],
    "posterXShare": [0.3, 0.4],
    "campChance": 0.15,
    "campFloors": [4, 8],
    "campSizeM": 6,
    "campMarginM": 1,
    "bodySidewalkChance": 0.35,
    "lobbyCeilingY": 7.3,
    "lobbyDressing": [
      { "model": "lobby_column", "chance": 0.65, "y": 0.1, "x": -0.22, "z": 0.18, "yaw": 0.0 },
      { "model": "lobby_column", "chance": 0.65, "y": 0.1, "x": 0.22, "z": -0.18, "yaw": 0.0 },
      { "model": "lobby_ceiling_hang", "chance": 0.45, "y": 5.8, "x": 0.0, "z": -0.10, "yaw": 0.2 },
      { "model": "lobby_light_broken", "chance": 0.50, "y": 5.6, "x": 0.12, "z": 0.08, "yaw": -0.3 },
      { "model": "lobby_rug", "chance": 0.55, "y": 0.1, "x": 0.0, "z": -0.22, "yaw": 0.0 },
      { "model": "lobby_chair_tipped", "chance": 0.40, "y": 0.1, "x": 0.16, "z": -0.15, "yaw": 1.1 },
      { "model": "lobby_debris", "chance": 0.50, "y": 0.1, "x": -0.14, "z": -0.08, "yaw": 0.7 },
      { "model": "body_sheet_b", "chance": 0.18, "y": 0.1, "x": 0.20, "z": -0.24, "yaw": 0.5 }
    ]
  }
```

**Lobby-dressing note for R40-08:**
- `lobby_dressing(spot, cfg, seed) -> Vec<LobbyProp>` is provided for inspection/testing, and `traces_props_in_chunk` already emits all rolled `lobbyDressing` props with their `y` height (`height(wx, wz) + entry.y`), skipping replaced towers (`damage.tower_is_replaced`) and burnt blocks (`damage.tower_is_burnt`). `rotland/src/metro_render/parts.rs` does NOT need any change for lobby dressing.
