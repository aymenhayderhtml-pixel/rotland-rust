"""Builds the 4x4 Outdoor Survival SUV variants and animated sub-parts:
  assets/models/vehicles/car_hatchback_{red,blue,olive}.glb  (body + collider + empties)
  assets/models/vehicles/car_door_{red,blue,olive}.glb       (hinged driver door)
  assets/models/vehicles/car_tailgate_{red,blue,olive}.glb   (hinged tailgate + spare tire)
  assets/models/vehicles/car_wheel.glb                       (shared off-road wheel)

Run: blender -b --python tools/blender/vehicles/build_car.py
Previews: assets/models/_previews/car_hatchback_<colour>{,_side,_rear}.png (fully assembled).
"""
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, HERE)

import bmesh  # noqa: E402
import bpy  # noqa: E402
import common  # noqa: E402
import car_parts  # noqa: E402

BUDGET = 4500  # measured 4,300 for the assembled vehicle (body 2,172 + door 156 + gate 468 + 4x376 wheels)
VARIANTS = [
    ("red", "car_red"),
    ("blue", "car_blue"),
    ("olive", "cloth_olive"),
]
OUT_DIR = os.path.join(common.MODELS_DIR, "vehicles")
FRONT_VIEW = (1.1, 1.0, 0.48)    # 3/4 from front right (+Y is front)
SIDE_VIEW = (-1.0, 0.0, 0.08)    # driver-side profile
REAR_VIEW = (-1.1, -1.0, 0.48)   # 3/4 from rear left


def _export_single_part(filename, mesh_name, bm):
    common.reset_scene()
    tris = len(bm.faces)
    common.object_from_bm(bm, mesh_name)  # frees bm
    os.makedirs(OUT_DIR, exist_ok=True)
    out_path = os.path.join(OUT_DIR, filename + ".glb")
    common._export_glb(out_path)
    print(f"CAR PART {filename}: {tris} tris -> {out_path}")
    return tris


def _build_assembled_preview(body_bm, door_bm, gate_bm, wheel_bm):
    """Combines copies of all animated parts at their rest locations for clean preview renders."""
    copies = [body_bm.copy()]

    door_copy = door_bm.copy()
    bmesh.ops.translate(door_copy, vec=car_parts.DOOR_HINGE, verts=list(door_copy.verts))
    copies.append(door_copy)

    gate_copy = gate_bm.copy()
    bmesh.ops.translate(gate_copy, vec=car_parts.TAILGATE_HINGE, verts=list(gate_copy.verts))
    copies.append(gate_copy)

    for wname, (wx, wy) in car_parts.WHEEL_POINTS.items():
        wcopy = wheel_bm.copy()
        if wx > 0.0:
            # Mirror right-side wheels 180 deg around Z so outer rim faces +X
            common.transform(wcopy, rot=(0.0, 0.0, math.pi))
        bmesh.ops.translate(wcopy, vec=(wx, wy, car_parts.WHEEL_RADIUS), verts=list(wcopy.verts))
        copies.append(wcopy)

    return common.combine(copies)


def build_variant(colour_key, body_color, wheel_tris):
    # 1. Build all parts so we can render an assembled preview before freeing bmeshes
    body_parts = car_parts.build_body_and_collider(body_color)
    door_bm = car_parts.build_door_left(body_color)
    gate_bm = car_parts.build_tailgate(body_color)
    wheel_bm = car_parts.build_wheel()

    body_bm = body_parts[0][1]
    preview_bm = _build_assembled_preview(body_bm, door_bm, gate_bm, wheel_bm)
    wheel_bm.free()

    # 2. Export separate hinged door & tailgate .glb parts
    door_tris = _export_single_part(f"car_door_{colour_key}", "door_L", door_bm)
    gate_tris = _export_single_part(f"car_tailgate_{colour_key}", "tailgate", gate_bm)

    # 3. Export main body .glb (body + collider + all game markers/empties)
    common.reset_scene()
    body_tris = 0
    for part_name, bm, loc in body_parts:
        if not part_name.endswith("-convcolonly"):
            body_tris += len(bm.faces)
        obj = common.object_from_bm(bm, part_name)  # frees bm
        obj.location = loc

    for label, loc in car_parts.EMPTIES:
        empty = bpy.data.objects.new(label, None)
        empty.empty_display_type = "PLAIN_AXES"
        empty.empty_display_size = 0.05
        empty.location = loc
        bpy.context.scene.collection.objects.link(empty)

    os.makedirs(OUT_DIR, exist_ok=True)
    name = f"car_hatchback_{colour_key}"
    out_path = os.path.join(OUT_DIR, name + ".glb")
    common._export_glb(out_path)

    # Hide collider & body for assembled preview renders
    for obj in bpy.context.scene.objects:
        if obj.type == "MESH":
            obj.hide_render = True

    preview = common.object_from_bm(preview_bm, name + "_preview")
    common.render_preview(name, preview, view=FRONT_VIEW)
    common.render_preview(name + "_side", preview, view=SIDE_VIEW)
    common.render_preview(name + "_rear", preview, view=REAR_VIEW)

    total_tris = body_tris + door_tris + gate_tris + 4 * wheel_tris
    ok = total_tris <= BUDGET
    print(
        f"CAR {name}: body={body_tris} door={door_tris} gate={gate_tris} "
        f"wheels=4x{wheel_tris} -> total={total_tris} tris (budget {BUDGET}) "
        f"{'OK' if ok else 'OVER BUDGET'} -> {out_path}"
    )
    return ok


def main():
    wheel_bm = car_parts.build_wheel()
    wheel_tris = _export_single_part("car_wheel", "wheel", wheel_bm)
    results = [build_variant(key, color, wheel_tris) for key, color in VARIANTS]
    return 0 if all(results) else 1


if __name__ == "__main__":
    sys.exit(main())
