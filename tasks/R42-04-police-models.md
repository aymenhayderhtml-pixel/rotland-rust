# R42-04: Police station (main building, props, layout data)

Owns:
- `tools/blender/places/police.py` + `tools/blender/places/police_props.py` (NEW; `build_all()` in `police.py`)
- the new .glb files below + previews
- `data/places/police.json` (NEW)

Not owned: `tools/blender/places/place_kit.py` (R42-03 owns it: import it, don't edit it; put any extra helper in your own file), `build_r42_models.py` (R42-06), every other script.
Read: tasks/R42-common.md (whole), tasks/R42-03-hospital-models.md ("Rules for every R42 building": follow them exactly, using your own script in the Blender command), `tools/blender/buildings/police.py` (the town one: do NOT change it), `tools/blender/metro/jam_car_shell.py` (car shape style for the police cars), `tools/blender/towers/lobby_props*.py`.

## The police station
- **`metro_police`**: 36 x 28 m, 3 floors (ground 4.2 m + 2 x 3.8 m), heavy stone/brick look.
  - Outside:
    - wide steps up to double doors under a "POLICE" sign
    - blue lamp posts each side of the door
    - barred lower windows
    - a side sally-port gate (open)
    - a flag pole (no flag; R41 flappers can't be used here)
    - roof antennas
  - Ground floor:
    - a front desk behind a glass screen (glass broken)
    - a waiting bench
    - an open office with 6 desks (computers, papers, chairs tipped)
    - a corridor of 3 holding cells (bars, one door hanging open, a bunk inside)
    - a locker room
    - an armoury behind a heavy mesh cage, its door forced
    - evidence shelves
    - debris, papers, stains, a barricade of desks against one inner door
- **Props:**
  - `police_car_wreck`: a sedan with a light bar, black/white doors, "POLICE" letters, broken windows, doors open
  - `police_car_burnt`: same shape, burnt, charred, no glass
  - `police_desk`, `holding_cell` (a standalone 3 x 3 m cell with bars and a door, for the yard)
  - `locker_row` (5 lockers, a few dented open)
  - `armory_cage` (a mesh cage with empty gun racks)
  - `riot_shield_pile`
  - `jersey_barrier` (concrete, 3 m)
  - `police_barricade` (a striped wooden sawhorse barrier, 2.5 m)
- **`data/places/police.json`:**
  - `crates`:
    - 3 `metro_locker` (`metro_police`) in the locker room / office
    - 2 `army_crate` in the armoury (`metro_police`, the best guns: R42-08 makes the table)
  - `props`: `police_car_wreck`s nose-in at the front, `police_barricade`s across the steps, a `riot_shield_pile`.
  - `yards.a` = the car lot: 5-6 police cars (2 burnt) in bays, a `jersey_barrier` line.
  - `yards.b` = a riot barricade across the lot: `jersey_barrier`s, `police_barricade`s, `riot_shield_pile`s, `body_bag`s, scorch (`ash_patch`).
  - `yards.c` = the impound: a fence line (use `chainlink`/fence models if they exist, else a simple `fence_panel` in your props) with wrecked civilian cars.
  - `yardCrates.a`: 2 `metro_locker` in car trunks (model `""` = invisible, table `metro_police`).

## Done when
All models exist within budget with good previews (including the cut-away), and the JSON is complete. Fill `## Worker notes`.

## Worker notes
