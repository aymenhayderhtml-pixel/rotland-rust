# R20-09: stop the flicker in props (street lamps, bus stop, car wrecks, tire pile...)

Read first: `tasks/R20-common.md`, `tasks/R19-08-zfight-camp-lake.md` (same job and rules), `tools/blender/common.py` (read only).
Files: tools/blender/props/barricade.py, tools/blender/props/bus_stop.py, tools/blender/props/car_wreck_sedan.py, tools/blender/props/car_wreck_van.py, tools/blender/props/dock_end.py, tools/blender/props/dock_section.py, tools/blender/props/dumpster.py, tools/blender/props/jersey_barrier.py, tools/blender/props/mailbox.py, tools/blender/props/road_sign.py, tools/blender/props/sign_fallen.py, tools/blender/props/street_lamp.py, tools/blender/props/stump.py, tools/blender/props/tire_pile.py, assets/models/props/*, assets/models/_previews/*, tasks/R20-09-zfight-props.md
Test: (Blender lane: no cargo)

Checker today (`python3 -I tools/find_coplanar.py assets/models/props/<name>.glb`), m2: car_wreck_van 1.87, tire_pile 1.56, car_wreck_sedan 0.82, bus_stop 0.47, road_sign 0.42, street_lamp 0.38, jersey_barrier 0.34, dumpster 0.33, mailbox 0.32, sign_fallen 0.29, barricade 0.17, stump 0.09, dock_end 0.08, dock_section 0.06. Biggest first. Goal: under 0.05 m2 each. Only rebuild the props you edit (don't commit other glbs/previews that changed by accident: `git checkout --` them; NOT rowboat, another lane owns it). Same rules: fix, rebuild, re-check, look at the preview; undo a model that got worse; before/after in Worker notes; commit as you go.

## Worker notes
