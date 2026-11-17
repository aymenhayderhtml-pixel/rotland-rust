"""Builds the hatchback in two colours: assets/models/vehicles/car_hatchback_{red,blue}.glb.

Run: blender -b --python tools/blender/vehicles/build_car.py
Each part is its own node (body, door_L, wheel_FL..RR, collision, empties), see car_parts.py.
Previews: assets/models/_previews/car_hatchback_<colour>.png (3/4 front) and _side.png.
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, HERE)

import bmesh  # noqa: E402
import bpy  # noqa: E402
import common  # noqa: E402
import car_parts  # noqa: E402

BUDGET = 2500
VARIANTS = [("car_hatchback_red", "car_red"), ("car_hatchback_blue", "car_blue")]
OUT_DIR = os.path.join(common.MODELS_DIR, "vehicles")
FRONT_VIEW = (1.0, 1.0, 0.55)  # 3/4 from the front right (the front faces +Y)
SIDE_VIEW = (1.0, 0.0, 0.10)  # straight from the right side


def _preview_bm(parts):
    """One joined copy of the visible parts (no collider), made before the parts are freed."""
    copies = []
    for part_name, bm, loc in parts:
        if part_name.endswith("-convcolonly"):
            continue
        copy = bm.copy()
        bmesh.ops.translate(copy, vec=loc, verts=list(copy.verts))
        copies.append(copy)
    return common.combine(copies)


def build_variant(name, body_color):
    common.reset_scene()
    parts = car_parts.build_parts(body_color)
    preview_bm = _preview_bm(parts)

    visible_tris = 0
    for part_name, bm, loc in parts:
        if not part_name.endswith("-convcolonly"):
            visible_tris += len(bm.faces)
        obj = common.object_from_bm(bm, part_name)  # frees bm
        obj.location = loc
    for label, loc in car_parts.EMPTIES:
        empty = bpy.data.objects.new(label, None)
        empty.empty_display_type = "PLAIN_AXES"
        empty.empty_display_size = 0.05
        empty.location = loc
        bpy.context.scene.collection.objects.link(empty)

    os.makedirs(OUT_DIR, exist_ok=True)
    out_path = os.path.join(OUT_DIR, name + ".glb")
    common._export_glb(out_path)
    # The collider box fills the cabin outline, so keep it out of the picture (the .glb keeps it)
    for obj in bpy.context.scene.objects:
        if obj.name.endswith("-convcolonly"):
            obj.hide_render = True

    # Preview object is added after the export, so it never ends up in the .glb
    preview = common.object_from_bm(preview_bm, name + "_preview")
    common.render_preview(name, preview, view=FRONT_VIEW)
    common.render_preview(name + "_side", preview, view=SIDE_VIEW)
    ok = visible_tris <= BUDGET
    print(f"CAR {name}: {visible_tris} tris (budget {BUDGET}) {'OK' if ok else 'OVER BUDGET'} -> {out_path}")
    return ok


def main():
    results = [build_variant(name, color) for name, color in VARIANTS]
    return 0 if all(results) else 1


if __name__ == "__main__":
    sys.exit(main())
