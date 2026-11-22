---
name: report-issue
description: The owner reports a bug or problem they saw while playing Rotland (usually a screenshot plus a short description). Save it to docs/ISSUES.md, look for the likely cause in the code, and report back in plain English. Use whenever the owner describes something wrong in the game, says "issue", "bug", "problem", or pastes a game screenshot with a complaint.
---

# Report an issue (owner's bug report)

The owner is not a programmer. They paste a screenshot and a few words (often with typos). Your job: **record it, find the likely cause, explain it simply.** Do NOT fix it unless the owner asks; the orchestrator session plans fixes into rounds.

## Steps
1. **Understand it.** Look at the image carefully. Restate the problem in one clear sentence. Only if it is truly unclear, ask ONE short question; otherwise go on.
2. **Save the picture.** Copy each pasted image into `docs/issues/` as `NNN-short-name.png` (NNN = the next issue number from `docs/ISSUES.md`, 3 digits). Attachments live under `/home/aymen/.t3/userdata/attachments/`.
3. **Find the likely cause (read only, ~10 minutes max).** Read `docs/STATUS.md` first, then `docs/CODEMAP.md` to find the right module, then only the files you need. Check `docs/ISSUES.md`, `docs/STATUS.md` and `docs/JESTER.md` for the same problem already known. Use clues in the screenshot: the position on the compass/minimap, FPS counter, HUD text, which model or screen. If it's a model problem (flicker, holes, floating parts), run `python3 -I tools/find_coplanar.py <model.glb>` for flicker. Don't run the game or cargo (another session may be using the laptop); don't edit code.
4. **Write the row** at the bottom of the table in `docs/ISSUES.md`: next number, today's date, status `open` or `checked`, where (place/screen/model), what the owner saw (their words, cleaned up), the likely cause with file paths (`path/file.rs:line`) and a suggested fix in one or two sentences, and the picture link `[pic](issues/NNN-short-name.png)`.
5. **Commit only those files:** `git add docs/ISSUES.md docs/issues/NNN-*.png && git commit -m "Issue NNN: <short title>"`. Never `git add -A`, never commit anything else, never push unless the owner asks, never touch other sessions' work.
6. **Answer the owner** in short, simple English (no jargon): what the problem is, what probably causes it, how big the fix is (small / medium / big), and "Saved as issue NNN". If it's already known, say so and which issue/status.

## Rules
- Safety rules from `AGENTS.md` apply (never write to the owner's saves, never `pkill -f`).
- Several problems in one message: one row each.
- If the owner later says it's fixed or not a problem, update the status column.
