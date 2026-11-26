# R37-01: weapon condition (wear)

Owns: game_core/src/items/stack.rs (the 2 new fields + helpers only), game_core/src/weapons/wear.rs (new), data/weapon_wear.json (new), game/tests/all/weapon_wear_tests.rs (new)
Read: tasks/R37-common.md, game_core/src/items/stack.rs, game_core/src/items/inventory*.rs (how stacks merge/split: stacks with wear or mods must never merge), game_core/src/weapons.rs, data/weapons.json, the save code that writes the inventory (grep `ItemStack` in save/).

- Add `wear`/`mods` to `ItemStack` exactly as in the shared names, with serde defaults so old saves load. Fix every place that builds an `ItemStack` literal (grep) to use `..` or `ItemStack::new`.
- `data/weapon_wear.json`: per weapon (all guns, bows, melee): `wearPerShot` / `wearPerHit`. Aim: a pistol lasts ~400 shots, an AK ~600, the bat ~300 hits, the crossbow ~250 shots. Globals: `jamBelow` 0.3, `jamChanceMax` 0.15, `spreadBelow` 0.5, `spreadMultMax` 1.6, `meleeDamageMultAtZero` 0.5 (melee keeps working but weaker; guns can't fire at 0).
- Pure fns from the shared names, with tests: wear goes down per shot, never below 0; jams only below jamBelow; spread grows smoothly; a fresh stack (`None`) counts as 1.0; old save JSON without the fields loads; stacks with different wear don't merge.
- Report the call sites rotland needs (where a shot / melee hit happens) to the lead. Don't edit rotland.
