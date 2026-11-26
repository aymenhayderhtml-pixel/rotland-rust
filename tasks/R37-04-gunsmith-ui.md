# R37-04: gunsmith screen

Owns: rotland_ui/src/gunsmith_ui/ (new: mod.rs, screen.rs, draw_buy.rs, draw_sell.rs, draw_repair.rs, draw_upgrade.rs, action.rs, tests.rs)
Read: tasks/R37-common.md, rotland_ui/src/camp_ui/draw_shop.rs, screen.rs, action.rs, view.rs (copy the look, fonts, colours, layout rules exactly), rotland_ui/src/hub_ui/item_text.rs (item names/tooltips), how icons are drawn there.

- Title bar: shop name ("Camp Gunsmith" / "Dusthollow Gun Shop") + your scrap count with the scrap icon. Tabs Buy | Sell | Repair | Upgrade (keys 1-4 and click).
- Buy: grid of icons with price; locked items greyed with "Camp tier N". Sell: your bag's sellable items with the scrap you'd get. Repair: your worn weapons with a condition bar and cost; "Repair all". Upgrade: pick a gun on the left, its 5 slots (Optic, Muzzle, Magazine, Grip, Stock) on the right with fitted mod / options + cost; before/after stat bars (damage, spread, kick, mag, noise, zoom).
- Must fit 1920×986 and 1280×720 without overlap (the camp board once overflowed: test both sizes like `camp_ui/tests.rs`).
- The screen takes a plain view-model struct (no game crate types beyond ids/numbers) and returns `GunsmithAction`. Report the mod lines for `rotland_ui/src/lib.rs`.
