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
If you can run it: before and after, under the lock, with scratch saves:
```bash
flock /tmp/rotland-heavy.lock cargo build --release
flock /tmp/rotland-heavy.lock target/release/rotland --save-dir=/tmp/r10perf --stats --seconds=60
```
Paste both stats outputs (avg FPS, 1% low / frame time) in notes. Say if the laptop was on battery. If you can't run it, say "not measured" — Claude will.

No behaviour change. Keep the frame order R10-01 documented.

## Worker notes
