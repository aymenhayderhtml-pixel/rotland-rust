# R27-04: guild "deliver" and "explore" tasks actually work

Files: game/src/guild/*, game/tests/all/guild*.rs, game/tests/all/main.rs, data/guild*.json, rotland/src/app/frame_guild.rs, rotland/src/camp_ui/*, tasks/R27-04-guild-hooks.md
Rules: `tasks/R27-common.md`.

**Goal:** guild tasks of kind "deliver" (bring items) and "explore" (reach a place) can be completed in the game. Today they have no hooks (see `docs/STATUS.md`, `tasks/R24-N6-guild-tasks.md` and its Worker notes, `rotland/src/app/frame_guild.rs` which already hooks clear/explore checks).
- Find exactly what is missing. Deliver: when the player hands in at the guild tab, check the inventory has the items, remove them, mark the task done (reuse the existing hand-in click if there is one). Explore: mark done when the player reaches the target spot (radius from data).
- **Do not invent rewards or rules.** Use what the guild data and code already define. If a rule is missing and would change how the game feels, write it as a question in Worker notes and do the rest.
- Tests for the pure parts (inventory check + removal, reach check).
