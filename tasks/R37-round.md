# R37 round: weapon shop (gunsmith). Lead file for the gate

Files: game_core/*, game_camp/*, game_player/*, game_world/*, game/*, rotland/*, rotland_ui/*, world/*, save/*, data/*, tools/blender/*, assets/models/*, assets/icons/*, assets/models.pack, tasks/R37-*, docs/WORLD.md
Rules: `tasks/LEAD.md` + `tasks/R37-common.md` (owner decisions + "R37 shared names").

## Tasks (8 subagents)
| Task | What | after |
|---|---|---|
| `R37-01-wear.md` | per-weapon condition: wear, jams, spread, broken; save | none |
| `R37-02-attachments.md` | attachment data + stat effects (pure) | none |
| `R37-03-gunsmith-logic.md` | shop logic: stock per camp tier / town, prices in scrap, buy, sell, repair, fit | none (use shared names for 01/02) |
| `R37-04-gunsmith-ui.md` | the shop screen (4 tabs) | none |
| `R37-05-models-shop.md` | `gunsmith_bench`, `gun_shop` building | none |
| `R37-06-models-attachments.md` | 9 attachment models + icons, mount points on guns | none |
| `R37-07-places.md` | gunsmith in camp (NPC + bench, camp upgrade tiers), a gun shop + shopkeeper in each town | none |
| `R37-08-wiring.md` | rotland: talk to NPC → screen, actions → logic, wear on fire, attachments change stats and show on the gun, condition bar on hotbar | after 01-04 report |

## Checks after the gate (target/play/rotland, `--save-dir=/tmp/r37-saves`, pictures to /tmp/r37/)
R37-08 adds test flags: `--gunsmith=<tab>` (opens the shop at the camp gunsmith), `--gunsmith-town=<key>`, `--give=<item>[:count]`, `--wear=<weapon>:<0..1>`, `--mods=<weapon>:<id,id>`.
- Shop screen, each tab: `--gunsmith=buy|sell|repair|upgrade --give=scrap_metal:60` (and one in a town: `--gunsmith-town=dusthollow`).
- Camp gunsmith bench + NPC (third person, looking at it); Dusthollow gun shop outside and inside.
- Guns with attachments, third AND first person: `--weapon=rifle --mods=rifle:scope_4x,suppressor`, `--weapon=pistol --mods=pistol:red_dot,ext_mag`, `--weapon=ak47 --mods=ak47:foregrip,drum_mag`, `--weapon=crossbow --mods=crossbow:scope_2x`. Attachments must sit ON the gun (no floating, no clipping into the hand), and the 4x scope must change the aim zoom.
- A worn gun: `--weapon=pistol --wear=pistol:0.1` shows a low condition bar.
- `tools/perf_scenes.sh --set all --seconds 20`: no scene slower than R36 by more than 3 FPS.
- An old save (copy any save from /tmp/r36-saves) still loads.

## Lead notes
- tasks 01-08 all written (Gemini lead); Claude final pass (Haiku) on 2026-11-26.
- gate: 1268 tests pass, clippy PASS, numbers/unused OK; the only flag is the expected scope flag on tools/README_TOOLS.md.
- build: `cargo build --profile play` OK from the r37 worktree (binary built 11:15, no newer source).
- screenshots (all 1920x986 unless noted; pictures in /tmp/r37/):
  - /tmp/r37/shop_buy.png: camp gunsmith Buy tab, scrap 44 shown. Readable, cards fit. Faint "Safe zone" label and compass strip show behind the top edge (cosmetic).
  - /tmp/r37/shop_sell.png: Sell tab lists bat, pistol, bandage, flashlight with sell prices. Fits.
  - /tmp/r37/shop_repair.png: Repair tab, both weapons at 100%, "Full condition" buttons. Fits.
  - /tmp/r37/shop_upgrade.png: Upgrade tab, pistol selected, optic/muzzle/magazine/grip/stock slots, stats before/after. Fits.
  - /tmp/r37/shop_buy_1280.png (1280x720): Buy tab, fits; some card descriptions cut with "..." (by design).
  - /tmp/r37/shop_upgrade_1280.png (1280x720): Upgrade tab, fits, no overlap.
  - /tmp/r37/town_shop_buy.png: Dusthollow Gun Shop, Buy tab (revolver, shotgun, pistol, katana, grenades, ammo, red dot, recoil pad). Fits. Top compass strip shows behind title edge (cosmetic).
  - /tmp/r37/town_shop_outside.png: `--town=dusthollow` view from the road, desert town buildings visible. Could NOT tell which building is the gun shop from this picture.
  - /tmp/r37/tp_rifle_3p.png, tp_rifle_1p.png: rifle with 4x scope and suppressor. 3p: scope sits on the receiver, muzzle tip dark. 1p: scope on top of receiver, attached. OK.
  - /tmp/r37/tp_pistol_3p.png, tp_pistol_1p.png: pistol with red dot and extended mag. 1p: red dot on slide, attached. 3p: too small to see the ext mag clearly.
  - /tmp/r37/tp_ak47_3p.png, tp_ak47_1p.png: AK with foregrip and drum mag. Drum hangs under the receiver, foregrip at the front. Looks attached in both views.
  - /tmp/r37/tp_crossbow_3p.png, tp_crossbow_1p.png: crossbow with 2x scope. Scope sits on the stock/bow rail, small but attached in 1p. Ammo shows 0 (no bolts in bag, expected).
  - /tmp/r37/worn_pistol.png: `--wear=pistol:0.1`; hotbar slot 2 shows a small red low-condition bar (checked by crop). Very small, may need a bigger bar.
  - /tmp/r37/oldsave_load.png: old save test (see save check below).
  - NOT TAKEN: camp gunsmith bench + NPC close-up (only seen as a stall with an NPC behind it at the right of tp_rifle_3p.png); Dusthollow shop INSIDE (no flag exists for interiors).
- FPS (tools/perf_scenes.sh --set all --seconds 20, /tmp/r37/perf.txt): camp 56.2, forest 40.1, city 52.8, night 61.0, zombies 64.3, desert 108.5, snow 128.3, swamp 128.5, mountains 127.9, town_desert 109.7, town_snow 100.7 (avg FPS). No per-scene R36 table exists in docs; only R36 number in STATUS is zombies 68.9 (now 64.3). Older R34 numbers: camp 138, forest 106. So camp and forest look much lower than before: needs a R36 comparison run before we pass the 3 FPS rule.
- save check: `/tmp/r36-saves` is EMPTY, so no R36 save exists. Used the repo's Godot fixture save/tests/fixtures/godot_save.json copied to a scratch slot1.json and run with `--load=1`: loads, shows Day 1 14:31, pistol in slot 2, health 86/100, no load warning. Not a true Rust R36 save.
- Claude must check:
  1. FPS: build R36 (main) play binary and run the same perf script; compare camp (56 now) and forest (40 now) and zombies (64 vs 68.9).
  2. Camp gunsmith bench + NPC close-up (not taken).
  3. Dusthollow gun shop outside (which building) and inside (shopkeeper behind counter) (not taken; no flag).
  4. Worn pistol bar size in hotbar (very small).
  5. Shop top edge: compass/"Safe zone"/"3694 m" labels show through the title area (cosmetic).
  6. Real R36 save: check with a true save when the owner provides one.

## Gate report
- date: 2026-11-26
- base: main
- files changed (137): assets/icons/bolt_quiver.png, assets/icons/drum_mag.png, assets/icons/ext_mag.png, assets/icons/foregrip.png, assets/icons/red_dot.png, assets/icons/scope_2x.png, assets/icons/scope_4x.png, assets/icons/stock_pad.png, assets/icons/suppressor.png, assets/models/buildings/gun_shop.glb, assets/models/props/gunsmith_bench.glb, assets/models/weapons/ak47.glb, assets/models/weapons/bolt_quiver.glb, assets/models/weapons/crossbow.glb, assets/models/weapons/drum_mag.glb, assets/models/weapons/ext_mag.glb, assets/models/weapons/foregrip.glb, assets/models/weapons/hunting_rifle.glb, assets/models/weapons/pistol.glb, assets/models/weapons/red_dot.glb, assets/models/weapons/revolver.glb, assets/models/weapons/rifle.glb, assets/models/weapons/scope_2x.glb, assets/models/weapons/scope_4x.glb, assets/models/weapons/shotgun.glb, assets/models/weapons/smg.glb, assets/models/weapons/stock_pad.glb, assets/models/weapons/suppressor.glb, data/attachments.json, data/camp_layout.json, data/camp_upgrades.json, data/gunsmith.json, data/items.json, data/settlements.json, data/towns.json, data/weapon_mounts.json, data/weapon_wear.json, game_camp/src/camp/npc.rs, game_camp/src/camp/resident.rs, game_camp/src/camp_upgrades/config.rs, game_camp/src/camp_upgrades/mod.rs, game_camp/src/camp_upgrades/types.rs, game_camp/src/gunsmith/actions.rs, game_camp/src/gunsmith/config.rs, game_camp/src/gunsmith/mod.rs, game_camp/src/gunsmith/stock.rs, game_camp/src/lib.rs, game_core/src/items/def.rs, game_core/src/items/hotbar.rs, game_core/src/items/inventory.rs, game_core/src/items/save.rs, game_core/src/items/stack.rs, game_core/src/lib.rs, game_core/src/weapons/attachments.rs, game_core/src/weapons/mod.rs, game_core/src/weapons.rs, game_core/src/weapons/wear.rs, game_player/src/gun_feel.rs, game/src/lib.rs, game/tests/all/attachments_tests.rs, game/tests/all/camp_tests.rs, game/tests/all/camp_upgrades_tests.rs, game/tests/all/gunsmith_tests.rs, game/tests/all/items_tests.rs, game/tests/all/main.rs, game/tests/all/weapon_wear_tests.rs, rotland/src/app/frame_actors_combat.rs, rotland/src/app/frame_actors.rs, rotland/src/app/frame_camp.rs, rotland/src/app/frame_input.rs, rotland/src/app/frame_sim_zombies.rs, rotland/src/app/frame_ui.rs, rotland/src/app/frame_world.rs, rotland/src/app/render_frame.rs, rotland/src/app/startup.rs, rotland/src/app/start_world.rs, rotland/src/app/state.rs, rotland/src/app/test_flags.rs, rotland/src/attachment_view.rs, rotland/src/camp.rs, rotland/src/camp_upgrade_view.rs, rotland/src/combat.rs, rotland/src/gun_feel_view.rs, rotland/src/gunsmith_play.rs, rotland/src/main.rs, rotland/src/play.rs, rotland/src/session.rs, rotland/src/shooting.rs, rotland_ui/src/camp_ui/upgrades/colors.rs, rotland_ui/src/camp_ui/upgrades/detail.rs, rotland_ui/src/camp_ui/upgrades/layout.rs, rotland_ui/src/camp_ui/view.rs, rotland_ui/src/gunsmith_ui/action.rs, rotland_ui/src/gunsmith_ui/draw_buy.rs, rotland_ui/src/gunsmith_ui/draw_repair.rs, rotland_ui/src/gunsmith_ui/draw_sell.rs, rotland_ui/src/gunsmith_ui/draw_upgrade.rs, rotland_ui/src/gunsmith_ui/mod.rs, rotland_ui/src/gunsmith_ui/screen.rs, rotland_ui/src/gunsmith_ui/tests.rs, rotland_ui/src/hud/draw.rs, rotland_ui/src/hud/hotbar.rs, rotland_ui/src/hud/state.rs, rotland_ui/src/lib.rs, save/src/systems/inventory.rs, save/src/systems/settlements.rs, save/tests/saves.rs, tasks/R37-01-wear.md, tasks/R37-02-attachments.md, tasks/R37-03-gunsmith-logic.md, tasks/R37-04-gunsmith-ui.md, tasks/R37-05-models-shop.md, tasks/R37-06-models-attachments.md, tasks/R37-07-places.md, tasks/R37-08-wiring.md, tasks/R37-round.md, tools/blender/buildings/gun_shop.py, tools/blender/build_r37_models.py, tools/blender/icons/item_meshes.py, tools/blender/props/gunsmith_bench.py, tools/blender/weapons/ak47.py, tools/blender/weapons/attachments.py, tools/blender/weapons/crossbow.py, tools/blender/weapons/hunting_rifle.py, tools/blender/weapons/pistol.py, tools/blender/weapons/revolver.py, tools/blender/weapons/rifle.py, tools/blender/weapons/shotgun.py, tools/blender/weapons/smg.py, tools/README_TOOLS.md, world/src/camp_layout/buildings.rs, world/src/camp_layout/mod.rs, world/src/camp_layout/tests.rs, world/src/camp_layout/types.rs, world/src/lib.rs, world/src/towns/lots.rs, world/src/towns/mod.rs
- tests passed: 1268
- clippy: PASS
- test: PASS
- checks: FAIL
- gate: FAIL (checks)
