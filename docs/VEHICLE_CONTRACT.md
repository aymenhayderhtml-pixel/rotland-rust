# Vehicle contract: the 4x4 Outdoor Survival SUV (V11-08 / vehicle overhaul)

Godot paths (scripts/, scenes/, project.godot) refer to the frozen Godot repo (github.com/aymenhayderhtml-pixel/rotland, on the owner's machine at ../zombi servival).

Art (Blender) and code (Rust, and the frozen Godot C#) both build against this file. **Don't change names without updating this contract and the model tests.**

## Files
| File | What |
|---|---|
| `tools/blender/vehicles/car_parts.py` | Multi-part 4x4 SUV geometry: body, driver door, tailgate + spare, shared off-road wheel, collider, empties (pure bmesh) |
| `tools/blender/vehicles/build_car.py` | Exports the three body/door/tailgate colours and shared wheel; `blender -b --python tools/blender/vehicles/build_car.py` |
| `assets/models/vehicles/car_hatchback_{red,blue,olive}.glb` | Body + convex collider + all markers (same geometry, different body colour) |
| `assets/models/vehicles/car_wheel.glb` | One chunky off-road wheel, origin at the wheel centre, axle along X |
| `assets/models/vehicles/car_door_{red,blue,olive}.glb` | Hinged driver door, origin at `door_hinge_L` |
| `assets/models/vehicles/car_tailgate_{red,blue,olive}.glb` | Hinged rear tailgate with mounted spare wheel, origin at `tailgate_hinge` |
| **Exported models** | **10 `.glb`s total:** `car_hatchback_{red,blue,olive}`, `car_wheel`, `car_door_{red,blue,olive}`, `car_tailgate_{red,blue,olive}` |
| `assets/models/_previews/car_hatchback_<colour>.png` | Assembled 3/4 front view |
| `assets/models/_previews/car_hatchback_<colour>_side.png` | Assembled side profile (wheels on the ground) |
| `assets/models/_previews/car_hatchback_<colour>_rear.png` | Assembled 3/4 rear view (tailgate + spare) |

The body `.glb` still uses the `car_hatchback_` file prefix so existing saves and spawn data keep working; it is the 4x4 now.

## Size and budget
- About **4.96 m long** including the bullbar and the tailgate's spare wheel, **1.88 m wide** over the fender flares (main body 1.72 m), **1.62 m to the roof** and about **1.93 m to the top of the roof rack gear**.
- Budget **4500 triangles** for the whole assembled vehicle (body + door + tailgate + 4 wheels); Iteration 6 measured **4040** (body 2172, door 156, tailgate 416, wheels 4 x 324).
- Resting on the ground: the lowest point is **y = 0**. Sill ground clearance is 0.40 m; the wheel-well ceiling is 0.90 m, which clears a full suspension bump (0.34 m wheel radius + 0.15 m travel = 0.83 m).

## Axes
- **Forward is −Z in Godot** (+Y in Blender). Left is −X, right is +X. Up is +Y.
- The origin is on the ground, at the centre between the axles (x = 0, z = 0 in Godot).
- Wheelbase **2.52 m** (axles at z = ±1.26 in Godot). Track **1.56 m** (x = ±0.78). Wheel radius **0.34 m**.

## Nodes (exact names; each is a separate node, not one mesh)
| Node | File | What | Origin |
|---|---|---|---|
| `body` | `car_hatchback_<colour>.glb` | Frame, sculpted body, cabin + interior, bullbar, skid plate, snorkel, roof rack, gear, lightbar, lights, rust | Ground centre |
| `collision-convcolonly` | `car_hatchback_<colour>.glb` | One box for physics (x ±0.84, y ±1.92, height 0.40 to 1.62). Godot makes it a `CollisionShape3D` | Ground centre |
| `door_L` | `car_door_<colour>.glb` | Driver door: panel, window frame + glass, handle, rub strip, left mirror | Hinge: Godot (−0.84, 0.72, −0.80) |
| `tailgate` | `car_tailgate_<colour>.glb` | Rear swing gate: panel, rear glass, handle, plate, mounted spare wheel | Hinge: Godot (0.74, 0.90, 1.90) |
| `wheel` | `car_wheel.glb` | Tyre with 8 tread lugs, dark recessed dish, 5 proud chrome spokes, locking hub, brake rotor | **Wheel centre**, axle along X |

A wheel instance can spin around its X axis and steer around its Y axis. The four wheel positions are the `wheel_FL` / `wheel_FR` / `wheel_RL` / `wheel_RR` markers on the body: front at Godot z −1.26, rear at z +1.26, x ±0.78, y 0.34.

## Empties (Godot `Node3D` markers, no geometry; all on the body `.glb`)
The body exports **16 markers**: driver seat and exit, trunk, four lights, lightbar, exhaust, hood smoke, two hinges, and four wheel centres.

| Name | Where (Godot metres, x, y, z) | Use |
|---|---|---|
| `seat_driver` | (−0.36, 0.72, −0.12) | Where the player sits (hips height) |
| `exit_L` | (−2.05, 0.00, −0.30) | Exit point: on the ground, left of the driver door |
| `trunk` | (0.00, 0.75, 2.35) | Storage interact point, behind the spare wheel |
| `headlight_L` / `headlight_R` | (−0.60, 0.80, −1.98) / (0.60, 0.80, −1.98) | SpotLight positions at the front |
| `taillight_L` / `taillight_R` | (−0.79, 0.84, 1.93) / (0.79, 0.84, 1.93) | Rear lamp glow positions |
| `lightbar` | (0.00, 1.68, −0.38) | Roof LED lightbar glow |
| `exhaust` | (0.46, 0.34, 2.04) | Exhaust smoke position at the back |
| `hood_smoke` | (0.00, 1.08, −1.45) | Engine damage smoke position under the hood |
| `door_hinge_L` | (−0.84, 0.72, −0.80) | Door swing pivot |
| `tailgate_hinge` | (0.74, 0.90, 1.90) | Tailgate swing pivot |
| `wheel_FL` / `wheel_FR` | (−0.78, 0.34, −1.26) / (0.78, 0.34, −1.26) | Front wheel centres |
| `wheel_RL` / `wheel_RR` | (−0.78, 0.34, 1.26) / (0.78, 0.34, 1.26) | Rear wheel centres |

Door and tailgate markers are informational: the separate part files already have their origin on the hinge, so code only has to rotate the loaded part around its own origin.

## Materials and colours
Palette sheet B (`palette_b.png`): `car_red` / `car_blue` / `cloth_olive` (body), `tire_black`, `chrome`, `glass_tint`, `rust`, `leather` (seats), `canvas_khaki` (bedroll).
Sheet A: `white` (headlights, plate), `red_paint` (tow shackles, jerry can), `metal` (skid plate, winch, crate strap), `gun_black` (frame, bullbar, rack, snorkel, tyres trim), `wood_crate` (supply crate), `carrot_orange` (indicators), `brass` (winch hook), `rock_dark` (differentials), `pants_dark` (dash, headrests).

## Self-test
`baker/tests/bake_test.rs` (Rust port; replaces the frozen `scripts/Debug/Tests/VehicleModelContract.cs`):
- the body, door, tailgate and wheel `.glb`s bake without error,
- every triangle is **CCW-outward** relative to its vertex normals (matches the `skinned` pipeline's `FrontFace::Ccw`),
- the body has every marker above, with the four wheel origins at wheel-centre height (0.34 m),
- forward is −Z (front wheels, headlights, lightbar and hood smoke at negative Godot z; tail lights, trunk and exhaust at positive z),
- the driver seat is on the left at hip height and `exit_L` is on the left, on the ground.
