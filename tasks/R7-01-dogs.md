# R7-01: Dogs — Rust port, logic only

Read `tasks/R7-common.md` first.

Port to `rust/game/src/dogs/` the logic (no rendering, no UI) of `scripts/Dogs/`: `Dog.cs`, `DogCombat.cs`, `DogConfig.cs` + `DogConfigData.cs` (`data/dog.json`), `DogFood.cs`, `DogGrowth.cs`, `DogManager.cs`, `DogMode.cs`, `DogMotion.cs`, `DogSteering.cs`, `Stray.cs`, `StrayDogSpawner.cs`, `DogBreeder.cs` (buying a puppy from the camp breeder: price, wallet). Skip `DogHud.cs`, `DogMenu.cs`, `DogNameDialog.cs`, `DogPackScreen.cs`, `DogVisual.cs` (R7-04 and Claude do those).
- Movement like the animals: `physics::CharacterMover` + `&PhysicsWorld` (read `rust/game/src/animals/motion.rs` and `manager_update.rs` for the pattern already used).
- A `Dogs` manager with `update(dt, &DogUpdateContext, &PhysicsWorld) -> DogFrame` (context: player position/forward/dead, zombies as `&[(u64, Vec3, bool dead)]`, animals as `&[(u64, Vec3, bool dead)]`, night). `DogFrame`: per dog draw state (id, model name e.g. `dog_tan`/`dog_dark`, position, yaw, clip name: idle/walk/run/sit/lie/bark/bite/die), bites on zombies/animals `(target_id, DamageInfo, knockback)`, sounds `(id, Vec3)`, toasts.
- Commands from the player as plain methods: set mode (follow / stay / guard / attack, whatever `DogMode.cs` has), feed (`feed(dog_id, &mut Inventory) -> Result<String,String>` using `DogFood` rules), rename, adopt a stray, buy from breeder (`&mut Wallet`).
- Dogs take hits like zombies: `hit(id, DamageInfo, knockback) -> killed` and `targets() -> Vec<MeleeTarget>` (so the player's swing can be told to skip them: add `is_friendly`).
- Save: `to_save()`/`from_save()` with the exact keys of `DogManager.cs` Save/Load (system id "dogs" or whatever the .cs uses: say which in the report).
Tests (`rust/game/tests/dogs_tests.rs`): data loads (count compared fields), a following dog walks toward the player on a flat physics world (copy `flat_world()` from `rust/game/tests/animals_tests.rs`), a dog bites a zombie in range, feeding removes the food item and restores hunger, growth puppy -> adult over the data's days, save round trip, breeder takes the right price.
