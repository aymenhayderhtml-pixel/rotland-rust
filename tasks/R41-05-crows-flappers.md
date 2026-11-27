# R41-05: Crows and flapping flags/plastic (runtime)

Owns:
- `game_world/src/crows/` (NEW: `mod.rs`, `flock.rs`, `tests.rs`, pure)
- `data/crows.json` (NEW)
- `rotland/src/metro_life.rs` (NEW, drawing + per-frame update)

Lines for the lead:
- `pub mod crows;` in `game_world/src/lib.rs` (+ facade re-export in `game/src/lib.rs` if the others do that)
- `mod metro_life;` in `rotland/src/main.rs`
- the call sites (R41-09 wires them; write exactly what to call, where and with what)

Not owned: `world/src/metro/life*.rs` (R41-03, read its `CrowPerch`/`Flapper` from the common names), models (R41-04).
Read: tasks/R41-common.md ("Crows + flappers runtime", the model names), `rotland/src/wildlife.rs` (how moving models are batched and drawn), `rotland/src/props.rs` (how a rigid model is looked up in the pack and drawn instanced), `game_world/src/animals/` (state machine + rng style), `engine/src/render/` only as needed to find the instanced draw call for rigid models.

## Data (`data/crows.json`)
```json
{
  "_why": "R41 silent life: crows startle when the player comes close.",
  "startleRadius": 12.0,
  "startleRadiusRunning": 20.0,
  "gunshotStartleRadius": 60.0,
  "takeoffSpeed": 4.0,
  "flySpeed": 9.0,
  "climbRate": 3.0,
  "goneHeight": 45.0,
  "returnAfter": [40.0, 90.0],
  "flapRate": 9.0,
  "glideShare": 0.35,
  "circlers": 6,
  "circleRadius": [40.0, 90.0],
  "circleHeight": [60.0, 110.0],
  "drawRadius": 90.0,
  "flapperRate": 2.2,
  "flapperDrawRadius": 70.0
}
```

## Pure logic (`game_world/src/crows/`)
- `CrowFlock::new(perch_pos, yaw, count, seed)`: crows sit at small offsets (within 1.2 m) around the perch. Each crow has its own phase and pecks now and then (perched <-> peck).
- `update(dt, player_pos, player_running, gunshot_at: Option<Vec3>)`:
  - `Perched` -> `Startled` when the player is inside the startle radius (bigger when running) or a gunshot is inside `gunshotStartleRadius`. Every crow takes off with a small random delay (0-0.4 s).
  - `Flying`: each crow climbs at `climbRate` and flies away from the player at `flySpeed`, with a little wobble.
  - `Gone` when above `goneHeight`.
  - `Returning` after a random `returnAfter` time, but only if the player is outside 2x the startle radius. The crows glide back down onto the perch, then go back to `Perched`.
- `instances(&self) -> impl Iterator<Item = CrowPose { pos: Vec3, yaw: f32, model: CrowModel /*Perched, Peck, Fly(0..4)*/ }>`. Flying crows glide (the f1 frame, held) for a `glideShare` of the time.
- Circlers: `Circlers::new(city_center, seed, cfg)` makes `circlers` crows that slowly circle high over the city, each with its own radius, height and speed. They always fly.
- `pub fn flap_frame(phase, time, rate) -> usize` for flags and plastic: `((time * rate + phase) * 4) mod 4`, so a smooth 4-frame loop.
- Tests (`tests.rs`):
  - The player far away keeps the crows perched. Walking within 12 m startles them. A gunshot at 50 m startles them, at 70 m it does not.
  - Crows rise and reach `Gone`, and return only when the player is away.
  - `flap_frame` cycles 0, 1, 2, 3, 0 in order.
  - Same seed gives the same poses.

## Drawing (`rotland/src/metro_life.rs`)
- Keeps one `CrowFlock` per `CrowPerch` within `drawRadius` of the player. Create them when they come in range, drop them when they leave. A flock that was startled stays "gone" until it returns, even if re-created: keep a small map perch-id -> return time.
- Each frame: update the flocks, then build instanced batches, one per model (`crow_perched`, `crow_peck`, `crow_fly_f0..f3`, `flag_torn_f0..f3`, `plastic_flap_f0..f3`).
  - Flappers within `flapperDrawRadius` use `flap_frame`.
  - Flags and plastic cast no shadow; crows cast none either (too small).
- Gets the perches and flappers from the `world` functions of R41-03 for loaded chunks. Cache them per chunk; don't re-run placement every frame.
- Must cost under 0.3 ms per frame with 150 crows + 80 flappers in view. Use plain Vec batches and no allocation per frame (reuse buffers).

## Done when
Code + tests written (the lead builds and tests). Fill `## Worker notes` with the exact wiring lines for R41-09.

## Worker notes
