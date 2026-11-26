# F1 clips: ladders, stairs, vaulting (for jobs A09, A10)

Files: tools/blender/characters/poses_climb.py, tools/blender/characters/poses.py, tasks/F1-clips-ladder-vault.md

Read `tasks/F-clips-common.md` first.
Ladder rungs are 0.3 m apart; the character faces the ladder (forward, -Y in Blender front view), hands at
shoulder height and above on the rungs, feet on rungs.
- `ladder_climb` (loop, 0.8 s): one full hand-over-hand + foot cycle = two rungs. Alternate left hand + right foot.
- `ladder_enter_bottom` (no loop, 0.5 s): from standing, reach up and step onto the first rung.
- `ladder_exit_top` (no loop, 0.9 s): pull up over the top, step forward, stand.
- `ladder_enter_top` (no loop, 0.9 s): turn around at the top and step down onto the ladder.
- `ladder_exit_bottom` (no loop, 0.4 s): step off the last rung to standing.
- `stairs_up` (loop, 0.7 s): higher knee lift than walking, slight forward lean.
- `stairs_down` (loop, 0.7 s): careful steps, slight lean back, knees bend more on landing.
- `vault_low` (no loop, 0.7 s): one hand on a ~1 m wall, legs swing over to the side, land.
- `step_over` (no loop, 0.6 s): high step over a ~0.5 m obstacle.

## Worker notes
- Clips created in `tools/blender/characters/poses_climb.py` and exported through `tools/blender/characters/poses.py`:
  - `ladder_climb`: 0.8 s, loop: True. Full alternating 2-rung hand-over-hand and foot-over-foot cycle (0.3 m rung spacing), hands grip rails/rungs at shoulder-width, seamless loop.
  - `ladder_enter_bottom`: 0.5 s, loop: False. Reaches up from standing and steps right foot onto 0.3 m first rung.
  - `ladder_exit_top`: 0.9 s, loop: False. Pulls up over top platform ledge, steps forward onto platform, stands upright.
  - `ladder_enter_top`: 0.9 s, loop: False. Crouches at platform top edge, reaches down to ladder rails/rungs, steps down backward onto top rung.
  - `ladder_exit_bottom`: 0.4 s, loop: False. Steps off lowest rung backward onto ground and stands upright.
  - `stairs_up`: 0.7 s, loop: True. Higher knee lift to clear stair risers, slight forward lean, natural alternating arm counter-swing, seamless loop.
  - `stairs_down`: 0.7 s, loop: True. Careful descent, slight lean back, knee absorption on impact, subtle natural arm counter-swing, seamless loop.
  - `vault_low`: 0.7 s, loop: False. Parkour speed vault over ~1 m obstacle with left hand planted on obstacle top, body rolled, legs tucked and swung over to the right side (+X), clean two-foot landing.
  - `step_over`: 0.6 s, loop: False. High hurdle step over ~0.5 m obstacle, lead right leg lifts high and plants over, trailing left leg lifts high over and plants to standing.
- Visual inspection of all 8-frame contact sheets confirmed:
  - No legs crossing or backwards knee/elbow bending.
  - Ground contacts maintain correct floor elevation; ladder feet align to 0.3 m rungs.
  - Vault and climb hand placements are clear of body collisions.
- Notes / known issues: None. All clips are in-place and ready for humanoid state machine integration in jobs A09 and A10.
