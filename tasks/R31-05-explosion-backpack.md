# R31-05: explosion effect + backpack you can see

Owns: rotland/src/explosion_view.rs, rotland/src/backpack_view.rs, rotland/src/player_visual.rs, data/effects.json (new or existing: explosion numbers), tools/blender/props/explosion_puff.py (new, optional), assets/models/props/explosion_puff.glb (optional)
Rules: `tasks/R31-common.md`.

**1. Explosion (`rotland/src/explosion_view.rs`):** `ExplosionView` with `spawn(&mut self, pos: Vec3, radius: f32)`, `update(dt)` and a draw method that fits how other small effect views draw (look at `rotland/src/bubbles.rs` or `projectiles_view.rs` for the pattern). The look:
- a bright flash light (warm orange, radius 3x the blast radius, fades over 0.25 s)
- a low-poly fireball that grows to ~0.6x radius in 0.15 s, then turns into dark grey smoke puffs that rise and fade over 2 s
- a camera shake if the player is within 3x the radius (use the existing camera shake/trauma if there is one; otherwise a small one in this file)
- a dark scorch circle on the ground that fades over 20 s.
Low cost: at most 4 explosions at once, pooled, and no allocation per frame (laptop has integrated graphics). Numbers go in `data/effects.json`. Unit-test the timing/fade math.

**2. Backpack on the player (parity: Godot `scripts/Player/PlayerBackpack.cs`):** models `assets/models/gear/backpack_small.glb` and `backpack_large.glb` already exist. When the player's bag item is a small or large backpack (find how the bag/backpack item is stored in the inventory), draw that model attached to the `chest` bone (offset so it sits on the back, not through the body). Draw nothing without a backpack, and hide it in first person. Put it in `rotland/src/backpack_view.rs` and call it from `player_visual.rs`. Put the offset in data (subagents don't run the game); the lead checks it with a screenshot.
