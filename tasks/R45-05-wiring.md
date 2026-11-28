# R45-05: Wiring (park and stadium into the world, the horde's real zombies, danger, stats)

Runs AFTER R45-01, 02, 03, 04 are merged (the lead merges them first and runs one build). Read first: `tasks/R45-common.md` (whole) and the Worker notes of R45-01..04.

Owns:
- `world/src/generator_contents.rs` (ONLY: a `metro_park_stadium_extras(coord)` method or two calls next to the places calls at ~line 143-160 and the crates at ~373-385: `park_in_chunk`, `stadium_in_chunk`. Props and crates only, no POIs).
- `rotland/src/stadium_horde_play.rs` (NEW): `StadiumHordeRuntime` (the real zombies of the horde: spawn, steer, despawn; pure decisions come from R45-04).
- `rotland/src/play_metro.rs` (ONLY `metro_profile_at`: the park calm and the stadium as a place for the danger boost; nothing else).
- `rotland/src/app/frame_world.rs` (ONLY one `tick` call next to the `metro_herds.tick` call at ~line 675, and the `stadium_horde` tick with `play.hour`).
- `rotland/src/props/rules.rs` and `rotland/src/streaming_metro.rs` (ONLY the name-prefix rules: `park_pond`, `park_path`, `stadium_pitch` have no collider; `stadium_pitch` has no shadow; copy the `rope_ladder_` / `green_smoke_` lines from R44).
- `rotland/src/app/stats.rs` (ONLY the five new keys in `R45-common.md`).
- `rotland/src/play.rs` (ONLY the field `pub stadium_horde: crate::stadium_horde_play::StadiumHordeRuntime` next to `metro_herds`, and its `Default` line).

Not owned: the pure logic (R45-02/03/04), the flags and labels (R45-06), `world/src/metro/mod.rs`, `game_zombie/src/lib.rs`, `rotland/src/main.rs` (lead adds the `mod` lines from the Worker notes).

Read (in this order):
- `rotland/src/play_metro.rs` `MetroHerdRuntime` (`tick`, lines ~55-200: the spawn loop with `zombies.spawn_kind("walker", pos)`, `z.herd_route = Some(route)`, `room = hard_cap - zombies.alive()`, the despawn by ids). Copy this shape for the stadium members. Do NOT change the avenue herd code.
- `rotland/src/app/frame_world.rs` ~`:668-684` (where `metro_herds.tick` is called and the `night` flag: `!(6.0..20.0).contains(&play.hour)`).
- `rotland/src/app/start_world.rs:234` (`restore_save`) and `rotland/src/app/save_load.rs:52` (`to_save`): the stadium horde needs NO save (it is a pure function of the hour; the real zombies are not saved, like the herds' blob).
- `game_zombie/src/zombie/brain_extras.rs` (`enter(ZombieState::Idle)` ~line 106, `enter_wander(self.home)` ~line 93: how a zombie is set to idle or wander; `set_wander_speed`).
- `rotland/src/play/spawn.rs:41-113` (herd members, `herd_route.is_some()`, are excluded from director hordes: a stadium member tagged with `herd_route` is excluded too).
- `rotland/src/app/stats.rs` (the R44 block ~lines 218-233).
- The R45-02/03/04 Worker notes (the `mod` lines, the gate positions, the park and stadium site functions).

## What to build (in this order)
1. **Chunk contents.** `world/src/generator_contents.rs`: `park_in_chunk(metro, chunk, hfn)` props + `stadium_in_chunk(metro, chunk, hfn)` props, in the metro block (~line 143-160), and their crates in the crates block (~373-385). Each `PropInstance` goes to `props`; each crate `LootSpot` to `crates`.
2. **Danger.** `rotland/src/play_metro.rs` `metro_profile_at`: `in_park = park_site(layout).is_some_and(|s| park_contains(&s, p))`. Call `metro_profile_near_park(cfg, district.into(), night, in_park)` when `in_park`; otherwise the existing path (the place boost). The stadium joins the `near_place` check: `stadium_site(layout).is_some_and(|s| s.centre.distance_squared(p) <= r2)`. Keep the R42 army-line / place logic unchanged. Inside the park, the park calm wins over the place boost (the park is not a place).
3. **The stadium horde runtime** (`rotland/src/stadium_horde_play.rs`):
   - `StadiumHordeRuntime { cfg: StadiumHordeConfig (from_repo), members: Vec<(usize /*member*/, u64 /*zombie id*/)>, site: Option<StadiumSite> }` with `#[derive(Default)]` (site lazy-init from `world_gen.metro_layout()` and `stadium_site`).
   - `pub const STADIUM_ROUTE_ID: u32 = u32::MAX; // new: DECISIONS 2026-11-28 (a route id no avenue uses: keeps these zombies out of the director and the avenue herd code)`. Members get `z.herd_route = Some(STADIUM_ROUTE_ID)`.
   - `tick(&mut self, hour: f32, player: Vec3, hard_cap: usize, zombies: &mut Zombies, physics: &PhysicsWorld, world_gen: &WorldGenerator)`, called once per frame from `frame_world.rs` after `metro_herds.tick` (throttle to about 4 times a second if the cost shows: say so in Worker notes if you add it).
   - Each frame: `split_counts(hour, cfg)`. For each member index `i in 0..total`:
     - inside the pitch group (`i < split.inside`): the spot is a pitch point (seeded `DotnetRandom` per member index, inside `pitch_half` of the site, or a fixed grid of `total` points on the pitch). Its state: `ZombieState::Idle`.
     - outside (`i >= split.inside`): the spot is `site.gates[gate_for_member(i, 4)]` plus a spread of 6-10 m. Its state: `ZombieState::Wander` with `home` = the gate (`enter_wander`).
     - A member is made REAL only when `wants_real(dist(player, spot), cfg)` and `hard_cap - zombies.alive() > 0`. Real members are spawned with `zombies.spawn_kind("walker", pos)`. Do not spawn more than the room allows.
     - A real member is despawned when `may_despawn(dist, chasing, cfg)` (chasing = its state is `Chase` or `Attack`), or when its member index no longer belongs to the group it was spawned in and it is not chasing. Use `zombies.despawn(&ids)`.
     - Members that die stay dead (remove them from `members`); a dead member is never respawned the same day (the count of inside members drops only through real kills: keep it simple, say what you did).
   - Real members of the returning group (dawn) that are far from the player despawn; the ones near the player keep chasing or wandering. Say what you did in Worker notes.
4. **Stats** (`rotland/src/app/stats.rs`): add `stadiumHordeInside` and `stadiumHordeOutside` (from `split_counts(play.hour)`, always pure counts), `stadiumHordeReal` (the count of live members in `members`), `metroDensity` (the density of the profile at the player: `metro_profile_at(...).map(|p| p.density)`; `0.0` when not in the city), `inPark` (bool, `park_contains`). Keys exactly as in `R45-common.md`.
5. **Props rules** (`rotland/src/props/rules.rs`, `rotland/src/streaming_metro.rs`): `park_pond` and `park_path` and `stadium_pitch` get no collider; `stadium_pitch` gets no shadow. Copy the R44 lines for `rope_ladder_` and `green_smoke_`.
6. **State** (`rotland/src/play.rs`): `pub stadium_horde: crate::stadium_horde_play::StadiumHordeRuntime` in `PlayState` (copy the `metro_herds` line).

## Number and tag rules
No new Rust numbers except `STADIUM_ROUTE_ID`, the throttle (if any) and the spread (6-10 m; tag `// new: DECISIONS 2026-11-28`). Every other number comes from the data files via the R45-02/03/04 modules.

## Checks you do yourself (run once, under the flock)
- `flock /tmp/rotland-heavy.lock target/debug/rotland --metro=core --frames=60 --save-dir=/tmp/r45-saves-w05 --stats=/tmp/r45-saves-w05/s.json` runs without a panic, and the file has the five new keys.
- Not in this task: the screenshots and the horde counts in game (R45-06 and the lead's checks).

## Done when
Everything above is wired; the gate passes (`tools/gate.sh --task tasks/R45-05-wiring.md`). Fill `## Worker notes` with:
- Files changed.
- The `mod` lines for `rotland/src/main.rs` (`mod stadium_horde_play;`) and `rotland/src/play.rs` if needed.
- How the stadium members are spawned at night (the spread, the gate), how they walk back at dawn, and what happens to a real member that dies.
- Whether the stadium members are idle on the pitch by day (the state name you used).
- The throttle decision.
- Any premise from R45-common that did not hold.

## Worker notes
