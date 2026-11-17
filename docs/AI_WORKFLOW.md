# AI workflow: who does what

```
 Owner (ideas, approves look & feel, plays the builds)
   │
 Claude Opus = ORCHESTRATOR
   │  plans milestones, writes tasks/*.md, reviews, merges, talks to owner
   ├── Haiku (cheap & fast)  → checks, builds, logs, small fixes, API lookups, reviews of simple tasks
   └── Gemini 3.8 Flash      → feature code, Blender .glb scripts, data JSON, larger implementation tasks
```

> **Current setup (2026-11-12):** Haiku + Gemini 3.8 Flash workers in parallel lanes (art / code / infra) in git worktrees. Fable reviews the roadmap at milestones. The full routine is in `.claude/skills/rotland-orchestrate/SKILL.md`.

## Loop for every task
1. **Opus** writes `tasks/NNN-name.md` from `tasks/TEMPLATE.md`, with a clear goal, files and acceptance checks.
2. **Gemini** (or Haiku for small jobs) implements it and fills in `## Worker notes`.
3. **Haiku** runs `dotnet build` and the headless Godot import, and reports errors.
4. **Opus** reviews the diff. If it's OK, Opus commits to git. If not, it goes back to the worker with notes.
5. At the end of each milestone, the **owner** plays it and gives feedback.

## Why this split
Opus usage on Claude Pro is limited and expensive, so Opus spends it on thinking and reviewing, not typing. Gemini Flash and Haiku are fast and cheap, and do well when the spec is clear. That's why task files must be precise.
