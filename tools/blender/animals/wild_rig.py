"""Armature for the wild animals (V12-01). Same bone names as the dog (docs/ANIMAL_CONTRACT.md).

Bone positions come from wild_species.py. Roll follows the dog: legs and ears use roll (0,1,0),
the body bones use (0,0,1), so a positive X rotation swings a leg forward or lifts the head.
"""
import bpy
from mathutils import Vector


def _roll(bone_name):
    if bone_name.startswith(("upper_", "lower_", "paw_", "ear_")) or bone_name == "root":
        return (0.0, 1.0, 0.0)
    return (0.0, 0.0, 1.0)


def _add_legs(eb, spec):
    knee, ankle, paw_len = spec["knee_z"], spec["ankle_z"], spec["paw_len"]
    for leg, (parent, x, y, top) in spec["legs"].items():
        upper = eb.new(f"upper_{leg}")
        upper.parent = eb[parent]
        upper.head = (x, y, top)
        upper.tail = (x, y, knee)
        upper.align_roll(Vector((0.0, 1.0, 0.0)))
        lower = eb.new(f"lower_{leg}")
        lower.parent = upper
        lower.head = (x, y, knee)
        lower.tail = (x, y + 0.01, ankle)
        lower.align_roll(Vector((0.0, 1.0, 0.0)))
        paw = eb.new(f"paw_{leg}")
        paw.parent = lower
        paw.head = (x, y + 0.01, ankle)
        paw.tail = (x, y + 0.01 + paw_len, 0.0)
        paw.align_roll(Vector((0.0, 1.0, 0.0)))


def create_armature(name, spec):
    """Armature in rest pose (standing, legs straight down). Returns the object."""
    arm_data = bpy.data.armatures.new(name + "_armature")
    arm_obj = bpy.data.objects.new(name + "_armature", arm_data)
    bpy.context.scene.collection.objects.link(arm_obj)
    bpy.context.view_layer.objects.active = arm_obj
    bpy.ops.object.mode_set(mode="EDIT")
    eb = arm_data.edit_bones
    for bone_name, (parent, head, tail) in spec["bones"].items():  # parents come first in the dict
        bone = eb.new(bone_name)
        if parent:
            bone.parent = eb[parent]
        bone.head = head
        bone.tail = tail
        bone.align_roll(Vector(_roll(bone_name)))
    _add_legs(eb, spec)
    bpy.ops.object.mode_set(mode="OBJECT")
    return arm_obj
