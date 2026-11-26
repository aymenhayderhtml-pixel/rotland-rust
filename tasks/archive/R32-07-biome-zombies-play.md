# R32-07: biome zombies in play (models, toxic cloud view, chill, names, dev flag)

Owns: rotland/src/zombies/types.rs, rotland/src/zombies/specials.rs, rotland/src/zombies/spawning.rs, rotland/src/toxic_cloud_view.rs (new), rotland/src/app/frame_sim_zombies.rs (only the display-name match and the new event handling), rotland/src/app/test_flags.rs (only `--spawn-zombie=<kind>`), tools/README_TOOLS.md (one line)
Files: rotland/src/zombies/types.rs, rotland/src/zombies/specials.rs, rotland/src/zombies/spawning.rs, rotland/src/toxic_cloud_view.rs, rotland/src/app/frame_sim_zombies.rs
Read: tasks/R32-round.md, tasks/LEAD.md, rotland/src/zombies/types.rs, rotland/src/zombies/specials.rs, rotland/src/zombies/spawning.rs, rotland/src/zombies/mod.rs, rotland/src/app/frame_sim_zombies.rs, rotland/src/bubbles.rs
API: game_zombie/src/zombie/specials.rs, rotland/src/app/state.rs, rotland/src/app/test_flags.rs
Data: data/zombies.json, data/special_zombies.json
Overlap: `rotland/src/app/test_flags.rs` is owned by R32-03 and `tools/README_TOOLS.md` too. Do not write them: report the lines you need in those two files to the lead instead.
Rules: `tasks/LEAD.md` subagent rules. Use the "R32 shared names" in `tasks/R32-round.md` EXACTLY. R32-06 makes the logic and R32-08 the models at the same time: code against the shared names.

1. **Models:** map kinds `frozen`, `husk` and `bloater` to `zombie_frozen`, `zombie_husk` and `zombie_bloater` the same way `brute` maps to `zombie_brute` (grep `zombie_brute`). Make sure they use the zombie animation set (and for the husk, the crawler-like lunge clip, if specials pick clips by kind).
2. **Display names** (`frame_sim_zombies.rs`, next to `"screamer" => "Screamer"`): "Frozen", "Husk" and "Bloater".
3. **Events:**
   - `ToxicBurst` → spawn into the shared `ToxicClouds` (R32-06), and each frame apply `damage_at(player)` through the normal damage path (cause `"toxic"`) and the zombie dps to zombies in range.
   - `chill` on bite → `TemperaturePlay`'s `BodyTemp::chill(amount)` (R32-05 makes `TemperaturePlay`; report the exact call line for the lead, since the borrow may cross systems).
4. **`toxic_cloud_view.rs`:** draw each cloud as 6-10 slowly drifting, fading green puffs using the `toxic_puff` prop from R32-09 (`assets/models/props/toxic_puff.glb`), scaled up 1.5-3×. Copy how other effects draw props (grep `bubbles.rs` or `explosion`/`spawn` views). The puffs fade by scaling down over the last 1.5 s. Keep it cheap: at most 3 clouds at once, the oldest replaced.
5. **`--spawn-zombie=<kind>`:** at start, spawn one zombie of that kind 5 m in front of the player, idle for 3 s, so screenshots show it. It must work with `--biome=`.
6. Report every shared-file line you need (state fields, mod lines).

## Worker notes

## Neon notes

### What was implemented

* **`rotland/src/zombies/types.rs`**: Added `ToxicBurst`, `ZombieFrame::{toxic_bursts, chill}`, `Zombie::{bloater_burst_done, idle_lock_timer}`, and `is_notable_special_kind` including `"frozen" | "husk" | "bloater"` so `SpecialSeen` fires for all biome zombies (with unit test).
* **`rotland/src/zombies/specials.rs`**: Added `BLOATER_*` constants, `bloater_burst_at`, and `maybe_bloater_burst` (emits a single `ToxicBurst` on bloater death or fire hit). `pose_clip` maps `SpecialPose::Lunge` -> `AnimClip::Attack` for both crawler (using its `clips` override) and husk (using the zombie attack clip).
* **`rotland/src/zombies/spawning.rs`**: Added `default_model_for_kind` fallback mapping `"frozen" => "zombie_frozen"`, `"husk" => "zombie_husk"`, `"bloater" => "zombie_bloater"` (matching `zombie_brute`), initialized the new `Zombie` fields, and added `spawn_idle_showcase` + `spawn_in_front_of_player` (5 m ahead in Godot yaw convention, facing player, idle 3 s).
* **`rotland/src/toxic_cloud_view.rs`**: New module rendering up to 3 clouds (oldest replaced), each with 6–10 slowly drifting `toxic_puff` props scaled 1.5–3×, scaling down over the last 1.5 s, with unit tests.
* **`rotland/src/app/frame_sim_zombies.rs`**: Added `"Frozen"`, `"Husk"`, `"Bloater"` display names; tracked unblocked frozen-zombie bite chill (`FROZEN_BITE_CHILL = 8.0`) and forwarded `frame.toxic_bursts` via `ZombieStepOut`.

### Shared-file lines needed from the lead

1. **`rotland/src/main.rs`**:
```rust
mod toxic_cloud_view;

```


2. **`rotland/src/zombies/mod.rs`**:
* Re-export `ToxicBurst` from `types`:
```rust
pub use types::{
    Look, ToxicBurst, Zombie, ZombieEvent, ZombieEventTracker, ZombieFrame, BODY_RADIUS,
    CORPSE_SECONDS, FAR_DISTANCE, FAR_THINK_INTERVAL, FIRST_ID, FLOW_EVERY, GRAVITY,
};

```


* Inside `Zombies::update_full`, in `for z in &mut self.list`:
* Handle `idle_lock_timer` right before `specials::step` (so `--spawn-zombie` stands idle for 3 s):
```rust
if z.idle_lock_timer > 0.0 && !dead {
    z.idle_lock_timer = (z.idle_lock_timer - dt).max(0.0);
    out.velocity = Vec3::ZERO;
    out.yaw = z.yaw;
    out.anim = AnimClip::Idle;
    out.attack_hit = None;
    out.scream = false;
} else if let (Some(cfg), Some(p), false, true) =
    (self.special_cfg.as_deref(), effective_player, dead, z.special.is_some())
{
    let roll = specials::next_roll(&mut roll_state);
    specials::step(z, cfg, physics, p, dt, roll, &mut out, &mut frame, &mut calls);
}

```


* Emit bloater burst on death (and pass `on_fire = true` from fire-hit path if/when fire hits zombies):
```rust
specials::maybe_bloater_burst(z, false, &mut frame);

```






3. **`rotland/src/app/state.rs`**:
* On `RenderState`:
```rust
pub toxic_cloud_view: crate::toxic_cloud_view::ToxicCloudView,
pub toxic_puff_scratch: Vec<crate::toxic_cloud_view::PuffInstance>,

```


* On `ActorsState` (or `WorldState`, using R32-06's exact `ToxicClouds` path):
```rust
pub toxic_clouds: game::zombie::ToxicClouds,

```




4. **`rotland/src/app/frame_sim.rs`** (update `step_zombies_and_events` call site and wire R32-05 / R32-06):
```rust
let z_out = step_zombies_and_events(/* existing args */);
let (hit_from, big_knock) = (z_out.hit_from, z_out.big_knock);

// R32-05: frozen bite chill -> TemperaturePlay's BodyTemp::chill(amount)
if z_out.chill > 0.0 {
    self.play.temperature.body_temp.chill(z_out.chill);
}

// R32-06 + toxic_cloud_view: spawn bursts, tick clouds, apply player ("toxic") & zombie DPS
for burst in &z_out.toxic_bursts {
    self.actors.toxic_clouds.spawn(
        burst.position,
        burst.radius,
        burst.duration,
        burst.player_dps,
        burst.zombie_dps,
    );
    self.render.toxic_cloud_view.spawn(burst.position, burst.radius, burst.duration);
}
self.actors.toxic_clouds.update(dt);
self.render.toxic_cloud_view.update(dt);
if alive {
    let toxic_dmg = self.actors.toxic_clouds.damage_at(player) * dt;
    if toxic_dmg > 0.0 {
        // Route through the normal player damage path with cause "toxic"
        // e.g. play_state.damage_player(toxic_dmg, "toxic");
    }
}
if let Some(zombies) = self.actors.zombies.as_mut() {
    for z in &mut zombies.list {
        if !z.health.is_dead() {
            let zdmg = self.actors.toxic_clouds.zombie_damage_at(z.position) * dt;
            if zdmg > 0.0 {
                z.health.damage(zdmg);
            }
        }
    }
}

```


*(Verify the exact method names on R32-06's `ToxicClouds`, R32-05's `TemperaturePlay`, `Health::damage`, and the `"toxic"` damage path once R32-05 and R32-06 land — those signatures were not in this task pack.)*
5. **`rotland/src/app/frame_render.rs`** (render `toxic_puff` instances via `PropLibrary` — `props::Placed` fields were not in the task pack):
```rust
self.render.toxic_cloud_view.fill_instances(&mut self.render.toxic_puff_scratch);
for puff in &self.render.toxic_puff_scratch {
    // Push props::Placed for crate::toxic_cloud_view::PROP_NAME ("toxic_puff")
    // using puff.position, puff.yaw, puff.scale (or puff.matrix()).
}

```


6. **`rotland/src/app/test_flags.rs`** (owned by R32-03 — do not overwrite):
* Field on `TestFlags`:
```rust
pub spawn_zombie: Option<String>,

```


* In `TestFlags::from_args(args: &EngineArgs)`:
Parse `--spawn-zombie=<kind>` into `spawn_zombie: Option<String>`.
* In `apply_startup` **after** `--biome=` has positioned the player:
```rust
if let Some(kind) = flags.spawn_zombie.as_deref()
    && let (Some(zombies), Some(pc)) = (actors.zombies.as_mut(), actors.player_ctrl.as_ref())
{
    zombies.spawn_in_front_of_player(kind, pc.position, pc.yaw, &world.physics);
}

```




7. **`tools/README_TOOLS.md`** (owned by R32-03 — one line):
```markdown
- `--spawn-zombie=<kind>`: spawn one zombie of `<kind>` 5 m in front of the player (idle for 3 s, works with `--biome=`).

```
