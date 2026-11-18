# R10-11: Bring docs/RUST_PARITY.md up to date — PHASE A (docs only, cheap model is fine)

Read `tasks/R10-common.md` first. This task **is** allowed to edit `docs/RUST_PARITY.md` (and only that file).

The audit was written at R7. Rounds R8 and R9 finished many rows since (see `docs/RUST_QUEUE.md`, "R8" and "R9" paragraphs, and `git log --oneline -15`). The doc still says "missing" for things that now work, which misleads planning.

## Do
1. For every row marked **missing** or **logic only**, check the current code: is it reached from `rotland/src/main.rs` (or the files main.rs calls)? Quote the file:line you found in the "Rust evidence" column. Known to be done since R8/R9 (verify each, don't trust this list blindly): tree felling, guards shooting, chat bubbles, profile stats, key rebinding panel, parked car colliders, stray dogs at POIs, POI building zombies + clear bounties, first-person weapon view, gun bash, car trunk, zombie voices, FOV / render scale / V-Sync settings, car headlights.
2. Fix the `rust/...` paths in the doc to the new layout (no `rust/` prefix).
3. Rewrite the "Top 15" at the end as the **current** top 15 still missing, ranked by player impact, with size (small/medium/large).
4. Add rows for anything new the owner asked for on GitHub that Godot doesn't have (mini-map #7, guild #6, more weapons #5, profile portrait #8), marked **new (not in Godot)**.
5. Update the header line ("as of R9, 2026-11-18").

Don't change code. Don't guess: if you can't tell, write "unsure" and why.

## Worker notes
