# R42-06: Army line models (tank, trucks, helicopter, barrier, camp) + the R42 build entry

Owns:
- `tools/blender/places/army_line.py` (`build_all()`), `tools/blender/places/army_vehicles.py`, `tools/blender/places/army_camp.py` (NEW)
- `tools/blender/places/__init__.py` (NEW, empty)
- `tools/blender/build_r42_models.py` (NEW). It calls, in order, `hospital.build_all`, `police.build_all`, `fire_station.build_all`, `army_line.build_all`, and R42-07's `zombie_looks.build_all` (each import guarded with try/except ImportError, printing which group was skipped). It ends with `R42 BUILD OK` or `R42 BUILD FAILED`.
- the new .glb files below + previews

Not owned: every other script.
Read: tasks/R42-common.md (whole), `tools/blender/build_r41_models.py` (the entry pattern), `tools/blender/metro/jam_truck.py` (the R38 army truck: `army_truck_jam`), `tools/blender/traces/traces.py` (`sandbags_line`), `tools/blender/damage/car_burnt_pile.py` (burnt look), `tools/blender/common.py`.

## Models
Budgets: vehicles 12k LOD0 (`_lod1` 1.2k), others 4k (`_lod1` where seen from far: tents, barriers, booth, kitchen).
- **`army_tank`:** a generic modern main battle tank, olive drab, about 9.5 x 3.6 m.
  - Hull: tracks with road wheels and skirts, an engine deck with grilles.
  - Turret: a long gun, a commander hatch OPEN, a machine gun on the hatch ring, smoke launchers, stowage baskets with bags.
  - Mud and soot on the lower hull, one track thrown (lying off the wheels).
  - Colliders: the hull box + a turret box (climbable onto the deck from the rear).
  - No markings beyond plain stencil numbers.
- **`army_truck_burnt`:** the R38 army truck shape, burnt: canvas gone (only the bare hoops), black/rust, no tyres, sitting on its rims.
- **`heli_crashed`:** a generic medium transport helicopter, olive.
  - Nose crushed into the ground, tilted 20 degrees.
  - Tail boom snapped and bent down, the cabin door slid open (seats inside).
  - Main rotor hub with 1-2 bent stub blades; skids collapsed. Soot round the engine.
  - About 15 m long.
- **`heli_rotor_blade`:** one long bent blade, 7 m.
- **Barrier:**
  - `sandbag_wall`: 6 m straight, 1.3 m high, with a firing slit.
  - `sandbag_corner`: an L-shaped nest with an MG mount, no gun.
  - `razor_wire`: an 8 m run of coiled wire on pickets (thin, 3 coils stacked like a pyramid).
  - `concrete_barrier`: a T-wall, 2 m wide, 3 m tall, with stencils.
- **Camp:**
  - `evac_tent`: a big white/olive frame tent, 8 x 5 m, open flap, "EVACUATION" painted on the side, a red-cross variant not needed.
  - `field_kitchen`: a trailer kitchen with pots.
  - `loudspeaker_pole`: a 6 m pole with 4 horn speakers and a cable.
  - `checkpoint_booth`: a small booth with a window, a lifted boom gate arm.
  - `army_gear_scatter`: helmets, packs, ammo cans and a stretcher on the ground, 4 x 3 m.
  - `flood_light`: a light tower on a trailer, lamps broken.
- Colliders on everything you can bump into (convex pieces); none on wire (it's thin; the lead may add a slow zone later).

## Blender run and checks
- `flock /tmp/rotland-blender.lock blender -b --python tools/blender/build_r42_models.py`.
  - The groups from other tasks may be skipped if missing.
  - Before you report, run it once more after the others have written their scripts, if they have.
- No `Traceback`. Look at every preview (the tank and the helicopter from 3 sides). Fix until each reads at a glance as "the army lost here".

## Done when
All models within budget with good previews, and `build_r42_models.py` works. Fill `## Worker notes`.

## Worker notes
