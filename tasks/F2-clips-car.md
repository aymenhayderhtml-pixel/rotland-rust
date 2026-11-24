# F2 clips: cars (for job A11)

Files: tools/blender/characters/poses_car.py, tools/blender/characters/poses.py, tasks/F2-clips-car.md

Read `tasks/F-clips-common.md` first.
The driver door is on the character's **left**. A seated driver: hips about 0.45 m lower than standing
(hips `loc` Y about -0.45), thighs almost level forward, knees bent ~80 degrees, hands on a wheel about
0.4 m in front of the chest at chest height.
- `car_open_door` (no loop, 0.6 s): standing beside the car facing it... facing forward along the car, reach left-down to the handle and pull it open.
- `car_get_in` (no loop, 1.0 s): turn, duck the head, sit down into the seat, pull legs in.
- `car_sit_drive` (loop, 2.0 s): seated, both hands on the wheel, tiny body sway.
- `car_steer_left` and `car_steer_right` (loop, 1.0 s, upper body): hands turn the wheel about 60 degrees.
- `car_sit_passenger` (loop, 2.0 s): seated, hands on thighs, looking around a bit.
- `car_get_out` (no loop, 1.0 s): legs out, stand up, step away.
- `car_thrown_out` (no loop, 1.2 s): thrown sideways out of the seat, tumble, land on the ground lying down (end pose lying on the ground, body not under the ground).

## Worker notes
- Clips created in `tools/blender/characters/poses_car.py`:
  - `car_open_door` (0.6 s, loop: False): standing beside the car facing forward, reaching left-down to door handle and pulling it open.
  - `car_get_in` (1.0 s, loop: False): turn left, duck head under car roof, sit down into driver seat (-0.45 m), bring legs into cabin.
  - `car_sit_drive` (2.0 s, loop: True): seated driver (hips loc Y -0.45 m, knees bent ~80 deg, thighs level forward), hands on wheel ~0.4 m in front of chest, subtle breathing/engine vibration sway.
  - `car_steer_left` (1.0 s, loop: True, upper body): hands turn wheel ~60 degrees counterclockwise with upper-body twist; legs unkeyed for animator layering.
  - `car_steer_right` (1.0 s, loop: True, upper body): hands turn wheel ~60 degrees clockwise with upper-body twist; legs unkeyed for animator layering.
  - `car_sit_passenger` (2.0 s, loop: True): seated passenger (hips loc Y -0.45 m), hands resting on thighs, looking out right window, then straight, then glancing left at driver.
  - `car_get_out` (1.0 s, loop: False): swing legs out through left door, duck head under roof, plant feet on ground outside, stand up clear of vehicle.
  - `car_thrown_out` (1.2 s, loop: False): thrown violently sideways to the left (+Z roll), midair tumble, lands on ground lying on left side; hips loc Y = -0.65 m, left arm resting along ground, final mesh min_z = +0.0128 m (body resting cleanly on ground, not sunken under ground).
- Wiring notes for Claude:
  - `tools/blender/characters/poses.py`: added `from poses_car import CLIPS as _CAR` and included `**_CAR` in `PLAYER_ONLY_CLIPS`.
  - Exported actions match RIG_CONTRACT humanoid 22-bone armature.
- Observations / nothing looks off:
  - Preview contact sheets checked across all 8 clips.
  - Seamless loop endpoints matched exactly at t=0.0 and t=length for looping clips.
  - Ingress and egress clips cleanly transition between ground standing height (0.0 m) and seated cabin height (-0.45 m).
