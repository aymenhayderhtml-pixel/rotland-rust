# R42-07: Zombie looks (soldier, police, medic, firefighter)

Owns:
- `tools/blender/characters/zombie_looks.py` (NEW, `build_all()` builds the 4 looks using the existing character recipe system)
- `tools/blender/characters/build_characters.py` (ONLY adding the 4 recipe entries and any new `extras` kinds they need; existing recipes and outputs must not change)
- `assets/models/characters/zombie_{soldier,police,medic,firefighter}.glb` (+ palettes/previews the pipeline makes)
- `data/zombies.json` (ONLY a new `extraLooks` list, if you choose that option under "Rust"; the existing `models` lists stay unchanged)
- `rotland/src/zombies/` (ONLY a new `spawn_kind_look(kind, pos, model) -> Option<u64>` next to `spawn_kind`, and whatever small change makes a zombie keep a chosen model), plus a unit test for it
- `game_zombie/src/poi_zombies/` (ONLY: `PoiSpawn` gets `model: Option<String>`, filled from the building's `zombieModels` with the manager's rng)
- `world/src/poi.rs` (ONLY: `PoiBuildingDef` gets `#[serde(default)] zombie_models: Vec<String>` read from `zombieModels`)

Not owned: `data/poi.json` (R42-08 adds the building entries with `zombieModels`), everything else.
Read: tasks/R42-common.md, `tools/blender/characters/build_characters.py` (the recipe dicts and `extras`, e.g. "scout"), `tools/blender/characters/humanoid.py`/`humanoid_body.py`, `rotland/src/zombies/` (spawn + how `models` from `zombies.json` pick a look), `game_zombie/src/poi_zombies/manager.rs`, `world/src/poi.rs`, `rotland/src/app/frame_sim_zombies.rs` (where POI spawns become zombies: R42-09 switches that call to `spawn_kind_look`).

## Looks (same rig, bones and size range as `zombie_01`; zombie skin, blood, torn cloth)
- `zombie_soldier`: an olive combat uniform, a plate carrier vest (torn), a helmet with goggles, boots, knee pads, a radio pouch. Hunched.
- `zombie_police`: a dark-blue uniform shirt with a badge patch, a duty belt, a cap OR a riot helmet (pick the cap; add a riot helmet extra if cheap), dark trousers.
- `zombie_medic`: green scrubs + a white coat (torn, blood-stained), a surgical mask hanging, a stethoscope.
- `zombie_firefighter`: a tan turnout coat and trousers with reflective stripes, a helmet, an air tank on the back, boots.
- Budgets and the export path: as the existing zombie looks. Run ONLY: `flock /tmp/rotland-blender.lock blender -b --python-expr "<import zombie_looks; build_all()>"` (path setup like the other scripts). There must be no `Traceback`. Look at the previews: each look reads at a glance at 10 m.

## Rust
- `PoiBuildingDef.zombie_models` (`zombieModels` in JSON, default empty).
- `PoiSpawn.model`: when the building has `zombieModels`, pick one per spawn (deterministic with the manager rng); else `None`.
- `spawn_kind_look(kind, pos, Some(model))` spawns `kind` (stats unchanged) but draws `model`. With `None` it behaves exactly like `spawn_kind`.
- The model id must be loaded/drawn like the other looks. Check how the zombie renderer collects look models and make sure ids that are only in `zombieModels` are loaded too (a list of extra looks from `poi.json`, or `zombies.json` key `extraLooks: [...]`, which you then own in `data/zombies.json`: say which).
- Tests:
  - `PoiBuildingDef` parses with and without `zombieModels`.
  - The spawn model is one of the list.
  - `spawn_kind_look(.., None)` equals `spawn_kind` (same stats/model choice for the same seed).

## Done when
The 4 looks exist with good previews; code + tests written (the lead builds and tests). Fill `## Worker notes` (the exact line for R42-09 in `frame_sim_zombies.rs`).

## Worker notes
