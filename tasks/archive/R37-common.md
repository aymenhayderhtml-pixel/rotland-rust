# R37 common: the weapon shop (gunsmith) round

Setup, subagent rules and the lead loop: `tasks/LEAD.md` (worktree `../rotland-r37`, branch `r37`, shared `CARGO_TARGET_DIR`).

## Owner decisions (2026-11-26)
- A **gunsmith** in the camp (grows with camp upgrades) AND a small **gun shop in each of the 4 far towns** (rarer, better stock; town shops are the only place for top-tier guns and some attachments).
- The gunsmith is paid in **scrap metal** (`scrap_metal`, an existing item; you hand it over from your bag). The general camp trader keeps using dollars: do not change it.
- Gunsmith services: **buy weapons, buy ammo** (bullets, bolts, grenades), **repair** worn weapons, **upgrade** weapons with attachments, and **buy your loot** (sell guns/items for scrap).
- Low-poly flat style, no sounds. Numbers in data files.

## R37 shared names (every task uses EXACTLY these)
- **Per-item data** (R37-01, `game_core/src/items/stack.rs`): `ItemStack` gains two optional fields, both `#[serde(default, skip_serializing_if = ...)]` so old saves load unchanged:
  - `pub wear: Option<f32>` (weapon condition 0.0..=1.0; `None` = brand new = 1.0)
  - `pub mods: Vec<String>` (attachment ids fitted to this weapon)
  Stacks with wear/mods never merge with other stacks. Helper fns: `ItemStack::condition(&self) -> f32`, `ItemStack::has_mod(&self, id) -> bool`.
- **Wear** (`game_core/src/weapons/wear.rs`, R37-01): `WearConfig` from `data/weapon_wear.json` (per weapon: wear per shot / per melee hit; at condition < `jamBelow` a jam chance; at < `spreadBelow` more spread; at 0.0 the weapon can't fire/swing until repaired). `fn apply_shot_wear(stack: &mut ItemStack, cfg: &WearConfig)`, `fn jam_roll(cond, cfg, rng) -> bool`, `fn spread_mult(cond, cfg) -> f32`.
- **Attachments** (`game_core/src/weapons/attachments.rs`, R37-02): `AttachmentDef { id, name, slot: AttachSlot, fits: Vec<String> /*weapon ids*/, model: String, price_scrap: u32, effects: AttachEffects }`, `AttachSlot { Optic, Muzzle, Magazine, Grip, Stock }`, `AttachEffects { spread_mult, kick_mult, noise_mult, mag_bonus, reload_mult, scope_fov: Option<f32> }`. `AttachmentRegistry::from_repo()` reads `data/attachments.json`. `fn effective_stats(base: &WeaponDef, mods: &[String], reg) -> WeaponDef` (pure). Ids: `scope_2x`, `scope_4x`, `red_dot`, `suppressor`, `ext_mag`, `drum_mag`, `foregrip`, `stock_pad`, `bolt_quiver` (crossbow).
- **Gunsmith** (`game_camp/src/gunsmith/`, R37-03): `GunsmithConfig` from `data/gunsmith.json`; `GunsmithStock` (save id `"gunsmith"`, restock each morning like `TraderStock`); `ShopKind { Camp { tier: u8 }, Town { key: String } }`; fns `buy(kind, item_id, bag) -> Result<..>`, `sell(item stack, bag) -> scrap`, `repair_cost(stack) -> u32`, `repair(stack, bag)`, `fit_cost(attachment) -> u32`, `fit(stack, attachment_id, bag)`, `remove_mod(stack, id, bag)` (gives the attachment back as an item). Scrap is taken from / given to the bag as `scrap_metal` stacks.
- **UI** (`rotland_ui/src/gunsmith_ui/`, R37-04): `GunsmithScreen` with tabs `Buy`, `Sell`, `Repair`, `Upgrade`; `GunsmithAction` enum returned to rotland.
- **Models** (`assets/models/...`, R37-05 and R37-06): `gunsmith_bench` (camp workbench with tools, gun rack, lamp), `gun_shop` (small town shop building with a big gun sign, walk-in like `house_small`), attachments `scope_2x`, `scope_4x`, `red_dot`, `suppressor`, `ext_mag`, `drum_mag`, `foregrip`, `stock_pad`, `bolt_quiver` (each with an inventory icon). Attachment models have their origin at the mount point.
- NPC: camp gunsmith resident id `npc_gunsmith`; town shopkeepers `shopkeeper_<town_key>`.
