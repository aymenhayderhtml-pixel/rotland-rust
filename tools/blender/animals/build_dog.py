"""Builds the rigged dog in two coats: assets/models/animals/dog_{tan,dark}.glb.

Run: blender -b --python tools/blender/animals/build_dog.py
Clips come from dog_poses.py (keyed at 60 fps). Previews go to assets/models/_previews:
<name>.png (rest pose), and <name>_<clip>.png (8 frames across each clip, one strip).
"""
import math
import os
import shutil
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, HERE)

import bpy  # noqa: E402
import numpy as np  # noqa: E402
from mathutils import Vector  # noqa: E402
import common  # noqa: E402
import dog_mesh  # noqa: E402
import dog_poses  # noqa: E402
import dog_rig  # noqa: E402

BUDGET = 1200
VARIANTS = [("dog_tan", "dog_tan", "dog_dark"), ("dog_dark", "dog_dark", "tire_black")]
ANIMALS_DIR = os.path.join(common.MODELS_DIR, "animals")
FRAME_W = 240
FRAME_H = 240
DOG_TARGET = Vector((0.0, 0.02, 0.32))
CAMERA_OFFSET = Vector((0.80, 1.60, 0.62))  # 3/4 view from the front right (the dog faces +Y)


def _setup_scene():
    scene = bpy.context.scene
    scene.render.fps = dog_poses.FPS
    scene.render.fps_base = 1.0
    scene.view_settings.view_transform = "Standard"  # keep palette colours exact
    cam_data = bpy.data.cameras.new("DogCamera")
    cam_data.angle = math.radians(40)
    cam = bpy.data.objects.new("DogCamera", cam_data)
    scene.collection.objects.link(cam)
    cam.location = DOG_TARGET + CAMERA_OFFSET
    cam.rotation_euler = (DOG_TARGET - cam.location).to_track_quat("-Z", "Y").to_euler()
    scene.camera = cam
    sun_data = bpy.data.lights.new("DogSun", "SUN")
    sun_data.energy = 3.0
    sun = bpy.data.objects.new("DogSun", sun_data)
    scene.collection.objects.link(sun)
    sun.rotation_euler = (math.radians(55), 0.0, math.radians(135))
    return scene


def _rest_all(arm_obj):
    for pb in arm_obj.pose.bones:
        pb.rotation_mode = "XYZ"
        pb.rotation_euler = (0.0, 0.0, 0.0)
        pb.location = (0.0, 0.0, 0.0)


def _key_clip(arm_obj, clip):
    """One Action per clip. Every bone gets a key on every frame, so no bone keeps a stale pose."""
    act = bpy.data.actions.new(clip)
    act.use_fake_user = True
    if not arm_obj.animation_data:
        arm_obj.animation_data_create()
    arm_obj.animation_data.action = act
    for t in dog_poses.sample_times(clip):
        frame = round(t * dog_poses.FPS) + 1
        pose = dog_poses.pose_at(clip, t)
        for pb in arm_obj.pose.bones:
            rot = pose.get(pb.name, (0.0, 0.0, 0.0))
            pb.rotation_euler = tuple(math.radians(r) for r in rot)
            pb.keyframe_insert(data_path="rotation_euler", frame=frame)
            if pb.name == "hips":
                pb.location = Vector(pose.get("hips_loc", (0.0, 0.0, 0.0)))
                pb.keyframe_insert(data_path="location", frame=frame)
    _rest_all(arm_obj)
    return act


def _frame_for(clip, t):
    return round(t * dog_poses.FPS) + 1


def _render_frame(path, size_w, size_h):
    scene = bpy.context.scene
    scene.render.resolution_x = size_w
    scene.render.resolution_y = size_h
    scene.render.resolution_percentage = 100
    scene.render.image_settings.file_format = "PNG"
    common._setup_workbench(scene)
    scene.render.filepath = path
    bpy.ops.render.render(write_still=True)


def _render_rest(name, arm_obj):
    os.makedirs(common.PREVIEW_DIR, exist_ok=True)
    arm_obj.animation_data.action = None
    _rest_all(arm_obj)
    bpy.context.view_layer.update()
    _render_frame(os.path.join(common.PREVIEW_DIR, name + ".png"), common.PREVIEW_SIZE, common.PREVIEW_SIZE)


def _contact_sheet(name, arm_obj, clip):
    """8 evenly spaced frames across the clip, joined left to right."""
    scene = bpy.context.scene
    length = dog_poses.CLIPS[clip][0]
    arm_obj.animation_data.action = bpy.data.actions[clip]
    temp_dir = tempfile.mkdtemp(prefix="rotland_dog_")
    try:
        frames = []
        for i in range(8):
            t = i * length / 8.0
            scene.frame_set(_frame_for(clip, t))
            bpy.context.view_layer.update()
            path = os.path.join(temp_dir, f"f{i}.png")
            _render_frame(path, FRAME_W, FRAME_H)
            img = bpy.data.images.load(path)
            pixels = np.array(img.pixels[:], dtype=np.float32).reshape(FRAME_H, FRAME_W, 4)
            frames.append(np.flipud(pixels))
            bpy.data.images.remove(img)
        sheet = np.flipud(np.hstack(frames))
        out = bpy.data.images.new(f"{name}_{clip}_sheet", width=FRAME_W * 8, height=FRAME_H, alpha=True)
        out.pixels.foreach_set(sheet.ravel())
        sheet_path = os.path.join(common.PREVIEW_DIR, f"{name}_{clip}.png")
        out.filepath_raw = sheet_path
        out.file_format = "PNG"
        out.save()
        bpy.data.images.remove(out)
        print(f"Contact sheet {name}_{clip} -> {sheet_path}")
    finally:
        shutil.rmtree(temp_dir, ignore_errors=True)
        arm_obj.animation_data.action = None


def build_variant(name, coat, accent):
    common.reset_scene()
    _setup_scene()
    arm_obj = dog_rig.create_armature(name)
    mesh_obj, _ = dog_mesh.build_mesh(arm_obj, name, coat, accent)
    tris = dog_mesh.triangle_count(mesh_obj)

    for clip in dog_poses.CLIPS:
        _key_clip(arm_obj, clip)
    arm_obj.animation_data.action = None
    _rest_all(arm_obj)

    os.makedirs(ANIMALS_DIR, exist_ok=True)
    out_glb = os.path.join(ANIMALS_DIR, name + ".glb")
    bpy.ops.object.select_all(action="DESELECT")
    arm_obj.select_set(True)
    mesh_obj.select_set(True)
    bpy.ops.export_scene.gltf(filepath=out_glb, export_format="GLB", use_selection=True,
                              export_yup=True, export_animation_mode="ACTIONS")

    _render_rest(name, arm_obj)
    for clip in dog_poses.CLIPS:
        _contact_sheet(name, arm_obj, clip)
    ok = tris <= BUDGET
    print(f"DOG {name}: {tris} tris (budget {BUDGET}) {'OK' if ok else 'OVER BUDGET'} -> {out_glb}")
    return ok


def main():
    results = [build_variant(name, coat, accent) for name, coat, accent in VARIANTS]
    return 0 if all(results) else 1


if __name__ == "__main__":
    sys.exit(main())
