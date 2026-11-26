# R37-08: wiring in the game (after 01-04 report)

Owns: rotland/src/gunsmith_play.rs (new), rotland/src/attachment_view.rs (new), rotland/src/app/* (only the lines needed), rotland/src/shooting*.rs, rotland/src/combat*.rs, rotland/src/fp_weapon.rs, rotland/src/player_visual.rs, rotland/src/dev_panel/* (test flags), rotland_ui/src/hud/* (condition bar on the hotbar slot), game_player/src/gun_feel.rs (read kick_mult), tools/README_TOOLS.md (new flags)
Read: tasks/R37-common.md, the 01-04 worker notes, how the camp trader screen opens and talks to the logic (grep `draw_shop`, `TraderStock`, `npc_trader` in rotland/), rotland/src/shooting.rs, rotland/src/fp_weapon.rs, rotland/src/player_visual.rs, how test flags are parsed (grep `--weapon=`).

- Press E near `npc_gunsmith` or a town shopkeeper → open `GunsmithScreen` for that shop; actions call the R37-03 logic; close with Esc; saves include the shops.
- Fire/melee apply wear (R37-01); jams play the existing "click"/dry-fire path; broken guns can't fire and show "Broken - repair at a gunsmith".
- Effective stats (R37-02) used everywhere a weapon's spread/mag/reload/noise/scope/kick is read for the held gun.
- Attachments drawn on the gun in third and first person at `data/weapon_mounts.json` positions, following the gun.
- Hotbar: a thin condition bar under weapon slots (green → yellow → red).
- Test flags from R37-round.md (`--gunsmith=`, `--gunsmith-town=`, `--give=`, `--wear=`, `--mods=`), documented in tools/README_TOOLS.md.
- Tests where logic lives outside rotland; keep rotland files under ~300 lines (split if needed).
