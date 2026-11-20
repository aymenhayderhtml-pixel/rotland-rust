# R5-04: Camp life (settlement NPCs) — Rust port, logic only

Read `tasks/R5-common.md` first (same rules: write-only, game crate, real data, Godot keys, seeded randomness, private submodules + `pub use`).

Port to `rust/game/src/camp/` the logic (no rendering) of `scripts/Settlement/`:
CampLifeConfig/CampLifeData (`data/camp_life.json`), CampState, CampPlanner, CampJob, CampEvent, CampClock, CampNext, CampMath, CampResident, CampLifeDirector, CampChat (lines picked by seed), SettlementNpc (roles: trader, banker, bounty board keeper, guards...), SettlementGuard (guard behaviour as data: post, patrol points, alert), SettlementWatcher (save id, which settlements were visited), SettlementConfig/Data (`data/settlements.json`).
- The world crate already places settlements: `world::WorldGenerator::settlements()` (see `rust/world/src/` settlement code). Reuse its positions and layout; do not re-port SettlementBuilder/Placer/Site/Ground (they are already in `world`), and say which parts you found already there.
- Output per frame: for each resident an id, role, model name, position, yaw, an animation name and an optional chat line, so Claude can draw them with the humanoid animator. Driven by `update(dt, game_hour, player_pos)`.
- `nearest_interactable(player_pos, facing) -> Option<(resident_id, role, prompt_text)>` using Godot's interact range and prompt texts.
- Save: the Godot ids used by these scripts (check `Save()` code; e.g. "settlements").
Tests: residents planned for 3 hours of game time follow camp_life.json schedules (count jobs), chat picks are stable for a fixed seed, interact finds the trader in front of the player and nothing behind, save round trip.
