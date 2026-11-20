# R6-01: Animals and hunting — Rust port, logic only

Read `tasks/R5-common.md` first (same rules: write-only, game crate, real data, Godot keys, seeded randomness, private submodules + `pub use`, mod.rs layout, tests from real data, clippy-clean, trace first frames).

Port to `rust/game/src/animals/` the logic (no rendering) of `scripts/Animals/*.cs` except `AnimalVisual.cs`:
AnimalConfig/Data + AnimalSpeciesData + AnimalBehaviourData + AnimalSpawnData (`data/animals.json`), AnimalState, AnimalSenses, AnimalAI (graze, wander, flee from player/zombies/noise, boar charge if the species does), AnimalMotion (move with `physics::CharacterMover` like `rust/rotland/src/zombies.rs` does for zombies; take `&PhysicsWorld`), AnimalHerd, AnimalCombat (health, being hit, boar attack damage), AnimalHunting, AnimalLoot + AnimalButcher/Data (butchering a corpse gives items like raw_meat and hides by species), AnimalSpawner + AnimalPool (spawn around the player by biome/time rules, despawn far).
- Output per animal per frame: id, species, model name, position, yaw, animation clip name, alive/dead, so Claude can draw it like zombies.
- Hits come in like zombies: `hit(id, DamageInfo, knockback) -> killed: bool`; targets list like `Zombies::targets()` (see `rust/rotland/src/zombies.rs` and `game::melee::MeleeTarget`).
- Butcher: `butcher(id, &mut Inventory) -> Vec<ItemStack>` (what was added) and the kill reported for bounties (`species`).
Tests: spawn picks on real data for a fixed seed (count them), flee moves away from the player, a boar charges and hits, damage kills, butcher gives the data's items into an Inventory, despawn far away.
