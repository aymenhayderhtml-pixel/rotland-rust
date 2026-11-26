# R36-05: explosion effect + backpack you can see

Owns: rotland/src/explosion_view.rs, rotland/src/backpack_view.rs, rotland/src/player_visual.rs, data/effects.json (new or existing: explosion numbers), tools/blender/props/explosion_puff.py (new, optional), assets/models/props/explosion_puff.glb (optional)
Rules: `tasks/R36-common.md`.

**1. Explosion (`rotland/src/explosion_view.rs`):** `ExplosionView` with `spawn(&mut self, pos: Vec3, radius: f32)`, `update(dt)` and a draw method that fits how other small effect views draw (look at `rotland/src/bubbles.rs` or `projectiles_view.rs` for the pattern). The look:
- a bright flash light (warm orange, radius 3x the blast radius, fades over 0.25 s)
- a low-poly fireball that grows to ~0.6x radius in 0.15 s, then turns into dark grey smoke puffs that rise and fade over 2 s
- a camera shake if the player is within 3x the radius (use the existing camera shake/trauma if there is one; otherwise a small one in this file)
- a dark scorch circle on the ground that fades over 20 s.
Low cost: at most 4 explosions at once, pooled, and no allocation per frame (laptop has integrated graphics). Numbers go in `data/effects.json`. Unit-test the timing/fade math.

**2. Backpack on the player (parity: Godot `scripts/Player/PlayerBackpack.cs`):** models `assets/models/gear/backpack_small.glb` and `backpack_large.glb` already exist. When the player's bag item is a small or large backpack (find how the bag/backpack item is stored in the inventory), draw that model attached to the `chest` bone (offset so it sits on the back, not through the body). Draw nothing without a backpack, and hide it in first person. Put it in `rotland/src/backpack_view.rs` and call it from `player_visual.rs`. Put the offset in data (subagents don't run the game); the lead checks it with a screenshot.

## Worker notes (Part 2: Backpack)

### Files changed
- `data/backpack.json` (new): `chest` bone attachment offset `[0.0, 0.14, 0.10]` and rotation `[0.0, 0.0, 0.0]` for `backpack_small` and `backpack_large` (matching `tools/blender/gear/backpacks.py` and `docs/RIG_CONTRACT.md`).
- `rotland/src/backpack_view.rs` (new): `BackpackKind`, `BackpackAttach`, `BackpackConfig`, `backpack_local_matrix`, `backpack_world_matrix`, `select_backpack`, `BackpackView`, and 3 unit tests (`select_backpack_picks_none_small_or_large`, `backpack_config_parses_json`, `backpack_world_matrix_attaches_behind_chest_bone`).
- `rotland/src/player_visual.rs`: added `chest_joint: Option<usize>` field (via `rig.joint_index("chest")`) and `pub fn chest_joint(&self) -> Option<usize>` accessor (kept free of `crate::backpack_view` imports so `rotland/examples/{pose_check,humanoid_view}.rs` continue to compile).

### Lines for the lead to add to shared files
1. `rotland/src/main.rs` (module list):
```rust
mod backpack_view;
```
2. `rotland/src/app/state.rs` (`RenderState` or `ActorsState`, e.g. on `ActorsState` alongside `player_visual`):
```rust
pub backpack_view: Option<crate::backpack_view::BackpackView>,
```
3. `rotland/src/app/start_world.rs` (alongside `actors.player_visual = Some(...)`, ~line 162):
```rust
actors.backpack_view = Some(crate::backpack_view::BackpackView::load(&gpu.device, pack));
```
(and `actors.backpack_view = None;` when resetting `actors.player_visual = None;`, ~line 49).
4. `rotland/src/app/render_frame.rs` (inside the on-foot `!pose.first_person` branch after `let (body, weapon) = pv.pose(...)` ~line 333):
```rust
if let Some(bv) = actors.backpack_view.as_ref() {
    let kind = play
        .play
        .as_ref()
        .and_then(|p| crate::backpack_view::select_backpack(&p.inventory));
    let char_world = Mat4::from_translation(pose.position) * Mat4::from_rotation_y(pose.yaw);
    let bone_base = body.instances[0].bone_base as usize;
    if let Some(pack_batch) = bv.draw_batch(pv, kind, pose.first_person, char_world, &render.bones, bone_base) {
        main.push(engine::render::SkinnedBatch {
            mesh: pack_batch.mesh,
            instances: pack_batch.instances.clone(),
        });
        shadow.push(pack_batch);
    }
}
```
(Optional: same in the `driven_car` branch ~line 279 using `seat` as `char_world` if the backpack should also show while driving.)

### Known issues / notes
- `data/items.json` currently has no `backpack_small` or `backpack_large` item definitions and `Inventory` (`game_core/src/items/inventory.rs`) has no dedicated `bag_id` field, so `select_backpack(&p.inventory)` checks `inv.has("backpack_large", 1)` / `inv.has("backpack_small", 1)` and returns `None` until those items are added (or when testing with `Some(BackpackKind::Small)` / `Some(BackpackKind::Large)`).
- Not built or run by subagent (per R36 rules).

