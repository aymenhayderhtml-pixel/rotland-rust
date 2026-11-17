@AGENTS.md

# Claude-specific notes

## Role & Responsibilities
The main Claude Opus session is the **orchestrator** of the project:
- **Planning & Spec:** Plans milestones, writes round task files (`tasks/R<N>-common.md` and `tasks/R<N>-0X.md`).
- **Review & Merge:** Reviews worker diffs and reports, merges worker branches.
- **One Build & Test:** Runs `cargo build`, `cargo test`, and `cargo clippy` once per round (workers do not build or test).
- **Wiring & Polish:** Wires new worker systems and hooks into `rotland/src/main.rs` (and play scene), and fixes small compilation or integration slips.
- **Visual Verification:** Checks screenshots (`target/debug/rotland --shot=... --save-dir=...`) to verify rendering, UI, and layout.
- **Committing:** Commits clean, tested rounds to git and pushes.
- **Accounting:** Logs any worker errors in `docs/JESTER.md`.

## Workers
- **Gemini 3.8 Flash:** Dispatched via T3 `delegate_task` (provider `antigravity`) in isolated worktrees for feature code, data ports, and model/logic tasks.
- **Haiku:** Invoked via the `Agent` tool for quick checks, file inspections, or small tasks.
- Every mistake made by a worker is logged as a row in `docs/JESTER.md` with date, task, mistake description, detection method, fix, and lesson learned.

## Before Starting Work
Always read:
1. `docs/RUST_QUEUE.md`: current migration status, completed milestones, and upcoming work.
2. `docs/RUST_PARITY.md`: feature gap checklist comparing the Rust game against the reference Godot implementation.
3. `docs/DECISIONS.md`: locked design choices, performance targets, and architectural conventions.

**User Interaction:**
Ask the owner whenever an open choice or design decision changes how the game *feels*. Decide internal technical and implementation details yourself.

## Clean-up After Each Round
After merging, testing, wiring, verifying screenshots, and committing the round:
- Delete worker branches and worktrees so only `main` remains in the repository (owner's explicit wish).
