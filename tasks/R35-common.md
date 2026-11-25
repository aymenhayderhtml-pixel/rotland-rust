# R35 common rules: ONE lead + subagents, ONE build (2026-11-25)

Why: in earlier rounds every worker built and tested on its own, so the slow laptop built the game again and again. This round has **one lead** (you, the Antigravity chat the owner pastes into). The lead hands out work to **subagents that only write code**. Then the lead builds and tests **once**.

## Setup (lead, once)
```bash
cd /home/aymen/Documents/rotland-rust
git worktree add ../rotland-r35 -b r35 main
cd ../rotland-r35
export CARGO_TARGET_DIR=/home/aymen/Documents/rotland-rust/target   # reuse the built crates; never build from nothing
```
Everyone (lead and subagents) works in `/home/aymen/Documents/rotland-r35`, on branch `r35`. **Do not give subagents their own copies of the folder.** They edit different files in the same folder, so there is nothing to merge.

## Subagent rules (copy these into every subagent's instructions)
- Read `AGENTS.md`, `tasks/R35-common.md`, then your task file. Open only the files your task needs (`docs/CODEMAP.md` tells you where things are).
- Edit ONLY the files your task file says you own. **Shared files** (`rotland/src/app/state.rs`, `rotland/src/app/mod.rs`, `rotland/src/main.rs`, `game/src/lib.rs` (the facade that re-exports everything), `game_player/src/lib.rs`, `game_world/src/lib.rs`, `game/tests/all/main.rs`) belong to the lead. Put the exact lines you need there in your report, and the lead adds them.
- **Never run `cargo`, `tools/gate.sh`, `blender` or the game.** Write careful code instead: match the real function names and types you read in the repo, and don't guess an API. Read the function before you call it.
- Port, don't invent: new numbers go in the task's `data/*.json` file, and every number in Rust code gets a tag `// data: <file> <key>` or `// new: DECISIONS 2026-11-25`.
- Files under ~300 lines. Pure logic goes in the `game_*` crates (since R30.5: `game_core` = items/weapons/damage, `game_player` = player/guns, `game_world` = projectiles/guns_fire/melee, `game_zombie`, `game_camp`; see `docs/CODEMAP.md`). A new module needs a `pub mod x;` line in its crate's `lib.rs` AND a re-export in `game/src/lib.rs` (put both in your report for the lead). Unit tests still go in `game/tests/all/<name>.rs` and use `game::...` paths.
- When done, fill `## Worker notes` in your task file (what changed, lines the lead must add to shared files, how to see it in the game, anything unsure) and report back to the lead. Don't commit.
- DO NOT touch `tools/blender/vehicles/*` or `assets/models/vehicles/*`. Never `pkill -f`. Never write to real save folders.

## Lead: build once, fix, finish
1. Start the subagents (all R35 tasks can run at the same time; each owns different files). Tasks that don't share files run at the same time. Inside one task, the pure-logic part (game/, data/, tests) can go first, then the wiring part (rotland/).
2. When all subagents have reported: add their shared-file lines yourself.
3. If any `tools/blender/**` file changed, rebuild those models ONCE (see the task file for the command). Check the log has no `Traceback`.
4. Build + test ONCE: `tools/gate.sh --task tasks/R35-round.md` in the FOREGROUND, and wait for it (it can take minutes; do not stop or end your turn while it runs). Read every error, fix it yourself (or send one subagent the exact error), and run the gate again. Loop until the last line is `GATE: PASS`.
5. Fill `## Lead notes` in `tasks/R35-round.md`: each task done or not, the last 5 lines of the final gate run, and anything Claude must check.
6. ONE commit on branch `r35` (`git add` only the files you changed, never `git add -A` or `commit -a`). Never push, never touch `main`. Then tell the owner "R35 done".

## R35 shared names (every subagent uses EXACTLY these, so the parts fit together)
- Weapon ids: `revolver`, `hunting_rifle`, `crossbow`, `grenade`. Ammo item ids: `revolver_ammo`, `crossbow_bolt` (the hunting rifle uses the existing `rifle_rounds`; a grenade is its own item).
- Model files: `assets/models/weapons/{revolver,hunting_rifle,crossbow,crossbow_bolt,grenade}.glb`.
- Explosion call (R35-05 makes it, R35-03 calls it): `crate::explosion_view::ExplosionView::spawn(&mut self, pos: glam::Vec3, radius: f32)`.
- Gun feel call (R35-01 makes it, the shooting code calls it): `crate::gun_feel_view::GunFeelView::on_shot(&mut self, weapon_id: &str)`.
- New animation clips (R35-04): `grenade_throw` (player, upper body), `crossbow_reload` (player, upper body).
- R30 may have split big files (`frame_sim`, `frame_actors`, `frame_team`, `combat`, `hud`). Find code with grep, and don't trust old line numbers.
