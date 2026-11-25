# R30-02: rescued survivors start where you found them + teammates kneel to revive

Owns: game/src/guild/companion/*, rotland/src/companion_view.rs, rotland/src/companion_model.rs, rotland/src/rescue_play.rs, rotland/src/app/frame_team.rs, game/tests/all/companion*.rs (new test file is fine), tools/blender/characters/poses_revive.py (new), tools/blender/characters/poses.py, assets/models/characters/*.glb
Rules: `tasks/R30-common.md`.

**1. Survivor starts at the rescue spot.** `CompanionSquad::sync(hired, camp)` (`game/src/guild/companion/squad.rs`) puts every NEW member at a camp slot. A survivor rescued 300+ m away therefore teleports to camp. Give `sync` (or a new small method) a way to say "this new member starts at position P". Then `rescue_play.rs` passes the survivor's `spot.position`, and the member starts there and follows the player. Hired-at-camp recruits still start at camp. Add a unit test.

**2. Teammates kneel when they revive you.** Right now the reviving teammate just stands (`CompanionAnim::Idle`; NPC models have no use/kneel clip). Add one shared clip `revive_kneel` (looping, ~1.2 s: kneel on one knee, lean forward, both hands pressing down in front as if bandaging). Use EXACTLY the clip format in `tasks/R27-common.md`. Put it in a new `tools/blender/characters/poses_revive.py`, registered in `CLIPS` (not `PLAYER_ONLY_CLIPS`) in `poses.py`, so NPCs get it. Add a `CompanionAnim` value for it, and play it in `frame_team.rs` `step_player_revive` while the teammate is beside the downed player.

**Lead rebuilds the characters ONCE (subagent doesn't):**
```bash
flock /tmp/rotland-blender.lock blender -b --python tools/blender/characters/build_characters.py > /tmp/r30-chars.log 2>&1
grep -c Traceback /tmp/r30-chars.log   # must be 0
grep -c CHARACTER /tmp/r30-chars.log   # 14 or more
```
Then look at `assets/models/_previews/npc_guard_revive_kneel.png` (or the nearest name): the knee must be on the ground, not floating or sunk.

## Worker notes
### Part 1 (Survivor starts at the rescue spot)
- **Files changed:**
  - `game/src/guild/companion/squad.rs`: added `spawn_overrides` + `CompanionSquad::set_spawn_position(&mut self, id: &str, position: Vec3)` and `take_spawn_override`. `CompanionSquad::sync` uses the recorded spawn position for newly created `MemberBody` entries instead of `camp_slot_offset`, consuming it once; camp-hired recruits without an override still start at camp.
  - `rotland/src/rescue_play.rs`: added `companions: Option<&mut CompanionSquad>` to `interact_tick` and calls `squad.set_spawn_position(&recruit.id, spot_pos)` when `try_rescue_nearest` succeeds.
  - `game/tests/all/companion_rescue_start.rs`: added 3 unit tests covering rescued survivor starting at `rescue_pos` and following the player, camp-hired recruits still starting at camp, and single-use consumption of the spawn override.
- **Lines the lead must add to shared/unowned files:**
  1. In `game/tests/all/main.rs` (after `mod companion_tests;`):
     ```rust
     mod companion_rescue_start;
     ```
  2. In `rotland/src/app/frame_world.rs` around line 323 (the only caller of `crate::rescue_play::interact_tick`), pass `play.companions.as_mut()`:
     ```rust
     crate::rescue_play::interact_tick(rescue, p.position, z, play_state, play.companions.as_mut(), pressed)
     ```
- **How to see it in the game:** Find a trapped survivor at a POI 300+ m from camp, clear nearby zombies, and press `E` to rescue them. They now spawn right at their rescue spot and follow you instead of teleporting to camp.
- **Anything unsure:** None. Did not run `cargo` or `tools/gate.sh` per R30 subagent rules.
