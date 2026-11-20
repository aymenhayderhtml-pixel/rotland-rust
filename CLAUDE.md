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
- **Routing:** see `docs/PLAN_0.2.md` "AI roles" (DeepSeek = main repo worker, Gemini Flash = script/render-verifiable work, Neon = chat-only via `tools/neon_pack.py`). All dispatched via T3 `delegate_task` in worktrees; at most 4 at once.
- **Haiku:** shares Opus's limit; tiny lookups only.
- Every mistake made by a worker is logged as a row in `docs/JESTER.md` with date, task, mistake description, detection method, fix, and lesson learned.

## Before Starting Work
Read `docs/STATUS.md` only (short). Open `docs/PLAN_0.2.md`, `docs/DECISIONS.md`, `docs/PARITY_OPEN.md` or `docs/JESTER.md` only when the task needs them. Keep STATUS.md current (under ~80 lines) at the end of each round.

**User Interaction:**
Ask the owner whenever an open choice or design decision changes how the game *feels*. Decide internal technical and implementation details yourself.

## Clean-up After Each Round
After merging, testing, wiring, verifying screenshots, and committing the round:
- Delete worker branches and worktrees so only `main` remains in the repository (owner's explicit wish).
