# R38-05: jam models (cars, bus, truck, luggage, bodies, gantry, barrier)

Owns: tools/blender/metro/ (new folder, one script per group, each under ~300 lines), tools/blender/build_r38_models.py (new), assets/models/vehicles/car_jam_*.glb + bus_city*.glb + army_truck_jam.glb (and their palette PNGs written by the build), assets/models/props/{luggage_pile,suitcase_open,clothes_spill,body_seated,body_sheet,highway_sign_gantry,highway_barrier}.glb (+ palette PNGs), assets/models/_previews/ (preview PNGs for these ids only), and the `jam` -> `seatOffsets` key in data/metro.json only (R38-01 creates that file: start this task after R38-01 is merged, then add only that key).
Read: tasks/R38-common.md ("Jam models", "Budgets", "Data file"), tools/blender/common.py (`Model` class: `add`, `add_collider`, `add_empty`, `finalize_and_export`; `run_model`; `make_box`, `make_frustum`, `combine`, `transform`; `palette_material`), tools/blender/palette.py (colour names: `car_red`, `car_blue`, `car_white`, `tire_black`, `chrome`, `glass_tint`, `rust`, `metal`, `dirt`, `sand_light`, `dry_brush`, `gun_black`, `white`, `shirt_blue`, `pants_denim`, `skin_pale`, `blood_dark`, `canvas_khaki`, `soil_dark`), tools/blender/vehicles/car_parts.py (`_loft`, `_box`, `_fender_flare`, `_build_wheel_mesh`, body sizes: half width 0.86, half length 1.94, roof 1.62, wheel radius 0.34), tools/blender/vehicles/build_car.py (the vehicle build pattern and `OUT_DIR`), tools/blender/props/car_wreck.py and car_wreck_van.py (wreck style: rust body, flat tyres, a hanging door), tools/blender/build_r37_models.py (the `main()` / failures pattern for one build file), tools/blender/buildings/kit.py (box helpers if useful), docs/ART_PIPELINE.md (origin at bottom centre, Y-up export, 1 unit = 1 m, previews).

Goal: the owner wants a dense, detailed, dying city. These models are the evacuation jam on the highway. Rich detail is allowed here; the frame budget is set by the lead.

## What to build
- **Cars** (`car_jam_sedan`, `car_jam_hatch`, `car_jam_suv`, `car_jam_pickup`, `car_jam_van`): one script builds all five from car_parts shapes, with their own sizes (sedan and hatch about 4.0 m long, SUV 4.5 m, pickup 5.2 m with a bed, van 5.0 m tall-roof). Each car has: doors open in a fixed pose (front driver door open about 60 degrees, one rear door open 90 degrees, the boot or tailgate open on some), seats and a steering wheel visible through the open door, cracked or missing windows (glass_tint panes, some holes, no glass in the door frame that is open), 1-2 flat tyres (squashed frustum), rust patches on the sills and the roof, dust (`dirt`/`dry_brush`) on the roof and bonnet. SUV and van: roof rack with luggage (2-4 boxes and a bag) on some; pickup: luggage under a tarp in the bed on some. Colours: sedan `car_blue`, hatch `car_red`, SUV `car_white`, pickup `car_red`, van `car_white`, with rust and dirt on top.
- **Burnt variants** (`<name>_burnt.glb`) of the five cars: same shape, no glass (window holes only), body painted `gun_black` with `rust` patches, tyres `gun_black`, dusty `soil_dark` on top. No orange fire colour beyond `rust`.
- **Bus** `bus_city`: upright city bus, about 12 m long, 2.5 m wide, 3.2 m high. Windows with some cracked and missing, a door open, a destination panel with no real brand text. Tyres flat.
- **Bus** `bus_city_side`: the same bus lying on its side (rotated 90 degrees on X, body lower than the wheels), origin on the ground at the lowest side, wheels up in the air.
- **Army truck** `army_truck_jam`: canvas-back 6-wheel truck (`canvas_khaki`), tailgate down, canvas torn, two flat tyres, a tarp with a few crates, no insignia.
- **Props**: `luggage_pile` (3-6 suitcases and bags stacked, about 1.5 m x 1.2 m), `suitcase_open` (one open suitcase with clothes), `clothes_spill` (flat cloth shapes and a shoe or two on the ground, about 2.5 m x 2 m; low poly, very flat, so it sits on the asphalt), `body_seated` (see seats below), `body_sheet` (a shape on the ground under a `white` / `snow_white` sheet, no gore), `highway_sign_gantry` (two posts and a sign board reading "NEW HALDEN" with exit arrows; no real brands or logos; about 9 m wide, 6 m high, origin bottom centre), `highway_barrier` (concrete jersey barrier, about 3 m long, same as the existing `jersey_barrier` shape but with a dented side).
- **body_seated**: slumped dead person in civilian clothes (`shirt_blue`, `pants_denim`, `skin_pale`, `hair_dark`, `blood_dark` only as small spots). Origin = the seat point (the middle of the seat cushion, see table). Sitting pose: torso leaning back and to one side, head down, arms in lap. Sized to fit a car seat.

## Seat points (contract for R38-04)
Offsets in metres from the car model origin (bottom centre, exported origin), +X right, +Y front, +Z up. A body in a car faces the car's front (yaw 0). The first entry is the driver seat. R38-05 writes these into `data/metro.json` as `jam.seatOffsets`:

| car | seats (x, y, z) in order |
|---|---|
| car_jam_sedan | driver (-0.38, 0.45, 0.42), passenger (0.38, 0.45, 0.42), rear_left (-0.38, -0.45, 0.42), rear_right (0.38, -0.45, 0.42) |
| car_jam_hatch | driver (-0.36, 0.42, 0.42), passenger (0.36, 0.42, 0.42), rear_left (-0.34, -0.50, 0.42) |
| car_jam_suv | driver (-0.40, 0.55, 0.48), passenger (0.40, 0.55, 0.48), rear_left (-0.40, -0.25, 0.48), rear_right (0.40, -0.25, 0.48) |
| car_jam_pickup | driver (-0.38, 0.30, 0.48), passenger (0.38, 0.30, 0.48) |
| car_jam_van | driver (-0.38, 0.95, 0.46), passenger (0.38, 0.95, 0.46), rear_left (-0.38, -0.40, 0.46) |

The burnt variant uses the same seats. Buses and the truck have no seat entries.
The seats are built where these points say, and the model origin must be the one `Model.finalize_and_export` gives (bottom centre of the visual bounding box). So keep each car's front and rear overhang equal, or measure the seats after the origin shift and correct the table, then report the change.

## Build and budgets
- `tools/blender/build_r38_models.py`: imports the `metro` scripts, runs each with `common.run_model`, writes `data/metro.json` `jam.seatOffsets` only through its own helper (or leave the JSON edit to the lead and report it). Prints `FAILED` lines and exits 1 on any failure, like `build_r37_models.py`. Does not touch the palette sheets or other models.
- Run: `blender -b --python tools/blender/build_r38_models.py`.
- Triangle targets (hard cap in brackets): sedan and hatch 7,000 (25,000); SUV, pickup, van 9,000 (25,000); bus and bus_city_side 14,000 (25,000); army truck 12,000 (25,000); props 2,000 to 6,000 each (15,000). Use the budget number in each `run_model` call.
- Model sizes and triangle numbers are set in the Python scripts, each with a `# new: DECISIONS 2026-11-26` comment (Blender scripts are the one place numbers may live, as in `build_r37_models.py`).
- Previews: every model writes its normal preview to `assets/models/_previews/`; the cars and bus also get a `_top.png` with `top_preview=True` so the inside shows.

## Done when
- The 20 `.glb` files exist with the ids above, each within its budget, and the build log has no `Traceback` and no `FAILED`.
- Each car's open doors, seats and steering wheel are visible in its preview; the burnt variants have no glass.
- `jam.seatOffsets` in `data/metro.json` lists the five cars with the table's values (or the corrected values, reported).
- `bus_city_side` origin is on the ground with the bus on its side.
- No sounds, no real brands, no extreme gore. No palette sheet changed.

## Worker notes
(fill in: what changed, any colour missing from the palette and what was used instead, measured triangles per model, corrected seat values, lines for the lead)
