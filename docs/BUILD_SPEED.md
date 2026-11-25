# R30.5 Build Speed Report

Hardware: Intel i5-10310U (4c/8t), 14 GB RAM, mold linker, `target-cpu=native`, shared `CARGO_TARGET_DIR`.

## Before (main @ bb131e7)

Measured via `tools/build_speed.sh --release`:

| Case | Time |
|---|---:|
| cargo build (1-line game/src/lib.rs) | 4.40s |
| cargo build (1-line rotland/src/main.rs) | 4.34s |
| cargo build (1-line engine/src/lib.rs) | 4.74s |
| cargo build (1-line humanoid/src/lib.rs) | 4.37s |
| cargo test --no-run (1-line game/src/lib.rs) | 55.95s |
| cargo test (1-line game/src/lib.rs) | 8.35s |
| cargo clippy --all-targets (1-line game/src/lib.rs) | 16.48s |
| cargo build --release (1-line game/src/lib.rs) | 148.17s |

### `cargo build --timings` (clean workspace crates)
- `game`: **25.81s**
- `rotland` (bin): **24.08s**
- `world`: **9.96s**
- `engine`: **9.40s**
- `humanoid`: **9.32s**
- `save`: **7.32s**

## Changes (Kept)
1. **Split `game` and `rotland` (`R30.5-02`):** Split `game` into `game_core`, `game_zombie`, `game_camp`, `game_player`, `game_world` (all free of `engine`/`winit` dependencies) + thin `game` facade, and moved UI modules into `rotland_ui`.
2. **Dev & Play profiles (`Cargo.toml`, `R30.5-03`):** Set `[profile.dev] opt-level = 0` with `opt-level = 2` only for hot crates (`world`, `humanoid`, `physics`, `engine`, `game_zombie`). Added `[profile.play]` (`lto = false`, `codegen-units = 16`, `incremental = true`) writing to `target/play/rotland`, and `[profile.dist]` (`lto = "thin"`, `codegen-units = 1`).

## After (r30.5)

Measured via `tools/build_speed.sh --release`:

| Case | Before | After | Change |
|---|---:|---:|---:|
| cargo build (1-line game/src/lib.rs) | 4.40s | 2.10s | -52% |
| cargo build (1-line rotland/src/main.rs) | 4.34s | 2.11s | -51% |
| cargo build (1-line engine/src/lib.rs) | 4.74s | 2.31s | -51% |
| cargo build (1-line humanoid/src/lib.rs) | 4.37s | 2.45s | -44% |
| cargo test --no-run (1-line game/src/lib.rs) | 55.95s | 27.96s | -50% |
| cargo test (1-line game/src/lib.rs) | 8.35s | 7.10s | -15% |
| cargo clippy --all-targets (1-line game/src/lib.rs) | 16.48s | 17.43s | +6% |
| cargo build --release (1-line game/src/lib.rs) | 148.17s | 116.93s | -21% |
| cargo build --profile play (1-line game/src/lib.rs) | — | 2.24s | 66x faster than release |
