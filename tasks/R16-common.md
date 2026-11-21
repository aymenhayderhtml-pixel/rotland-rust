# R16 common rules (0.2 push, day 1 morning: foundations)

Plan: `docs/PLAN_0.2_PUSH.md`. Owner decisions: `docs/DECISIONS.md` rows dated 2026-11-21. Rules: `AGENTS.md`, and `tasks/R13-common.md` ("Everyone" and "Rust/data lanes" parts apply to you).

- This round builds **foundations**: pure logic + data + tests. No wiring into `rotland/` unless your task says so (Claude wires on day 2). Make the public API small and clear, and describe it in Worker notes ("Wiring notes for Claude": the exact calls the game loop should make).
- **Unused check:** the gate fails on a new `pub` item that nothing uses. Your tests count as uses, so test every public item.
- **Numbers:** every number in `game/src`, `world/src` code goes in your JSON file and gets `// data: <file>.json <key>`, or is a math exemption. Owner-decided numbers may use `// new: DECISIONS 2026-11-21`.
- **Save:** if your system has state that must survive a save, give it one `#[derive(Serialize, Deserialize, Default)]` state struct with `#[serde(default)]` on every field, plus a test that a JSON round-trip keeps it and that `{}` loads as the default. Don't touch the `save` crate (Claude hooks it in).
- **Shared files:** only the files on your `Files:` line. `data/items.json` belongs to R16-03 this round. Don't edit `Files:` (the gate reads it from main).
- Wait for every command to finish before ending your turn. Commit once when the gate passes.
