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
