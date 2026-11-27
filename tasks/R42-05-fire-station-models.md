# R42-05: Fire station (main building, trucks, props, layout data)

Owns:
- `tools/blender/places/fire_station.py` + `tools/blender/places/fire_props.py` (NEW; `build_all()` in `fire_station.py`)
- the new .glb files below + previews
- `data/places/fire_station.json` (NEW)

Not owned: `place_kit.py` (R42-03: import only), `build_r42_models.py` (R42-06), every other script.
Read: tasks/R42-common.md (whole), tasks/R42-03-hospital-models.md ("Rules for every R42 building": follow them exactly), `tools/blender/metro/jam_truck.py` + `jam_bus.py` (big vehicle style), `tools/blender/towers/brick.py` (brick facade style).

## The fire station
- **`metro_fire_station`**: 34 x 26 m, 2 floors + a 4-storey hose-drying tower at one corner, brick with stone trim.
  - **Front:** 3 tall garage bays (5 m wide, 5 m high):
    - bay 1 roller door half up
    - bay 2 door gone (the opening is open and walkable)
    - bay 3 `bay_door_torn` hanging off its rail (also a separate model for the yard)
  - A "FIRE DEPT" sign and a red light over each bay; a side office door.
  - **Inside (ground floor):**
    - the bay hall with painted floor lines, hoses on the walls, a pole hole in the ceiling with a brass pole
    - a gear room with turnout coats on hooks and helmets
    - lockers, a kitchen/day room (table, chairs, counter)
    - a stair (blocked by rubble)
  - Debris and stains as elsewhere.
- **Trucks** (category `vehicles`, 10k LOD0, `_lod1` 1k; R38 car quality: wheels, mirrors, cab interior, ladders, hose reels, compartments, lights):
  - `fire_truck`: a pumper. Red with a white stripe, chrome bumper. "FIRE DEPT" letters.
  - `ladder_truck`: a longer truck with a turntable aerial ladder, folded.
  - `fire_truck_burnt`: the pumper burnt black, tyres gone.
- **Props:** `hose_rack`, `gear_rack` (coats + helmets on a rack), `bay_door_torn`, `kitchen_counter`.
- **`data/places/fire_station.json`:**
  - `props`: `fire_truck` parked IN bay 1 (under the half door), `ladder_truck` half out of bay 2 nosing onto the forecourt, cones, hose lines on the ground.
  - `crates`: 2 `metro_locker` in the gear room (`metro_fire`), 1 `med_cabinet` in the day room (`metro_medical`).
  - `yards.a` = the training yard: a `fire_truck_burnt`, a training tower stub (stacked containers are fine if a model exists, else `rubble_field_m`), hoses, cones.
  - `yards.b` = staging: 2 `jersey_barrier`s, a `police_car_burnt` (R42-04 makes it; check the name), `body_bag`s.
  - `yards.c` = an overgrown lot (no props needed; R42-01 paints it, and the lead may enable overgrowth there: write a note).
  - `yardCrates.a`: 1 `metro_locker` (`metro_fire`).

## Done when
All models exist within budget with good previews (including the cut-away and each truck from the side and the front), and the JSON is complete. Fill `## Worker notes`.

## Worker notes
