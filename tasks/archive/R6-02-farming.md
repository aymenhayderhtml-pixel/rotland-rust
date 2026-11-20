# R6-02: Farming — Rust port, logic only

Read `tasks/R5-common.md` first (same rules).

Port to `rust/game/src/farming/` the logic of `scripts/Farming/*.cs` except `PlotNode.cs` visuals:
FarmConfig/Data (`data/farming.json`), CropDef, CropStage, FarmRules, PlotRecord, PlotAction, FarmManager (+ .Actions: till, plant seeds from the Inventory, water with the watering can, harvest into the Inventory; + .Trample: zombies/animals walking over plots; + .Save under Godot's save id, check `FarmManager.Save.cs`).
- Crops grow with game time (take `hours_passed` / day changes, like `world::time::Clock`).
- Output per plot: position, crop id, stage index, stage model name (from the data), watered, so Claude can draw them.
- Plots are placed by the player: `till(position) -> Result<plot_id, Reason>` with Godot's rules (spacing, ground, inside/outside camp if the C# checks it).
Tests: grow a crop through all stages on real data hours (count stages), watering rules, harvest gives the data's items, trample, save round trip with a hand-written Godot-shaped fixture.
