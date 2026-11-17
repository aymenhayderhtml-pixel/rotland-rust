---
name: blender-art
description: Rotland's script-only Blender pipeline for low-poly .glb models (props, weapons, buildings, characters, item icons). Use for any art/model task; Blender is run only via `blender -b --python`.
---

# Rotland Blender art pipeline

Art is **only** Python scripts run with `blender -b --python <script>` (Blender 5.3, snap with classic confinement). No asset packs, no downloads, no add-ons beyond built-ins.

## Where things are
- `tools/blender/palette.py`: the shared palette (a named 4-column × 8-row color grid, 64×128 px) and the generator for `assets/textures/palette.png`. To add a color, use a free cell; never re-order existing cells (old models depend on their UVs).
- `tools/blender/common.py`: `reset_scene`, `palette_material`, `paint`/`paint_faces` (set UVs to a palette cell), builders (`make_box/cylinder/cone/icosphere`), `add_collider` (`-convcolonly` suffix → Godot convex collision only), `finalize_and_export(name, category)` (apply transforms, origin bottom-center, Y-up glb), `render_preview` (Workbench; Cycles CPU fallback), and the triangle budget check.
- `tools/blender/props/*.py`: one script per model. `tools/blender/build_all.py` builds everything and the contact sheet.
- Output: `assets/models/<category>/<name>.glb` plus `assets/models/_previews/<name>.png`.

## Rules
- Low-poly, **flat-shaded**, colors from palette UVs only (one shared material named `palette`). Muted survival colors.
- 1 unit = 1 m. Origin at bottom-center (z min = 0). In Blender the **front faces +Y**, which is −Z in Godot.
- Triangle budgets: props ≤ 150–450 (see the ModelsTriangleBudget test table), characters ≤ 1500, weapons ≤ 300, buildings ≤ 3000 shell.
- Colliders: simple meshes named `<x>-convcolonly`. Nothing walk-through (bushes) gets one.
- **Weapons:** origin at the grip center, pointing +Y (Blender), and an optional empty `grip_L` for two-handed weapons.
- **Characters:** follow `docs/RIG_CONTRACT.md` exactly (bone names, A-pose, rigid weights, clip names, pose tables in `tools/blender/characters/poses.py`, contact sheets of 8 frames per clip).
- **Item icons:** 128×128 PNG renders, transparent background, consistent 3/4 angle, saved to `assets/icons/<item_id>.png`.
- After exporting, run the Godot import (`godot-mono --headless --path . --import`), then the model self-tests (`--selftest=ModelsLoad,ModelsTriangleBudget,ModelsHaveColliders,ModelsOriginAtGround`, plus `RigContract` for characters).
- Look at your previews with the image Read tool and describe them honestly in the Worker notes (does it read as the object?).
- Never commit `__pycache__`.
