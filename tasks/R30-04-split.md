# R30-04: split the biggest files (PHASE 2: start only after R30-01..03 are all in)

Owns: rotland/src/app/frame_sim*.rs, rotland/src/app/frame_actors*.rs, rotland/src/app/frame_team*.rs, rotland/src/combat*.rs, rotland/src/hud/*, rotland/src/app/mod.rs, rotland/src/main.rs
Rules: `tasks/R30-common.md`.

**Goal:** these files grew far past the ~300-line rule, and every AI now has to read 600-700 lines to change one thing:
`rotland/src/app/frame_sim.rs` (~720), `rotland/src/hud/mod.rs` (~700), `rotland/src/app/frame_actors.rs` (~670), `rotland/src/combat.rs` (~660), `rotland/src/app/frame_team.rs` (~610).
Split each into topic files of under ~350 lines (for example `frame_sim_zombies.rs`, `frame_sim_team.rs`, `combat_downed.rs`, `hud/bars.rs`). Move tests with their code.

**Rules:** move code only. **No behaviour change**: same functions, same order of calls each frame, same numbers. Keep public function names (callers keep working, or update the callers in the files you own). You can use one subagent per file at the same time, since each file is separate. `rotland/src/app/mod.rs` and `main.rs` `mod` lines are added by the lead.
