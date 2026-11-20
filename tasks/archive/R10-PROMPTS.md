# R10 prompts (copy one into each AI)

Setup once: `bash tasks/R10-setup.sh A` makes folders `../rotland-r10-01` … `../rotland-r10-11`.
Open each AI **in its own folder**, paste its prompt. Phase B (09, 10): only after Claude says R10-01 is merged; then `bash tasks/R10-setup.sh B`.

## Template (same for every lane; change XX and the file name)
```
You are a worker on Rotland, a Rust game. You are in your own git worktree on branch r10-XX.
1. Read AGENTS.md, then tasks/R10-common.md, then tasks/R10-XX-<name>.md. Follow them exactly.
2. Do only that task. Edit only the files it lists under "You own". Never edit rotland/src/main.rs (unless your task is R10-01).
3. Port numbers from the Godot reference at "../zombi servival/" (full path: /home/aymen/Documents/zombi servival/). Never invent values; open every function before calling it.
4. Write-only: never run cargo, the game, the baker, Blender, or pkill. Claude builds once.
5. When done, fill in "## Worker notes" at the bottom of your task file (files changed, public API, wiring for Claude, tests + where each number comes from, cargo output or "not checked", questions).
6. Make ONE commit on branch r10-XX with a message starting "R10-XX:". Don't push. Don't touch main.
Take your time and be thorough: a lazy or made-up answer gets logged in docs/JESTER.md.
```

## Which AI for which lane
| Lane | Task file | Size | Best kind of AI |
|---|---|---|---|
| 01 | R10-01-split-main.md | big, risky | Strongest coder that **can run cargo** (e.g. Codex / GPT high, Gemini Pro). Start first. |
| 02 | R10-02-model-fixes.md | medium | One that can run Blender headless; good with 3D/Python |
| 03 | R10-03-two-hand-swing.md | medium | Strong at math (IK, rotations) |
| 04 | R10-04-hud-pack.md | big | Good Rust + egui; Gemini Flash OK |
| 05 | R10-05-bag-crafting-ui.md | medium | Gemini Flash OK |
| 06 | R10-06-zombie-behaviours.md | big | Strong coder; careful with porting Godot |
| 07 | R10-07-data-sounds.md | small | Any (Flash / cheap model) |
| 08 | R10-08-no-crash-data.md | medium | Careful coder; can run cargo is a plus |
| 11 | R10-11-parity-refresh.md | small, docs | Any cheap model |
| 09 | R10-09-saves.md (Phase B) | medium | Careful coder |
| 10 | R10-10-frame-perf.md (Phase B) | medium | One that can run the game for `--stats` |

## When a worker says it's done
Tell Claude: "R10-XX done" (several at once is fine). Claude merges, builds and tests once, wires, takes screenshots, commits, and logs mistakes.
