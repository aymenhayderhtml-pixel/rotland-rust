# R41-02: Herds walking down the avenues

Owns:
- `world/src/metro/herds.rs` (NEW, pure routes) and `world/src/metro/herds_tests.rs` (NEW)
- `world/src/metro/config.rs` (ONLY the new optional `herds: HerdsConfig` field, `#[serde(default)]`)
- `data/metro.json` (ONLY the new top-level key `herds`)
- `game_zombie/src/street_herds.rs` (NEW, runtime state, pure) and `game/tests/all/street_herds.rs` (NEW)

Lines for the lead:
- `pub mod herds;` and `#[cfg(test)] mod herds_tests;` in `world/src/metro/mod.rs`, plus any re-export in `world/src/lib.rs`.
- `mod street_herds;` + `pub use street_herds::*;` in `game_zombie/src/lib.rs`.
- `mod street_herds;` in `game/tests/all/main.rs`.

Not owned: `rotland/*` (R41-09 wires it), `game_zombie/src/horde/*` (read only; call its public API).
Read: tasks/R41-common.md, `world/src/metro/layout.rs` (`MetroLayout`, `MetroStreet`, block grid, avenues vs streets), `world/src/metro/damage.rs` (`DamageSites`: blocked street segment of the fallen tower), `world/src/metro/traces.rs` (car walls: which street ends are walled), `game_zombie/src/horde/` (whole), `game_zombie/src/zombie/` (how a zombie notices the player: senses), `data/metro.json`.

## Data (`data/metro.json` -> `herds`)
```json
"herds": {
  "_why": "R41: slow herds along the avenues; a herd only becomes zombies near the player.",
  "count": { "edge": 2, "midtown": 4, "core": 4 },
  "size": [12, 30],
  "speed": 0.55,
  "routeLegs": [4, 9],
  "activeRadius": 170,
  "despawnRadius": 210,
  "spreadM": 7.0,
  "aggroShare": 0.6
}
```

## Pure routes (`world/src/metro/herds.rs`)
- `HerdsConfig` (serde, the keys above, sensible `Default`).
- `herd_routes(layout, cfg, seed)`: for each district, place `count` herds. Each route is a CLOSED loop of street-centre points that walks along avenues (prefer the long avenues; streets are fine for the turns), turning only at crossings, with `routeLegs` legs.
  - Never enter a reserved block.
  - Never go through the fallen-tower blocked segment or a car-wall street end.
  - Never use the highway entrance avenue's first block.
  - Deterministic per seed.
- `route_pos(route, time_s)`: position at `distance = (time_s * speed) mod loop_length` along the polyline, and yaw facing the walking direction (Godot yaw convention: forward = `(-sin yaw, -cos yaw)`).
- Tests (`herds_tests.rs`):
  - Same seed gives the same routes; another seed gives different ones.
  - The counts per district match the data.
  - Every point lies on a street (`layout.street_at(p).is_some()`), and no point is in a reserved block or the blocked fallen segment.
  - Each route is a closed loop: the last point joins the first along a street.
  - `route_pos` moves exactly `speed * dt` along the path, and the yaw follows the direction.

## Runtime (`game_zombie/src/street_herds.rs`, pure, no physics)
- `StreetHerds::new(routes: Vec<HerdRoute>, cfg: HerdsConfig)`.
- `update(&mut self, world_time: f64, player: Vec3, alive: &dyn Fn(u32) -> usize, ground: &dyn Fn(f32, f32) -> f32) -> StreetHerdOrders`. Here `alive(route)` tells how many zombies of that herd are still alive.
  - When the player comes within `activeRadius` of a dormant herd's `route_pos`, emit `spawn` with `size` positions. They are spread in a loose blob, `spreadM` wide, behind the leader along the route, with y from `ground`.
  - When the player is beyond `despawnRadius` and the herd is not chasing, emit `despawn_routes`. The herd goes back to dormant.
  - A herd whose zombies were all killed stays gone until the next in-game day (it remembers the day number).
  - `pub fn leader_target(&self, route: u32, world_time: f64) -> Vec3`: the next point a walking herd heads for (R41-09 sets it on the `Horde` each tick).
  - `pub fn on_noticed(&mut self, route: u32)`: once any member notices the player, the herd is "chasing". When `aggroShare` of the members chase, the whole herd chases. R41-09 calls this from the zombie notice event.
- Size per herd comes from the route (`size`, rolled in `herd_routes` from `cfg.size`).
- Save: `to_save_state()` / `from_save_state()` with only the per-route "killed on day N" list (`serde`). R41-09 stores it in the save's `extra` map.
- Tests (`game/tests/all/street_herds.rs`):
  - A player far away gives no spawn. Walking in gives one spawn of `size` positions. Walking out gives a despawn.
  - The positions are within `spreadM` of the route.
  - A killed herd does not respawn the same day but does the next.
  - The save round-trips.

## Done when
Code + tests written (the lead builds and tests). Fill `## Worker notes`.

## Worker notes
