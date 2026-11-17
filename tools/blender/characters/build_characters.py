"""Builds humanoid character models, animations, previews, and contact sheets.

Characters:
- human_player: survivor with light skin, blue shirt, denim pants, dark hair
- zombie_01: decayed walker with green skin, stained rotted shirt with blood patches,
            dark pants, slightly hunched posture
- zombie_02: broader and a little bigger, faded red shirt, one bare arm (torn sleeve)
- zombie_03: thin and tall, pale skin, torn blue hospital gown

Clips are listed in poses.py. The player gets every non-zombie clip; zombies get ZOMBIE_CLIPS.

Exports:
- assets/models/characters/human_player.glb, zombie_01.glb, zombie_02.glb, zombie_03.glb
- assets/models/_previews/<name>.png (rest pose 384x384)
- assets/models/_previews/<name>_<clip>.png (8-frame contact sheet, 200x260 per frame)
"""
import math
import os
import shutil
import sys
import tempfile

import bpy
import numpy as np
from mathutils import Euler, Matrix, Vector

HERE = os.path.dirname(os.path.abspath(__file__))
TOOLS_DIR = os.path.abspath(os.path.join(HERE, ".."))
ROOT_DIR = os.path.abspath(os.path.join(TOOLS_DIR, "..", ".."))
sys.path.insert(0, TOOLS_DIR)
sys.path.insert(0, HERE)

import common  # noqa: E402
import humanoid  # noqa: E402
import palette  # noqa: E402
import poses  # noqa: E402

CHARACTERS_DIR = os.path.join(ROOT_DIR, "assets", "models", "characters")
PREVIEWS_DIR = os.path.join(ROOT_DIR, "assets", "models", "_previews")
FPS = 30
BUDGET = 5000

VARIANTS = [
    {
        "name": "human_player",
        "skin": "skin_light",
        "hair": "hair_dark",
        "shirt": "shirt_blue",
        "pants": "pants_denim",
        "shoes": "shoe_dark",
        "blood": None,
        "hunched": False,
    },
    {
        "name": "zombie_01",
        "skin": "skin_zombie",
        "hair": "hair_dark",
        "shirt": "shirt_zombie",
        "pants": "pants_dark",
        "shoes": "shoe_dark",
        "blood": "blood_dark",
        "hunched": True,
        # Degrees added to every clip: forward lean (negative X = forward)
        "posture": {"spine": (-10, 0, 0), "chest": (-8, 0, 0), "neck": (-4, 0, 0), "head": (-6, 0, 0)},
    },
    {
        "name": "zombie_02",
        "skin": "skin_zombie",
        "hair": "hair_dark",
        "shirt": "shirt_red",
        "pants": "pants_denim",
        "shoes": "shoe_dark",
        "blood": "blood_dark",
        "hunched": False,
        "bare_arm": "L",
        # Broader and a bit bigger than zombie_01
        "proportions": {"width": 1.12, "depth": 1.06, "height": 1.02},
    },
    {
        "name": "zombie_03",
        "skin": "skin_pale",
        "hair": "hair_dark",
        "shirt": "gown_blue",
        "pants": "pants_dark",
        "shoes": "shoe_dark",
        "blood": "blood_dark",
        "hunched": False,
        # Thinner and taller (still inside the 1.70-1.90 m height rule)
        "proportions": {"width": 0.88, "depth": 0.94, "height": 1.04},
    },
    # Settlement NPCs (V1-15): same rig and clips as the player
    {
        "name": "npc_trader",
        "skin": "skin_light",
        "hair": "rock_light",
        "shirt": "bark_light",
        "pants": "pants_dark",
        "shoes": "shoe_dark",
        "blood": None,
        "hunched": False,
        "extras": "trader",
    },
    {
        "name": "npc_guard",
        "skin": "skin_light",
        "hair": "hair_dark",
        "shirt": "shoe_dark",
        "pants": "pants_dark",
        "shoes": "shoe_dark",
        "blood": None,
        "hunched": False,
        "extras": "guard",
    },
    {
        "name": "npc_survivor",
        "skin": "skin_light",
        "hair": "wood_crate",
        "shirt": "leaf_mid",
        "pants": "pants_denim",
        "shoes": "shoe_dark",
        "blood": None,
        "hunched": False,
        "extras": "survivor",
    },
    # Villager (V11-03): a woman's look with narrower shoulders and long hair
    {
        "name": "npc_villager",
        "skin": "skin_light",
        "hair": "bark_dark",
        "shirt": "gown_blue",
        "pants": "pants_dark",
        "shoes": "shoe_dark",
        "blood": None,
        "hunched": False,
        "extras": "villager",
        "proportions": {"width": 0.92, "depth": 0.94, "height": 0.98},
    },
]

# Zombies only use this subset. Everything else in poses.CLIPS is for the player.
ZOMBIE_CLIPS = [
    "idle", "walk", "run", "die", "hit", "zombie_shamble", "zombie_attack",
    # Phase 3 zombie clips (V11-06)
    "zombie_stagger", "zombie_lurch", "zombie_crawl", "zombie_bang",
    "zombie_eat", "zombie_scream", "zombie_runner_sprint",
]


def clips_for(variant):
    """Clip names this character gets exported with, in poses.CLIPS order.

    Player-only clips (poses.PLAYER_ONLY_CLIPS) go to human_player alone, so NPC glbs stay unchanged.
    """
    if variant["name"].startswith("zombie_"):
        return [c for c in poses.CLIPS if c in ZOMBIE_CLIPS]
    names = [c for c in poses.CLIPS if not c.startswith("zombie_")]
    if variant["name"] == "human_player":
        names += list(poses.PLAYER_ONLY_CLIPS)
    return names


def setup_character_camera(scene, hunched=False):
    """Sets up a camera framed for 3/4 front view of the character with comfortable margins."""
    cam_data = bpy.data.cameras.new("CharacterCamera")
    cam_data.angle = math.radians(44)
    cam = bpy.data.objects.new("CharacterCamera", cam_data)
    scene.collection.objects.link(cam)

    target = Vector((0.0, -0.05, 0.82))
    # 3/4 front view: +X (right), +Y (in front of facing direction), +Z (above)
    cam.location = Vector((1.6, 2.85, 1.45))
    direction = target - cam.location
    cam.rotation_euler = direction.to_track_quat("-Z", "Y").to_euler()
    scene.camera = cam

    # Sun light
    sun_data = bpy.data.lights.new("CharacterSun", "SUN")
    sun_data.energy = 3.5
    sun = bpy.data.objects.new("CharacterSun", sun_data)
    scene.collection.objects.link(sun)
    sun.rotation_euler = (math.radians(55), 0.0, math.radians(135))
    return cam


def _with_offsets(pose, offsets):
    """Adds per-bone rotation offsets to every keyed pose (absent bones count as rest)."""
    out = dict(pose)
    for bone, delta in offsets.items():
        entry = out.get(bone, (0, 0, 0))
        if isinstance(entry, dict):
            rot = entry.get("rot", (0, 0, 0))
            out[bone] = {**entry, "rot": tuple(r + d for r, d in zip(rot, delta))}
        else:
            out[bone] = tuple(r + d for r, d in zip(entry, delta))
    return out


def apply_pose_keyframes(arm_obj, clip_name, clip_data, posture=None):
    """Creates a Blender Action and keys bone transforms from the pose table.

    Arms-down and the variant's posture are added on top of every keyframe.
    Every bone gets keys in every clip: bones the clip does not mention are held at rest.
    Without this, a bone keeps whatever the previous clip left it at (e.g. a bent leg
    in an upper-body clip).
    """
    act = bpy.data.actions.new(name=clip_name)
    act.use_fake_user = True
    if not arm_obj.animation_data:
        arm_obj.animation_data_create()
    arm_obj.animation_data.action = act
    for pb in arm_obj.pose.bones:
        pb.rotation_mode = "XYZ"
        pb.rotation_euler = (0.0, 0.0, 0.0)
        pb.location = (0.0, 0.0, 0.0)

    offsets = {arm: (0, 0, down) for arm, down in poses.ARM_DOWN_OFFSET.items()}
    offsets.update(posture or {})
    frames_dict = {t: _with_offsets(p, offsets) for t, p in clip_data["frames"].items()}
    keyed = {bone for pose in frames_dict.values() for bone in pose if bone in arm_obj.pose.bones}
    for bone_name in arm_obj.pose.bones.keys():
        if bone_name in keyed:
            continue
        pb = arm_obj.pose.bones[bone_name]
        pb.rotation_euler = tuple(math.radians(r) for r in offsets.get(bone_name, (0, 0, 0)))
        pb.location = (0.0, 0.0, 0.0)
        pb.keyframe_insert(data_path="rotation_euler", frame=1)
        pb.keyframe_insert(data_path="location", frame=1)

    for t_sec, pose in frames_dict.items():
        frame_idx = round(t_sec * FPS) + 1
        for bone_name, transform in pose.items():
            if bone_name not in arm_obj.pose.bones:
                continue
            pb = arm_obj.pose.bones[bone_name]
            pb.rotation_mode = "XYZ"

            if isinstance(transform, dict):
                rot = transform.get("rot")
                loc = transform.get("loc")
            else:
                rot = transform
                loc = None

            if rot is not None:
                pb.rotation_euler = (math.radians(rot[0]), math.radians(rot[1]), math.radians(rot[2]))
                pb.keyframe_insert(data_path="rotation_euler", frame=frame_idx)
            if loc is not None:
                pb.location = Vector(loc)
                pb.keyframe_insert(data_path="location", frame=frame_idx)
    return act


def render_rest_preview(name, arm_obj, mesh_obj):
    """Renders 384x384 rest-pose preview to assets/models/_previews/<name>.png."""
    scene = bpy.context.scene
    # Set to rest pose (frame 0 without action)
    if arm_obj.animation_data:
        arm_obj.animation_data.action = None
    for pb in arm_obj.pose.bones:
        pb.rotation_euler = (0, 0, 0)
        pb.location = (0, 0, 0)
    bpy.context.view_layer.update()

    setup_character_camera(scene)
    scene.render.resolution_x = common.PREVIEW_SIZE
    scene.render.resolution_y = common.PREVIEW_SIZE
    scene.render.resolution_percentage = 100
    scene.render.image_settings.file_format = "PNG"
    scene.view_settings.view_transform = "Standard"

    common._setup_workbench(scene)
    out_path = os.path.join(PREVIEWS_DIR, f"{name}.png")
    scene.render.filepath = out_path
    bpy.ops.render.render(write_still=True)
    print(f"Rest preview {name} -> {out_path}")


def frame_camera(cam, wide):
    """Aims the existing camera. `wide` pulls back for clips that spread the body (die)."""
    target = Vector((0.0, -0.05, 0.82))
    offset = Vector((1.6, 2.9, 0.63))
    angle = 44
    if wide:
        target = Vector((0.0, -0.55, 0.6))
        offset = offset * 1.3
        angle = 46
    cam.data.angle = math.radians(angle)
    cam.location = target + offset
    cam.rotation_euler = (target - cam.location).to_track_quat("-Z", "Y").to_euler()


def render_contact_sheet(name, arm_obj, clip_name, clip_data):
    """Renders 8 evenly spaced frames across the clip and stitches them into a 1600x260 PNG."""
    scene = bpy.context.scene
    act = bpy.data.actions.get(clip_name)
    arm_obj.animation_data.action = act
    frame_camera(scene.camera, wide=(clip_name == "die"))

    length = clip_data["length"]
    # 8 sample times
    times = [i * (length / 8.0) for i in range(8)]

    frame_w = 200
    frame_h = 260
    scene.render.resolution_x = frame_w
    scene.render.resolution_y = frame_h
    scene.render.resolution_percentage = 100
    scene.render.image_settings.file_format = "PNG"
    scene.view_settings.view_transform = "Standard"
    common._setup_workbench(scene)

    temp_dir = tempfile.mkdtemp(prefix="rotland_contact_")
    try:
        frame_arrays = []
        for i, t in enumerate(times):
            frame_num = round(t * FPS) + 1
            scene.frame_set(frame_num)
            bpy.context.view_layer.update()

            frame_path = os.path.join(temp_dir, f"frame_{i}.png")
            scene.render.filepath = frame_path
            bpy.ops.render.render(write_still=True)

            img = bpy.data.images.load(frame_path)
            pixels = np.array(img.pixels[:], dtype=np.float32).reshape(frame_h, frame_w, 4)
            # Flip vertically because Blender images are stored bottom-up
            pixels = np.flipud(pixels)
            frame_arrays.append(pixels)
            bpy.data.images.remove(img)

        # Stitch horizontally: 8 x 200 = 1600 width, 260 height
        sheet = np.hstack(frame_arrays)
        sheet_flipped = np.flipud(sheet)

        sheet_img = bpy.data.images.new("contact_sheet", width=frame_w * 8, height=frame_h, alpha=True)
        sheet_img.pixels.foreach_set(sheet_flipped.ravel())
        out_path = os.path.join(PREVIEWS_DIR, f"{name}_{clip_name}.png")
        sheet_img.filepath_raw = out_path
        sheet_img.file_format = "PNG"
        sheet_img.save()
        bpy.data.images.remove(sheet_img)
        print(f"Contact sheet {name}_{clip_name} -> {out_path}")
    finally:
        shutil.rmtree(temp_dir, ignore_errors=True)


def build_one_character(variant):
    name = variant["name"]
    common.reset_scene()
    palette.write_palette_png()

    arm_obj, mesh_obj = humanoid.build_humanoid(variant)
    tris = len(mesh_obj.data.polygons)
    ok = tris <= BUDGET
    if not ok:
        print(f"ERROR: {name} has {tris} triangles, over budget {BUDGET}!")

    # Render rest preview before setting animation actions
    render_rest_preview(name, arm_obj, mesh_obj)

    # Keyframe this character's clips from poses.py (one action each, exported as-is)
    for clip_name in clips_for(variant):
        clip_data = poses.ALL_CLIPS[clip_name]
        apply_pose_keyframes(arm_obj, clip_name, clip_data, variant.get("posture"))
        render_contact_sheet(name, arm_obj, clip_name, clip_data)

    # Export to GLB with all actions
    os.makedirs(CHARACTERS_DIR, exist_ok=True)
    out_glb = os.path.join(CHARACTERS_DIR, f"{name}.glb")

    bpy.ops.object.select_all(action="DESELECT")
    arm_obj.select_set(True)
    mesh_obj.select_set(True)

    bpy.ops.export_scene.gltf(
        filepath=out_glb,
        export_format="GLB",
        use_selection=True,
        export_yup=True,
        export_animation_mode="ACTIONS",
    )
    print(f"CHARACTER {name}: {tris} tris (budget {BUDGET}) {'OK' if ok else 'OVER BUDGET'} -> {out_glb}")
    return tris, ok


def _only_names():
    """`blender -b --python build_characters.py -- --only=npc_survivor,npc_villager` builds just those."""
    argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    for arg in argv:
        if arg.startswith("--only="):
            return set(arg.split("=", 1)[1].split(","))
    return None


def main():
    os.makedirs(CHARACTERS_DIR, exist_ok=True)
    os.makedirs(PREVIEWS_DIR, exist_ok=True)

    only = _only_names()
    results = []
    for variant in [v for v in VARIANTS if only is None or v["name"] in only]:
        tris, ok = build_one_character(variant)
        results.append((variant["name"], tris, ok))

    print("\n--- CHARACTER BUILD SUMMARY ---")
    all_ok = True
    for name, tris, ok in results:
        status = "OK" if ok else "FAILED"
        print(f"{name}: {tris} tris / {BUDGET} budget [{status}]")
        if not ok:
            all_ok = False
    return 0 if all_ok else 1


if __name__ == "__main__":
    sys.exit(main())
