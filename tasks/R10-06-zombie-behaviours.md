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
