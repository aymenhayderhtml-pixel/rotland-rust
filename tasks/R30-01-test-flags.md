# R30-01: test flags so Claude can see downed, revive and rescue in screenshots

Owns: rotland/src/app/test_flags.rs (new), rotland/src/app/startup.rs, rotland/src/app/frame_actors.rs (only `check_scripted_timers`), tools/README_TOOLS.md
Rules: `tasks/R30-common.md`.

**Why:** `--die-at` calls `kill()`, which skips the new knocked-down state, so Claude can't take a picture of it. Add dev flags (all off by default, no change to normal play):
- `--down-at=<seconds>`: at that game time, deal a lethal hit through the SAME path a zombie bite uses (`Combat::take_damage_with_downed` with `actors.downed`), so the player goes down and crawls exactly like in play.
- `--team=<n>`: at start, add n free rescued-style recruits with order Follow next to the player (use `GuildRecruits::add_rescued` from R29; names from `data/guild.json` recruits). With `--down-at`, one of them should walk over and revive.
- `--tp-rescue`: at start, put the player 6 m from the nearest unrescued survivor spot (`play.rescue`), facing it, so the survivor and the "[E] Rescue" prompt are in the screenshot.
- `--hold-e`: hold E the whole run (so self revive with a bandage can be pictured).
Put the parsing and the "what to do" logic in `rotland/src/app/test_flags.rs` with unit tests. Add one example per flag to `tools/README_TOOLS.md` (next to the R28 `--pose` flags), like:
`flock /tmp/rotland-heavy.lock target/debug/rotland --save-dir=/tmp/scratch --shot=/tmp/down.png --down-at=1 --frames=120 --cam-yaw=90`

## Worker notes
- **Files changed:**
  - `rotland/src/app/test_flags.rs` (new): `TestFlags` parsing (`--down-at`, `--team`, `--tp-rescue`, `--hold-e`), `tp_rescue_pose`, `build_rescued_recruits`, `apply_startup`, `tick_timers`, and unit tests.
  - `rotland/src/app/startup.rs`: parses `session.test_flags` via `TestFlags::from_args(args)` and calls `super::test_flags::apply_startup(gpu, render, world, actors, play, session)` at the end of `init`.
  - `rotland/src/app/frame_actors.rs`: updated `check_scripted_timers` to accept `render: &mut RenderState` and `audio: &mut super::state::AudioState`, and call `super::test_flags::tick_timers(session, render, actors, play, audio)`.
  - `tools/README_TOOLS.md`: added `## Downed, revive & rescue screenshot flags (used with --shot=<png>)` with one example per flag.
- **Lines the lead must add to shared files:**
  1. `rotland/src/app/mod.rs` (e.g. after `pub mod startup;`):
     ```rust
     pub mod test_flags;
     ```
  2. `rotland/src/app/state.rs`:
     - In `pub struct SessionState` (e.g. after `pub anim_shot: super::anim_shot::AnimShotState,`):
       ```rust
       pub test_flags: super::test_flags::TestFlags,
       ```
     - In `impl Default for SessionState` (e.g. after `anim_shot: super::anim_shot::AnimShotState::default(),`):
       ```rust
       test_flags: super::test_flags::TestFlags::default(),
       ```
  3. `rotland/src/main.rs` (lines 128-133, `check_scripted_timers` call in `Game::update`):
     ```rust
     app::frame_actors::check_scripted_timers(
         &mut self.session,
         &mut self.render,
         &self.world,
         &mut self.actors,
         &mut self.play,
         &mut self.audio,
     );
     ```
- **How to see it in the game:**
  - `--down-at=1 --frames=120 --cam-yaw=90`: player takes lethal zombie-bite-path damage at 1s and enters the `downed_crawl` state.
  - `--team=1 --down-at=1 --frames=180 --cam-yaw=90`: spawns a free Follow teammate next to the player who walks over and revives when downed.
  - `--tp-rescue --frames=30`: teleports the player 6 m from the nearest unrescued survivor spot facing it so the survivor and `[E] Rescue` prompt are visible.
  - `--give=bandage --down-at=1 --hold-e --frames=180 --cam-yaw=90`: holds `E` continuously so self-revive with a bandage triggers while downed.
- **Known / unsure:**
  - Did not run `cargo` or `tools/gate.sh` per R30 subagent rules; needs the lead's shared-file additions and gate run to compile and verify.
