# R37-06: attachment models + mount points

Owns: tools/blender/weapons/attachments.py (new: builds the 9 attachment glbs), tools/blender/weapons/<gun>.py (ONLY adding empty mount nodes), data/weapon_mounts.json (new)
Read: tasks/R37-common.md, tools/blender/weapons/rifle.py, pistol.py, ak47.py, smg.py, shotgun.py, revolver.py, hunting_rifle.py, crossbow.py (sizes, axes, existing `grip_L` node pattern), tools/blender/common.py, palette.py, tools/blender/icons/. Never run blender.

- 9 models (ids from shared names), 40-400 tris, dark metal palette; origin at the mount point, +Z/-Z the same as the guns (barrel along -Z).
- Each gun gets empty nodes `mount_optic`, `mount_muzzle`, `mount_magazine`, `mount_grip`, `mount_stock` where they make sense (like `grip_L`). Also write the same positions into `data/weapon_mounts.json` (per weapon, per slot: [x,y,z]) so rotland can place attachments without reading glb nodes.
- Icons: add the 9 to the icon render list (report the lines if the list is in another file).
- Changing gun scripts must not move existing geometry (only new empty nodes).

## Worker notes
- **Files created/changed:**
  - `tools/blender/weapons/attachments.py` (new, 134 lines): builds all 9 attachment `.glb` models into `assets/models/weapons/<id>.glb` via `common.run_model(att_id, "weapons", 400, ..., grip=(0.0, 0.0, 0.0))`. Uses Sheet A dark-metal palette (`gun_black`, `metal`, `rock_dark`, `water_blue`, `red_paint`, `brass`), origin `(0, 0, 0)` at the mount point, `+Y` forward in Blender (`-Z` in engine space), estimated ~92–316 tris each (within 40–400). Exposes `ICON_BUILDERS`, `BUILDERS`, `icon_meshes(att_id)`, `build_<id>(model)`, and `main()`.
  - `data/weapon_mounts.json` (new): engine-space `[x, y, z]` mount positions per weapon (`pistol`, `revolver`, `shotgun`, `rifle`, `hunting_rifle`, `ak47`, `smg`, `crossbow`) and slot (`optic`, `muzzle`, `magazine`, `grip`, `stock`), converted from Blender build coords via `[bx - gx, bz - gz, gy - by]` (verified against `common.py` `export_yup=True` + `baker/src/transform.rs` + `data/weapons.json` `offHandGrip`).
  - Gun scripts modified (ONLY `model.add_empty(...)` added, no existing geometry moved):
    - `tools/blender/weapons/pistol.py`: `mount_optic`, `mount_muzzle`, `mount_magazine`
    - `tools/blender/weapons/revolver.py`: `mount_optic`
    - `tools/blender/weapons/shotgun.py`: `mount_optic`, `mount_grip`, `mount_stock`
    - `tools/blender/weapons/rifle.py`: `mount_optic`, `mount_muzzle`, `mount_stock`
    - `tools/blender/weapons/hunting_rifle.py`: `mount_optic`, `mount_muzzle`, `mount_stock`
    - `tools/blender/weapons/ak47.py`: `mount_optic`, `mount_muzzle`, `mount_magazine`, `mount_grip`, `mount_stock`
    - `tools/blender/weapons/smg.py`: `mount_optic`, `mount_muzzle`, `mount_magazine`, `mount_grip`
    - `tools/blender/weapons/crossbow.py`: `mount_optic`, `mount_magazine`
- **Icon list lines needed in `tools/blender/icons/item_meshes.py`:**
  1. Near the weapon imports (around line 214, after `import grenade as grenade_weapon  # noqa: E402`):
     ```python
     import attachments as weapon_attachments  # noqa: E402
     ```
  2. Inside `BUILDERS = { ... }` (around line 384, after `"grenade": grenade,`):
     ```python
         **weapon_attachments.ICON_BUILDERS,
     ```
     (Or explicitly: `"scope_2x": weapon_attachments.ICON_BUILDERS["scope_2x"],` etc. for `scope_2x`, `scope_4x`, `red_dot`, `suppressor`, `ext_mag`, `drum_mag`, `foregrip`, `stock_pad`, `bolt_quiver`.)
- **How to build & verify (Lead):**
  - Run `blender -b --python tools/blender/weapons/build_weapons.py` (re-exports the 8 updated gun `.glb`s with the new `mount_*` empty nodes) and `blender -b --python tools/blender/build_r37_models.py` (builds the 9 attachment `.glb`s + 9 attachment icons + `contact_sheet_r37.png`), then `cargo run --release -p baker`.
- **Notes / anything unsure:**
  - `data/attachments.json` (written by R37-02) sets `"model": "res://assets/models/attachments/<id>.glb"`, whereas `attachments.py` exports to `assets/models/weapons/<id>.glb` per Lead instructions (`common.run_model(id, "weapons", ...)`). `ModelPack::find` looks up by stem (`<id>`), and `data/budgets.json` has a cap for `"weapons"` (3000) but none for `"attachments"`, so `"weapons"` is the right export folder; R37-08 should look up by stem `<id>` (or check `data/attachments.json` path if it strips prefix/stem).
  - `rifle.py` and `hunting_rifle.py` already have built-in scope geometry (`z=0.105` and `z=0.095` in build space). Since existing gun geometry cannot be moved, `scope_4x` is sized with its tube at `z=0.048` above `mount_optic` (`z=0.054`, putting the tube at `z=0.102`) and larger bells (`r=0.027`) so it cleanly envelopes the built-in scope.
  - Note: Blender/cargo/gate were not run per subagent rules; only `python3` AST + JSON syntax validation was run.
