# R37-06: attachment models + mount points

Owns: tools/blender/weapons/attachments.py (new: builds the 9 attachment glbs), tools/blender/weapons/<gun>.py (ONLY adding empty mount nodes), data/weapon_mounts.json (new)
Read: tasks/R37-common.md, tools/blender/weapons/rifle.py, pistol.py, ak47.py, smg.py, shotgun.py, revolver.py, hunting_rifle.py, crossbow.py (sizes, axes, existing `grip_L` node pattern), tools/blender/common.py, palette.py, tools/blender/icons/. Never run blender.

- 9 models (ids from shared names), 40-400 tris, dark metal palette; origin at the mount point, +Z/-Z the same as the guns (barrel along -Z).
- Each gun gets empty nodes `mount_optic`, `mount_muzzle`, `mount_magazine`, `mount_grip`, `mount_stock` where they make sense (like `grip_L`). Also write the same positions into `data/weapon_mounts.json` (per weapon, per slot: [x,y,z]) so rotland can place attachments without reading glb nodes.
- Icons: add the 9 to the icon render list (report the lines if the list is in another file).
- Changing gun scripts must not move existing geometry (only new empty nodes).
