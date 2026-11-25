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
