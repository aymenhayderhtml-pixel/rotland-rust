"""Dog armature (V11-08). Bone names and layout follow docs/ANIMAL_CONTRACT.md.

Facing: +Y in Blender (-Z in Godot). Right is +X, so the left-side bones use -X.
Bone roll: leg bones pointing down use roll (0,1,0) so their local X is the dog's sideways
axis; spine, head and tail bones use roll (0,0,1). A positive X rotation then swings a leg
forward, lifts the nose, and so on. Poses in dog_poses.py rely on this.
"""
import bpy
from mathutils import Vector

# name, parent, head, tail, roll vector (world direction for the bone's local Z)
BONES = [
    ("root", None, (0.0, 0.0, 0.0), (0.0, 0.0, 0.10), (0.0, 1.0, 0.0)),
    ("hips", "root", (0.0, -0.22, 0.50), (0.0, -0.10, 0.50), (0.0, 0.0, 1.0)),
    ("spine", "hips", (0.0, -0.10, 0.50), (0.0, 0.06, 0.52), (0.0, 0.0, 1.0)),
    ("chest", "spine", (0.0, 0.06, 0.52), (0.0, 0.24, 0.52), (0.0, 0.0, 1.0)),
    ("neck", "chest", (0.0, 0.24, 0.52), (0.0, 0.30, 0.60), (0.0, 0.0, 1.0)),
    ("head", "neck", (0.0, 0.30, 0.60), (0.0, 0.45, 0.64), (0.0, 0.0, 1.0)),
    ("jaw", "head", (0.0, 0.34, 0.57), (0.0, 0.46, 0.56), (0.0, 0.0, 1.0)),
    ("ear_L", "head", (-0.05, 0.30, 0.67), (-0.07, 0.27, 0.77), (0.0, 1.0, 0.0)),
    ("ear_R", "head", (0.05, 0.30, 0.67), (0.07, 0.27, 0.77), (0.0, 1.0, 0.0)),
    ("tail_1", "hips", (0.0, -0.24, 0.52), (0.0, -0.34, 0.60), (0.0, 0.0, 1.0)),
    ("tail_2", "tail_1", (0.0, -0.34, 0.60), (0.0, -0.42, 0.66), (0.0, 0.0, 1.0)),
]

# Legs: hip or shoulder joint, then upper (thigh), lower (shin) and paw. Left is -X.
LEG_JOINTS = {
    "FL": ("chest", -0.09, 0.16), "FR": ("chest", 0.09, 0.16),
    "BL": ("hips", -0.09, -0.22), "BR": ("hips", 0.09, -0.22),
}
LEG_KNEE_Z = 0.27
LEG_ANKLE_Z = 0.06
LEG_NAMES = ["FL", "FR", "BL", "BR"]


def _add_leg_bones(eb):
    for leg, (parent, x, y) in LEG_JOINTS.items():
        upper = eb.new(f"upper_{leg}")
        upper.parent = eb[parent]
        upper.head = (x, y, 0.50)
        upper.tail = (x, y, LEG_KNEE_Z)
        upper.align_roll(Vector((0.0, 1.0, 0.0)))
        lower = eb.new(f"lower_{leg}")
        lower.parent = upper
        lower.head = (x, y, LEG_KNEE_Z)
        lower.tail = (x, y + 0.01, LEG_ANKLE_Z)
        lower.align_roll(Vector((0.0, 1.0, 0.0)))
        paw = eb.new(f"paw_{leg}")
        paw.parent = lower
        paw.head = (x, y + 0.01, LEG_ANKLE_Z)
        paw.tail = (x, y + 0.10, 0.0)
        paw.align_roll(Vector((0.0, 1.0, 0.0)))


def create_armature(name):
    """Makes the armature in rest pose (standing, legs straight down). Returns the object."""
    arm_data = bpy.data.armatures.new(name + "_armature")
    arm_obj = bpy.data.objects.new(name + "_armature", arm_data)
    bpy.context.scene.collection.objects.link(arm_obj)
    bpy.context.view_layer.objects.active = arm_obj
    bpy.ops.object.mode_set(mode="EDIT")
    eb = arm_data.edit_bones
    for bone_name, parent, head, tail, roll in BONES:
        bone = eb.new(bone_name)
        if parent:
            bone.parent = eb[parent]
        bone.head = head
        bone.tail = tail
        bone.align_roll(Vector(roll))
    _add_leg_bones(eb)
    bpy.ops.object.mode_set(mode="OBJECT")
    return arm_obj


def bone_names():
    """Every bone name, in creation order (the contract lists them)."""
    names = [b[0] for b in BONES]
    for leg in LEG_NAMES:
        names += [f"upper_{leg}", f"lower_{leg}", f"paw_{leg}"]
    return names
