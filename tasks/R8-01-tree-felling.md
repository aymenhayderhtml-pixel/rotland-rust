# R8-01: Tree felling — Rust port, logic only

Read `tasks/R8-common.md` first.

Port `scripts/World/TreeFelling.cs`, `TreeHit.cs` and `TreeStrike.cs` to `rust/game/src/trees/` (mod.rs + small files). The data the C# reads comes from data files; find them (search `data/` for the keys the C# uses) and load them the same way.
- Trees in Rust are scatter instances (`rust/world/src/scatter.rs`, `is_tree()`). Give each tree a stable id string from its chunk coord and its index in the chunk (document the format; it must be the same after reload).
- API: a `TreeFelling` manager. `strike(tree_id, tree_pos, tree_kind, weapon_id, skills, player_pos) -> TreeStrikeResult` (hits left, wood given, felled yes/no, sound event); `is_felled(id)`; `update(dt)` advances the fall animation and returns, per falling tree, its tilt angle and fall direction (away from the player) so Claude can tilt the model; regrowth if Godot has it.
- Wood goes into an `&mut Inventory` (open `rust/game/src/items/` for the real `add` signature). Skill XP if Godot gives any (open `rust/game/src/skills/` for `add_xp`).
- Save: the exact Godot keys and system id (find where the C# saves felled trees: WorldState or its own system). Say them in your report.
Tests (`rust/game/tests/trees_tests.rs`): a strike with an axe vs bare hands vs a non-axe weapon, hits needed traced from the data, wood count, falling angle reaches the end, felled trees stay felled after a save round trip.

## Worker notes

### Files changed
- `rust/game/src/trees/mod.rs` (new): Private submodules and re-exports for the tree felling system.
- `rust/game/src/trees/strike.rs` (new): Port of `TreeHit.cs` and `TreeStrike.cs`. Spatial arc/range query (`find_tree_strike`), stable tree id (`tree_id`, `parse_tree_id`), and tree model filter (`is_tree_kind`).
- `rust/game/src/trees/falling.rs` (new): Active falling tree animation tracking (`FallingTree`), advancing tilt angle from 0.0 to `FRAC_PI_2` (90 degrees) along the horizontal vector pointing away from the player.
- `rust/game/src/trees/save.rs` (new): Serialization and deserialization of fallen trees compatible with Godot `WorldState.cs` save data (`to_save_fallen_trees`, `from_save_fallen_trees`).
- `rust/game/src/trees/felling.rs` (new): `TreeFelling` manager port of `TreeFelling.cs`. Handles `strike`, `strike_weapon`, `is_felled`, `update(dt)`, `apply_fallen`, wood generation in `3..=5`, and `TreeStrikeResult::add_to_inventory`.
- `rust/game/src/lib.rs`: Registered `pub mod trees;` and `pub use trees::*;`.
- `rust/game/tests/trees_tests.rs` (new): 9 integration tests covering axe felling in 4 hits, bare hands immunity, non-axe immunity, wood drop range and inventory weight/count integration, fall animation tilt/direction, save/load roundtrips, spatial arc/range queries, non-tree filtering, and chunk prop hiding.
- `tasks/R8-01-tree-felling.md`: Added worker notes.

### Public API
- `TreeFelling`:
  - `TreeFelling::new() -> Self`: Loads `data/weapons.json` via `paths::repo_path` if available.
  - `TreeFelling::with_weapons(weapons: Option<Arc<WeaponRegistry>>) -> Self`
  - `TreeFelling::with_seed(seed: u64) -> Self`
  - `pub fn strike(&mut self, tree_id: &str, tree_pos: Vec3, tree_kind: &str, weapon_id: Option<&str>, skills: Option<&SkillSystem>, player_pos: Vec3) -> TreeStrikeResult`
  - `pub fn strike_weapon(&mut self, tree_id: &str, tree_pos: Vec3, tree_kind: &str, weapon_id: &str, skills: Option<&SkillSystem>, player_pos: Vec3) -> TreeStrikeResult`
  - `pub fn is_felled(&self, tree_id: &str) -> bool`
  - `pub fn is_prop_felled(&self, chunk: IVec2, prop_index: usize) -> bool`
  - `pub fn update(&mut self, dt: f32) -> Vec<FallingTree>`
  - `pub fn apply_fallen<'a>(&self, chunk: IVec2, props: impl IntoIterator<Item = (usize, &'a str)>) -> Vec<usize>`
  - `pub fn to_save(&self) -> serde_json::Value`
  - `pub fn from_save(&mut self, data: &serde_json::Value)`
  - `pub fn can_chop(&self, weapon_id: &str) -> bool`
  - `pub fn set_seed(&mut self, seed: u64)`
  - `pub fn set_fixed_wood(&mut self, count: Option<u32>)`
  - `pub fn set_weapons(&mut self, weapons: Arc<WeaponRegistry>)`
- `TreeStrikeResult`:
  - `pub hits_left: u32`
  - `pub wood_given: u32`
  - `pub felled: bool`
  - `pub sound_event: Option<&'static str>`
  - `pub fn add_to_inventory(&self, inventory: &mut Inventory) -> u32`
- `FallingTree`:
  - `pub id: String`
  - `pub position: Vec3`
  - `pub fall_dir: Vec3`
  - `pub tilt_angle: f32` (radians, 0.0 to `FRAC_PI_2`)
  - `pub progress: f32` (0.0 to 1.0)
  - `pub elapsed: f32`
  - `pub duration: f32`
  - `pub finished: bool`
  - `pub fn update(&mut self, dt: f32)`
  - `pub fn tilt_degrees(&self) -> f32`
- Spatial & Identifier Helpers:
  - `TreeHit`: `pub chunk: IVec2`, `pub index: usize`, `pub point: Vec3`, `pub fn tree_id(&self) -> String`
  - `TreeCandidate<'a>`: `pub chunk: IVec2`, `pub index: usize`, `pub position: Vec3`, `pub model: &'a str`
  - `find_tree_strike(origin: Vec3, forward: Vec3, range: f32, arc_deg: f32, candidates: impl IntoIterator<Item = TreeCandidate>) -> Option<TreeHit>`
  - `tree_id(chunk: IVec2, index: usize) -> String` (format: `"tree_{chunk.x}_{chunk.y}_{index}"`)
  - `parse_tree_id(id: &str) -> Option<(IVec2, usize)>`
  - `is_tree_kind(kind: &str) -> bool`
- Constants:
  - `TREE_FELL_HITS: u32 = 4` (also `TreeFelling::FELL_HITS`)
  - `TREE_WOOD_MIN: u32 = 3` (also `TreeFelling::WOOD_MIN`)
  - `TREE_WOOD_MAX: u32 = 5` (also `TreeFelling::WOOD_MAX`)
  - `TREE_DROP_OFFSET: f32 = 1.5` (also `TreeFelling::DROP_OFFSET`)
  - `TREE_ITEM_HALF_HEIGHT: f32 = 0.125`
  - `TREE_FALL_DURATION: f32 = 1.2`
  - `WOOD_ITEM_ID: &str = "wood"`
  - `HIT_WOOD_SOUND: &str = "hit_wood"`
  - `TREE_WORLDSTATE_SAVE_ID: &str = "worldstate"`
  - `TREE_FALLEN_KEY: &str = "fallen_trees"`

### Save keys
- System ID: `"worldstate"` (matches Godot `scripts/World/WorldState.cs:SaveId`)
- Save key: `"fallen_trees"` (list of string ids `["tree_0_0_1", ...]`)
- Partial hit counts are not saved (matches Godot `TreeFelling.cs`).

### Traced test numbers
- `FELL_HITS = 4` (`scripts/World/TreeFelling.cs:15`): An axe requires exactly 4 strikes to fell a tree. Hits 1, 2, 3 leave 3, 2, 1 hits remaining. The 4th hit fells the tree (`hits_left = 0`). Subsequent strikes on an already felled tree yield 0 wood and no sound.
- `WoodMin = 3, WoodMax = 5` (`scripts/World/TreeFelling.cs:16-17`): Random wood dropped is always within `3..=5`.
- Axe tag `"chop"` (`data/weapons.json:21`): Only weapons with tag `"chop"` (axe) count towards felling. Non-axe weapons (bat, knife, machete, pistol, rifle, shotgun) and bare hands (`None` or `""`) leave hits remaining at 4 with no wood and no sound.
- Fall animation duration and tilt: Duration is 1.2s. At `dt = 0.6s` (halfway), progress is 0.5, tilt angle is `0.5 * FRAC_PI_2 = 0.7853982 rad` (45.0 degrees), `finished = false`. At `dt = 1.2s` (full duration), progress reaches 1.0, tilt angle reaches `FRAC_PI_2 = 1.5707963 rad` (90.0 degrees), and `finished = true`. On subsequent frames, finished animations are automatically pruned.
- Fall direction vector: Player at `(10, 0, 0)`, tree at `(13, 0, 4)`. Displacement away from player is `(3, 0, 4)`. Normalized horizontal vector is `(0.6, 0.0, 0.8)`.
- Wood item weight: 1.0 kg (`data/items.json:50`). Felling a tree yielding 4 wood adds 4.0 kg to the player's inventory.
- Spatial strike query: Player facing +Z `(0, 0, 1)`, range 2.0, arc 80 degrees (40 deg half-arc). Tree at `(0, 0, 1.5)` is selected. Tree at `(0, 0, 2.5)` is out of range. Tree at `(1.8, 0, 0)` (90 deg) is outside the arc. Non-tree prop `rock_01` at `(0, 0, 0.8)` is ignored.

### Exact wiring steps for the orchestrator (Claude)
1. **Instantiate manager**:
   Add `pub tree_felling: TreeFelling` to the gameplay / world state struct in `rust/rotland/src/play.rs` (initialized with `TreeFelling::new()`).
2. **Melee strike check**:
   On player melee attack impact:
   - Collect candidate scatter props within chunk(s) near the player.
   - Call `find_tree_strike(player_chest, player_forward, weapon.range, weapon.arc_deg, candidates)`.
   - If a `TreeHit` is found:
     - Call `let result = tree_felling.strike(&hit.tree_id(), hit.point, candidate_model, Some(&weapon.id), Some(&skills), player_position);`.
     - If `let Some(sfx) = result.sound_event`: trigger audio playback for `sfx` (`"hit_wood"`).
     - If `result.wood_given > 0`: call `result.add_to_inventory(&mut player.inventory);`. If leftover > 0, spawn a dropped world item box at `hit.point + Vec3::X * TREE_DROP_OFFSET`.
3. **Falling model animation update**:
   In `play.rs` frame update (`update(dt)`):
   - Call `let falling = tree_felling.update(dt);`.
   - For each `f in falling`:
     - Axis of rotation: `let axis = Vec3::Y.cross(f.fall_dir).normalize();`.
     - Apply rotation `Quat::from_axis_angle(axis, f.tilt_angle)` to the tree model instance transform.
     - When `f.finished`: set transform scale to zero or replace with stump.
4. **Chunk props loading**:
   When a chunk loads its props:
   - Query `let felled_indices = tree_felling.apply_fallen(chunk_coord, chunk.props.iter().map(|(i, p)| (i, p.model.as_str())));`.
   - For each `index` in `felled_indices`, hide the prop (scale to zero and disable collider).
5. **Save / Load wiring**:
   - In `WorldState` save: write `worldstate_dict["fallen_trees"] = tree_felling.to_save()["fallen_trees"]`.
   - In `WorldState` load: call `tree_felling.from_save(&worldstate_dict)`.
