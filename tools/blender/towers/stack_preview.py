"""Renders stack_<style>_<size>.png previews by importing and stacking exported tower GLBs.

Run standalone after all tower GLBs exist:
`flock /tmp/rotland-blender.lock blender -b --python tools/blender/towers/stack_preview.py`
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))

import bpy  # noqa: E402
from mathutils import Vector  # noqa: E402

import common  # noqa: E402

BUILDINGS_DIR = os.path.join(common.MODELS_DIR, "buildings")
STYLE_SIZES = [
    ("brick", 16),
    ("brick", 24),
    ("stone", 24),
    ("stone", 32),
    ("glass", 24),
    ("glass", 32),
    ("slab", 24),
    ("slab", 32),
    ("deco", 24),
    ("deco", 32),
]


def _import_visual_meshes(glb_path):
    """Imports `glb_path`, deletes colliders/empties, and returns the visual mesh objects."""
    before = set(bpy.data.objects)
    bpy.ops.import_scene.gltf(filepath=glb_path)
    added = [obj for obj in bpy.data.objects if obj not in before]
    visuals = []
    for obj in added:
        if obj.type != "MESH" or obj.name.endswith("-convcolonly"):
            bpy.data.objects.remove(obj, do_unlink=True)
        else:
            visuals.append(obj)
    return visuals


def _z_bounds(objs):
    """World-space (min_z, max_z) across `objs`."""
    bpy.context.view_layer.update()
    zs = []
    for obj in objs:
        for corner in obj.bound_box:
            zs.append((obj.matrix_world @ Vector(corner)).z)
    return min(zs), max(zs)


def render_stack(style, size):
    """Stacks base + floor + floor_b + crown for `(style, size)` and renders `stack_<style>_<size>.png`."""
    name = f"stack_{style}_{size}"
    parts = [
        f"tw_{style}_{size}_base",
        f"tw_{style}_{size}_floor",
        f"tw_{style}_{size}_floor_b",
        f"tw_{style}_{size}_crown",
    ]
    paths = [os.path.join(BUILDINGS_DIR, p + ".glb") for p in parts]
    missing = [p for p in paths if not os.path.exists(p)]
    if missing:
        print(f"STACK {name}: MISSING {', '.join(os.path.basename(p) for p in missing)}")
        return name, False

    common.reset_scene()
    cur_z = 0.0
    all_visuals = []
    for path in paths:
        meshes = _import_visual_meshes(path)
        if not meshes:
            print(f"STACK {name}: no visual mesh in {os.path.basename(path)}")
            return name, False
        min_z, max_z = _z_bounds(meshes)
        shift = cur_z - min_z
        for obj in meshes:
            obj.location.z += shift
        cur_z += max_z - min_z
        all_visuals.extend(meshes)

    bpy.ops.object.select_all(action="DESELECT")
    for obj in all_visuals:
        obj.select_set(True)
    bpy.context.view_layer.objects.active = all_visuals[0]
    if len(all_visuals) > 1:
        bpy.ops.object.join()
    joined = bpy.context.view_layer.objects.active
    joined.name = name
    common.render_preview(name, joined, view=(1.0, 1.0, 0.55))
    return name, True


def build_all():
    return [render_stack(style, size) for style, size in STYLE_SIZES]


if __name__ == "__main__":
    failed = [name for name, ok in build_all() if not ok]
    if failed:
        print("STACK PREVIEW FAILED:", ", ".join(failed))
        sys.exit(1)
    print("STACK PREVIEW OK")
