# R27-03 (A18): teammates and camp NPCs use the new animation system

Files: game/src/team_anim*.rs, game/src/lib.rs, game/tests/all/team_anim_tests.rs, game/tests/all/main.rs, data/team_anim.json, rotland/src/app/frame_team.rs, rotland/src/app/team_*.rs, rotland/src/team_*.rs, rotland/src/companions*.rs, tools/blender/characters/poses.py, tools/blender/characters/build_characters.py, assets/models/characters/*.glb, assets/models/_previews/*, tasks/R27-03-team-anim.md
Rules: `tasks/R27-common.md`.

**Goal (animation only, no AI or gameplay change):** teammates (R23 followers, `data/companions.json`) and camp guards hold their weapon in the right stance, aim and recoil when they shoot, swing melee with the right clip, walk with directional legs, and flinch when hit, by calling the same `HumanoidAnimator` API the player uses (`set_weapon`, `set_aiming`, `on_gun_fired`, `play_action`, `set_move_velocity`). Keep many NPCs cheap with `set_lod` / `set_update_interval` by distance (numbers in `data/team_anim.json`).
- First find how teammates and guards are drawn and animated today (grep `HumanoidAnimator`, `team_models`, `play_action` in rotland/src).
- NPC models only get clips from `CLIPS` in `tools/blender/characters/poses.py`, not `PLAYER_ONLY_CLIPS` (see `build_characters.py`). If an NPC needs a stance/aim clip it lacks, move it to the shared set and rebuild (rules in R27-common).
- Put the state → animator mapping (which clip, when to aim, LOD interval) in a pure module with tests; the rotland side only calls it.
- If a file you need isn't on `Files:`, write the exact edit under Worker notes.

## Worker notes

### Files changed
- `tools/blender/characters/poses.py`: moved `_STANCE`, `_AIM`, and `_MOVE` from `PLAYER_ONLY_CLIPS` to `CLIPS` so all `npc_*` models receive weapon ready stances, aim/reload clips, and directional movement clips (`move_walk_back`, `move_run_back`, `move_strafe_l`, `move_strafe_r`, `move_run_strafe_l`, `move_run_strafe_r`, `move_turn_l`, `move_turn_r`).
- `assets/models/characters/{human_player,npc_*}.glb` and `assets/models/_previews/*`: rebuilt via `tools/blender/characters/build_characters.py`.
- `data/team_anim.json`: added role weapon/attack clip mappings (`fighter`, `guard`, `scavenger`, `campGuard`), LOD distance/interval tiers, movement speeds, and hit-flinch config.
- `game/src/team_anim.rs` + `game/src/lib.rs`: added pure state-to-animator mapping (`TeamAnimConfig`, `TeamLodConfig`, `NpcAnimRole`, `NpcAnimTrack`, `NpcAnimInput`, `NpcAnimPlan`, `plan_npc_anim`).
- `game/tests/all/team_anim_tests.rs` + `game/tests/all/main.rs`: added unit tests for `data/team_anim.json` loading, LOD intervals, melee/ranged attack triggers, flinch on hit, downed/recovery transitions, and resident held poses.
- `rotland/src/app/frame_team.rs`: extended `TeamFigure` with `anim_role`, `velocity`, `just_hit`, `distance_to_player`, tracked per-teammate velocity/health/edge triggers in `TeamScratch`, and added `apply_npc_anim` calling `HumanoidAnimator` (`set_update_interval`, `set_weapon`, `set_aiming`, `reset_alive`, `play_die`, `on_gun_fired`, `play_action`, `hold_pose`/`release_pose`, `set_move_velocity`, `update`).

### How to see it in the game
- Once the wiring notes below are applied, hire teammates (Fighter, Guard, Scavenger) or visit camp guards when zombies approach: guards hold rifles in `stance_rifle_ready`, aim (`aim_rifle_shoulder`), recoil on shots (`on_gun_fired`), fighters swing 2H melee (`melee_2h` with `bat`), scavengers swing 1H melee (`melee_1h` with `knife`), all move with directional legs (`set_move_velocity`) and flinch (`hit`) when damaged, and distant/offscreen NPCs throttle updates via `set_update_interval`.

### Wiring notes for Claude
The following files are outside this task's `Files:` list and still need to be wired to use `game::plan_npc_anim` + `frame_team::apply_npc_anim`:
1. **`rotland/src/companion_model.rs`**:
   - Replace `clocks: HashMap<String, (&'static str, f32)>` with `animators: HashMap<String, humanoid::HumanoidAnimator>` and `tracks: HashMap<String, game::NpcAnimTrack>`, storing `skeleton: pack::Skeleton` on `Look` (in addition to `rig: Rig`).
   - In `advance(&mut self, dt: f32, figures: &[TeamFigure])`, for each `f` look up `look = &self.looks[li]`, get/insert `HumanoidAnimator::new(&look.skeleton, &look.clips)` in `self.animators`, call `game::plan_npc_anim(game::TeamAnimConfig::load_shared(), &NpcAnimInput { role: f.anim_role, companion_anim: Some(f.draw.anim), camp_clip: None, aiming_at_target: false, just_fired: false, just_hit: f.just_hit, velocity: f.velocity, yaw: f.draw.yaw, distance_to_camera: f.distance_to_player, on_screen: true }, track)`, and call `crate::app::frame_team::apply_npc_anim(animator, &plan, dt)`.
   - In `pose(&mut self, figures: &[TeamFigure], bones: &mut Vec<Mat4>)` (note `&mut self`), call `animator.write_skin(&look.rig, &mut bones[base..])` instead of `look.rig.sample(clip, t, ...)`.
2. **`rotland/src/camp_npcs.rs`**:
   - Store `skeleton: pack::Skeleton` on `Look` and replace `clocks: HashMap<String, (String, f32)>` with `animators: HashMap<String, humanoid::HumanoidAnimator>` and `tracks: HashMap<String, game::NpcAnimTrack>`, plus `prev_positions: HashMap<String, Vec3>`.
   - Update `update(&mut self, dt: f32, hour: f32, player: Option<Vec3>, fired_guards: &[String])`: for each figure, compute `role = if f.is_guard { NpcAnimRole::CampGuard } else { NpcAnimRole::Resident }`, `aiming_at_target = self.aim.contains_key(&f.id)`, `just_fired = fired_guards.iter().any(|id| id == &f.id)`, `velocity`, and `distance_to_camera = player.map_or(0.0, |p| p.distance(f.position))`; call `game::plan_npc_anim` and `crate::app::frame_team::apply_npc_anim`.
   - In `pose(&mut self, player: Option<Vec3>, bones: &mut Vec<Mat4>)` (note `&mut self`), call `animator.write_skin(&look.rig, &mut bones[base..])`.
3. **`rotland/src/app/frame_sim.rs`**:
   - Collect `fired_guards: Vec<String>` from `gf.shots` (before `for shot in gf.shots`) and pass `&fired_guards` to `npcs.update(dt, hour, player, &fired_guards)`.
4. **`rotland/src/app/render_frame.rs`**:
   - Change `actors.team_models.as_ref()` to `actors.team_models.as_mut()` and `actors.camp_npcs.as_ref()` to `actors.camp_npcs.as_mut()` so `pose` can take `&mut self` for `write_skin`.

### Known issues / unsure
- `CampNpcs::update` currently has no frustum visibility flag, so `on_screen: true` is assumed unless frustum culling is passed in; distance tiers (`0.0` / `0.04` / `0.1`) still apply.

## Gate report
- date: 2026-11-24
- base: main
- files changed (15): assets/models/characters/npc_banker.glb, assets/models/characters/npc_guard.glb, assets/models/characters/npc_mechanic.glb, assets/models/characters/npc_scout.glb, assets/models/characters/npc_survivor.glb, assets/models/characters/npc_trader.glb, assets/models/characters/npc_villager.glb, data/team_anim.json, game/src/lib.rs, game/src/team_anim.rs, game/tests/all/main.rs, game/tests/all/team_anim_tests.rs, rotland/src/app/frame_team.rs, tasks/R27-03-team-anim.md, tools/blender/characters/poses.py
- tests passed: 1022
- clippy: PASS
- test: PASS
- checks: PASS
- gate: PASS
