# R29-01: knocked down before death (downed, crawl, revive)

Owns: game/src/downed.rs, data/downed.json, game/tests/all/downed.rs, rotland/src/combat.rs, rotland/src/downed_play.rs, rotland/src/app/frame_actors.rs, rotland/src/app/frame_team.rs, rotland/src/hud/* (only a new small file for the downed bar + the line that draws it)
Rules: `tasks/R29-common.md`. Owner decision: `docs/DECISIONS.md` row 2026-11-25.

**Goal:** when the player's health reaches 0, they are knocked down instead of dying at once.
- **Downed:** the player falls and plays the existing `downed_crawl` clip (made in R27, polished in R28). They can only crawl slowly. No weapons, no shooting, no blocking, no sprint, no jump, no car.
- **Bleed-out:** a timer (30 s) counts down. Each zombie bite while downed takes 4 s off it. At 0 the normal death runs (use the existing death path in `rotland/src/combat.rs` `take_bites`: `dead_for`, death animation, respawn).
- **Revive by a teammate:** a recruit with order Follow within 12 m walks to the player and kneels for 4 s (use the existing `use`/bandage-type clip on the teammate if one exists; otherwise just stand still). Then the player stands up with 25 health.
- **Self revive:** hold E for 6 s with at least 1 bandage in the bag. It uses up 1 bandage, and the player stands up with 25 health.
- **Downed again within 60 s of getting up = real death.** Drowning and falling into deep water still kill at once (no downed).
- **HUD:** while downed, show a centered bar with "DOWNED 23 s" and the hint "[Hold E] Bandage" (only if you have one) or "Waiting for a teammate…".
- Zombies keep attacking a downed player (that's what shortens the timer).

**Parts:**
- A (pure logic, start first): `game/src/downed.rs`: a `Downed` state machine (`Up` / `Down { bleed, revive_progress, by }` / `Dead`) with `on_lethal_hit(now)`, `bite()`, `tick(dt, reviving_teammate_in_range, self_revive_held, has_bandage) -> DownedEvent` (None / Revived / BledOut / BandageUsed), and the 60 s rule. Numbers in `data/downed.json` (bleedOutSeconds 30, biteSeconds 4, teammateRange 12, reviveSeconds 4, selfReviveSeconds 6, reviveHealth 25, crawlSpeedMul 0.35, reDownWindowSeconds 60). Tests in `game/tests/all/downed.rs` cover every event and the 60 s rule.
- B (wiring, after A): `rotland/src/downed_play.rs` glue. In `combat.rs` route the lethal hit through `Downed` (not for drowning). In `frame_actors.rs` block the inputs and slow the movement while downed, and play `downed_crawl` (full body). In `frame_team.rs` make one Follow teammate walk over and revive. Add the HUD bar.
- Save: a downed player who quits loads as up with 25 health. Don't add save fields.
