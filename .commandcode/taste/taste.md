# Taste

## Workflow
- Keep existing tests passing untouched when extending test coverage — append new tests rather than modifying existing ones — and do not change pinned config/data values (e.g. `data/vehicles.json`) unless explicitly asked. Confidence: 0.85
- Long-running/heavy cargo commands in the rotland-rust repo should be serialized via `flock /tmp/rotland-heavy.lock`. Confidence: 0.7
- After implementing a change, run the verification commands and report their actual output/exit codes (e.g. `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, release build) rather than just claiming the work compiles — the user should be able to test it right away. When given verification commands, run them in the given order and report the real output/exit codes. Confidence: 0.9
- When work is organized into passes/iterations with explicit boundaries, stay strictly within the current pass's scope and stop at the boundary until asked — e.g. "do NOT start physics, camera, audio, or HUD changes yet", or "stop after verifying this iteration". Confidence: 0.8

## Communication
- When auditing a codebase, wants an exhaustive list of every issue found (explicitly not a "top 10" summary). Confidence: 0.65
