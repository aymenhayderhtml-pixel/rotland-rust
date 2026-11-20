# R10-10: Stop allocating memory every frame (GitHub #16) — PHASE B

**Start only after Claude merged R10-01 into main** (`git merge main` in your worktree first). The line numbers in issue #16 are from the old main.rs; find the same code in the new `rotland/src/app/` files (R10-01's "Where to wire things" table helps).

Target laptop: Intel UHD, steady 30+ FPS. Goal: zero new heap allocations per frame in normal play.

## You own
- The per-frame files under `rotland/src/app/` that contain the code below (not the save/load file — R10-09 owns that)
- `rotland/src/props.rs`, `rotland/src/streaming.rs` — only for the dog-sites cache hook (PERF-003)

## Fix
1. **PERF-001**: zombie positions, noise lists, car solids (with cloned id strings) collected into new `Vec`s every frame → keep reusable scratch buffers in the owning struct (`clear()` + `extend`), or pass slices/iterators. Car solids: use an index or `&str`, not cloned `String`s.
2. **PERF-002**: bag item ids copied into a new `Vec<String>` each frame for hints → only rebuild when the inventory changes (use a change counter / dirty flag; if the inventory has none, add a cheap `generation: u32` — that's `game/src/items/` which R10-08 owns, so write it as a wiring note instead and use a local hash/length check meanwhile).
3. **PERF-003**: dog POI sites rebuilt over all loaded chunks every frame → rebuild only when a chunk loads/unloads (hook in streaming).
4. **PERF-004**: shadow pass clones each skinned batch's instance list → borrow it.
5. Search the per-frame files for any other `collect::<Vec<_>>()`, `.clone()` of Vecs/Strings, `format!` in hot loops, and list them in notes (fix the easy ones).

## Measure
Write-only round: don't build or run. Claude measures before/after with `--stats`.

No behaviour change. Keep the frame order R10-01 documented.

## Worker notes

### Files changed
- `rotland/src/streaming.rs`: Added cached `dog_sites` and `ladders` collections rebuilt only when chunks load/unload (`changed || applied`); added generation counter; replaced `wanted()` 49-element `Vec` allocations with inlined loops; added scratch buffers (`far_scratch`, `missing_scratch`, `rx_scratch`).
- `rotland/src/app/state.rs`: Added persistent scratch vectors to `RenderState` (`scratch_beds`), `WorldState` (`near_crates_scratch`, `poi_zombie_sites`, `tracked_zombie_scratch`, `cleared_poi_scratch`, `camps`), `ActorsState` (`bites_scratch`, `zombie_pos_scratch`, `voice_zombies_scratch`, `guard_posts_scratch`, `guard_targets_scratch`, `zombie_status_scratch`, `noises_scratch`, `animal_status_scratch`, `car_solids_scratch`, `car_targets_scratch`, `car_gone_scratch`), and `PlaySide` (`hint_ctx`, `hint_inventory_hash`, `active_hint_id`).
- `rotland/src/app/frame_camp.rs`: Persistent `hint_ctx` reused in `update_camp_and_stats`; `npc_distances` and `needs` updated in-place without `String` cloning; inventory slots hashed so `inventory_items` set is only rebuilt on change; `fill_keys` text template only runs when active hint changes; replaced `def.id.clone()` in `play.skill_seen` with `get_mut`.
- `rotland/src/app/frame_cars.rs`: Updated `car_ramming` to accept scratch buffers for `solids_scratch`, `targets_scratch`, and `gone_scratch`; reuses `String` capacities in-place via `clear()` + `push_str()`.
- `rotland/src/app/frame_sim.rs`: Converted `update_sim` to zero per-frame allocations using `std::mem::take` pattern on scratch buffers; cached `world.poi_zombie_sites` (invalidated when streamer generation increments); reused zombie and animal status scratch slices across wildlife and dog updates; reused guard posts/targets scratch buffers; took dog sites directly from `streamer.dog_sites` slice.
- `rotland/src/app/frame_actors.rs`: Replaced per-frame `chunks.values().flat_map(...).collect()` for ladders with borrowing `streamer.ladders.as_slice()`.
- `rotland/src/app/frame_world.rs`: Replaced crate queries `Vec<LootSpot>` allocation with `world.near_crates_scratch`.
- `rotland/src/app/frame_ui.rs`: Cached `world.camps` lazily so settlement `Vec<Vec2>` is never allocated per frame while hub is open.
- `rotland/src/app/frame_input.rs`: Avoided calling `input.pressed_keys()` (which allocates a `Vec<Key>`) when UI menus are closed.
- `rotland/src/app/render_frame.rs`: Reused `render.scratch_beds` for farm bed placement instead of allocating a new `Vec<Placed>` per frame.

### New public API
- `Streamer::dog_sites: Vec<DogPoiSite>` — exposed slice of all dog POI sites on loaded chunks.
- `Streamer::ladders: Vec<LadderRail>` — exposed slice of all ladder rails on loaded chunks.
- `Streamer::generation: u64` — monotonic revision counter incremented on chunk load/unload.
- `Streamer::rebuild_dog_sites(&mut self)` — updates `dog_sites` from loaded chunks.
- `Streamer::rebuild_ladders(&mut self)` — updates `ladders` from loaded chunks.
- `car_ramming(...)` — updated signature to take scratch buffers:
  `solids_scratch: &mut Vec<(String, Mat4, bool)>`, `targets_scratch: &mut Vec<(u64, Vec3)>`, `gone_scratch: &mut Vec<glam::IVec2>`.

### Wiring for Claude
1. **PERF-002 (`generation: u32` in `Inventory`)**:
   `game/src/items/inventory.rs` is owned by R10-08, so we did not modify it. We currently fingerprint inventory state using a fast, zero-allocation `DefaultHasher` over `inventory.slots()`. Once R10-08 adds a dirty flag or `generation: u32` to `Inventory`, `update_camp_and_stats` in `rotland/src/app/frame_camp.rs` can directly check `inventory.generation != play.hint_inventory_gen` instead of hashing.
2. **PERF-004 (`Cow<'a, [SkinInstance]>` in `engine::render::SkinnedBatch`)**:
   In `rotland/src/app/render_frame.rs`:
   `shadow.extend(zombies.iter().map(|b| SkinnedBatch { mesh: b.mesh, instances: b.instances.clone() }));`
   `SkinnedBatch` in `engine/src/render/skinned.rs` has field `pub instances: Vec<SkinInstance>`. Since `engine/` is forbidden from modification in our file list, we cannot change `instances` to `Cow<'a, [SkinInstance]>` or `&'a [SkinInstance]` without editing engine files. We recommend Claude or the engine owner change `SkinnedBatch.instances` to `Cow<'a, [SkinInstance]>` so shadow pass can borrow `Cow::Borrowed(&b.instances)` with zero cloning.

### List of allocations removed and any left
- **Removed**:
  - PERF-001: Car solids `Vec<(String, Mat4, bool)>` with cloned id strings -> scratch buffer with in-place `push_str`.
  - PERF-001: Car targets `Vec<(u64, Vec3)>` -> `car_targets_scratch`.
  - PERF-001: Car gone chunk keys `Vec<IVec2>` -> `car_gone_scratch`.
  - PERF-001: Zombie positions `Vec<(u64, Vec3)>` for sounds -> `zombie_pos_scratch`.
  - PERF-001: Zombie voices `Vec<VoiceZombie>` -> `voice_zombies_scratch`.
  - PERF-001: Guard posts `Vec<GuardPost>` -> `guard_posts_scratch` with in-place id updates.
  - PERF-001: Guard targets `Vec<GuardTarget>` -> `guard_targets_scratch`.
  - PERF-001: Zombie status `Vec<(u64, Vec3, bool)>` -> `zombie_status_scratch` (shared between wildlife & dogs).
  - PERF-001: Noises `Vec<(Vec3, f32)>` -> `noises_scratch`.
  - PERF-001: Animal status `Vec<(u64, Vec3, bool)>` -> `animal_status_scratch`.
  - PERF-001: Zombie bites `Vec<f32>` -> `bites_scratch`.
  - PERF-001: Ladder rails `Vec<LadderRail>` -> cached in `Streamer`, borrowed as slice `&[LadderRail]`.
  - PERF-001: Crate interaction `Vec<LootSpot>` -> `near_crates_scratch`.
  - PERF-001: Farm beds `Vec<Placed>` -> `scratch_beds`.
  - PERF-001: Menu input `input.pressed_keys()` -> skipped when menu is closed.
  - PERF-001: Settlement camps `Vec<Vec2>` in hub UI -> cached lazily in `world.camps`.
  - PERF-001: Skill seen `play.skill_seen.insert(def.id.clone(), ...)` -> `get_mut` to avoid cloning string key every frame.
  - PERF-002: Bag item IDs `Vec<String>` in hint context -> cached, only updated when inventory hash changes; `npc_distances` and `needs` updated in-place; `fill_keys` string template executed only when hint changes.
  - PERF-003: Dog POI sites rebuilt across all chunks -> cached on chunk load/unload in `Streamer::dog_sites`, borrowed as slice `&[DogPoiSite]`.
  - PERF-003: POI zombie sites -> cached in `world.poi_zombie_sites`, invalidated by `streamer.generation`.
  - PERF-003: Streamer unload `far` chunk collection -> `far_scratch`.
  - PERF-003: Streamer `wanted()` 49-element `Vec<IVec2>` -> inline nested coordinate loops.
  - PERF-003: Streamer missing chunks -> `missing_scratch`.
  - PERF-003: Streamer rx try_iter collection -> `rx_scratch`.
- **Left**:
  - PERF-004: SkinnedBatch instances clone for shadow pass (requires changing `engine/src/render/skinned.rs` to use `Cow`, noted for Claude above).
  - Toast string clone when new toast is shown (rare event, not per-frame).
  - Level-up overlay trigger string clone when skill level increases (rare event).

### Verification
- not checked (write-only round)

### Questions / known issues
- None. Frame ordering, logic flow, and exact Godot/R10 behavior have been strictly preserved.
