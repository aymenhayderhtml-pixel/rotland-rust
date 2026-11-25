# R32-07: biome zombies in play (models, toxic cloud view, chill, names, dev flag)

Owns: rotland/src/zombies/types.rs, rotland/src/zombies/specials.rs, rotland/src/zombies/spawning.rs, rotland/src/toxic_cloud_view.rs (new), rotland/src/app/frame_sim_zombies.rs (only the display-name match and the new event handling), rotland/src/app/test_flags.rs (only `--spawn-zombie=<kind>`), tools/README_TOOLS.md (one line)
Files: rotland/src/zombies/types.rs, rotland/src/zombies/specials.rs, rotland/src/zombies/spawning.rs, rotland/src/toxic_cloud_view.rs, rotland/src/app/frame_sim_zombies.rs
Read: tasks/R32-round.md, tasks/LEAD.md, rotland/src/zombies/types.rs, rotland/src/zombies/specials.rs, rotland/src/zombies/spawning.rs, rotland/src/zombies/mod.rs, rotland/src/app/frame_sim_zombies.rs, rotland/src/bubbles.rs
API: game_zombie/src/zombie/specials.rs, rotland/src/app/state.rs, rotland/src/app/test_flags.rs
Data: data/zombies.json, data/special_zombies.json
Overlap: `rotland/src/app/test_flags.rs` is owned by R32-03 and `tools/README_TOOLS.md` too. Do not write them: report the lines you need in those two files to the lead instead.
Rules: `tasks/LEAD.md` subagent rules. Use the "R32 shared names" in `tasks/R32-round.md` EXACTLY. R32-06 makes the logic and R32-08 the models at the same time: code against the shared names.

1. **Models:** map kinds `frozen`, `husk` and `bloater` to `zombie_frozen`, `zombie_husk` and `zombie_bloater` the same way `brute` maps to `zombie_brute` (grep `zombie_brute`). Make sure they use the zombie animation set (and for the husk, the crawler-like lunge clip, if specials pick clips by kind).
2. **Display names** (`frame_sim_zombies.rs`, next to `"screamer" => "Screamer"`): "Frozen", "Husk" and "Bloater".
3. **Events:**
   - `ToxicBurst` → spawn into the shared `ToxicClouds` (R32-06), and each frame apply `damage_at(player)` through the normal damage path (cause `"toxic"`) and the zombie dps to zombies in range.
   - `chill` on bite → `TemperaturePlay`'s `BodyTemp::chill(amount)` (R32-05 makes `TemperaturePlay`; report the exact call line for the lead, since the borrow may cross systems).
4. **`toxic_cloud_view.rs`:** draw each cloud as 6-10 slowly drifting, fading green puffs using the `toxic_puff` prop from R32-09 (`assets/models/props/toxic_puff.glb`), scaled up 1.5-3×. Copy how other effects draw props (grep `bubbles.rs` or `explosion`/`spawn` views). The puffs fade by scaling down over the last 1.5 s. Keep it cheap: at most 3 clouds at once, the oldest replaced.
5. **`--spawn-zombie=<kind>`:** at start, spawn one zombie of that kind 5 m in front of the player, idle for 3 s, so screenshots show it. It must work with `--biome=`.
6. Report every shared-file line you need (state fields, mod lines).

## Worker notes
