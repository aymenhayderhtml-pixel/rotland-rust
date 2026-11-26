# R37-02: attachments (data + stat effects)

Owns: game_core/src/weapons/attachments.rs (new), data/attachments.json (new), data/items.json (one item per attachment), game/tests/all/attachments_tests.rs (new)
Read: tasks/R37-common.md, game_core/src/weapons.rs (WeaponDef fields: spread, magazine, reload, noise, scope), data/weapons.json, data/items.json (item format), data/gun_feel.json (kick).

- Attachments (ids from shared names), which guns they fit, and effects (balanced, small): scopes set `scope_fov` (2x ≈ 40, 4x ≈ 20) and lower aim spread; red_dot a little less hip+aim spread, short-range only (pistols, smg, ak47, revolver); suppressor `noise_mult` 0.35 (pistol, smg, ak47, rifle, hunting_rifle); ext_mag +50% mag (pistol, smg, ak47), drum_mag +150% mag but reload ×1.3 (ak47, smg); foregrip kick ×0.7 (ak47, smg, shotgun); stock_pad kick ×0.8 (rifles, shotgun); bolt_quiver reload ×0.75 (crossbow).
- One slot each: a weapon has at most one mod per `AttachSlot`.
- Items in `data/items.json` (category `attachment`, weight, icon name = id, value).
- `effective_stats` is pure and tested: each effect, two mods stacking, wrong-weapon mods ignored, unknown ids ignored.
- `kick_mult` must reach gun feel: report to the lead how `gun_feel` should read it (don't edit game_player).
