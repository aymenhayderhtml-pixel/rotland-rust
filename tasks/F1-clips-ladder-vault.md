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
