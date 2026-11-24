# A17 (Neon): zombies that blend, keep their feet in time and die by direction

Files: game/src/zombie/anim_fade.rs, game/tests/all/zombie_anim_fade.rs, data/zombie_anim.json, tools/blender/characters/poses_zombie_v2.py, tasks/A17-zombie-anim.md, game/src/zombie/mod.rs, game/tests/all/main.rs, rotland/src/zombies/rendering.rs, rotland/src/zombies/types.rs, rotland/src/zombies/mod.rs, rotland/src/zombies/combat_glue.rs, rotland/src/zombies/spawning.rs, tools/blender/characters/poses.py, tools/blender/characters/build_characters.py
Read: rotland/src/zombies/rendering.rs, rotland/src/zombies/types.rs, rotland/src/zombies/combat_glue.rs, engine/src/anim.rs, tools/blender/characters/poses_air.py
API: game/src/zombie/mod.rs, game/src/zombie/brain.rs, rotland/src/zombies/mod.rs, rotland/src/zombies/spawning.rs

Rotland is a low-poly 3D zombie survival game in Rust (custom engine). This is job 17 of 20 of a deep character animation system. Zombies do not use the player's animator; each zombie samples exactly one clip per frame (`rotland/src/zombies/rendering.rs`, clip chosen by `clip_name` in `types.rs`, time `z.anim_time` reset to 0 on every clip change in `rotland/src/zombies/mod.rs`). There can be 100+ zombies on a laptop with integrated graphics, so this must stay cheap.

**The problem:**
1. Clip changes **pop** instantly (walk to attack to stagger), which looks robotic.
2. Walk and run clips play at a fixed speed whatever the real speed, so **feet slide**.
3. Every death is the same `die` clip, falling the same way whatever direction the hit came from.

**After this job:**
- Clip changes **crossfade** (`rig.sample_blend` already exists in `engine/src/anim.rs`). Fade time per change from data (short into attack/hit, longer between idle/walk/run). Death fades in quickly.
- Walk/run/crawl playback **matches the real ground speed** (`z.velocity` horizontal) to the clip's reference speed from data, clamped.
- **Deaths by direction:** the knockback direction of the last hit (already passed to `Zombies::hit`) is stored; on death, hit from the front → `zombie_die_back` (falls backward), from behind → `zombie_die_forward`; otherwise the old `die`. Crawler and specials keep their own death clips (their `def.clips` override, see `clip_name`).
- **LOD:** zombies farther than `blendMaxDistance` from the camera skip crossfading (single sample as today).
- Gameplay unchanged: brain, speeds, hits, damage and timing of attacks stay the same (`anim_time` is still what the game uses for its own timing if it does; if you change how it advances, keep a separate visual time and leave `anim_time` as it is).

## Output rules (important)
- Output **complete files only for**: `game/src/zombie/anim_fade.rs`, `game/tests/all/zombie_anim_fade.rs`, `data/zombie_anim.json`, `tools/blender/characters/poses_zombie_v2.py`, and `rotland/src/zombies/rendering.rs` (it is short).
- **Do not output** other files. In `=== NOTES ===`, give **every** edit for the rest (`game/src/zombie/mod.rs`, `game/tests/all/main.rs`, `rotland/src/zombies/types.rs`, `mod.rs`, `combat_glue.rs`, `spawning.rs`, `poses.py`, `build_characters.py`): file, exact existing lines to find (copied exactly, unique in that file), and the complete replacement. A helper applies them literally. If you cannot see an exact line you need, write `// HELPER:` saying what to find.
- **Keep the whole answer under 40,000 characters.** `anim_fade.rs` under 220 lines, tests under 170 lines. Do not stop until the final line `END OF EDITS`.
- No new crates. No sounds. **No allocation per frame per zombie** (state is `Copy`; the rendering may reuse one scratch buffer owned by `Zombies` if it needs one).

## `game/src/zombie/anim_fade.rs` (pure logic, glam + serde only)
```rust
pub struct ZombieAnimConfig { /* serde camelCase; Default = the json values */ }
impl ZombieAnimConfig { pub fn from_json(text: &str) -> Result<Self, serde_json::Error>; pub fn from_repo() -> Option<Self>; }
/// Per-zombie visual animation state (Copy). Clips are identified by a u64 key (hash of the clip name).
#[derive(Clone, Copy, Debug, Default)]
pub struct AnimFade { /* current key + visual time, previous key + time, fade left/total */ }
impl AnimFade {
    /// Call every frame with the clip chosen this frame. Starts a fade when the key changes.
    pub fn update(&mut self, dt: f32, key: u64, rate: f32, fade_seconds: f32);
    pub fn time(&self) -> f32;
    /// (previous key, previous time, weight of the CURRENT clip 0..1) while fading, else None.
    pub fn blend(&self) -> Option<(u64, f32, f32)>;
}
pub fn clip_key(name: &str) -> u64; // stable FNV-1a
/// Playback rate for a locomotion clip from ground speed (clamped by config).
pub fn locomotion_rate(speed: f32, ref_speed: f32, cfg: &ZombieAnimConfig) -> f32;
/// Death clip from the hit direction (flat knockback) and the zombie's yaw (forward = (-sin yaw, 0, -cos yaw)).
pub fn death_clip(knock_dir: glam::Vec3, yaw: f32) -> &'static str; // "zombie_die_back" | "zombie_die_forward" | "die"
```
A fade during a fade starts from the blended pose's dominant clip (no long chains). Finite at dt 0 and 0.25.

## `data/zombie_anim.json`
Fade seconds (default, into attack, into hit, into death, between locomotion), reference speeds for `walk`, `zombie_shamble`, `zombie_lurch`, `run`, `zombie_runner_sprint`, `zombie_crawl`, `zombie_crawler_crawl` (the speed in m/s each clip looks right at; guess from the clips), min/max rate (0.6 / 1.6), `blendMaxDistance` (25 m), front/back death angle (60 degrees from the facing). Choose values and list them.

## New clips: `tools/blender/characters/poses_zombie_v2.py`
Same format as `poses_air.py`; export `CLIPS`; names start with `zombie_` (zombie glbs only). **Sign rules (a previous answer got these backwards):** hips `rot` X negative = lean forward, hips `loc` Y negative = lower; spine/chest/head X negative = bend forward; `upper_arm_*` X positive = arm forward/up (0 = hanging); `forearm_*` X positive = elbow bends; `thigh_*` X positive = leg forward; `shin_*` X **negative** = knee bends. Zombies have a hunched rest posture added by the builder; key the body relative to it.
- `zombie_die_forward` (no loop, 1.3 s): pushed from behind, stumbles forward, falls flat on the face, arms limp. Ends lying on the ground (no part below it).
- `zombie_die_back` (no loop, 1.3 s): hit from the front, knees buckle, falls on the back. Ends lying on the ground.
Edits: add the import to `poses.py` and merge into `CLIPS`; add both names to `ZOMBIE_CLIPS` in `build_characters.py`.

## Wiring (you write these as NOTES edits)
- `Zombie` gets `pub fade: AnimFade` and `pub last_knock: Vec3` (set in `spawning.rs` to defaults). In `combat_glue.rs` `hit()`, store the flat knockback direction in `last_knock` when non-zero.
- `rendering.rs` (full file): for each zombie choose the clip as today, but use `death_clip` for `AnimClip::Die` when the zombie's own clip is plain `die` and the look has that clip; call `fade.update` (rate from `locomotion_rate` for locomotion clips, else 1); sample with `sample_blend` while fading and within `blendMaxDistance` of the camera position (add a `camera: Vec3` parameter to `pose`, update its caller with a `// HELPER:` note if you cannot see it), else `sample`. Death clips hold their last frame. Since `pose` takes `&self`, update the fades where zombies are updated each frame (`mod.rs`, next to `z.anim_time += dt;`) and only read them in `rendering.rs`; the clip name and rate needed there must then be computed in `mod.rs` too (move `clip_name` use as needed; the look's clip list is reachable through `self.looks[z.look]`).

## Tests: `game/tests/all/zombie_anim_fade.rs`
Compact: data parses and equals Default; same key = no fade; key change = fade from weight 0 to 1 over the fade time; fade during fade stays finite and ends; time advances by rate; locomotion rate clamps; death_clip front/back/side cases with yaw 0 and yaw PI/2; clip_key stable and different for different names; no NaN at dt 0 and 0.25.

Every new number in Rust code needs a source tag on its line (`// data: zombie_anim.json key` or `// new: DECISIONS 2026-11-24`).

## Worker notes

- Applied all Neon NOTES edits 1-9 directly (game/src/zombie/mod.rs, game/tests/all/main.rs, rotland/src/zombies/types.rs, combat_glue.rs, spawning.rs, rotland/src/zombies/mod.rs, tools/blender/characters/poses.py, build_characters.py ZOMBIE_CLIPS).
- Extra caller change (note 7): `rotland/src/app/render_frame.rs` now calls `z.pose(render.camera.position, &mut render.bones)`.
- `poses_zombie_v2.py` sign rules were checked by reading: hips X negative = forward, spine/chest/head X negative = forward, shin X negative = knee bends. Die forward uses hips X to -90 and die back uses +90.
- Clippy `--all-targets`: 0 warnings. `cargo test`: all pass, including the 9 zombie_anim_fade tests.
- Blender rebuild of the zombie looks (zombie_01, 02, 03, brute, screamer, crawler) is run one at a time. Death clips fall back to plain `die` until the looks are rebuilt.
- Owner files untouched: car_hatchback_*, settlement.*, car_parts.py.
- Visual check of `assets/models/_previews/zombie_01_zombie_die_forward.png`: the fall overshoots. The last frames have the legs raised and the head low, so it is NOT a flat face-down fall yet. Needs a pose fix (hips rot / leg keys in `poses_zombie_v2.py`, then rebuild zombie looks and bake).
- Visual check of `assets/models/_previews/zombie_01_zombie_die_back.png`: the body ends at an angle (about 45 degrees, legs down), not flat on the back. Also needs a pose fix.
- Gate run: clippy PASS, test PASS (971), checks FAIL only on scope (owner files, plus `data/car_anim.json`, `humanoid/src/car_anim*.rs`, `tasks/A11-car-seat.md`, which I did not touch).

## Neon notes

### 1. `game/src/zombie/mod.rs`

// HELPER: Add `pub mod anim_fade;` alongside the existing `pub mod` declarations in `game/src/zombie/mod.rs`.

### 2. `game/tests/all/main.rs`

// HELPER: Add `mod zombie_anim_fade;` alongside the existing `mod` declarations in `game/tests/all/main.rs`.

### 3. `rotland/src/zombies/types.rs`

Find:

```rust
use game::zombie::{AnimClip, AttackHit, BrainOutput, Noise, SpecialState, ZombieBrain, ZombieDef};

```

Replace with:

```rust
use game::zombie::anim_fade::{AnimFade, clip_key, death_clip};
use game::zombie::{AnimClip, AttackHit, BrainOutput, Noise, SpecialState, ZombieBrain, ZombieDef};

```

Find:

```rust
impl Look {
    pub fn clip(&self, name: &str) -> Option<&pack::Clip> {
        self.clips.iter().find(|c| c.name == name)
    }
}

```

Replace with:

```rust
impl Look {
    pub fn clip(&self, name: &str) -> Option<&pack::Clip> {
        self.clips.iter().find(|c| c.name == name)
    }

    pub fn clip_by_key(&self, key: u64) -> Option<&pack::Clip> {
        self.clips.iter().find(|c| clip_key(&c.name) == key)
    }
}

```

Find:

```rust
    pub anim: AnimClip,
    pub anim_time: f32,
    pub dead_for: f32,

```

Replace with:

```rust
    pub anim: AnimClip,
    pub anim_time: f32,
    pub fade: AnimFade,
    pub last_knock: Vec3,
    pub dead_for: f32,

```

Find:

```rust
    if let Some(c) = z.def.clips.get(default_name) {
        return c.as_str();
    }
    default_name
}

```

Replace with:

```rust
    if let Some(c) = z.def.clips.get(default_name) {
        return c.as_str();
    }
    default_name
}

/// Resolves the clip name including directional death clips when the look provides them.
pub(super) fn resolved_clip_name<'a>(z: &'a Zombie, look: &Look) -> &'a str {
    let base = clip_name(z);
    if z.anim == AnimClip::Die && base == "die" {
        let directional = death_clip(z.last_knock, z.yaw);
        if look.clip(directional).is_some() {
            return directional;
        }
    }
    base
}

```

### 4. `rotland/src/zombies/combat_glue.rs`

Find:

```rust
        if result.dealt > 0.0 {
            z.knock += Vec3::new(knockback.x, 0.0, knockback.z);
            z.pending_hit = Some(damage);
        }

```

Replace with:

```rust
        if result.dealt > 0.0 {
            let flat_knock = Vec3::new(knockback.x, 0.0, knockback.z);
            if flat_knock.length_squared() > 0.0 {
                z.last_knock = flat_knock;
            }
            z.knock += flat_knock;
            z.pending_hit = Some(damage);
        }

```

### 5. `rotland/src/zombies/spawning.rs`

// HELPER: In every `Zombie { ... }` struct literal in `rotland/src/zombies/spawning.rs`, add:

```rust
fade: game::zombie::anim_fade::AnimFade::default(),
last_knock: Vec3::ZERO,

```

### 6. `rotland/src/zombies/mod.rs`

// HELPER:

1. Add `pub anim_cfg: game::zombie::anim_fade::ZombieAnimConfig,` to `pub struct Zombies`, initialized with `anim_cfg: game::zombie::anim_fade::ZombieAnimConfig::from_repo().unwrap_or_default(),` in `Zombies::new`.
2. In the per-frame zombie update loop, record `let prev_anim = z.anim;` before `z.anim` is updated, and right next to `z.anim_time += dt;`, update `z.fade`:

```rust
let look = &self.looks[z.look];
let name = types::resolved_clip_name(z, look);
let speed = glam::Vec2::new(z.velocity.x, z.velocity.z).length();
let rate = self
    .anim_cfg
    .ref_speed(name)
    .map_or(1.0, |ref_s| {
        game::zombie::anim_fade::locomotion_rate(speed, ref_s, &self.anim_cfg)
    });
let fade_secs = self.anim_cfg.fade_seconds_for(prev_anim, z.anim);
z.fade.update(dt, game::zombie::anim_fade::clip_key(name), rate, fade_secs);

```

### 7. Caller of `Zombies::pose`

// HELPER: Find the call site of `zombies.pose(&mut bones)` in `rotland/src/` and pass the camera world position as the first argument: `zombies.pose(camera_pos, &mut bones)`.

### 8. `tools/blender/characters/poses.py`

// HELPER: Import `CLIPS as ZOMBIE_V2_CLIPS` from `poses_zombie_v2` and merge `ZOMBIE_V2_CLIPS` into `CLIPS` (e.g. `CLIPS.update(ZOMBIE_V2_CLIPS)`).

### 9. `tools/blender/characters/build_characters.py`

// HELPER: Add `"zombie_die_forward"` and `"zombie_die_back"` to `ZOMBIE_CLIPS`.

END OF EDITS

## Gate report
- date: 2026-11-24
- base: main
- files changed (40): assets/models/buildings/settlement.glb, assets/models/characters/zombie_01.glb, assets/models/characters/zombie_02.glb, assets/models/characters/zombie_03.glb, assets/models/characters/zombie_brute.glb, assets/models/characters/zombie_crawler.glb, assets/models/characters/zombie_screamer.glb, assets/models/_previews/car_hatchback_blue.png, assets/models/_previews/car_hatchback_blue_rear.png, assets/models/_previews/car_hatchback_blue_side.png, assets/models/_previews/car_hatchback_olive.png, assets/models/_previews/car_hatchback_olive_rear.png, assets/models/_previews/car_hatchback_olive_side.png, assets/models/_previews/car_hatchback_red.png, assets/models/_previews/car_hatchback_red_rear.png, assets/models/_previews/car_hatchback_red_side.png, assets/models/vehicles/car_hatchback_blue.glb, assets/models/vehicles/car_hatchback_olive.glb, assets/models/vehicles/car_hatchback_red.glb, data/car_anim.json, data/zombie_anim.json, game/src/zombie/anim_fade.rs, game/src/zombie/mod.rs, game/tests/all/main.rs, game/tests/all/zombie_anim_fade.rs, humanoid/src/car_anim.rs, humanoid/src/car_anim_tests.rs, rotland/src/app/render_frame.rs, rotland/src/zombies/combat_glue.rs, rotland/src/zombies/mod.rs, rotland/src/zombies/rendering.rs, rotland/src/zombies/spawning.rs, rotland/src/zombies/types.rs, tasks/A11-car-seat.md, tasks/A17-zombie-anim.md, tools/blender/buildings/settlement.py, tools/blender/characters/build_characters.py, tools/blender/characters/poses.py, tools/blender/characters/poses_zombie_v2.py, tools/blender/vehicles/car_parts.py
- tests passed: 971
- clippy: PASS
- test: PASS
- checks: FAIL
- gate: FAIL (checks)
