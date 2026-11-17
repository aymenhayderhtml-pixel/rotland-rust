"""Backpacks worn on the player's back (V11-03): backpack_small.glb and backpack_large.glb.

Build coordinates (1 unit = 1 m). The origin is the attach point on the upper back, at the top
front edge of the bag. The bag hangs down (-Z) and sits behind the body (-Y); its front face
touches the back of the chest.

Attach: put the glb under a BoneAttachment3D on the `chest` bone. The attach point is at rest
0.14 m above the chest bone's head and 0.10 m behind it (Godot offset (0, 0.14, 0.10)).

Also writes assets/models/_previews/<name>_worn.png: the rest-pose player with the bag on, for review.

Run: blender -b --factory-startup --python tools/blender/gear/backpacks.py
"""
import math
import os
import sys

from mathutils import Vector

HERE = os.path.dirname(os.path.abspath(__file__))
TOOLS_DIR = os.path.abspath(os.path.join(HERE, ".."))
sys.path.insert(0, TOOLS_DIR)
sys.path.insert(0, os.path.join(TOOLS_DIR, "characters"))

import bpy  # noqa: E402
import build_characters  # noqa: E402
import common  # noqa: E402
import humanoid  # noqa: E402
import palette  # noqa: E402

# Attach point in rest-pose Blender coordinates: chest bone head (0, 0, 1.22) + (0, -0.10, 0.14)
ATTACH_WORLD = (0.0, -0.10, 1.36)


def _straps(width):
    """Two straps on the back of the chest, rising from the top of the bag (visible from behind)."""
    return [common.make_box((0.04, 0.012, 0.08), loc=(side * width, -0.006, 0.0), color="leather")
            for side in (-1, 1)]


def small_parts():
    return [
        common.make_box((0.26, 0.12, 0.30), loc=(0.0, -0.06, -0.15), color="canvas_khaki"),  # body
        common.make_box((0.28, 0.13, 0.07), loc=(0.0, -0.065, -0.025), color="cloth_olive"),  # top flap
        *_straps(0.09),
    ]


def large_parts():
    bedroll = common.make_frustum(0.055, 0.055, 0.0, 0.26, segments=8, color="canvas_khaki")
    common.transform(bedroll, rot=(0.0, math.pi / 2, 0.0), loc=(-0.13, -0.09, -0.42))
    return [
        common.make_box((0.30, 0.18, 0.40), loc=(0.0, -0.09, -0.20), color="cloth_olive"),  # body
        common.make_box((0.32, 0.19, 0.12), loc=(0.0, -0.0925, -0.04), color="leather"),  # top flap
        common.make_box((0.05, 0.12, 0.20), loc=(0.165, -0.08, -0.26), color="canvas_khaki"),  # side pocket
        bedroll,  # rolled sleeping mat under the bag
        *_straps(0.10),
    ]


SPECS = [
    # name, triangle budget, part builder, preview view (from behind, left side)
    ("backpack_small", 200, small_parts, (-1.0, -1.0, 0.6)),
    ("backpack_large", 300, large_parts, (-1.0, -1.0, 0.6)),
]


def render_worn(name, build):
    """Rest-pose human_player with the bag on, from behind. Preview only, not exported."""
    common.reset_scene()
    palette.write_palette_png()
    palette.write_palette_png_b()
    humanoid.build_humanoid(build_characters.VARIANTS[0])  # human_player, rest pose

    bag = common.combine(build())
    obj = common.object_from_bm(bag, name + "_worn")
    obj.location = Vector(ATTACH_WORLD)

    scene = bpy.context.scene
    scene.render.resolution_x = common.PREVIEW_SIZE
    scene.render.resolution_y = common.PREVIEW_SIZE
    scene.render.resolution_percentage = 100
    scene.render.image_settings.file_format = "PNG"
    scene.view_settings.view_transform = "Standard"
    common._setup_workbench(scene)

    cam_data = bpy.data.cameras.new("WornCamera")
    cam_data.angle = math.radians(40)
    cam = bpy.data.objects.new("WornCamera", cam_data)
    scene.collection.objects.link(cam)
    target = Vector((0.0, -0.05, 1.0))
    cam.location = Vector((-1.7, -2.9, 1.5))
    cam.rotation_euler = (target - cam.location).to_track_quat("-Z", "Y").to_euler()
    scene.camera = cam

    path = os.path.join(common.PREVIEW_DIR, f"{name}_worn.png")
    os.makedirs(common.PREVIEW_DIR, exist_ok=True)
    scene.render.filepath = path
    bpy.ops.render.render(write_still=True)
    print(f"worn preview {name} -> {path}")


def main():
    palette.write_palette_png()
    palette.write_palette_png_b()
    failures = []
    for name, budget, build, view in SPECS:
        common.reset_scene()
        model = common.Model(name, "gear", budget, grip=(0.0, 0.0, 0.0), view=view)
        for bm in build():
            model.add(bm)
        _, ok = model.finalize_and_export()
        if not ok:
            failures.append(name)
        render_worn(name, build)
    if failures:
        print("GEAR FAILED:", ", ".join(failures))
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
