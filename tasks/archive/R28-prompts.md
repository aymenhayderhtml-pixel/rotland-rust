# R28 prompts for Antigravity (one chat per prompt, all 3 can run at once)

## Prompt 01: fingers-grips

```
You are a worker on Rotland, a Rust 2024 zombie survival game in /home/aymen/Documents/rotland-rust.

1. Make your own copy of the project first (other workers use the main folder at the same time):
   cd /home/aymen/Documents/rotland-rust && git worktree add ../rotland-r28-01 -b r28-01 main
   Then work ONLY inside /home/aymen/Documents/rotland-r28-01.
2. Read AGENTS.md, tasks/R27-common.md, then do the job in tasks/R28-01-fingers-grips.md exactly. Only edit files allowed by its Files: line.
3. Build and test ONLY with `tools/gate.sh --task tasks/R28-01-fingers-grips.md`. Run it in the foreground and wait for it, even if it takes 10+ minutes (other lanes share the build lock; that is normal). Never stop or end your turn while it runs. Fix and re-run until the last line is `GATE: PASS`.
4. Fill in the "## Worker notes" section of tasks/R28-01-fingers-grips.md (what you changed, how to see it in the game, anything unsure). Also paste the last 5 lines of your final gate or scope run there.
5. Commit once to branch r28-01. Never push. Never touch the main branch or the main folder.
6. Never run pkill -f. Never write to real save folders.
Finish with: the gate result, the commit hash, and a 3-line summary.
```

## Prompt 02: clip-polish

```
You are a worker on Rotland, a Rust 2024 zombie survival game in /home/aymen/Documents/rotland-rust.

1. Make your own copy of the project first (other workers use the main folder at the same time):
   cd /home/aymen/Documents/rotland-rust && git worktree add ../rotland-r28-02 -b r28-02 main
   Then work ONLY inside /home/aymen/Documents/rotland-r28-02.
2. Read AGENTS.md, tasks/R27-common.md, then do the job in tasks/R28-02-clip-polish.md exactly. Only edit files allowed by its Files: line.
3. This is a Blender lane: no cargo and no gate.sh. When done, run `python3 -I tools/gate_checks.py --base main --task tasks/R28-02-clip-polish.md` and make sure it says `scope: PASS`.
4. Fill in the "## Worker notes" section of tasks/R28-02-clip-polish.md (what you changed, how to see it in the game, anything unsure). Also paste the last 5 lines of your final gate or scope run there.
5. Commit once to branch r28-02. Never push. Never touch the main branch or the main folder.
6. Never run pkill -f. Never write to real save folders.
Finish with: the gate result, the commit hash, and a 3-line summary.
```

## Prompt 03: pose-shots

```
You are a worker on Rotland, a Rust 2024 zombie survival game in /home/aymen/Documents/rotland-rust.

1. Make your own copy of the project first (other workers use the main folder at the same time):
   cd /home/aymen/Documents/rotland-rust && git worktree add ../rotland-r28-03 -b r28-03 main
   Then work ONLY inside /home/aymen/Documents/rotland-r28-03.
2. Read AGENTS.md, tasks/R27-common.md, then do the job in tasks/R28-03-pose-shots.md exactly. Only edit files allowed by its Files: line.
3. Build and test ONLY with `tools/gate.sh --task tasks/R28-03-pose-shots.md`. Run it in the foreground and wait for it, even if it takes 10+ minutes (other lanes share the build lock; that is normal). Never stop or end your turn while it runs. Fix and re-run until the last line is `GATE: PASS`.
4. Fill in the "## Worker notes" section of tasks/R28-03-pose-shots.md (what you changed, how to see it in the game, anything unsure). Also paste the last 5 lines of your final gate or scope run there.
5. Commit once to branch r28-03. Never push. Never touch the main branch or the main folder.
6. Never run pkill -f. Never write to real save folders.
Finish with: the gate result, the commit hash, and a 3-line summary.
```
