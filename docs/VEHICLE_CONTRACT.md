# Vehicle contract: the hatchback (V11-08)

Godot paths (scripts/, scenes/, project.godot) refer to the frozen Godot repo (github.com/aymenhayderhtml-pixel/rotland, on the owner's machine at ../zombi servival).

Art (Blender) and code (Godot C#) both build against this file. **Don't change names without updating this contract and `VehicleModelContract`.**

## Files
| File | What |
|---|---|
| `tools/blender/vehicles/car_parts.py` | Geometry: body, door, wheels, collider, empties (pure bmesh) |
| `tools/blender/vehicles/build_car.py` | Builds both colours and exports the glbs; `blender -b --python tools/blender/vehicles/build_car.py` |
| `assets/models/vehicles/car_hatchback_red.glb`, `car_hatchback_blue.glb` | The two colour variants (same geometry, different body colour) |
| `assets/models/_previews/car_hatchback_<colour>.png` | 3/4 front view |
| `assets/models/_previews/car_hatchback_<colour>_side.png` | Side view (check the wheels sit on the ground) |

## Size and budget
- About 3.9 m long, 1.7 m wide, 1.45 m tall (about 3.92 x 1.66 x 1.42 m in the file).
- Budget **2500 triangles** (actual: 460).
- Resting on the ground: the lowest point is **y = 0**. Ground clearance (body underside) is 0.26 m.

## Axes
- **Forward is −Z in Godot** (+Y in Blender). Left is −X, right is +X. Up is +Y.
- The origin is on the ground, at the centre between the axles (x = 0, z = 0 in Godot).
- Wheelbase 2.44 m (axles at z = ±1.22 in Godot). Track 1.44 m (x = ±0.72).

## Nodes (exact names; each is a separate node, not one mesh)
| Node | What | Origin |
|---|---|---|
| `body` | Lower body, cabin, bumpers, headlights, tail lights, rust patch. Cabin sides are `glass_tint` | Ground centre |
| `door_L` | Driver door panel (left side) | Its front (hinge) edge at mid height, Godot (−0.80, 0.55, −0.95) |
| `wheel_FL`, `wheel_FR`, `wheel_RL`, `wheel_RR` | Tyre + chrome hub, axle along X | **Wheel centre** (radius 0.30 m) |
| `collision-convcolonly` | One box for physics (x ±0.78, y ±1.88, height 0.26 to 1.42). Godot makes it a `CollisionShape3D` | Ground centre |

Wheels are in front at `wheel_FL` / `wheel_FR` (Godot z −1.22) and at the back at `wheel_RL` / `wheel_RR` (z +1.22). Each wheel node can spin around its X axis and steer around its Y axis.

## Empties (Godot `Node3D` markers, no geometry)
| Name | Where (Godot metres, x, y, z) | Use |
|---|---|---|
| `seat_driver` | (−0.32, 0.50, −0.20) | Where the player sits (hips height) |
| `exit_L` | (−2.00, 0.00, −0.50) | Exit point: 1.2 m left of the driver door, on the ground |
| `trunk` | (0.00, 0.60, 2.20) | Storage interact point, behind the car |
| `headlight_L` / `headlight_R` | (−0.55, 0.66, −1.93) / (0.55, 0.66, −1.93) | SpotLight positions at the front |
| `exhaust` | (0.30, 0.20, 1.97) | Exhaust smoke position at the back |

## Materials and colours
Palette sheet B (`palette_b.png`): `car_red` / `car_blue` (body), `tire_black`, `chrome`, `glass_tint`, `rust`. Sheet A: `white` and `red_paint` (lights), `metal` (bumpers).

## Self-test
`VehicleModelContract` (`scripts/Debug/Tests/VehicleModelContract.cs`): both glbs have every node above, the four wheel origins are at the wheel centre height (0.30 m) and their lowest points are at y = 0, forward is −Z (front wheels, headlights, trunk, exhaust), the driver seat is on the left at hip height, `exit_L` is 1.5 m or more to the left and on the ground, and the size is in range.
