# AI Workflow: Rotland Rust Development

```
 Owner (approves look & feel, reviews play runs, plays release builds)
   │
 Claude Opus = ORCHESTRATOR
   │  plans milestones, writes tasks, merges, single build+test pass, wires main.rs, commits & pushes
   ├── Haiku (cheap & fast)  → quick checks, file inspections, small verification tasks
   └── Gemini 3.8 Flash      → feature implementation in parallel git worktrees (write-only)
```

## The Round Workflow

Work proceeds in discrete, structured rounds (e.g. R7, R8, R9, R10). Each round follows an end-to-end cycle:

### 1. Plan & Task Creation (Claude Opus)
- Review `docs/RUST_QUEUE.md`, `docs/RUST_PARITY.md`, and `docs/DECISIONS.md`.
- Formulate the round's milestone tasks.
- Create `tasks/R<N>-common.md` containing common rules, shared crate dependencies, coordinate rules, and recent lessons from `docs/JESTER.md`.
- Create individual worker task specifications `tasks/R<N>-01.md`, `tasks/R<N>-02.md`, etc., defining exact files, Godot C# reference files to port, data files, and expected public APIs.

### 2. Parallel Worker Execution (Gemini 3.8 Flash)
- Worker sub-agents are launched in parallel git worktrees (via T3 `delegate_task`, provider `antigravity`).
- Each worker reads `AGENTS.md`, `GEMINI.md`, the round common rules, and its specific task file.
- **Workers do NOT run builds or tests.** Workers write the implementation and unit tests, verify types and logic by reading code, and write their report in `## Worker notes`.
- In their worktree, each worker commits once to its own task branch. Workers never push and never touch `main`.

### 3. Review & Merge (Claude Opus)
- Opus inspects worker reports and git diffs against the task specifications.
- Worker branches are merged into the orchestrator branch.

### 4. Single Unified Build, Test & Clippy Pass (Claude Opus)
- Owner's rule: **one build and test run per round**, executed by Claude Opus, not by workers.
- Run dev build and tests under lock:
  ```bash
  flock /tmp/rotland-heavy.lock cargo test
  flock /tmp/rotland-heavy.lock cargo clippy --all-targets
  ```
- Claude fixes any compile errors, borrow-checker issues, or clippy warnings directly.

### 5. Wiring into Game Loop (Claude Opus)
- Claude wires new features, systems, and UI panels into `rotland/src/main.rs`, `play.rs`, or other core orchestrating modules.
- Workers never edit `main.rs` directly; they supply the exact wiring snippet in their `## Worker notes`.

### 6. Visual & Runtime Screenshot Check (Claude Opus)
- If the round touches rendering, lighting, characters, vehicles, world generation, or egui UI, Claude runs a scripted screenshot verification under the shared lock:
  ```bash
  flock /tmp/rotland-heavy.lock target/debug/rotland --save-dir=/tmp/scratch_saves --shot=/tmp/x/check.png [options]
  ```
- Inspect the generated PNG and JSON stats to verify that models, textures, UI text, and colors (including keeping the ground green) render as expected.

### 7. Commit & Push (Claude Opus)
- Once compilation, tests, clippy, wiring, and visual checks pass, Claude commits the unified round to git and pushes to `main`.

### 8. Documentation & Accounting Update (Claude Opus)
- Update `docs/RUST_QUEUE.md` marking the round completed and outlining the next round.
- Update `docs/RUST_PARITY.md` reflecting completed features and reducing the gap against Godot.
- Log any worker compilation, borrow, logic, or clippy mistakes into `docs/JESTER.md`.

### 9. Worktree & Branch Clean-up (Claude Opus)
- Delete the worker git worktrees and temporary branches so that only `main` remains clean (owner's preference).

---

## Why This Split
- **Cost & Rate Limits:** Opus reasoning is reserved for orchestration, architecture, reviewing, and wiring. Gemini Flash provides fast, cost-effective parallel coding.
- **Build Bottlenecks:** Multiple parallel builds on a 4-core laptop cause thrashing and CPU saturation. Having workers operate write-only while Claude runs a single unified build/test pass ensures speed, clean merges, and predictable compilation.
