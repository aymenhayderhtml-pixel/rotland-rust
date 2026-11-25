"""Humanoid character builder for Rotland.

Builds a low-poly character mesh skinned to the standard 22-bone humanoid armature
defined in docs/RIG_CONTRACT.md.

Facing: +Y in Blender (-Z in Godot)
Right hand: +X
Height: 1.80 m (standing at origin z=0)
"""
import math
import os
import sys

import bmesh
import bpy
from mathutils import Euler, Matrix, Vector

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
import common  # noqa: E402
import humanoid_body  # noqa: E402
import npc_extras  # noqa: E402
import palette  # noqa: E402

BONE_NAMES = [
    "root",
    "hips",
    "spine",
    "chest",
    "neck",
    "head",
    "shoulder_L",
    "upper_arm_L",
    "forearm_L",
    "hand_L",
    "shoulder_R",
    "upper_arm_R",
    "forearm_R",
    "hand_R",
    "thigh_L",
    "shin_L",
    "foot_L",
    "toe_L",
    "thigh_R",
    "shin_R",
    "foot_R",
    "toe_R",
    "thumb_1_L",
    "thumb_2_L",
    "fingers_1_L",
    "fingers_2_L",
    "thumb_1_R",
    "thumb_2_R",
    "fingers_1_R",
    "fingers_2_R",
]


# A-pose: arms hang 30 degrees below horizontal. Lengths in metres.
ARM_DIR = Vector((0.866, 0.0, -0.5))
ARM_UPPER = 0.27
ARM_FORE = 0.25
ARM_HAND = 0.16


def arm_points(sign, sh_y, sh_z):
    """Shoulder, elbow, wrist and hand-tip joints of one arm (sign +1 = right, +X)."""
    d = Vector((sign * ARM_DIR.x, 0.0, ARM_DIR.z))
    shoulder = Vector((sign * 0.19, sh_y, sh_z))
    elbow = shoulder + d * ARM_UPPER
    wrist = elbow + d * ARM_FORE
    tip = wrist + d * ARM_HAND
    return shoulder, elbow, wrist, tip


class MeshBuilder:
    def __init__(self):
        self.bm = bmesh.new()
        self.pal_layer = self.bm.faces.layers.int.new("pal")
        self.bone_layer = self.bm.verts.layers.int.new("bone")

    def add_box(self, p_min, p_max, color, bone_name, rot=None, center_offset=None):
        """Adds a box aligned with axes or rotated around a center."""
        x0, y0, z0 = p_min
        x1, y1, z1 = p_max
        verts_co = [
            (x0, y0, z0), (x1, y0, z0), (x1, y1, z0), (x0, y1, z0),  # bottom: -Z
            (x0, y0, z1), (x1, y0, z1), (x1, y1, z1), (x0, y1, z1),  # top: +Z
        ]
        if rot is not None:
            c = center_offset if center_offset else Vector(((x0 + x1) / 2, (y0 + y1) / 2, (z0 + z1) / 2))
            rot_mat = Euler(rot, "XYZ").to_matrix()
            verts_co = [c + rot_mat @ (Vector(p) - c) for p in verts_co]

        b_idx = BONE_NAMES.index(bone_name)
        new_verts = [self.bm.verts.new(p) for p in verts_co]
        for v in new_verts:
            v[self.bone_layer] = b_idx

        # 6 quad faces with correct outward winding
        # 0: bottom (-Z), 1: top (+Z), 2: front (+Y), 3: back (-Y), 4: left (-X), 5: right (+X)
        faces_def = [
            (3, 2, 1, 0),  # bottom (-Z)
            (4, 5, 6, 7),  # top (+Z)
            (2, 3, 7, 6),  # front (+Y)
            (0, 1, 5, 4),  # back (-Y)
            (0, 4, 7, 3),  # left (-X)
            (1, 2, 6, 5),  # right (+X)
        ]
        color_idx = palette.key_of(color)
        for face_idx in faces_def:
            face = self.bm.faces.new([new_verts[i] for i in face_idx])
            face[self.pal_layer] = color_idx
        return new_verts

    def add_tapered_box(self, bottom_corners, top_corners, color, bone_name):
        """Adds a 6-sided prism connecting 4 bottom corners and 4 top corners."""
        b_idx = BONE_NAMES.index(bone_name)
        color_idx = palette.key_of(color)

        v_bottom = [self.bm.verts.new(p) for p in bottom_corners]
        v_top = [self.bm.verts.new(p) for p in top_corners]
        for v in v_bottom + v_top:
            v[self.bone_layer] = b_idx

        # Bottom (-Z)
        self.bm.faces.new([v_bottom[3], v_bottom[2], v_bottom[1], v_bottom[0]])[self.pal_layer] = color_idx
        # Top (+Z)
        self.bm.faces.new([v_top[0], v_top[1], v_top[2], v_top[3]])[self.pal_layer] = color_idx
        # Sides
        for i in range(4):
            j = (i + 1) % 4
            self.bm.faces.new([v_bottom[i], v_bottom[j], v_top[j], v_top[i]])[self.pal_layer] = color_idx

    def add_limb(self, p0, p1, r0, r1, color, bone_name):
        """Square tube from joint p0 to joint p1 (radius r0 at p0, r1 at p1)."""
        p0 = Vector(p0)
        p1 = Vector(p1)
        axis = (p1 - p0).normalized()
        side = axis.cross(Vector((0, 1, 0))).normalized()
        up = axis.cross(side)  # side x up == axis, so winding matches add_tapered_box

        def ring(c, r):
            return [c + (side * sx + up * sy) * r for sx, sy in ((-1, -1), (1, -1), (1, 1), (-1, 1))]

        self.add_tapered_box(ring(p0, r0), ring(p1, r1), color, bone_name)

    def add_loft(self, sections, color, bone_name, segs=8, axis=None):
        """Closed tube through rings. sections: [(center, r_side, r_up)]; color: one name or one per band.

        Rings lie across `axis` (default: first to last center). For a vertical tube r_side is the
        X half-width and r_up the Y half-depth; along Y, r_side is X and r_up is Z.
        """
        centers = [Vector(c) for c, _, _ in sections]
        a = Vector(axis) if axis is not None else centers[-1] - centers[0]
        a.normalize()
        travel = centers[-1] - centers[0]
        if travel.length_squared > 1e-10 and travel.dot(a) < 0.0:
            a = -a
        ref = Vector((0, 0, 1)) if abs(a.z) < 0.9 else Vector((0, 1, 0))
        side = ref.cross(a).normalized()
        up = a.cross(side).normalized()
        bone_indices = (
            [BONE_NAMES.index(bone_name)] * len(sections)
            if isinstance(bone_name, str)
            else [BONE_NAMES.index(b) for b in bone_name]
        )
        colors = [color] * (len(sections) - 1) if isinstance(color, str) else list(color)
        phase = math.pi / segs
        rings = []
        for c, (_, rs, ru), b_idx in zip(centers, sections, bone_indices):
            ring = []
            for i in range(segs):
                t = phase + 2 * math.pi * i / segs
                v = self.bm.verts.new(c + side * (math.cos(t) * rs) + up * (math.sin(t) * ru))
                v[self.bone_layer] = b_idx
                ring.append(v)
            rings.append(ring)
        for k in range(len(rings) - 1):
            ci = palette.key_of(colors[k])
            r0, r1 = rings[k], rings[k + 1]
            for i in range(segs):
                j = (i + 1) % segs
                self.bm.faces.new([r0[i], r0[j], r1[j], r1[i]])[self.pal_layer] = ci
        # Winding is fixed for the whole mesh in finalize()
        self.bm.faces.new(list(reversed(rings[0])))[self.pal_layer] = palette.key_of(colors[0])
        self.bm.faces.new(rings[-1])[self.pal_layer] = palette.key_of(colors[-1])

    def add_ellipsoid(self, center, radii, color, bone_name, segs=8, rings=5):
        """Low-poly ellipsoid (a loft of `rings` rings along Z, with small flat caps)."""
        c = Vector(center)
        sections = []
        for k in range(rings):
            th = math.pi * (k + 0.5) / rings
            sections.append(((c.x, c.y, c.z - radii[2] * math.cos(th)),
                             radii[0] * math.sin(th), radii[1] * math.sin(th)))
        self.add_loft(sections, color, bone_name, segs=segs, axis=(0, 0, 1))

    def finalize(self, name, scale=(1.0, 1.0, 1.0)):
        mesh = bpy.data.meshes.new(name)
        bmesh.ops.scale(self.bm, vec=Vector(scale), verts=list(self.bm.verts))
        # Each part is built with outward-facing faces; avoid global recalc_face_normals
        # which inverts faces on overlapping shells (pelvis/torso, knees/shins, laces/shoes - REND-002)
        bmesh.ops.triangulate(self.bm, faces=list(self.bm.faces))
        self.bm.normal_update()

        # Capture palette keys before conversion/free, then use each key's own sheet.
        keys = [face[self.pal_layer] for face in self.bm.faces]
        uv_layer = self.bm.loops.layers.uv.new("UVMap")
        for face in self.bm.faces:
            color_name = palette.name_of_key(face[self.pal_layer])
            uv = palette.uv_center(color_name)
            for loop in face.loops:
                loop[uv_layer].uv = uv

        self.bm.verts.index_update()
        self.vert_bones = [v[self.bone_layer] for v in self.bm.verts]
        self.bm.to_mesh(mesh)
        for poly in mesh.polygons:
            poly.use_smooth = False
        sheets = ["A"] + [s for s in ("B", "C") if any(palette.sheet_of_key(k) == s for k in keys)]
        for sheet in sheets:
            mesh.materials.append(common.palette_material(sheet))
        for poly, key in zip(mesh.polygons, keys):
            poly.material_index = sheets.index(palette.sheet_of_key(key))
        obj = bpy.data.objects.new(name, mesh)
        bpy.context.scene.collection.objects.link(obj)
        self.bm.free()
        return obj


def _scaled(p, s):
    return Vector((p[0] * s[0], p[1] * s[1], p[2] * s[2]))


def create_armature(name, hunched=False, scale=(1.0, 1.0, 1.0), include_fingers=True):
    """Creates the humanoid armature matching docs/RIG_CONTRACT.md in A-pose.

    `scale` (x width, y depth, z height) is applied to every bone so the rig matches the mesh.
    """
    arm_data = bpy.data.armatures.new(name + "_armature")
    arm_obj = bpy.data.objects.new(name + "_armature", arm_data)
    bpy.context.scene.collection.objects.link(arm_obj)
    bpy.context.view_layer.objects.active = arm_obj
    bpy.ops.object.mode_set(mode="EDIT")

    eb = arm_data.edit_bones

    # 1. Root
    root = eb.new("root")
    root.head = (0.0, 0.0, 0.0)
    root.tail = (0.0, 0.0, 0.20)

    # 2. Hips
    hips = eb.new("hips")
    hips_z = 0.90 if not hunched else 0.88
    hips.head = (0.0, 0.0, hips_z)
    hips.tail = (0.0, 0.0 if not hunched else 0.03, hips_z + 0.14)
    hips.parent = root

    # 3. Spine
    spine = eb.new("spine")
    spine.head = hips.tail
    spine.tail = (0.0, 0.0 if not hunched else 0.07, hips_z + 0.32)
    spine.parent = hips

    # 4. Chest
    chest = eb.new("chest")
    chest.head = spine.tail
    chest.tail = (0.0, 0.0 if not hunched else 0.12, hips_z + 0.56)
    chest.parent = spine

    # 5. Neck
    neck = eb.new("neck")
    neck.head = chest.tail
    neck.tail = (0.0, 0.0 if not hunched else 0.15, hips_z + 0.64)
    neck.parent = chest

    # 6. Head
    head = eb.new("head")
    head.head = neck.tail
    head.tail = (0.0, 0.0 if not hunched else 0.18, 1.80 if not hunched else 1.77)
    head.parent = neck

    # Shoulders and arms: joints come from arm_points(), the same points the mesh uses
    sh_y = 0.0 if not hunched else 0.08
    sh_z = hips_z + 0.52
    for sign, side in ((1, "R"), (-1, "L")):
        shoulder, elbow, wrist, tip = arm_points(sign, sh_y, sh_z)
        sh = eb.new("shoulder_" + side)
        sh.head = (0.08 * sign, sh_y, sh_z)
        sh.tail = shoulder
        sh.parent = chest
        ua = eb.new("upper_arm_" + side)
        ua.head = shoulder
        ua.tail = elbow
        ua.parent = sh
        fa = eb.new("forearm_" + side)
        fa.head = elbow
        fa.tail = wrist
        fa.parent = ua
        hd = eb.new("hand_" + side)
        hd.head = wrist
        hd.tail = tip
        hd.parent = fa
        for bone in (ua, fa, hd):
            bone.align_roll(Vector((0, 1, 0)))

        if include_fingers:
            a = (tip - wrist).normalized()
            back = Vector((sign * 0.5, 0.0, 0.866))
            across = Vector((0.0, 1.0, 0.0))

            f_base = wrist + a * 0.072
            f_mid = f_base + a * 0.036 - back * 0.004
            f_tip = f_base + a * 0.070 - back * 0.014
            f1 = eb.new("fingers_1_" + side)
            f1.head = f_base
            f1.tail = f_mid
            f1.parent = hd
            f2 = eb.new("fingers_2_" + side)
            f2.head = f_mid
            f2.tail = f_tip
            f2.parent = f1

            t0 = wrist + a * 0.022 + across * 0.028 - back * 0.006
            t1 = wrist + a * 0.064 + across * 0.046 - back * 0.012
            t_mid = (t0 + t1) * 0.5
            th1 = eb.new("thumb_1_" + side)
            th1.head = t0
            th1.tail = t_mid
            th1.parent = hd
            th2 = eb.new("thumb_2_" + side)
            th2.head = t_mid
            th2.tail = t1
            th2.parent = th1

            for bone in (f1, f2, th1, th2):
                bone.align_roll(Vector((0, 1, 0)))

    # Legs (Right & Left)
    thigh_z = hips_z - 0.02
    knee_z = 0.50
    ankle_z = 0.08
    ball_y = 0.10
    toe_y = 0.18

    # Right Leg
    t_r = eb.new("thigh_R")
    t_r.head = (0.10, 0.0, thigh_z)
    t_r.tail = (0.10, 0.0, knee_z)
    t_r.parent = hips
    t_r.align_roll(Vector((0, 1, 0)))

    s_r = eb.new("shin_R")
    s_r.head = t_r.tail
    s_r.tail = (0.10, 0.0, ankle_z)
    s_r.parent = t_r
    s_r.align_roll(Vector((0, 1, 0)))

    f_r = eb.new("foot_R")
    f_r.head = s_r.tail
    f_r.tail = (0.10, ball_y, 0.03)
    f_r.parent = s_r
    f_r.align_roll(Vector((0, 0, 1)))

    toe_r = eb.new("toe_R")
    toe_r.head = f_r.tail
    toe_r.tail = (0.10, toe_y, 0.02)
    toe_r.parent = f_r
    toe_r.align_roll(Vector((0, 0, 1)))

    # Left Leg
    t_l = eb.new("thigh_L")
    t_l.head = (-0.10, 0.0, thigh_z)
    t_l.tail = (-0.10, 0.0, knee_z)
    t_l.parent = hips
    t_l.align_roll(Vector((0, 1, 0)))

    s_l = eb.new("shin_L")
    s_l.head = t_l.tail
    s_l.tail = (-0.10, 0.0, ankle_z)
    s_l.parent = t_l
    s_l.align_roll(Vector((0, 1, 0)))

    f_l = eb.new("foot_L")
    f_l.head = s_l.tail
    f_l.tail = (-0.10, ball_y, 0.03)
    f_l.parent = s_l
    f_l.align_roll(Vector((0, 0, 1)))

    toe_l = eb.new("toe_L")
    toe_l.head = f_l.tail
    toe_l.tail = (-0.10, toe_y, 0.02)
    toe_l.parent = f_l
    toe_l.align_roll(Vector((0, 0, 1)))

    for bone in eb:
        bone.head = _scaled(bone.head, scale)
        bone.tail = _scaled(bone.tail, scale)

    bpy.ops.object.mode_set(mode="OBJECT")
    return arm_obj


def build_humanoid(variant):
    """Builds the skinned humanoid mesh and armature.

    variant dict keys:
    - name: str (e.g. 'human_player', 'zombie_01')
    - skin: palette color name
    - hair: palette color name
    - shirt: palette color name
    - pants: palette color name
    - shoes: palette color name
    - blood: optional palette color for tears/patches
    - hunched: bool
    - bare_arm: "R" or "L" to show skin instead of a sleeve on that upper arm (torn sleeve)
    - proportions: dict with optional "width", "depth", "height" multipliers (default 1.0)
    - extras: optional NPC look: "trader", "guard" or "survivor" (see npc_extras.py)
    """
    name = variant.get("name", "humanoid")
    skin = variant.get("skin", "skin_light")
    hair = variant.get("hair", "hair_dark")
    shirt = variant.get("shirt", "shirt_blue")
    pants = variant.get("pants", "pants_denim")
    shoes = variant.get("shoes", "shoe_dark")
    blood = variant.get("blood", None)
    hunched = variant.get("hunched", False)
    bare_arm = variant.get("bare_arm")
    prop = variant.get("proportions", {})
    scale = (prop.get("width", 1.0), prop.get("depth", 1.0), prop.get("height", 1.0))

    include_fingers = not name.startswith("zombie_")
    arm_obj = create_armature(name, hunched=hunched, scale=scale, include_fingers=include_fingers)

    mb = MeshBuilder()
    look = {"skin": skin, "hair": hair, "shirt": shirt, "pants": pants, "shoes": shoes,
            "blood": blood, "bare_arm": bare_arm, "name": name, "zombie": name.startswith("zombie_")}
    for key, default in {
        "hair_style": "short_swept", "jacket": None, "gloves": None,
        "knee_pads": False, "holster": False, "beard": None,
        "chest_rig": False, "long_sleeves": False, "watch": False,
        "cargo_pockets": name in {"human_player", "npc_guard", "npc_survivor"},
    }.items():
        look[key] = variant.get(key, default)
    humanoid_body.build_body(mb, look, hunched, arm_points)

    # NPC extras (apron, helmet, rucksack...) use the same bones, so weights stay rigid
    npc_extras.add_extras(mb, variant.get("extras"), arm_points=arm_points)

    # Finalize mesh
    mesh_obj = mb.finalize(name, scale)
    mesh_obj.parent = arm_obj

    # Create vertex groups and assign rigid weights (100% on exactly one bone)
    _assign_rigid_weights(mesh_obj, mb.vert_bones)

    # Add Armature modifier
    mod = mesh_obj.modifiers.new(name="Armature", type="ARMATURE")
    mod.object = arm_obj

    tris = len(mesh_obj.data.polygons)
    print(f"Humanoid '{name}': {tris} triangles (budget 5000)")
    return arm_obj, mesh_obj


def _assign_rigid_weights(mesh_obj, bone_of_vert):
    """Each vertex is 100% on the bone of the part it was built from (exact, not guessed)."""
    groups = {}
    for v_idx, b_idx in enumerate(bone_of_vert):
        bone = BONE_NAMES[b_idx]
        if bone not in groups:
            groups[bone] = mesh_obj.vertex_groups.new(name=bone)
        groups[bone].add([v_idx], 1.0, "REPLACE")
