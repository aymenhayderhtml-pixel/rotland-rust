# R29-02: rescue survivors in the world (they join your team)

Owns: game/src/rescue.rs, game/src/rescue_place.rs, data/rescue.json, game/tests/all/rescue.rs, rotland/src/rescue_play.rs, rotland/src/rescue_view.rs, rotland/src/app/frame_world.rs (only the E-prompt / interact lines for survivors), game/src/guild/save.rs (one new field with `#[serde(default)]`), game/src/guild/recruits.rs (only a small "add a rescued recruit for free" function, if one doesn't exist)
Rules: `tasks/R29-common.md`. Owner decision: `docs/DECISIONS.md` row 2026-11-25.

**Goal:** survivors are trapped in some houses out in the world, with zombies around. Clear the zombies, walk up, press E, and they join your team for free and follow you.
- **Placement (pure, seeded):** from the world seed, choose some house/shop POIs (look at how `game/src/poi_zombies` picks POIs and copies their rules). Chance per POI is 0.15, at most 6 survivors in the world, and none within 300 m of camp. The same seed must always give the same places. Each survivor gets a name and role from the same lists the guild recruits use (`game/src/guild/roller.rs`), plus a stable id like `rescue_<n>`.
- **Zombies:** each survivor spot has at least 3 zombies near it (use the existing POI zombie spawning; only raise the count there if needed).
- **Model:** the `npc_survivor` model, standing with the idle clip, drawn only within normal NPC view distance (copy how `rotland/src/camp_npcs.rs` draws NPCs).
- **Rescue:** when no living zombie is within 12 m of the survivor and the player is within 2.5 m, show "[E] Rescue <name>". If the team is full (the existing guild team-size limit), show "Team full" instead. On E the survivor becomes a hired `GuildRecruit` (no cost) with order Follow, uses the existing teammate code from then on, and disappears from the spot. Show a short message: "<name> joined your team".
- **Save:** save the list of rescued ids (new `#[serde(default)]` field in the guild save), so a rescued survivor never shows up again.

**Parts:**
- A (pure logic, start first): `rescue.rs` (state + can-rescue check), `rescue_place.rs` (seeded placement), `data/rescue.json` (chancePerPoi 0.15, maxSurvivors 6, minDistFromCamp 300, clearRadius 12, talkRange 2.5, minZombiesNear 3), and tests in `game/tests/all/rescue.rs` (same seed gives same places, max 6, camp distance, clear/not clear, team full, rescued never returns).
- B (wiring, after A): `rescue_play.rs` (tick, prompt, E), `rescue_view.rs` (the model), the prompt hook in `frame_world.rs`, the save field.
