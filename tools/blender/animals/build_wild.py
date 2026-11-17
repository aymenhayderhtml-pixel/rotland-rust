"""Builds the wild animals (V12-01): deer, deer_buck, rabbit, boar. Contract: docs/ANIMAL_CONTRACT.md.

Run all:   blender -b --python tools/blender/animals/build_wild.py
Run some:  blender -b --python tools/blender/animals/build_wild.py -- rabbit boar
Writes assets/models/animals/<name>.glb. Previews go to assets/models/_previews: <name>.png (rest
pose) and <name>_<clip>.png (8 frames across each clip, one strip). Also prints a ground check per
clip: the lowest mesh point (below 0 means a leg goes through the floor).
Temp frames go to $ROTLAND_TMP when set (the scratchpad), else the system temp folder.
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
import wild_mesh  # noqa: E402
import wild_poses  # noqa: E402
import wild_rig  # noqa: E402
import wild_species  # noqa: E402

ANIMALS_DIR = os.path.join(common.MODELS_DIR, "animals")
FRAME_W = 240
FRAME_H = 240
CAMERA_DIR = Vector((0.8, 1.6, 0.62)).normalized()  # 3/4 view from the front right (the animal faces +Y)
TEMP_ROOT = os.environ.get("ROTLAND_TMP") or None


def _setup_scene(center, radius):
    scene = bpy.context.scene
    scene.render.fps = wild_poses.FPS
    scene.render.fps_base = 1.0
    scene.view_settings.view_transform = "Standard"  # keep palette colours exact
    cam_data = bpy.data.cameras.new("WildCamera")
    cam_data.angle = math.radians(40)
    cam = bpy.data.objects.new("WildCamera", cam_data)
    scene.collection.objects.link(cam)
    cam.location = center + CAMERA_DIR * (radius / math.sin(math.radians(20)) * 1.1)
    cam.rotation_euler = (center - cam.location).to_track_quat("-Z", "Y").to_euler()
    scene.camera = cam
    sun_data = bpy.data.lights.new("WildSun", "SUN")
    sun_data.energy = 3.0
    sun = bpy.data.objects.new("WildSun", sun_data)
    scene.collection.objects.link(sun)
    sun.rotation_euler = (math.radians(55), 0.0, math.radians(135))
    return scene


def _rest_all(arm_obj):
    for pb in arm_obj.pose.bones:
        pb.rotation_mode = "XYZ"
        pb.rotation_euler = (0.0, 0.0, 0.0)
        pb.location = (0.0, 0.0, 0.0)


def _set_pose(arm_obj, pose):
    for pb in arm_obj.pose.bones:
        rot = pose.get(pb.name, (0.0, 0.0, 0.0))
        pb.rotation_mode = "XYZ"
        pb.rotation_euler = tuple(math.radians(r) for r in rot)
        pb.location = Vector(pose.get("hips_loc", (0.0, 0.0, 0.0))) if pb.name == "hips" else Vector()


def _floor_lift(arm_obj, mesh_obj, pose):
    """How far the body must rise so that no vertex is below the floor in this pose (0 if none is)."""
    _set_pose(arm_obj, pose)
    bpy.context.view_layer.update()
    return max(0.0, -_lowest_z(mesh_obj))


def _key_clip(arm_obj, mesh_obj, clips, clip):
    """One Action per clip. Every bone gets a key on every frame, so no bone keeps a stale pose.

    A frame whose legs or head would go through the floor is lifted by the body (hips), only as far
    as needed. Those frames are measured first, with no action assigned, so the action cannot
    override the test pose.
    """
    if not arm_obj.animation_data:
        arm_obj.animation_data_create()
    arm_obj.animation_data.action = None
    keyed = []
    for t in wild_poses.sample_times(clips, clip):
        pose = dict(wild_poses.pose_at(clips, clip, t))
        lift = _floor_lift(arm_obj, mesh_obj, pose)
        if lift > 0.0:
            x, y, z = pose.get("hips_loc", (0.0, 0.0, 0.0))
            pose["hips_loc"] = (x, y, z + lift)
        keyed.append((t, pose))
    act = bpy.data.actions.new(clip)
    act.use_fake_user = True
    arm_obj.animation_data.action = act
    for t, pose in keyed:
        frame = _frame_for(t)
        for pb in arm_obj.pose.bones:
            rot = pose.get(pb.name, (0.0, 0.0, 0.0))
            pb.rotation_euler = tuple(math.radians(r) for r in rot)
            pb.keyframe_insert(data_path="rotation_euler", frame=frame)
            if pb.name == "hips":
                pb.location = Vector(pose.get("hips_loc", (0.0, 0.0, 0.0)))
                pb.keyframe_insert(data_path="location", frame=frame)
    _rest_all(arm_obj)
    return act


def _frame_for(t):
    return round(t * wild_poses.FPS) + 1


def _render_frame(path, size_w, size_h):
    scene = bpy.context.scene
    scene.render.resolution_x = size_w
    scene.render.resolution_y = size_h
    scene.render.resolution_percentage = 100
    scene.render.image_settings.file_format = "PNG"
    common._setup_workbench(scene)
    scene.render.filepath = path
    bpy.ops.render.render(write_still=True)


def _lowest_z(mesh_obj):
    """Lowest world z of the posed (evaluated) mesh."""
    evaluated = mesh_obj.evaluated_get(bpy.context.evaluated_depsgraph_get())
    mesh = evaluated.to_mesh()
    try:
        return min((mesh_obj.matrix_world @ v.co).z for v in mesh.vertices)
    finally:
        evaluated.to_mesh_clear()


def _ground_check(name, arm_obj, mesh_obj, clips):
    print(f"GROUND {name} (lowest point per clip, metres; below 0 = through the floor):")
    for clip in clips:
        arm_obj.animation_data.action = bpy.data.actions[clip]
        lows = []
        for t in wild_poses.sample_times(clips, clip):
            bpy.context.scene.frame_set(_frame_for(t))
            bpy.context.view_layer.update()
            lows.append(_lowest_z(mesh_obj))
        print(f"  {clip:7s} min {min(lows):+.3f}  max {max(lows):+.3f}")
    arm_obj.animation_data.action = None


def _rest_preview(name, arm_obj, size):
    previews = os.path.join(common.PREVIEW_DIR)
    os.makedirs(previews, exist_ok=True)
    arm_obj.animation_data.action = None
    _rest_all(arm_obj)
    bpy.context.view_layer.update()
    _render_frame(os.path.join(previews, name + ".png"), size, size)


def _contact_sheet(name, arm_obj, clips, clip):
    """8 evenly spaced frames across the clip, joined left to right."""
    scene = bpy.context.scene
    length = clips[clip][0]
    arm_obj.animation_data.action = bpy.data.actions[clip]
    temp_dir = tempfile.mkdtemp(prefix=f"rotland_{name}_", dir=TEMP_ROOT)
    try:
        frames = []
        for i in range(8):
            scene.frame_set(_frame_for(i * length / 8.0))
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
    finally:
        shutil.rmtree(temp_dir, ignore_errors=True)
        arm_obj.animation_data.action = None


def _remove_side_textures(name):
    """Deletes the palette PNGs the glTF exporter writes next to the glb (the glb embeds them)."""
    for fname in os.listdir(ANIMALS_DIR):
        if fname.startswith(name + "_palette") and fname.endswith((".png", ".png.import")):
            os.remove(os.path.join(ANIMALS_DIR, fname))


def build_model(name, spec):
    """One model: rig, mesh, clips, export, ground check, previews. Returns True if within budget."""
    common.reset_scene()
    arm_obj = wild_rig.create_armature(name, spec)
    mesh_obj, _ = wild_mesh.build_mesh(arm_obj, name, spec)
    tris = wild_mesh.triangle_count(mesh_obj)

    corners = [mesh_obj.matrix_world @ Vector(c) for c in mesh_obj.bound_box]
    lo = Vector((min(c.x for c in corners), min(c.y for c in corners), min(c.z for c in corners)))
    hi = Vector((max(c.x for c in corners), max(c.y for c in corners), max(c.z for c in corners)))
    _setup_scene((lo + hi) / 2, (hi - lo).length / 2)

    clips = wild_poses.CLIPS[spec["style"]]
    for clip in clips:
        _key_clip(arm_obj, mesh_obj, clips, clip)
    arm_obj.animation_data.action = None
    _rest_all(arm_obj)

    os.makedirs(ANIMALS_DIR, exist_ok=True)
    out_glb = os.path.join(ANIMALS_DIR, name + ".glb")
    bpy.ops.object.select_all(action="DESELECT")
    arm_obj.select_set(True)
    mesh_obj.select_set(True)
    bpy.ops.export_scene.gltf(filepath=out_glb, export_format="GLB", use_selection=True,
                              export_yup=True, export_animation_mode="ACTIONS")
    _remove_side_textures(name)  # the exporter also writes <name>_palette*.png; the glb embeds them

    _ground_check(name, arm_obj, mesh_obj, clips)
    size = common.PREVIEW_SIZE
    _rest_preview(name, arm_obj, size)
    for clip in clips:
        _contact_sheet(name, arm_obj, clips, clip)
    budget = spec["budget"]
    ok = tris <= budget
    height = hi.z - lo.z
    length = hi.y - lo.y
    print(f"WILD {name}: {tris} tris (budget {budget}) {'OK' if ok else 'OVER BUDGET'}, "
          f"height {height:.2f} m, length {length:.2f} m -> {out_glb}")
    return ok


def _picked_names():
    argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    names = argv or list(wild_species.SPECIES)
    unknown = [n for n in names if n not in wild_species.SPECIES]
    if unknown:
        raise SystemExit(f"unknown species: {unknown}")
    return names


def main():
    results = [build_model(name, wild_species.SPECIES[name]) for name in _picked_names()]
    return 0 if all(results) else 1


if __name__ == "__main__":
    sys.exit(main())
