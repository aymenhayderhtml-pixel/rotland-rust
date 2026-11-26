# R20-02: hold the bat properly, and make the hotbar decide what is in your hand (issues 002 + 007 part 2)

Read first: `tasks/R20-common.md`, `docs/ISSUES.md` rows 002 and 007 (+ pictures), `humanoid/src/weapon.rs`, `humanoid/src/animator.rs` (~line 270-290, grip IK), `rotland/src/app/start_world.rs` (~lines 120-150: bat set straight on the player), `rotland/src/play.rs` (~line 100, start inventory), `game/src/items/hotbar.rs`, `data/start_inventory.json`.
Files: humanoid/src/weapon.rs, humanoid/src/animator.rs, rotland/src/app/start_world.rs, rotland/src/play.rs, game/src/items/hotbar.rs, game/src/items/mod.rs, game/tests/all/*, data/start_inventory.json, tasks/R20-02-bat-in-hand.md
Test: hotbar

1. Bat (issue 002): `weapon_hold_for("bat")` returns no rotation, and the bat model points along +Y, so it hangs down toward the ground. Give the bat (and the other melee two-handers that are built the same way: check `tools/blender/weapons/axe.py`, `crowbar.py`, `katana.py` read only) a hold rotation so it points up and slightly back over the shoulder at rest, like a batter ready to swing. Update the test in `weapon.rs`. Write the angle you chose and why in Worker notes; Claude checks it in a screenshot.
2. Hand vs hotbar (issue 007): today the bat is put in the hand directly while the hotbar comes from `start_inventory.json`, so the player can hold a bat with an empty hotbar. Make the hotbar the only source of truth: the item in the hand is always the equipped hotbar slot (or nothing). At a new game the bat is in slot 1 and equipped. When a save loads and the hand item is not in the hotbar, put it into the first free slot (or the bag if full) and equip it. Remove the direct "set bat on player" path.
3. Tests (pure logic in `game/`): new game -> slot 1 = bat, equipped; loaded save with hand item but empty hotbar -> fixed up; unequip -> hand empty.

## Worker notes

### Angle chosen and rationale (issue 002)
- Angle chosen: `Vec3::new(90.0, 0.0, 0.0)` for "bat", "axe", "crowbar", and "katana" in `weapon_hold_for`.
- Rationale: The weapon models are constructed with their handle/haft along negative Y and barrel/blade/head along positive Y. Hand grip specifies `HAND_GRIP_ROT_DEG` of +90° on X and `REST_POSE_DEG` of -20° on X. Combining with a hold rotation of +90° on X yields a total rotation of 160° on X (`grip * mount * hold`). In the humanoid idle pose, the right arm hangs downward with a slight forward elbow pitch (~22°); rotating the +Y model axis by 160° points the bat/axe/crowbar/katana upward and ~20° backward over the right shoulder at rest, like a batter ready to swing. This matches the standard `[90, 0, 0]` hold angle specified across all weapons in `data/weapons.json`.

### Changes made
1. `humanoid/src/weapon.rs`:
   - Updated `weapon_hold_for()` to return `(Vec3::ZERO, Vec3::new(90.0, 0.0, 0.0))` for `"bat" | "axe" | "crowbar" | "katana"`, tagged `// data: weapons.json knife`.
   - Updated `hold_offsets_for_known_weapons` test to assert the new 90° hold rotation for bat, axe, crowbar, and katana.
2. `game/src/items/hotbar.rs`:
   - Added `Hotbar::new_game(inventory)` to initialize the hotbar for a new game with slot 1 (index 0) equipped.
   - Added `Hotbar::equip_slot()`.
   - Added `Hotbar::fixup_equipped_from_save(hand_item, inventory)` to ensure that on save restore, the hand item is present in the hotbar (or placed into the first free hotbar slot / bag if hotbar is full) and equipped.
3. `rotland/src/play.rs`:
   - Initialized `PlayState::new` with `Hotbar::new_game(&inventory)`.
   - Added `PlayState::equipped_weapon()` and `PlayState::equipped_hand_change()`.
4. `rotland/src/app/start_world.rs`:
   - Removed direct hardcoded `"bat"` initialization on player and combat. Instead, initialize `PlayState` first, then derive `actors.combat` and `actors.player_visual` from the hotbar's equipped weapon (or none if unarmed).
   - On save restore, invoke `play_state.hotbar.fixup_equipped_from_save(saved_equipped, &mut play_state.inventory)` and synchronize the hand via `apply_hand`.
5. `game/tests/all/items_tests.rs`:
   - Added tests for `hotbar_new_game_starts_with_slot_1_bat_equipped`, `hotbar_loaded_save_with_hand_item_but_empty_hotbar_fixed_up`, `hotbar_unequip_leaves_hand_empty`, and `hotbar_loaded_save_moves_from_bag_to_hotbar_or_uses_bag_if_full`.

### How to test
- Run `tools/gate.sh --task tasks/R20-02-bat-in-hand.md`.
- In-game verification: start a new game (`--save-dir=/tmp/scratch_saves`) and verify slot 1 bat is equipped in hand and over shoulder.

### Wiring notes for Claude
- None required; all changes are self-contained within the permitted files.

### Known issues
- None.

## Gate report
- date: 2026-11-22
- base: main
- files changed (6): game/src/items/hotbar.rs, game/tests/all/items_tests.rs, humanoid/src/weapon.rs, rotland/src/app/start_world.rs, rotland/src/play.rs, tasks/R20-02-bat-in-hand.md
- tests passed: 6
- clippy: PASS
- test: PASS
- checks: PASS
- gate: PASS
