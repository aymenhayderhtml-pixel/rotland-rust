"""Renders a 128x128 transparent icon for every item in data/items.json into assets/icons/<id>.png.

Weapons use their weapon models (tools/blender/weapons); other items use item_meshes.py.
Also writes assets/icons/_sheet.png (a labelled grid, via make_sheet.py when python3 has Pillow).

Run: blender -b --python tools/blender/icons/render_icons.py
"""
import importlib
import json
import os
import shutil
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, os.path.join(HERE, "..", "weapons"))

import bpy  # noqa: E402
from mathutils import Vector  # noqa: E402
import common  # noqa: E402
import item_meshes  # noqa: E402

ITEMS_PATH = os.path.join(common.ROOT, "data", "items.json")
ICON_DIR = os.path.join(common.ROOT, "assets", "icons")
SIZE = 128
VIEW_DIR = Vector((1.0, -1.0, 0.7)).normalized()  # same 3/4 angle for every item
PADDING = 1.12  # a little empty margin around each item
OUTLINE = (0.05, 0.05, 0.05)  # dark outline so light items read on dark UI


class _Collector:
    """Stands in for common.Model: keeps the bmeshes a weapon script builds, ignores the rest."""

    def __init__(self):
        self.bms = []

    def add(self, bm):
        self.bms.append(bm)

    def add_collider(self, label, bm):
        pass

    def add_part(self, label, bm):
        self.bms.append(bm)

    def add_empty(self, label, loc):
        pass


def _meshes_for(item_id, item):
    if "weaponId" in item:
        collector = _Collector()
        importlib.import_module(item["weaponId"]).build(collector)
        return collector.bms
    return item_meshes.BUILDERS[item_id]()


def _setup_scene():
    scene = bpy.context.scene
    scene.render.engine = "BLENDER_WORKBENCH"
    scene.render.resolution_x = SIZE
    scene.render.resolution_y = SIZE
    scene.render.resolution_percentage = 100
    scene.render.film_transparent = True
    scene.render.image_settings.file_format = "PNG"
    scene.render.image_settings.color_mode = "RGBA"
    scene.view_settings.view_transform = "Standard"
    shading = scene.display.shading
    shading.light = "STUDIO"
    shading.color_type = "TEXTURE"
    shading.show_object_outline = True
    shading.object_outline_color = OUTLINE
    scene.display.render_aa = "16"
    return scene


def _frame_camera(scene, obj):
    """Orthographic camera at the 3/4 angle, sized so the item fills the square (tight, not a fixed distance)."""
    corners = [Vector(c) for c in obj.bound_box]
    center = sum(corners, Vector()) / 8.0
    cam_data = bpy.data.cameras.new("IconCamera")
    cam_data.type = "ORTHO"
    cam_data.clip_start = 0.01
    cam_data.clip_end = 100.0
    cam = bpy.data.objects.new("IconCamera", cam_data)
    scene.collection.objects.link(cam)
    cam.location = center + VIEW_DIR * 10.0
    cam.rotation_euler = (center - cam.location).to_track_quat("-Z", "Y").to_euler()
    rot = cam.rotation_euler.to_matrix()
    right, up = rot.col[0], rot.col[1]
    rs = [(c - center).dot(right) for c in corners]
    us = [(c - center).dot(up) for c in corners]
    # Re-center the camera on the item's projected box, so the framing is symmetric
    mid_r = (min(rs) + max(rs)) / 2
    mid_u = (min(us) + max(us)) / 2
    cam_data.ortho_scale = max(max(rs) - min(rs), max(us) - min(us), 0.05) * PADDING
    cam.location = center + right * mid_r + up * mid_u + VIEW_DIR * 10.0
    scene.camera = cam


def _render_icon(item_id, bms):
    common.reset_scene()
    scene = _setup_scene()
    merged = common.combine(bms)
    obj = common.object_from_bm(merged, item_id)
    _frame_camera(scene, obj)
    scene.render.filepath = os.path.join(ICON_DIR, item_id + ".png")
    if os.path.exists(scene.render.filepath):
        os.remove(scene.render.filepath)
    bpy.ops.render.render(write_still=True)
    if not os.path.exists(scene.render.filepath):
        raise RuntimeError(f"icon {item_id} was not written")


def _make_sheet():
    python = shutil.which("python3")
    if python is None:
        print("SHEET skipped: python3 not found")
        return
    result = subprocess.run([python, "-I", os.path.join(HERE, "make_sheet.py")],
                            capture_output=True, text=True)
    print(result.stdout.strip() or result.stderr.strip())


def _only_ids():
    """`-- --only=a,b` renders just those icons; the other icon files are left as they are."""
    for arg in sys.argv:
        if arg.startswith("--only="):
            return set(arg.split("=", 1)[1].split(","))
    return None


def main():
    os.makedirs(ICON_DIR, exist_ok=True)
    with open(ITEMS_PATH, encoding="utf-8") as f:
        items = json.load(f)
    only = _only_ids()
    for item_id, item in items.items():
        if only is not None and item_id not in only:
            continue
        if "weaponId" not in item and item_id not in item_meshes.BUILDERS:
            # No mesh builder yet (e.g. fishing_rod): keep the existing PNG instead of stopping the run.
            print("SKIP (no mesh builder)", item_id)
            continue
        _render_icon(item_id, _meshes_for(item_id, item))
        print("ICON", item_id)
    if only is None:
        _make_sheet()
    print(f"ICONS OK: {len(items)} icons in {ICON_DIR}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
