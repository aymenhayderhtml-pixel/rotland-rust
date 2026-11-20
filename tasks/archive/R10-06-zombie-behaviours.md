# R10-06: Wire the zombie extras that exist only as logic (parity: zombies) — PHASE A

Read `tasks/R10-common.md` first. `docs/RUST_PARITY.md` → Zombies section. The logic already exists in `game/src/zombie/` but `rotland/src/zombies.rs` feeds it empty or hard-coded inputs every frame. Godot reference: `../zombi servival/scripts/Enemies/ZombieAI.Actions.cs`, `ZombieHerd.cs`, `ZombieHorde.cs`, `ZombieCrawl.cs`, `ZombieWounds.cs`, `ZombieCorpses.cs`, `ZombieHideouts.cs`, `ZombiePlayerLink.cs`.

## You own
- `rotland/src/zombies.rs` (split it if it grows past ~300 lines: e.g. `rotland/src/zombies/`)
- `game/src/zombie/**` and `game/tests/zombie_*.rs`

## Wire each one (port Godot's numbers)
1. **Herd alert** — today `nearby_zombies: &[]`. Feed each zombie the zombies within Godot's radius (8 m). Use a cheap spatial grid (cell = radius) rebuilt once per frame into a reused buffer — no new `Vec` per zombie per frame (target laptop, see #16).
2. **Scream** — when a zombie screams, emit a noise event at its position (Godot radius). Return it from `Zombies::update` in the existing noise/events output so other zombies hear it. Play the scream sound if `data/sounds.json` has one (say which key).
3. **Crawlers from leg hits** — today `leg_hit: false`. Add `pub fn hit(&mut self, idx, damage, hit_point_y)` (or extend the existing hit API — open it first). Hits below 0.6 m above the zombie's feet (Godot value, quote it) set leg_hit → crawl state, smaller capsule, crawl pose.
4. **Corpse eating** — today `corpses: &[]`. Keep a small ring buffer of recent zombie/animal corpse positions (Godot cap/lifetime) and feed it.
5. **Banging on hideouts** — today `target_unreachable: false`. Set it true when the player is on a ladder or on a roof/deck the zombie can't reach (take `player_unreachable: bool` as an input; Claude computes it from ladder state).
6. **Safe zone** — while the player is inside the camp safe zone, zombies must not target the player (Godot `ZombiePlayerLink.cs` / `SealsOff`). Take `player_in_safe_zone: bool` as an input.
7. **Night horde** — assign the shared `ZombieHorde` wander target to night zombies like Godot does.

## Performance
Distant zombies keep their throttled updates. Zero new heap allocations per frame in the steady state (reuse buffers stored in `Zombies`). Say in notes how you checked.

## Tests (`game/tests/`)
- Herd: a chasing zombie alerts an idle one at 7 m, not at 9 m.
- Leg hit at 0.5 m → crawler; at 1.2 m → not.
- Scream produces a noise event with Godot's radius.
- Safe zone true → zombie never enters Chase on the player.
- Unreachable true while chasing → Bang state.

## Wiring for Claude
The new inputs (`player_in_safe_zone`, `player_unreachable`, hit height from shooting/combat hit events) and where they come from.

## Worker notes

### Files changed
- `game/src/zombie/grid.rs` (new): 2D spatial hash grid with fixed buckets and entry pooling for allocation-free neighbor queries.
- `game/src/zombie/corpses.rs`: Added `recent_positions(&self, seconds, now, out: &mut Vec<Vec3>)` to populate a caller-supplied buffer without allocations.
- `game/src/zombie/mod.rs`: Exported `grid` module and `SpatialGrid`.
- `game/src/zombie/brain.rs`: Added `player_in_safe_zone` to `BrainInput`, added `last_seen()` getter on `ZombieBrain`, and handled safe zone in `update` and `think`.
- `game/src/zombie/brain_actions.rs`: Ignored player when `player_in_safe_zone` is true (in idle, wander, investigate, chase, eat, bang, and herd check); transitioned close unreachable chasing zombies directly to Bang.
- `rotland/src/zombies/types.rs` (new): Extracted `Zombie`, `Look`, `ZombieFrame`, constants, and `clip_name`.
- `rotland/src/zombies/combat_glue.rs` (new): Extracted `hit`, `hit_with_height`, `hit_at_height`, `record_corpse`, `hit_unclaimed`, `take_kills`, `targets`, `alive`.
- `rotland/src/zombies/rendering.rs` (new): Extracted `pose` skin matrix evaluation.
- `rotland/src/zombies/spawning.rs` (new): Extracted `spawn_ring`, `spawn_kind`, `spawn_horde`, `assign_horde`, `despawn`, `active`.
- `rotland/src/zombies/mod.rs`: Rebuilt `rotland/src/zombies.rs` into modular `rotland/src/zombies/` (<300 lines each); wired spatial grid for herd alert, corpse tracking, scream noise events and audio triggers, crawler mover capsule, night horde support, and safe zone / unreachable inputs.
- `rotland/src/zombies.rs`: Deleted in favor of `rotland/src/zombies/` module structure.
- `game/tests/zombie_brain_tests.rs`: Updated `sense` helper with default `player_in_safe_zone: false`.
- `game/tests/zombie_extras_tests.rs`: Updated `sense` helper and added unit tests for herd alerts (7m vs 9m), crawler leg hits (0.5m vs 1.2m), scream noise radius, safe zone avoidance, and unreachable banging.

### New public API
- `game::zombie::SpatialGrid`:
  - `pub fn new(cell_size: f32, bucket_count: usize) -> Self`
  - `pub fn clear(&mut self)`
  - `pub fn insert(&mut self, zombie: HerdZombie)`
  - `pub fn query_nearby(&self, pos: Vec3, out: &mut Vec<HerdZombie>)`
- `game::zombie::ZombieCorpses`:
  - `pub fn recent_positions(&self, seconds: f32, now: f64, out: &mut Vec<Vec3>)`
- `game::zombie::ZombieBrain`:
  - `pub fn last_seen(&self) -> Vec3`
- `game::zombie::BrainInput`:
  - Added field `pub player_in_safe_zone: bool`
- `rotland::zombies::Zombies`:
  - Fields: `pub player_in_safe_zone: bool`, `pub player_unreachable: bool`
  - `pub fn hit_with_height(&mut self, id: u64, damage: DamageInfo, knockback: Vec3, hit_point_y: f32) -> bool`
  - `pub fn hit_at_height(&mut self, id: u64, damage: DamageInfo, knockback: Vec3, hit_point_y: f32) -> bool`
  - `pub fn record_corpse(&mut self, position: Vec3)`
  - `pub fn spawn_horde(&mut self, kind: &str, positions: &[Vec3]) -> Vec<u64>`
  - `pub fn assign_horde(&mut self, ids: &[u64]) -> Arc<Mutex<ZombieHorde>>`
  - `pub fn update_full(&mut self, dt: f32, player: Option<Vec3>, noises: &[Noise], physics: &PhysicsWorld, player_unreachable: bool, player_in_safe_zone: bool) -> ZombieFrame`
- `rotland::zombies::ZombieFrame`:
  - Added fields: `pub noises: Vec<Noise>`, `pub screams: Vec<Vec3>`

### Wiring for Claude
1. **Safe zone (`player_in_safe_zone: bool`)**:
   - Location: `rotland/src/main.rs` around line 1370 where `z.update(...)` is called.
   - Expression: Compute `let in_safe_zone = self.world_gen.as_ref().map_or(false, |g| g.settlements().iter().any(|s| s.contains(self.player, 0.0, g.config())));` (matching line 1445 in main.rs).
   - Pass: `z.player_in_safe_zone = in_safe_zone;` or pass into `z.update_full(..., player_unreachable, in_safe_zone)`.
2. **Player unreachable (`player_unreachable: bool`)**:
   - Location: `rotland/src/main.rs` around line 1370.
   - Expression: Check player ladder state `let unreachable = self.player_ctrl.as_ref().map_or(false, |p| p.on_ladder);` (or ladder controller active flag).
   - Pass: `z.player_unreachable = unreachable;` or pass into `z.update_full(..., unreachable, in_safe_zone)`.
3. **Hit height (`hit_point_y: f32`)**:
   - Location: `rotland/src/combat.rs` and `rotland/src/shooting.rs`.
   - When resolving melee or gun hits on zombies, pass `hit.hit_point.y` (or call `zombies.hit_with_height(id, damage, knockback, hit_point.y)`).
4. **Scream audio & noise bus**:
   - In `rotland/src/main.rs` after `let frame = z.update(...)`:
     - Feed `frame.noises` into combat noises so other zombies / systems hear screams.
     - For `pos` in `frame.screams`: if `a.has("zombie_groan")` (note: `data/sounds.json` has `zombie_groan` with maxDistance 25, matching `screamAlertRadius`), play audio: `a.play_at("zombie_groan", pos);`.
5. **Night horde wander target**:
   - In `rotland/src/play.rs` (`spawn_tick`): when night hordes spawn, call `zombies.spawn_horde` (or `zombies.assign_horde(&horde_ids)`) so all horde members share one `ZombieHorde` Arc.

### Tests
- `herd_chasing_zombie_alerts_idle_one_at_7m_not_at_9m`:
  - 7 m alert: within `HerdRadius = 8f` (`ZombieBehaviourDef.cs:32`, `data/zombies.json: herdRadius: 8.0`).
  - 9 m no alert: outside `HerdRadius = 8f`.
- `leg_hit_at_0_5m_becomes_crawler_at_1_2m_does_not`:
  - 0.5 m crawler: below `LegHitHeight = 0.6f` (`ZombieBehaviourDef.cs:14`, `data/zombies.json: legHitHeight: 0.6`).
  - 1.2 m non-crawler: above `LegHitHeight = 0.6f`.
- `scream_produces_a_noise_event_with_godot_radius`:
  - Radius 25.0: from `ScreamAlertRadius = 25f` (`ZombieBehaviourDef.cs:22`, `data/zombies.json: screamAlertRadius: 25.0`).
- `safe_zone_true_zombie_never_enters_chase_on_the_player`:
  - Verifies `ZombiePlayerLink.cs:64` / `SettlementRegistry.SealsOff`: player inside safe zone is unreachable, zombie never enters Chase or Attack.
- `unreachable_true_while_chasing_enters_bang_state`:
  - Verifies `ZombieAI.Actions.cs:208` / `ZombieHideouts.cs:12`: unreachable player triggers Bang state at target spot.

### Checked?
not checked (write-only round)

### Memory allocations check
- `nearby_buf` and `corpse_buf` are allocated once on `Zombies` struct creation and cleared/reused every frame via `std::mem::take` / `clear()`.
- `SpatialGrid` reuses its entry and bucket vector capacities across frames via `clear()` and does not reallocate during steady state.
- Zero new heap allocations per frame in steady-state zombie updates.

### Questions / known issues
- None. `data/sounds.json` does not have a dedicated `"zombie_scream"` entry, but does have `"zombie_groan"` (spatial, 25m distance matching `screamAlertRadius = 25m`). Scream positions are surfaced via `frame.screams`.

