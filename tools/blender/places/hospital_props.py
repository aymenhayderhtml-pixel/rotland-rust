"""Hospital props for the R42 New Halden place: beds, trolleys, curtain, chairs, cage, bags, tent, ambulance.

Categories: `props` (4k each), `body_bag_pile` and `triage_tent` get 6k, `ambulance_wreck` is a
`vehicles` model at 10k with an `_lod1` at 1/10. Signs use generic words only ("AMBULANCE", a red cross).
Run: see `places/hospital.py` (`build_all`).
"""
import math
import os
import random
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
BLENDER = os.path.dirname(HERE)
for _p in (HERE, BLENDER,
           os.path.join(BLENDER, "buildings"),
           os.path.join(BLENDER, "metro"),
           os.path.join(BLENDER, "vehicles"),
           os.path.join(BLENDER, "overgrowth")):
    if _p not in sys.path:
        sys.path.insert(0, _p)

import bmesh  # noqa: E402
from mathutils import Matrix, Vector  # noqa: E402

import car_parts  # noqa: E402
import common  # noqa: E402
import place_kit  # noqa: E402
from jam_util import box, pbox, wheel_parts  # noqa: E402

PROP_BUDGET = 4000
PILE_BUDGET = 6000
TENT_BUDGET = 6000
VEHICLE_BUDGET = 10000

FRONT_VIEW = (1.0, 1.0, 0.65)
STEEL = "metal"
SOFT = "white"


def _beam(p0, p1, r, color=STEEL, segs=6):
    return car_parts._beam(p0, p1, r, color=color, segments=segs)


def _wheel(model, x, y, z, r, half_w, color="gun_black", tyre=True):
    """Castor or wheel lying on its axle along X at (x, y, z)."""
    parts = wheel_parts(x, y, r, half_w, tyre=tyre, rim_color="metal")
    for bm in parts:
        if z:
            bmesh.ops.translate(bm, verts=list(bm.verts), vec=(0.0, 0.0, z))
        model.add(bm)


# ---------------------------------------------------------------- gurney


def _gurney(model):
    """Stretcher trolley, 1.9 x 0.72 m: tube frame on four castors, padded mattress with a raised
    backrest wedge, two side rails (one dropped) and a lower shelf tray."""
    hw, hl = 0.32, 0.9
    frame_z = 0.56
    # Frame rails, top and a lower stretcher
    for sx in (-hw, hw):
        model.add(_beam((sx, -hl, frame_z), (sx, hl, frame_z), 0.026))
        model.add(_beam((sx, -hl, 0.34), (sx, hl, 0.34), 0.020))
    for sy in (-hl, hl):
        model.add(_beam((-hw, sy, frame_z), (hw, sy, frame_z), 0.026))
        model.add(_beam((-hw, sy, 0.34), (hw, sy, 0.34), 0.020))
    # Legs and castors
    for sx in (-hw, hw):
        for sy in (-0.74, 0.74):
            model.add(_beam((sx, sy, 0.34), (sx, sy, 0.15), 0.020))
            _wheel(model, sx, sy, 0.0, 0.075, 0.035)
    # Lower shelf tray
    model.add(pbox((-hw + 0.03, -hl + 0.04, 0.34), (hw - 0.03, hl - 0.04, 0.36), "rock_light", bevel=0.004))
    # Mattress pad on the frame, with a stain and a pillow
    model.add(pbox((-hw, -hl, frame_z), (hw, hl, frame_z + 0.09), SOFT, bevel=0.025))
    model.add(pbox((-0.16, 0.1, frame_z + 0.085), (0.1, 0.5, frame_z + 0.098), "blood_dark", bevel=0.004))
    model.add(pbox((-hw + 0.04, -hl + 0.05, frame_z + 0.09), (hw - 0.04, -hl + 0.36, frame_z + 0.16),
                   "bone_white", bevel=0.02))
    # Raised backrest wedge at the head end (-Y)
    wedge = common.make_box((hw * 2.0, 0.42, 0.07), color=SOFT)
    common.transform(wedge, rot=(math.radians(-26.0), 0.0, 0.0), loc=(0.0, -hl + 0.30, frame_z + 0.22))
    model.add(wedge)
    # Side rails: two tubes per side on three short uprights; the right one is dropped flat
    for sx in (-hw - 0.03, hw + 0.03):
        z_rail = 0.80 if sx > 0 else 1.02
        model.add(_beam((sx, -0.5, z_rail), (sx, 0.6, z_rail), 0.018))
        model.add(_beam((sx, -0.5, z_rail - 0.26), (sx, 0.6, z_rail - 0.26), 0.013))
        for sy in (-0.45, 0.05, 0.55):
            model.add(_beam((sx, sy, frame_z + 0.06), (sx, sy, z_rail), 0.014))
    # A chart hanging off the head end
    model.add(pbox((-0.11, -hl - 0.05, 0.70), (0.11, -hl - 0.01, 0.92), "white", bevel=0.004))
    model.add_collider("gurney", box((-hw - 0.06, -hl - 0.06, 0.0), (hw + 0.06, hl + 0.06, 1.1), "white"))


# ---------------------------------------------------------------- hospital bed


def _hospital_bed(model):
    """2.05 x 0.98 m ward bed: frame, castors, mattress, blanket, metal head and foot boards, one raised rail."""
    hw, hl = 0.49, 1.02
    for sx in (-1.0, 1.0):
        model.add(_beam((sx * hw, -hl, 0.42), (sx * hw, hl, 0.42), 0.03))
    for sy in (-hl, hl):
        model.add(_beam((-hw, sy, 0.42), (hw, sy, 0.42), 0.03))
    for sx in (-hw, hw):
        for sy in (-0.82, 0.82):
            model.add(_beam((sx, sy, 0.42), (sx, sy, 0.20), 0.024))
            _wheel(model, sx, sy, 0.0, 0.07, 0.032)
    # Mattress platform, mattress, pillow, blanket
    model.add(pbox((-hw - 0.02, -hl, 0.42), (hw + 0.02, hl, 0.48), "rock_light", bevel=0.008))
    model.add(pbox((-hw, -0.98, 0.48), (hw, 0.98, 0.66), SOFT, bevel=0.04))
    model.add(pbox((-0.32, 0.52, 0.66), (0.32, 0.9, 0.76), "bone_white", bevel=0.03))
    model.add(pbox((-hw + 0.01, -0.98, 0.655), (hw - 0.01, 0.30, 0.70), "gown_blue", bevel=0.02))
    model.add(pbox((-0.2, -0.6, 0.70), (0.12, -0.3, 0.712), "blood_dark", bevel=0.004))
    # Head board (+Y) and foot board (-Y): tube frames with vertical bars
    for sy, tall in ((hl - 0.02, 1.18), (-hl + 0.02, 1.02)):
        for sx in (-hw, hw):
            model.add(_beam((sx, sy, 0.40), (sx, sy, tall), 0.024))
        model.add(_beam((-hw, sy, tall), (hw, sy, tall), 0.024))
        model.add(_beam((-hw, sy, (tall + 0.4) / 2.0), (hw, sy, (tall + 0.4) / 2.0), 0.018))
        for bx in (-0.3, -0.1, 0.1, 0.3):
            model.add(_beam((bx, sy, 0.40), (bx, sy, tall), 0.013))
    # One raised side rail (right), the left one dropped against the frame
    model.add(_beam((hw + 0.04, -0.6, 0.98), (hw + 0.04, 0.6, 0.98), 0.02))
    for sy in (-0.55, 0.0, 0.55):
        model.add(_beam((hw + 0.04, sy, 0.66), (hw + 0.04, sy, 0.98), 0.016))
    model.add(_beam((-hw - 0.05, -0.6, 0.44), (-hw - 0.05, 0.6, 0.44), 0.018))
    # A chart clip hanging off the foot board
    model.add(pbox((-0.12, -hl - 0.05, 0.72), (0.12, -hl - 0.01, 0.94), "white", bevel=0.004))
    model.add_collider("bed", box((-hw - 0.07, -hl - 0.06, 0.0), (hw + 0.07, hl + 0.06, 1.2), "white"))


# ---------------------------------------------------------------- ward curtain


def _ward_curtain(model):
    """2.1 m ceiling rail with 14 fabric folds hanging to 2.0 m, torn hem, a gap and one bare end."""
    rail_z = 2.06
    half = 1.05
    model.add(_beam((-half, 0.0, rail_z), (half, 0.0, rail_z), 0.026, color=STEEL, segs=8))
    for sx in (-half, half):
        model.add(pbox((sx - 0.07, -0.05, rail_z + 0.02), (sx + 0.07, 0.05, rail_z + 0.09), STEEL, bevel=0.006))
        model.add(_beam((sx, 0.0, rail_z + 0.08), (sx, 0.0, rail_z + 0.30), 0.014, color=STEEL))  # ceiling rod
        model.add(pbox((sx - 0.09, -0.09, rail_z + 0.29), (sx + 0.09, 0.09, rail_z + 0.32), "rock_dark", bevel=0.004))
    # Fabric: folds are thin tilted panels hanging from the rail, each with its own bottom edge
    rng = random.Random(4213)
    folds = 13
    x = -half + 0.06
    step = (half * 2.0 - 0.42) / folds
    for i in range(folds):
        # A tear: fold 8 hangs short and fold 9 is missing, leaving the rail bare
        if i == 9:
            # hanger only
            model.add(_beam((x, 0.0, rail_z), (x, 0.0, rail_z - 0.10), 0.012, color=STEEL))
            x += step
            continue
        bottom = 0.42 if i == 8 else (0.05 + rng.uniform(0.0, 0.16))
        if i == 5:
            bottom = 0.62  # a big rip lower down
        w = step * 0.90
        depth = 0.055 + 0.02 * ((i % 3) - 1)
        tilt = 0.05 * (1 if i % 2 else -1)
        bm = common.make_box((w, depth, rail_z - bottom), color="gown_blue")
        common.transform(bm, rot=(0.0, tilt, 0.0),
                         loc=(x + w / 2.0, 0.0, (rail_z + bottom) / 2.0))
        model.add(bm)
        # Ragged lower hem strip, a shade darker and offset from the fold
        hem = common.make_box((w * 0.55, depth * 0.7, 0.16), color="shirt_blue")
        common.transform(hem, rot=(0.0, tilt * 1.6, 0.35), loc=(x + w * 0.42, 0.01, bottom + 0.05))
        model.add(hem)
        # Hanger ring
        model.add(_beam((x + w / 2.0, 0.0, rail_z + 0.02), (x + w / 2.0, 0.0, rail_z - 0.09), 0.011,
                        color="gun_black"))
        x += step
    # One torn-off strip hanging by a thread near the middle
    model.add(_beam((0.08, 0.0, rail_z - 0.05), (0.10, 0.02, 1.15), 0.045, color="gown_blue", segs=4))
    model.add(pbox((0.0, -0.03, 0.95), (0.18, 0.03, 1.16), "gown_blue", bevel=0.01))


# ---------------------------------------------------------------- wheelchair


def _wheelchair(model):
    """Folding chair: 2 spoked wheels (r 0.31), 2 front castors, seat, back, armrests, footplates."""
    model.add(pbox((-0.23, -0.22, 0.42), (0.23, 0.22, 0.47), "pants_denim", bevel=0.02))   # seat
    model.add(pbox((-0.22, -0.26, 0.47), (0.22, -0.21, 0.92), "pants_denim", bevel=0.02))  # back
    model.add(pbox((-0.2, -0.27, 0.72), (0.2, -0.22, 0.86), "shirt_blue", bevel=0.006))    # back strap
    # Big wheels on either side of the seat
    for sx in (-0.3, 0.3):
        model.add(_spoked_wheel(sx, 0.0, 0.31, 0.31))
    # Frame tubes
    for sx in (-0.28, 0.28):
        model.add(_beam((sx, 0.0, 0.31), (sx, 0.0, 0.44), 0.02))
        model.add(_beam((sx, 0.16, 0.16), (sx, -0.16, 0.44), 0.018))
        model.add(_beam((sx, 0.0, 0.44), (sx, 0.0, 0.86), 0.018))       # back upright
        model.add(_beam((sx, 0.16, 0.30), (sx, 0.30, 0.62), 0.015))     # front upright
        model.add(_beam((sx, 0.62, 0.62), (sx, 0.30, 0.62), 0.016))     # armrest
        model.add(_beam((sx, 0.30, 0.20), (sx, 0.52, 0.16), 0.015))     # footrest arm
        model.add(pbox((sx - 0.07, 0.42, 0.14), (sx + 0.07, 0.62, 0.17), STEEL, bevel=0.008))
        # Front castor
        model.add(_beam((sx, 0.30, 0.30), (sx, 0.34, 0.10), 0.014))
        model.add(_beam((sx - 0.04, 0.34, 0.09), (sx + 0.04, 0.34, 0.09), 0.02))
    model.add(_beam((-0.28, 0.02, 0.36), (0.28, 0.02, 0.36), 0.016))
    model.add_collider("chair", box((-0.36, -0.3, 0.0), (0.36, 0.66, 0.95), "white"))


def _spoked_wheel(x, y, z, r):
    """Wheel with a rim, 6 spokes and a hub, axle along X (built for the wheelchair)."""
    parts = [common.make_frustum(r * 0.97, r * 0.97, -0.03, 0.03, segments=18, color="gun_black"),
             common.make_frustum(r * 0.90, r * 0.90, -0.02, 0.02, segments=18, color="metal")]
    for k in range(6):
        a = math.pi * k / 6.0
        spoke = common.make_box((0.012, r * 1.8, 0.012), color="chrome")
        common.transform(spoke, rot=(a, 0.0, 0.0))
        parts.append(spoke)
    parts.append(common.make_frustum(0.04, 0.04, -0.05, 0.05, segments=8, color="chrome"))
    bm = common.combine(parts)
    common.transform(bm, rot=(0.0, math.pi / 2.0, 0.0), loc=(x, y, z))
    return bm


# ---------------------------------------------------------------- IV stand


def _iv_stand(model):
    """Stainless drip stand: 5-leg base with castors, telescopic pole to 1.95 m, two bags and a tube."""
    for k in range(5):
        a = 2.0 * math.pi * k / 5.0
        tip = (math.cos(a) * 0.30, math.sin(a) * 0.30, 0.06)
        model.add(_beam((0.0, 0.0, 0.10), tip, 0.016))
        model.add(common.make_frustum(0.03, 0.03, 0.06, 0.10, segments=6, center=tip[:2], color="gun_black"))
    model.add(common.make_frustum(0.035, 0.028, 0.10, 1.60, segments=8, center=(0.0, 0.0), color="chrome"))
    model.add(common.make_frustum(0.024, 0.020, 1.60, 1.96, segments=8, center=(0.0, 0.0), color="chrome"))
    # Hooks
    for sx in (-1.0, 1.0):
        model.add(_beam((0.0, 0.0, 1.92), (sx * 0.13, 0.0, 1.86), 0.010, color=STEEL))
        model.add(_beam((sx * 0.13, 0.0, 1.87), (sx * 0.13, 0.0, 1.80), 0.008, color=STEEL))
    # Two fluid bags hanging from the hooks (one nearly empty and sagging)
    model.add(pbox((-0.04, -0.06, 1.48), (0.10, 0.06, 1.82), "water_blue", bevel=0.02))
    model.add(pbox((-0.03, -0.05, 1.44), (0.09, 0.05, 1.50), "white", bevel=0.006))
    model.add(pbox((-0.18, -0.05, 1.62), (0.07, 0.05, 1.80), "white", bevel=0.02))
    # Drip chamber and tube running down
    model.add(common.make_frustum(0.022, 0.022, 1.32, 1.44, segments=6, center=(0.03, 0.0), color="water_blue"))
    model.add(_beam((0.03, 0.0, 1.32), (0.05, 0.01, 0.9), 0.006, color="water_blue", segs=4))
    model.add(_beam((0.05, 0.01, 0.9), (0.02, 0.02, 0.55), 0.006, color="water_blue", segs=4))
    # Small clamp and a roll of tape on the pole
    model.add(pbox((0.03, -0.03, 1.10), (0.07, 0.03, 1.16), "gun_black", bevel=0.004))
    model.add(common.make_frustum(0.055, 0.055, 1.20, 1.27, segments=8, center=(0.0, 0.0), color="white"))
    model.add_collider("stand", box((-0.16, -0.16, 0.0), (0.16, 0.16, 1.9), "white"))


# ---------------------------------------------------------------- reception desk


def _reception_desk(model):
    """3.3 x 0.95 x 1.12 m counter: laminate top with a raised ledge, modesty panels, monitor, papers, phone."""
    model.add(pbox((-1.65, -0.44, 0.0), (1.65, 0.44, 1.02), "bark_light", bevel=0.012))
    model.add(pbox((-1.70, -0.50, 1.02), (1.70, 0.50, 1.12), "rock_light", bevel=0.014))
    model.add(pbox((-1.68, 0.44, 1.12), (1.68, 0.50, 1.24), "limestone", bevel=0.01))    # raised ledge
    model.add(pbox((-1.60, -0.40, 0.0), (1.60, 0.40, 0.06), "soot", bevel=0.006))        # kick recess
    # Front pilasters and a recessed panel with the hospital band colour
    for x in (-1.55, -0.42, 0.42, 1.55):
        model.add(pbox((x - 0.06, 0.42, 0.06), (x + 0.06, 0.47, 1.02), "bark_dark", bevel=0.006))
    model.add(pbox((-1.45, 0.455, 0.12), (1.45, 0.475, 0.96), "gown_blue", bevel=0.006))
    model.add(pbox((-1.0, 0.47, 0.55), (1.0, 0.485, 0.62), "white", bevel=0.003))        # painted band
    # Dead monitor, keyboard, phone, papers, a toppled pen pot
    model.add(pbox((-0.75, -0.16, 1.12), (-0.43, 0.04, 1.15), "gun_black", bevel=0.004))
    model.add(pbox((-0.63, -0.06, 1.15), (-0.55, 0.0, 1.24), "gun_black", bevel=0.004))
    model.add(pbox((-0.84, -0.13, 1.24), (-0.34, 0.05, 1.56), "gun_black", bevel=0.01))
    model.add(pbox((-0.81, -0.14, 1.27), (-0.37, -0.13, 1.53), "soot", bevel=0.003))
    model.add(pbox((-0.78, -0.34, 1.12), (-0.40, -0.18, 1.145), "gun_black", bevel=0.004))
    model.add(pbox((0.35, -0.2, 1.12), (0.62, 0.0, 1.18), "white", bevel=0.006))
    model.add(pbox((0.37, -0.17, 1.18), (0.45, 0.01, 1.24), "white", bevel=0.006))
    for loc, rot, col in (((0.02, -0.16, 1.125), 0.2, "white"), ((0.06, -0.09, 1.132), -0.35, "bone_white"),
                          ((0.86, -0.04, 1.125), 0.5, "white"), ((1.2, 0.1, 1.125), -0.2, "bone_white")):
        sheet = common.make_box((0.22, 0.28, 0.006), color=col)
        common.transform(sheet, rot=(0.0, 0.0, rot), loc=loc)
        model.add(sheet)
    model.add(common.make_frustum(0.05, 0.05, 1.12, 1.24, segments=8, center=(1.1, -0.1), color="metal"))
    model.add_collider("desk", box((-1.7, -0.5, 0.0), (1.7, 0.5, 1.12), "white"))


# ---------------------------------------------------------------- pharmacy cage


def _pharmacy_cage(model):
    """3.0 x 2.2 x 2.6 m wire cage: mesh panels on 3 sides plus the roof, a door leaf swung open,
    stocked shelves inside and a solid counter half across the front."""
    hw, hd, top = 1.5, 1.1, 2.6
    post = 0.05
    # Corner + mid posts, plain boxes (a post at 5 cm needs no bevel)
    for sx in (-hw, -0.05, hw):
        for sy in (-hd, hd):
            model.add(pbox((sx - post, sy - post, 0.0), (sx + post, sy + post, top), STEEL, bevel=0.0))
    # Top and bottom frames
    for z in (0.0, top - 0.06):
        for sy in (-hd, hd):
            model.add(pbox((-hw, sy - 0.03, z), (hw, sy + 0.03, z + 0.05), STEEL, bevel=0.0))
        for sx in (-hw, hw):
            model.add(pbox((sx - 0.03, -hd, z), (sx + 0.03, hd, z + 0.05), STEEL, bevel=0.0))

    def mesh_panel(axis, coord, u0, u1, z0, z1, step=0.30):
        """Thin mesh: uprights every `step` plus three rails (flat quads are too thin to see)."""
        u = u0
        while u <= u1 + 1e-6:
            if axis == 1:
                model.add(pbox((u - 0.012, coord - 0.012, z0), (u + 0.012, coord + 0.012, z1),
                               "chrome", bevel=0.0))
            else:
                model.add(pbox((coord - 0.012, u - 0.012, z0), (coord + 0.012, u + 0.012, z1),
                               "chrome", bevel=0.0))
            u += step
        for zr in (z0 + (z1 - z0) * 0.33, z0 + (z1 - z0) * 0.66):
            if axis == 1:
                model.add(pbox((u0, coord - 0.01, zr), (u1, coord + 0.01, zr + 0.02), "chrome", bevel=0.0))
            else:
                model.add(pbox((coord - 0.01, u0, zr), (coord + 0.01, u1, zr + 0.02), "chrome", bevel=0.0))

    mesh_panel(1, -hd, -hw, hw, 0.09, top - 0.06)                 # back
    mesh_panel(0, -hw, -hd, hd, 0.09, top - 0.06)                 # left side
    mesh_panel(0, hw, -hd, hd, 0.09, top - 0.06)                  # right side
    # Front: solid on the left half (the counter), mesh on the right, the doorway left open
    model.add(pbox((-hw, hd - 0.03, 0.09), (-0.2, hd + 0.03, top - 0.06), "white", bevel=0.0))
    mesh_panel(1, hd, -0.2, hw, 1.45, top - 0.06)
    # Door leaf swung 100 degrees open on the right hinge
    leaf = [
        pbox((-0.02, -0.03, 0.0), (0.02, 0.03, 1.34), STEEL, bevel=0.0),
        pbox((0.90, -0.03, 0.0), (0.94, 0.03, 1.34), STEEL, bevel=0.0),
        pbox((-0.02, -0.03, 0.0), (0.94, 0.03, 0.04), STEEL, bevel=0.0),
        pbox((-0.02, -0.03, 1.30), (0.94, 0.03, 1.34), STEEL, bevel=0.0),
        pbox((-0.02, -0.03, 0.62), (0.94, 0.03, 0.65), STEEL, bevel=0.0),
        pbox((0.62, -0.05, 0.58), (0.72, 0.05, 0.72), "brass", bevel=0.0),
    ]
    for i in (1, 2, 3):
        leaf.append(pbox((i * 0.24 - 0.012, -0.012, 0.04), (i * 0.24 + 0.012, 0.012, 1.30),
                         "chrome", bevel=0.0))
    door = common.combine(leaf)
    common.transform(door, rot=(0.0, 0.0, math.radians(-100.0)), loc=(hw - 0.05, hd + 0.05, 0.09))
    model.add(door)
    # Shelves inside with medicine boxes and bottles on them (a few, generously sized)
    for z in (0.62, 1.25, 1.88):
        model.add(pbox((-hw + 0.08, -hd + 0.06, z), (hw - 0.08, -0.15, z + 0.04), "chrome", bevel=0.0))
        x = -hw + 0.2
        rng = random.Random(97 + int(z * 10))
        while x < hw - 0.4:
            w = rng.uniform(0.22, 0.42)
            col = ("white", "bone_white", "wood_crate", "gown_blue")[rng.randrange(4)]
            model.add(pbox((x, -hd + 0.15, z + 0.04), (x + w, -hd + 0.44, z + 0.32), col, bevel=0.0))
            if rng.random() < 0.6:
                model.add(pbox((x + 0.04, -hd + 0.44, z + 0.14), (x + w - 0.04, -hd + 0.46, z + 0.26),
                               "red_paint", bevel=0.0))
            x += w + rng.uniform(0.10, 0.20)
    # Counter on the solid half of the front, with a clipboard
    model.add(pbox((-hw - 0.02, hd - 0.30, 1.10), (-0.2, hd + 0.28, 1.16), "rock_light", bevel=0.0))
    model.add(pbox((-1.1, hd - 0.1, 1.16), (-0.7, hd + 0.12, 1.19), "white", bevel=0.0))
    model.add_collider("cage", box((-hw - 0.08, -hd - 0.08, 0.0), (hw + 0.08, hd + 0.08, top), "white"))


# ---------------------------------------------------------------- waiting chairs


def _waiting_chairs(model):
    """Row of four moulded waiting seats on one steel beam, with armrests and one tipped-back backrest."""
    xs = (-0.93, -0.31, 0.31, 0.93)
    # Beam and legs
    model.add(pbox((-1.2, -0.03, 0.34), (1.2, 0.03, 0.40), STEEL, bevel=0.008))
    for sx in (-1.05, 1.05):
        model.add(pbox((sx - 0.03, -0.16, 0.0), (sx + 0.03, 0.16, 0.06), "gun_black", bevel=0.006))
        model.add(_beam((sx, 0.0, 0.06), (sx, 0.0, 0.36), 0.026))
        model.add(_beam((sx, -0.14, 0.02), (sx, 0.14, 0.36), 0.016))
        model.add(_beam((sx, 0.14, 0.02), (sx, -0.14, 0.36), 0.016))
    # Armrests between and outside the seats
    for sx in (-1.16, -0.62, 0.0, 0.62, 1.16):
        model.add(pbox((sx - 0.035, -0.24, 0.62), (sx + 0.035, 0.24, 0.66), "shirt_blue", bevel=0.012))
        model.add(_beam((sx, 0.0, 0.40), (sx, 0.0, 0.63), 0.022))
    for i, sx in enumerate(xs):
        model.add(pbox((sx - 0.26, -0.22, 0.40), (sx + 0.26, 0.24, 0.47), "shirt_blue", bevel=0.018))
        lean = -0.35 if i == 2 else 0.0  # third backrest tipped back and cracked
        back = pbox((sx - 0.25, -0.25, 0.47), (sx + 0.25, -0.19, 0.92), "shirt_blue", bevel=0.018)
        common.transform(back, rot=(lean, 0.0, 0.0), loc=(0.0, 0.0, 0.0))
        model.add(back)
        if i == 2:
            model.add(pbox((sx - 0.2, -0.30, 0.60), (sx + 0.05, -0.2, 0.64), "soot", bevel=0.002))
        if i == 1:  # a bag left on the seat
            model.add(pbox((sx - 0.16, -0.12, 0.47), (sx + 0.16, 0.14, 0.72), "leather", bevel=0.03))
        if i == 3:  # papers scattered on the seat
            for k, (dx, dz) in enumerate(((0.0, 0.0), (0.1, 0.02), (-0.08, -0.03))):
                sheet = common.make_box((0.2, 0.26, 0.005), color="white" if k % 2 else "bone_white")
                common.transform(sheet, rot=(0.0, 0.0, 0.3 * k), loc=(sx + dx, 0.02 + dz, 0.475))
                model.add(sheet)
    model.add_collider("chairs", box((-1.22, -0.26, 0.0), (1.22, 0.26, 0.95), "white"))


# ---------------------------------------------------------------- cot


def _cot(model):
    """Folding army cot, 1.9 x 0.72 m: canvas bed, X legs, steel side rails, thin mattress and a folded blanket."""
    hw, hl = 0.36, 0.95
    # Canvas bed, slightly sagging
    model.add(pbox((-hw, -hl, 0.35), (hw, hl, 0.40), "canvas_khaki", bevel=0.02))
    model.add(pbox((-hw + 0.02, -0.6, 0.30), (hw - 0.02, 0.4, 0.36), "canvas_khaki", bevel=0.02))
    # Side rails
    for sx in (-hw - 0.03, hw + 0.03):
        model.add(_beam((sx, -hl, 0.41), (sx, hl, 0.41), 0.024, color=STEEL))
    for sy in (-hl, hl):
        model.add(_beam((-hw - 0.03, sy, 0.41), (hw + 0.03, sy, 0.41), 0.024, color=STEEL))
    # Folding X legs at both ends
    for sy in (-0.78, 0.78):
        model.add(_beam((-hw - 0.06, sy - 0.22, 0.02), (hw + 0.06, sy + 0.22, 0.40), 0.02, color=STEEL))
        model.add(_beam((hw + 0.06, sy - 0.22, 0.02), (-hw - 0.06, sy + 0.22, 0.40), 0.02, color=STEEL))
        model.add(_beam((-hw - 0.06, sy - 0.22, 0.02), (hw + 0.06, sy - 0.22, 0.02), 0.016, color=STEEL))
        model.add(pbox((-hw - 0.1, sy * 0.6 - 0.03, 0.40), (hw + 0.1, sy * 0.6 + 0.03, 0.435), STEEL, bevel=0.006))
    # Thin mattress, folded blanket at the foot, flat pillow at the head
    model.add(pbox((-hw + 0.01, -hl + 0.03, 0.40), (hw - 0.01, hl - 0.03, 0.48), "bone_white", bevel=0.02))
    model.add(pbox((-hw + 0.03, -0.65, 0.48), (hw - 0.03, 0.1, 0.54), "cloth_olive", bevel=0.02))
    model.add(pbox((-hw + 0.03, 0.55, 0.48), (hw - 0.03, 0.9, 0.55), "white", bevel=0.025))
    model.add_collider("cot", box((-hw - 0.12, -hl - 0.05, 0.0), (hw + 0.12, hl + 0.05, 0.55), "white"))


# ---------------------------------------------------------------- body bags


def _bag_parts(rng, length=1.9, width=0.62, z=0.0, color="pants_dark", detail=2):
    """One closed black body bag lying along Y: shaped body, zip, two straps, a toe tag.
    `detail` is the icosphere subdivision for the body lumps (2 for a single bag, 1 in the pile)."""
    parts = []
    parts.append(pbox((-width / 2, -length / 2, z), (width / 2, length / 2, z + 0.10), color, bevel=0.04))
    # Body lumps under the vinyl so it reads as a person
    lumps = ((0.62, 0.20, 0.16, 0.14), (0.30, 0.26, 0.24, 0.13),
             (-0.15, 0.24, 0.30, 0.14), (-0.62, 0.18, 0.28, 0.11))
    if detail < 2:
        lumps = ((0.45, 0.24, 0.34, 0.13), (-0.45, 0.20, 0.34, 0.10))
    for oy, rx, ry, rz in lumps:
        parts.append(common.make_icosphere(subdiv=detail, seed=int(abs(oy) * 100) + 3,
                                           radius=(rx, ry, rz), loc=(0.0, oy, z + 0.09), color=color))
    # Zip along the top and two straps
    parts.append(pbox((-0.012, -length / 2 + 0.1, z + 0.16), (0.012, length / 2 - 0.1, z + 0.175),
                      "gun_black", bevel=0.0))
    for sy in (-0.45, 0.45):
        parts.append(pbox((-width / 2 - 0.02, sy - 0.05, z + 0.02), (width / 2 + 0.02, sy + 0.05, z + 0.15),
                          "leather", bevel=0.006))
    parts.append(pbox((width / 2 - 0.02, length / 2 - 0.25, z + 0.06), (width / 2 + 0.04, length / 2 - 0.09, z + 0.13),
                      "white", bevel=0.004))
    return parts


def _pile_bag(rng, color="pants_dark"):
    """One bag of the heap: a plain shaped sack (cheap, they overlap and are seen from far)."""
    parts = [common.make_box((0.60, 1.85, 0.14), loc=(0.0, 0.0, 0.09), color=color),
             common.make_icosphere(subdiv=1, seed=rng.randrange(999), radius=(0.26, 0.80, 0.12),
                                   loc=(0.0, 0.0, 0.16), color=color),
             pbox((-0.30, -0.12, 0.02), (0.30, 0.05, 0.16), "leather", bevel=0.005)]
    return parts


def _body_bag(model):
    """One black body bag on the ground with a tag and a strap pulled loose."""
    rng = random.Random(31)
    for bm in _bag_parts(rng, z=0.0):
        model.add(bm)
    model.add(pbox((-0.34, -0.2, 0.0), (0.0, 0.15, 0.03), "pants_dark", bevel=0.01))  # settled skirt
    model.add_collider("bag", box((-0.36, -1.0, 0.0), (0.36, 1.0, 0.3), "white"))


def _body_bag_pile(model):
    """Heaped pile of 18 body bags on a 5 x 4 m patch of ground, three layers, one strap dangling."""
    rng = random.Random(5507)
    spots = []
    # Bottom layer: 10 bags side by side, roughly parallel
    for i in range(10):
        x = -2.1 + (i % 5) * 1.05 + rng.uniform(-0.12, 0.12)
        y = -1.5 + (i // 5) * 1.5 + rng.uniform(-0.15, 0.15)
        spots.append((x, y, 0.0, rng.uniform(-0.35, 0.35), 1.0))
    # Middle layer: 6 bags lying across the lower ones
    for i in range(6):
        x = -1.7 + (i % 4) * 1.15 + rng.uniform(-0.1, 0.1)
        y = -0.9 + (i // 4) * 1.6 + rng.uniform(-0.1, 0.1)
        spots.append((x, y, 0.16, 1.5708 + rng.uniform(-0.25, 0.25), 1.0))
    # Top layer: 2 bags dropped on the heap
    for i in range(2):
        spots.append((-0.7 + i * 1.3, 0.2 + rng.uniform(-0.2, 0.2), 0.34, rng.uniform(-0.4, 0.4), 1.0))
    for x, y, z, yaw, scale in spots:
        bag = common.combine(_pile_bag(rng))
        common.transform(bag, rot=(rng.uniform(-0.05, 0.05), rng.uniform(-0.05, 0.05), yaw), loc=(x, y, z))
        model.add(bag)
    # Loose straps and a clipboard dropped beside the pile
    model.add(pbox((2.0, -1.4, 0.02), (2.35, -1.1, 0.03), "leather", bevel=0.004))
    model.add(pbox((-2.45, 1.2, 0.02), (-2.1, 1.5, 0.035), "white", bevel=0.004))
    model.add_collider("pile", box((-2.6, -2.0, 0.0), (2.6, 2.0, 0.6), "white"))


# ---------------------------------------------------------------- triage tent


def _triage_tent(model):
    """6 x 4 m open-sided white triage tent: 4 corner poles, ridge, tarp roof with a red cross, guy ropes."""
    hw, hd = 3.0, 2.0
    eave_z, ridge_z = 2.2, 2.9
    # Poles: four corners plus two ridge posts
    for sx in (-hw, hw):
        for sy in (-hd, hd):
            model.add(pbox((sx - 0.05, sy - 0.05, 0.0), (sx + 0.05, sy + 0.05, eave_z), STEEL, bevel=0.006))
            model.add(pbox((sx - 0.16, sy - 0.16, 0.0), (sx + 0.16, sy + 0.16, 0.05), "gun_black", bevel=0.006))
    for sy in (-hd, hd):
        model.add(pbox((-hw, sy - 0.04, eave_z - 0.06), (hw, sy + 0.04, eave_z), STEEL, bevel=0.004))
    model.add(_beam((0.0, -hd, ridge_z), (0.0, hd, ridge_z), 0.04, color=STEEL))
    for sy in (-hd, hd):
        model.add(_beam((0.0, sy, eave_z), (0.0, sy, ridge_z), 0.035, color=STEEL))
    # Roof: two tarp panels from the ridge down to each eave, with a sagging overhang
    for sx in (-1.0, 1.0):
        top = common.new_bm()
        pts = [(0.0, -hd - 0.35, ridge_z), (0.0, hd + 0.35, ridge_z),
               (sx * (hw + 0.35), hd + 0.35, eave_z - 0.10), (sx * (hw + 0.35), -hd - 0.35, eave_z - 0.10)]
        common.add_quad(top, pts, (0.0, 0.0, 1.0), "white")
        model.add(top)
        under = common.new_bm()
        common.add_quad(under, [(p[0] * 0.999, p[1], p[2] - 0.03) for p in reversed(pts)],
                        (0.0, 0.0, -1.0), "bone_white")
        model.add(under)
    # Red cross painted on both roof panels
    for sx in (-1.0, 1.0):
        for du, dv, col in ((0.0, 0.0, "red_paint"),):
            cx = sx * 1.55
            cz = (ridge_z + eave_z - 0.10) / 2.0 + 0.12
            bar = common.make_box((0.9, 0.30, 0.02), color=col)
            common.transform(bar, rot=(0.0, sx * -0.19, 0.0), loc=(cx, 0.0, cz + 0.02))
            model.add(bar)
            bar2 = common.make_box((0.30, 0.9, 0.02), color=col)
            common.transform(bar2, rot=(0.0, sx * -0.19, 0.0), loc=(cx, 0.0, cz + 0.03))
            model.add(bar2)
    # Side skirt of tarp hanging at the back, and a small sign board on the front eave
    model.add(pbox((-hw + 0.1, -hd + 0.02, 0.3), (hw - 0.1, -hd + 0.07, eave_z - 0.05), "white", bevel=0.01))
    model.add(pbox((-0.7, hd - 0.06, eave_z - 0.45), (0.7, hd - 0.01, eave_z - 0.12), "white", bevel=0.006))
    for bm in place_kit.letters_bms("TRIAGE", (0.0, hd - 0.04, eave_z - 0.42), 0.16, 0.24, (0.0, 1.0, 0.0),
                                    color="red_paint", depth=0.02):
        model.add(bm)
    # Guy ropes to pegs
    for sx in (-hw, hw):
        for sy in (-hd, hd):
            peg = (sx * 1.28, sy * 1.20, 0.0)
            model.add(_beam((sx, sy, eave_z - 0.02), peg, 0.012, color="canvas_khaki", segs=4))
            model.add(pbox((peg[0] - 0.04, peg[1] - 0.04, 0.0), (peg[0] + 0.04, peg[1] + 0.04, 0.22),
                           "wood_crate", bevel=0.006))
    model.add_collider("tent", box((-hw - 0.2, -hd - 0.2, 0.0), (hw + 0.2, hd + 0.2, eave_z), "white"))


# ---------------------------------------------------------------- ambulance


def _ambulance(model):
    """Boxy ambulance wreck, 5.7 x 2.3 m: cab + box body, both rear doors and the side door open,
    one flat tyre, smashed windscreen, faded stripes, a red cross and mirror "AMBULANCE" on the hood."""
    rng = random.Random(6180)
    hl, hw = 2.85, 1.15
    body = "car_white"
    # Chassis and body shell
    model.add(pbox((-hw + 0.05, -hl + 0.15, 0.42), (hw - 0.05, hl - 0.10, 0.62), "gun_black", bevel=0.02))
    model.add(pbox((-hw, -hl, 0.55), (hw, -0.35, 2.55), body, bevel=0.04))          # box body
    model.add(pbox((-hw + 0.02, -0.35, 0.55), (hw - 0.02, 0.55, 2.20), body, bevel=0.03))  # crew cab
    model.add(pbox((-hw + 0.06, 0.40, 0.55), (hw - 0.06, hl - 0.35, 1.28), body, bevel=0.03))  # bonnet
    # Rounded box roof with the light bar
    model.add(pbox((-hw + 0.02, -hl + 0.02, 2.50), (hw - 0.02, -0.05, 2.66), body, bevel=0.05))
    model.add(pbox((-0.5, -0.3, 2.66), (0.5, 0.1, 2.76), "metal", bevel=0.01))
    model.add(pbox((-0.44, -0.24, 2.74), (-0.06, 0.04, 2.80), "red_paint", bevel=0.006))
    model.add(pbox((0.06, -0.24, 2.74), (0.44, 0.04, 2.80), "water_blue", bevel=0.006))
    model.add(pbox((0.30, -0.2, 2.79), (0.42, 0.0, 2.82), "gun_black", bevel=0.0))  # broken lens
    # Windscreen (raked) smashed: dark hole with two crack bars and a shard
    ws = common.new_bm()
    common.add_quad(ws, [(-hw + 0.1, 0.42, 1.30), (hw - 0.1, 0.42, 1.30),
                         (hw - 0.16, 0.62, 2.12), (-hw + 0.16, 0.62, 2.12)], (0.0, 1.0, 0.4), "soot")
    model.add(ws)
    for s in (-1.0, 1.0):
        model.add(_beam((s * 0.55, 0.44, 1.40), (-s * 0.30, 0.60, 2.02), 0.018, color="glass_dark", segs=4))
    model.add(_beam((-0.2, 0.60, 2.10), (-0.1, 0.44, 1.35), 0.012, color="glass_dark", segs=4))
    # Side windows of the cab, cracked and one gone
    for sx in (-1.0, 1.0):
        x = sx * (hw - 0.01)
        model.add(pbox((x, -0.28, 1.34), (x + sx * 0.02, 0.40, 2.02), "soot", bevel=0.004))
        model.add(_beam((x, -0.2, 1.42), (x, 0.3, 1.9), 0.012, color="glass_dark", segs=4))
    # Side crew door on the right, swung out and hanging
    door = common.combine([
        pbox((-0.02, -0.02, 0.0), (0.02, 0.02, 1.60), body, bevel=0.01),
        pbox((0.86, -0.02, 0.0), (0.90, 0.02, 1.60), body, bevel=0.01),
        pbox((-0.02, -0.02, 0.0), (0.90, 0.02, 0.08), body, bevel=0.01),
        pbox((-0.02, -0.02, 1.52), (0.90, 0.02, 1.60), body, bevel=0.01),
        pbox((0.12, -0.03, 0.62), (0.74, 0.03, 1.34), "soot", bevel=0.004),
        pbox((0.66, 0.02, 0.55), (0.78, 0.08, 0.70), "chrome", bevel=0.006),
    ])
    common.transform(door, rot=(0.0, 0.0, math.radians(-72.0)), loc=(hw - 0.02, 0.36, 0.60))
    model.add(door)
    # Rear doors, both swung open
    for sx in (-1.0, 1.0):
        leaf = common.combine([
            pbox((0.0, -0.02, 0.0), (0.04, 0.02, 1.90), body, bevel=0.008),
            pbox((1.05, -0.02, 0.0), (1.09, 0.02, 1.90), body, bevel=0.008),
            pbox((0.0, -0.02, 0.0), (1.09, 0.02, 0.07), body, bevel=0.008),
            pbox((0.0, -0.02, 1.83), (1.09, 0.02, 1.90), body, bevel=0.008),
            pbox((0.16, 0.02, 1.10), (0.92, 0.05, 1.70), "soot", bevel=0.004),
        ])
        common.transform(leaf, rot=(0.0, 0.0, math.radians(-95.0 if sx > 0 else 95.0)),
                         loc=(sx * 1.02 if sx > 0 else sx * 1.02, -hl - 0.02, 0.62))
        model.add(leaf)
    # Open rear: dark interior, a stretcher rail and a dangling strap
    model.add(pbox((-hw + 0.06, -hl + 0.03, 0.62), (hw - 0.06, -hl + 0.10, 1.90), "soot", bevel=0.004))
    model.add(pbox((-hw + 0.12, -hl + 0.12, 0.78), (hw - 0.12, -hl + 0.55, 0.86), "chrome", bevel=0.01))
    model.add(_beam((0.5, -hl - 0.12, 2.1), (0.56, -hl - 0.02, 1.4), 0.02, color="leather", segs=4))
    # Stripes (faded), cross, and markings on the box side
    for sx in (-1.0, 1.0):
        x = sx * hw
        model.add(pbox((x, -hl + 0.15, 1.35), (x + sx * 0.02, -0.45, 1.55), "red_paint", bevel=0.0))
        model.add(pbox((x, -1.5, 1.05), (x + sx * 0.03, -1.1, 1.15), "red_paint", bevel=0.0))
        model.add(pbox((x, -1.5, 1.20), (x + sx * 0.03, -1.1, 1.30), "red_paint", bevel=0.0))
        # Red cross on the box side
        cx = x + sx * 0.03
        model.add(pbox((cx, -1.95, 1.95), (cx + sx * 0.03, -1.30, 2.05), "red_paint", bevel=0.0))
        model.add(pbox((cx, -1.70, 1.62), (cx + sx * 0.03, -1.55, 2.38), "red_paint", bevel=0.0))
        # Rust and soot patches
        for i in range(3):
            py = rng.uniform(-2.4, 0.2)
            pz = rng.uniform(0.7, 2.3)
            model.add(pbox((x + sx * 0.012, py, pz), (x + sx * 0.03, py + rng.uniform(0.15, 0.45),
                                                      pz + rng.uniform(0.12, 0.4)), "rust_dark", bevel=0.0))
    # Mirror lettering on the bonnet (reads correctly in a mirror)
    for bm in place_kit.letters_bms("AMBULANCE", (0.0, 1.95, 1.30), 0.115, 0.30, (0.0, 1.0, 0.0),
                                    color="soot", depth=0.02, mirror=True):
        model.add(bm)
    # Front bumper hanging, grille, headlights, mirrors
    model.add(pbox((-hw + 0.02, hl - 0.16, 0.42), (hw - 0.02, hl + 0.04, 0.62), "gun_black", bevel=0.02))
    model.add(pbox((hw - 0.3, hl - 0.14, 0.30), (hw - 0.05, hl + 0.06, 0.46), "gun_black", bevel=0.015))
    for k in range(4):
        model.add(pbox((-0.5 + k * 0.26, hl - 0.16, 0.66), (-0.36 + k * 0.26, hl - 0.10, 1.02), "metal", bevel=0.0))
    for sx in (-1.0, 1.0):
        model.add(pbox((sx * 0.72 - 0.14, hl - 0.14, 0.70), (sx * 0.72 + 0.14, hl - 0.06, 0.90),
                       "bone_white" if sx < 0 else "soot", bevel=0.01))
        model.add(_beam((sx * hw, 0.30, 1.45), (sx * (hw + 0.26), 0.34, 1.72), 0.02, color="gun_black", segs=4))
        model.add(pbox((sx * (hw + 0.24) - 0.03, 0.28, 1.60), (sx * (hw + 0.24) + 0.05, 0.40, 1.84),
                       "gun_black", bevel=0.008))
    # Wheels: front-left flat, rear-right tyre missing (rim on the ground)
    wheel_parts_list = (
        (hw - 0.08, 1.75, 0.46, 0.16, 0.72, True),
        (-hw + 0.08, 1.75, 0.46, 0.16, 1.0, True),
        (hw - 0.08, -1.62, 0.46, 0.16, 1.0, True),
        (-hw + 0.08, -1.62, 0.46, 0.16, 1.0, False),
    )
    for x, y, r, hwid, flat, tyre in wheel_parts_list:
        for bm in wheel_parts(x, y, r, hwid, flat=flat, tyre=tyre):
            model.add(bm)
    model.add_collider("ambulance", box((-hw - 0.06, -hl - 0.08, 0.0), (hw + 0.06, hl + 0.06, 2.6), "white"))


def _ambulance_lod1(model):
    """Far LOD: two clean boxes, a windscreen slab, four wheel boxes and one stripe."""
    model.add(pbox((-1.15, -0.35, 0.5), (1.15, 2.85, 2.2), "car_white", bevel=0.02))
    model.add(pbox((-1.15, -2.85, 0.5), (1.15, -0.35, 2.55), "car_white", bevel=0.02))
    model.add(pbox((-1.2, 0.4, 1.3), (1.2, 0.7, 2.1), "soot", bevel=0.0))
    model.add(pbox((-1.18, -2.4, 1.35), (1.18, -0.5, 1.6), "red_paint", bevel=0.0))
    for x, y in ((1.05, 1.75), (-1.05, 1.75), (1.05, -1.62), (-1.05, -1.62)):
        model.add(pbox((x - 0.16, y - 0.42, 0.0), (x + 0.16, y + 0.42, 0.9), "gun_black", bevel=0.02))
    model.add_collider("ambulance", box((-1.2, -2.9, 0.0), (1.2, 2.9, 2.6), "white"))


# ---------------------------------------------------------------- build list


def build_all():
    """Builds every hospital prop. Returns [(name, ok)]."""
    front = FRONT_VIEW
    specs = [
        ("gurney", "props", PROP_BUDGET, _gurney, front),
        ("hospital_bed", "props", PROP_BUDGET, _hospital_bed, front),
        ("ward_curtain", "props", PROP_BUDGET, _ward_curtain, (0.6, 1.0, 0.2)),
        ("wheelchair", "props", PROP_BUDGET, _wheelchair, front),
        ("iv_stand", "props", PROP_BUDGET, _iv_stand, front),
        ("reception_desk_h", "props", PROP_BUDGET, _reception_desk, front),
        ("pharmacy_cage", "props", PROP_BUDGET, _pharmacy_cage, front),
        ("waiting_chairs", "props", PROP_BUDGET, _waiting_chairs, front),
        ("cot", "props", PROP_BUDGET, _cot, front),
        ("body_bag", "props", PROP_BUDGET, _body_bag, (0.8, 1.0, 0.7)),
        ("body_bag_pile", "props", PILE_BUDGET, _body_bag_pile, (1.0, 1.0, 0.9)),
        ("triage_tent", "props", TENT_BUDGET, _triage_tent, (1.0, 1.0, 0.5)),
        ("ambulance_wreck", "vehicles", VEHICLE_BUDGET, _ambulance, front),
        ("ambulance_wreck_lod1", "vehicles", VEHICLE_BUDGET // 10, _ambulance_lod1, front),
    ]
    out = []
    for name, category, budget, fn, view in specs:
        ok = common.run_model(name, category, budget, fn, view=view)
        out.append((name, ok))
    return out


if __name__ == "__main__":
    failed = [n for n, ok in build_all() if not ok]
    print("PROPS DONE" if not failed else f"PROPS FAILED: {failed}")
    sys.exit(1 if failed else 0)
