# R35 prompt (paste everything inside the box into Hydrogen High in Antigravity)

```
You are the LEAD for round R35 of the Rotland game (Rust repo at /home/aymen/Documents/rotland-rust): WEAPONS + GUN FEEL.

First read: AGENTS.md, then tasks/R35-common.md (the rules and the "R35 shared names"; follow them exactly), then tasks/R35-round.md and its five task files:
- tasks/R35-01-gun-feel.md          (guns kick and flash)
- tasks/R35-02-weapon-models.md     (5 new models + icons)
- tasks/R35-03-new-weapons.md       (revolver, hunting rifle, crossbow, grenade)
- tasks/R35-04-weapon-anims.md      (grenade throw + crossbow reload, grips)
- tasks/R35-05-explosion-backpack.md (explosion effect + visible backpack)

How we work (the laptop is slow, so the game builds only ONCE):
1. Make ONE worktree as R35-common.md says (../rotland-r35, branch r35 from main, CARGO_TARGET_DIR shared). No separate folder copies.
2. Start about 8 Hydrogen subagents at the same time. Suggested split: gun-feel logic, gun-feel wiring, weapon models, new-weapons data+loot, grenade logic, grenade/crossbow wiring, animations+grips, explosion+backpack. Give each the "Subagent rules" AND the "R35 shared names" from R35-common.md word for word, plus its task file. Subagents write code only: no cargo, no gate, no blender, no game.
3. When all have reported: add the shared-file lines, build the weapons + icons once, rebuild the characters once, then run `tools/gate.sh --task tasks/R35-round.md` in the foreground and wait. Fix errors and run it again until the last line is GATE: PASS.
4. Run `cargo run --release -p baker` once, then take the 4 check screenshots listed in R35-round.md and look at them. Fix anything clearly wrong (a gun floating out of the hand, a backpack inside the body).
5. Fill "## Lead notes" in tasks/R35-round.md (each task done or not, last 5 lines of the final gate, screenshot paths, anything Claude must check). ONE commit on branch r35 (git add only your changed files). Never push, never touch main.
6. Tell me: "R35 done" plus a short summary.
```
