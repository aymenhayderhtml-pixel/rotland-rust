# R20-02: hold the bat properly, and make the hotbar decide what is in your hand (issues 002 + 007 part 2)

Read first: `tasks/R20-common.md`, `docs/ISSUES.md` rows 002 and 007 (+ pictures), `humanoid/src/weapon.rs`, `humanoid/src/animator.rs` (~line 270-290, grip IK), `rotland/src/app/start_world.rs` (~lines 120-150: bat set straight on the player), `rotland/src/play.rs` (~line 100, start inventory), `game/src/items/hotbar.rs`, `data/start_inventory.json`.
Files: humanoid/src/weapon.rs, humanoid/src/animator.rs, rotland/src/app/start_world.rs, rotland/src/play.rs, game/src/items/hotbar.rs, game/src/items/mod.rs, game/tests/all/*, data/start_inventory.json, tasks/R20-02-bat-in-hand.md
Test: hotbar

1. Bat (issue 002): `weapon_hold_for("bat")` returns no rotation, and the bat model points along +Y, so it hangs down toward the ground. Give the bat (and the other melee two-handers that are built the same way: check `tools/blender/weapons/axe.py`, `crowbar.py`, `katana.py` read only) a hold rotation so it points up and slightly back over the shoulder at rest, like a batter ready to swing. Update the test in `weapon.rs`. Write the angle you chose and why in Worker notes; Claude checks it in a screenshot.
2. Hand vs hotbar (issue 007): today the bat is put in the hand directly while the hotbar comes from `start_inventory.json`, so the player can hold a bat with an empty hotbar. Make the hotbar the only source of truth: the item in the hand is always the equipped hotbar slot (or nothing). At a new game the bat is in slot 1 and equipped. When a save loads and the hand item is not in the hotbar, put it into the first free slot (or the bag if full) and equip it. Remove the direct "set bat on player" path.
3. Tests (pure logic in `game/`): new game -> slot 1 = bat, equipped; loaded save with hand item but empty hotbar -> fixed up; unequip -> hand empty.

## Worker notes
