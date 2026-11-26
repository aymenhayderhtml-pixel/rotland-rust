"""Fallen tower section (`tw_fallen_section` and `_lod1`) lying on its side across a street.

24 m wide (Y) x 24 m high (Z) x 72 m long (X), origin at bottom centre (Z=0 at ground).
+X cut end is open ~6 floors deep showing vertical floor slabs, soot-stained interior walls,
ripped window frames, hanging pipes and cable runs.
"""
import math
import os
import random
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, os.path.join(HERE, "..", "buildings"))
sys.path.insert(0, os.path.join(HERE, "..", "towers"))

import bmesh  # noqa: E402
from mathutils import Matrix, Vector  # noqa: E402

import common  # noqa: E402
import kit  # noqa: E402

BUDGET_LOD0 = 30000  # data: budgets.json buildings
BUDGET_LOD1 = 3000  # new: DECISIONS 2026-11-26 (10% LOD1 budget for fallen section)
SECTION_LEN = 72.0  # new: DECISIONS 2026-11-26 (60-80 m broken tower section length)
TOWER_SIZE = 24.0  # new: DECISIONS 2026-11-26 (24 m tower footprint width and height)
FLOOR_H = 3.8  # new: DECISIONS 2026-11-26 (standard tower floor height)
BAY_W = 4.0  # new: DECISIONS 2026-11-26 (standard 4 m facade bay)
WALL_T = 0.45  # new: DECISIONS 2026-11-26 (perimeter wall thickness on cut section)
SLAB_T = 0.26  # new: DECISIONS 2026-11-26 (floor slab thickness)
NUM_FLOORS = 18  # new: DECISIONS 2026-11-26 (18 x 3.8 m = 68.4 m + jagged ends = 72 m)
OPEN_X_START = 11.4  # new: DECISIONS 2026-11-26 (+X cut end open 6 floors deep: 34.2 - 22.8)
CLOSED_X_START = -34.2  # new: DECISIONS 2026-11-26 (-X end of regular floor grid)


def _angled_bar(model, a, b, w, t, color):
    va, vb = Vector(a), Vector(b)
    d = vb - va
    if d.length < 1e-4:  # new: DECISIONS 2026-11-26
        return
    z_axis = d.normalized()
    up = Vector((0.0, 0.0, 1.0)) if abs(z_axis.z) < 0.9 else Vector((1.0, 0.0, 0.0))  # new: DECISIONS 2026-11-26
    x_axis = up.cross(z_axis).normalized()
    y_axis = z_axis.cross(x_axis).normalized()
    bm = common.make_box((w, t, d.length), color=color)
    mat = Matrix.Translation((va + vb) / 2) @ Matrix((x_axis, y_axis, z_axis)).transposed().to_4x4()
    bmesh.ops.transform(bm, matrix=mat, verts=list(bm.verts))
    bm.normal_update()
    model.add(bm)


def _pipe(model, a, b, radius, color="metal"):
    va, vb = Vector(a), Vector(b)
    d = vb - va
    if d.length < 1e-4:  # new: DECISIONS 2026-11-26
        return
    z_axis = d.normalized()
    up = Vector((0.0, 0.0, 1.0)) if abs(z_axis.z) < 0.9 else Vector((1.0, 0.0, 0.0))  # new: DECISIONS 2026-11-26
    x_axis = up.cross(z_axis).normalized()
    y_axis = z_axis.cross(x_axis).normalized()
    bm = common.make_frustum(radius, radius, -d.length / 2, d.length / 2, segments=6, color=color)  # new: DECISIONS 2026-11-26
    mat = Matrix.Translation((va + vb) / 2) @ Matrix((x_axis, y_axis, z_axis)).transposed().to_4x4()
    bmesh.ops.transform(bm, matrix=mat, verts=list(bm.verts))
    bm.normal_update()
    model.add(bm)


def _window_cell(k, x0, x1, face_axis, face_coord, sign, u_c, state, is_open_zone=False):
    """One 3.8 m x 4.0 m facade window bay on the fallen tower (+Y, -Y, or +Z face)."""
    wx0, wx1 = x0 + 0.82, x0 + 2.68  # new: DECISIONS 2026-11-26 (deco window span along floor height)
    hw = 1.15  # new: DECISIONS 2026-11-26 (half bay window width)
    g0, g1 = sorted((face_coord, face_coord + sign * 0.03))  # new: DECISIONS 2026-11-26
    s0, s1 = sorted((face_coord, face_coord + sign * 0.11))  # new: DECISIONS 2026-11-26

    # Intact dark glass on a few bays; most are shattered soot or empty dark cavities
    glass_col = "glass_dark" if state == 0 else "soot"  # new: DECISIONS 2026-11-26
    if face_axis == 2:  # +Z top face (y is u_c)
        if not (is_open_zone and state == 4):
            k.box((wx0, u_c - hw, g0), (wx1, u_c + hw, g1), glass_col)
        k.box((wx0 - 0.12, u_c - hw - 0.10, s0), (wx0, u_c + hw + 0.10, s1), "limestone_dark")
        k.box((wx1, u_c - hw - 0.10, s0), (wx1 + 0.12, u_c + hw + 0.10, s1), "limestone_dark")
        if state in (0, 3):  # center mullion
            k.box((wx0, u_c - 0.04, s0), (wx1, u_c + 0.04, g1), "metal")  # new: DECISIONS 2026-11-26
        elif state == 2:  # wooden boards
            k.box((wx0 + 0.45, u_c - hw - 0.05, face_coord + 0.04),
                  (wx0 + 0.62, u_c + hw + 0.05, face_coord + 0.09), "bark_light")  # new: DECISIONS 2026-11-26
            k.box((wx1 - 0.62, u_c - hw - 0.05, face_coord + 0.04),
                  (wx1 - 0.45, u_c + hw + 0.05, face_coord + 0.09), "bark_light")  # new: DECISIONS 2026-11-26
        elif state == 3:
            _angled_bar(k.model, (wx0 + 0.2, u_c - 0.7, face_coord + 0.05),
                        (wx1 - 0.2, u_c + 0.6, face_coord + 0.05), 0.04, 0.04, "soot")  # new: DECISIONS 2026-11-26
        if is_open_zone and state in (1, 4):  # ripped frame bar sticking up
            _angled_bar(k.model, (wx0 + 0.2, u_c - hw, face_coord + 0.05),
                        (wx0 + 0.9, u_c - hw + 0.35, face_coord + 0.65), 0.05, 0.05, "metal")  # new: DECISIONS 2026-11-26
    else:  # +Y or -Y side face (z is u_c)
        if not (is_open_zone and state == 4):
            k.box((wx0, g0, u_c - hw), (wx1, g1, u_c + hw), glass_col)
        k.box((wx0 - 0.12, s0, u_c - hw - 0.10), (wx0, s1, u_c + hw + 0.10), "limestone_dark")
        k.box((wx1, s0, u_c - hw - 0.10), (wx1 + 0.12, s1, u_c + hw + 0.10), "limestone_dark")
        if state in (0, 3):
            k.box((wx0, s0, u_c - 0.04), (wx1, g1, u_c + 0.04), "metal")  # new: DECISIONS 2026-11-26
        elif state == 2:
            k.box((wx0 + 0.45, face_coord + sign * 0.04, u_c - hw - 0.05),
                  (wx0 + 0.62, face_coord + sign * 0.09, u_c + hw + 0.05), "bark_light")  # new: DECISIONS 2026-11-26
        if is_open_zone and state in (1, 4):
            _angled_bar(k.model, (wx1 - 0.2, face_coord + sign * 0.04, u_c + hw),
                        (wx1 + 0.4, face_coord + sign * 0.55, u_c + hw - 0.3), 0.05, 0.05, "metal")  # new: DECISIONS 2026-11-26


def _add_exterior_shell(k, rng):
    """12 closed floors (-34.2..+11.4) + 6 hollow cut floors (+11.4..+34.2) + long piers and window bays."""
    hs = TOWER_SIZE / 2.0
    inset = 0.25  # new: DECISIONS 2026-11-26 (matches deco.py recessed wall inset)
    wf = hs - inset
    top_z = TOWER_SIZE - inset

    # Solid core for the closed rear section (-34.2..+11.4)
    k.box((CLOSED_X_START, -wf, 0.0), (OPEN_X_START, wf, top_z), "limestone")
    # Bulkhead wall closing the back of the 6-floor open cavity at X = OPEN_X_START
    k.box((OPEN_X_START - 0.3, -wf, 0.0), (OPEN_X_START, wf, top_z), "soot")  # new: DECISIONS 2026-11-26

    # Hollow shell for the 6 open floors (+11.4..+34.2), leaving top-end bays partly caved in
    end_x = OPEN_X_START + 6 * FLOOR_H  # 34.2 m
    k.box((OPEN_X_START, -wf, 0.0), (end_x, wf, WALL_T), "concrete_grey")  # bottom wall lying on ground
    k.box((OPEN_X_START, -wf, WALL_T), (end_x, -wf + WALL_T, top_z), "limestone")  # -Y wall
    k.box((OPEN_X_START, wf - WALL_T, WALL_T), (end_x, wf, top_z), "limestone")  # +Y wall
    # Top (+Z) wall caved back on the last 2 floors so _top preview also sees inside
    k.box((OPEN_X_START, -wf, top_z - WALL_T), (end_x - 7.6, wf, top_z), "limestone", roof=True)  # new: DECISIONS 2026-11-26
    k.box((end_x - 7.6, -wf, top_z - WALL_T), (end_x - 2.2, -2.0, top_z), "limestone", roof=True)  # new: DECISIONS 2026-11-26
    k.box((end_x - 7.6, 3.5, top_z - WALL_T), (end_x, wf, top_z), "limestone", roof=True)  # new: DECISIONS 2026-11-26

    # Interior pale plaster lining inside the open cavity with soot patches
    k.box((OPEN_X_START + 0.02, -wf + WALL_T, WALL_T), (end_x - 1.5, -wf + WALL_T + 0.03, top_z - WALL_T), "rock_light")
    k.box((OPEN_X_START + 0.02, wf - WALL_T - 0.03, WALL_T), (end_x - 0.8, wf - WALL_T, top_z - WALL_T), "rock_light")

    # Longitudinal Art Deco piers along X on +Z, +Y, -Y faces
    n_bays = int(round(TOWER_SIZE / BAY_W))
    for i in range(n_bays + 1):
        u = -hs + i * BAY_W
        hw = 0.34 if i in (0, n_bays) else 0.25  # new: DECISIONS 2026-11-26
        pier_end = end_x if i != 3 else end_x - 5.5  # broken middle pier on top  # new: DECISIONS 2026-11-26
        k.box((CLOSED_X_START, u - hw, top_z), (pier_end, u + hw, TOWER_SIZE), "limestone_dark", roof=(i in (2, 3, 4)))
        if i > 0:
            z_u = i * BAY_W
            k.box((CLOSED_X_START, wf, z_u - hw), (end_x - (2.0 if i == 5 else 0.0), hs, z_u + hw), "limestone_dark")
            k.box((CLOSED_X_START, -hs, z_u - hw), (end_x - (1.8 if i == 4 else 0.0), -wf, z_u + hw), "limestone_dark")

    # Floor slab bands + window bays per floor
    for f_idx in range(NUM_FLOORS):
        x0 = CLOSED_X_START + f_idx * FLOOR_H
        x1 = x0 + FLOOR_H
        is_open = x0 >= OPEN_X_START - 0.01
        is_top_hole = f_idx >= NUM_FLOORS - 2  # new: DECISIONS 2026-11-26

        # Floor band around perimeter
        k.box((x0, -hs, 0.0), (x0 + 0.22, hs, 0.4), "limestone_dark")
        k.box((x0, -hs, 0.0), (x0 + 0.22, -wf, TOWER_SIZE), "limestone_dark")
        k.box((x0, wf, 0.0), (x0 + 0.22, hs, TOWER_SIZE), "limestone_dark")
        if not is_top_hole:
            k.box((x0, -hs, top_z), (x0 + 0.22, hs, TOWER_SIZE), "limestone_dark", roof=is_open)

        # Window bays: +Z top face, +Y front side, -Y back side
        for b in range(n_bays):
            if is_top_hole and b in (2, 3):  # caved-in top opening  # new: DECISIONS 2026-11-26
                continue
            u_c = -hs + (b + 0.5) * BAY_W
            r = rng.random()
            st_top = 0 if r < 0.22 else (2 if r < 0.36 else (3 if r < 0.62 else (4 if is_open else 1)))  # new: DECISIONS 2026-11-26
            _window_cell(k, x0, x1, 2, top_z, 1, u_c, st_top, is_open_zone=is_open)

        for side_sign in (1, -1):
            for b in range(1, n_bays):  # skip bottom row (z=0..4) partially buried in rubble
                if f_idx == NUM_FLOORS - 1 and b >= 4:  # jagged cut corner  # new: DECISIONS 2026-11-26
                    continue
                z_c = (b + 0.5) * BAY_W
                r = rng.random()
                st = 0 if r < 0.18 else (2 if r < 0.32 else (3 if r < 0.60 else 1))  # new: DECISIONS 2026-11-26
                _window_cell(k, x0, x1, 1, side_sign * wf, side_sign, z_c, st, is_open_zone=is_open)


def _add_exposed_interior(k, rng):
    """Vertical floor slabs (6 floors deep), pale interior walls with soot, pipes and cable runs."""
    hs = TOWER_SIZE / 2.0
    in_w = hs - WALL_T - 0.25
    top_in = TOWER_SIZE - WALL_T - 0.25

    for idx in range(6):
        fx = OPEN_X_START + (idx + 1) * FLOOR_H
        # Floor slab split into 2-3 jagged concrete panels so edges look broken
        y_split = -2.0 + (idx % 3) * 1.5  # new: DECISIONS 2026-11-26
        z_top_a = top_in - (2.8 if idx >= 4 else 0.4)  # new: DECISIONS 2026-11-26
        z_top_b = top_in - (5.2 if idx == 5 else (1.5 if idx == 4 else 0.2))  # new: DECISIONS 2026-11-26
        k.box((fx - SLAB_T, -in_w, WALL_T), (fx, y_split, z_top_a), "concrete_grey")
        k.box((fx - SLAB_T, y_split + 1.6, WALL_T), (fx, in_w, z_top_b), "concrete_stain")  # doorway gap
        # Header lintel above doorway gap
        k.box((fx - SLAB_T, y_split, 11.0), (fx, y_split + 1.6, min(z_top_a, z_top_b)), "limestone_dark")
        # Soot blotch on the pale slab face
        k.box((fx, -in_w + 1.5, 4.0 + idx * 1.1), (fx + 0.04, y_split - 0.5, 11.5 + idx * 0.8), "soot")

        # Pale interior partition walls (running along X between floor slabs)
        px0 = fx - FLOOR_H + 0.02
        py = -5.5 + (idx % 2) * 10.5  # new: DECISIONS 2026-11-26
        k.box((px0, py, WALL_T), (fx - SLAB_T, py + 0.24, 15.0 - idx * 0.7), "rock_light")
        k.box((px0 + 0.5, py - 0.03, 5.0), (fx - 0.8, py + 0.27, 9.5), "soot")  # char stain on interior wall

        # Interior collapsed rubble pile resting on the bottom wall inside the open end
        if idx >= 2:
            rz = WALL_T + 1.6 + (idx - 2) * 0.25  # new: DECISIONS 2026-11-26
            k.box((px0 + 0.2, -in_w + 0.4, WALL_T), (fx - 0.3, in_w - 0.4, rz), "concrete_stain")

    # Jagged broken slab teeth jutting out past +34.2 m to +36.0 m (stepped low so interior shows cleanly)
    end_x = OPEN_X_START + 6 * FLOOR_H
    jagged = [
        ((end_x - 0.2, -11.6, 0.0), (35.9, -6.5, 6.5), "limestone"),
        ((end_x - 0.2, -6.5, 0.0), (35.3, 1.5, 2.4), "concrete_grey"),
        ((end_x - 0.2, 6.2, 0.0), (35.8, 11.6, 9.2), "limestone_dark"),
        ((end_x - 0.2, -11.6, 13.5), (35.5, -8.5, 21.5), "limestone_dark"),
        ((end_x - 0.2, 8.2, 14.0), (35.6, 11.6, 20.5), "limestone"),
    ]  # new: DECISIONS 2026-11-26
    for lo, hi, col in jagged:
        k.box(lo, hi, col)

    # -X broken stump extensions (-36.0 to -34.2) with exposed rebar and shattered concrete
    stump_x = CLOSED_X_START
    for lo, hi, col in (
        ((-36.0, -11.8, 0.0), (stump_x + 0.2, -3.5, 18.0), "limestone_dark"),
        ((-35.8, -3.5, 0.0), (stump_x + 0.2, 5.2, 11.5), "concrete_stain"),
        ((-35.5, 5.2, 0.0), (stump_x + 0.2, 11.8, 21.0), "limestone"),
    ):  # new: DECISIONS 2026-11-26
        k.box(lo, hi, col)
    for k_r in range(8):
        ry = -9.5 + k_r * 2.6  # new: DECISIONS 2026-11-26
        rz = 2.5 + (k_r % 4) * 4.2  # new: DECISIONS 2026-11-26
        _angled_bar(k.model, (stump_x + 0.2, ry, rz), (-36.0, ry + 0.3, rz - 0.6), 0.06, 0.06, "rust_dark")

    # Hanging pipes & cable runs poking out of the +X cut end
    pipes = [
        ((24.0, -8.2, 14.5), (35.8, -8.2, 14.2), 0.14, "metal"),
        ((26.5, -7.6, 13.8), (35.4, -7.2, 12.8), 0.10, "rust_dark"),
        ((23.0, 7.5, 16.0), (35.6, 7.8, 15.2), 0.15, "metal"),
        ((27.0, 2.5, 10.5), (35.9, 2.2, 8.8), 0.12, "metal"),
    ]  # new: DECISIONS 2026-11-26
    for a, b, rad, col in pipes:
        _pipe(k.model, a, b, rad, col)
        # Bent hanging tip
        _pipe(k.model, b, (b[0] + 0.35, b[1] + rng.uniform(-0.4, 0.4), b[2] - 2.6), rad * 0.9, "rust_dark")

    # Drooping cable runs hanging from exposed ceiling/slabs
    cables = [
        ((31.2, -5.2, 16.5), (34.8, -5.0, 11.2), (35.2, -4.8, 5.5)),
        ((32.5, 1.8, 13.8), (34.9, 2.1, 8.5), (34.6, 2.4, 2.5)),
        ((30.4, 6.2, 17.5), (34.6, 5.9, 12.4), (35.1, 5.5, 6.2)),
        ((33.0, -1.5, 11.0), (35.3, -1.2, 5.8), (35.5, -0.9, 1.2)),
    ]  # new: DECISIONS 2026-11-26
    for p0, p1, p2 in cables:
        _angled_bar(k.model, p0, p1, 0.07, 0.07, "gun_black")
        _angled_bar(k.model, p1, p2, 0.06, 0.06, "gun_black")


def _add_colliders(k):
    """Solid core collider + hollow walls/floor around the +X exposed interior."""
    hs = TOWER_SIZE / 2.0
    end_x = OPEN_X_START + 6 * FLOOR_H
    k.collider((-35.8, -hs + 0.2, 0.0), (OPEN_X_START + 0.2, hs - 0.2, TOWER_SIZE - 0.2), "core")  # new: DECISIONS 2026-11-26
    k.collider((OPEN_X_START, -hs + 0.2, 0.0), (end_x + 1.0, hs - 0.2, 1.6), "cut_floor")  # new: DECISIONS 2026-11-26
    k.collider((OPEN_X_START, -hs + 0.2, 1.6), (end_x + 0.6, -hs + 1.0, TOWER_SIZE - 0.5), "cut_wall_l")
    k.collider((OPEN_X_START, hs - 1.0, 1.6), (end_x + 0.6, hs - 0.2, TOWER_SIZE - 0.5), "cut_wall_r")
    k.collider((OPEN_X_START, -hs + 1.0, TOWER_SIZE - 1.4), (end_x - 3.0, hs - 1.0, TOWER_SIZE - 0.2), "cut_top")


def build_lod0(model):
    k = kit.Kit(model)
    rng = random.Random(4004)  # new: DECISIONS 2026-11-26
    _add_exterior_shell(k, rng)
    _add_exposed_interior(k, rng)
    _add_colliders(k)


def build_lod1(model):
    k = kit.Kit(model)
    hs = TOWER_SIZE / 2.0
    # Main silhouette box + open end recess + floor bands + window stripes (no colliders)
    k.box((-35.8, -hs, 0.0), (OPEN_X_START, hs, TOWER_SIZE), "limestone")  # new: DECISIONS 2026-11-26
    end_x = OPEN_X_START + 6 * FLOOR_H
    k.box((OPEN_X_START, -hs, 0.0), (end_x, hs, 1.2), "concrete_grey")
    k.box((OPEN_X_START, -hs, 1.2), (end_x, -hs + 1.2, TOWER_SIZE), "limestone")
    k.box((OPEN_X_START, hs - 1.2, 1.2), (end_x, hs, TOWER_SIZE), "limestone")
    k.box((OPEN_X_START, -hs + 1.2, TOWER_SIZE - 1.2), (end_x - 4.0, hs - 1.2, TOWER_SIZE), "limestone")
    k.box((OPEN_X_START - 0.2, -hs + 1.2, 1.2), (OPEN_X_START + 0.2, hs - 1.2, TOWER_SIZE - 1.2), "soot")
    for f_idx in range(NUM_FLOORS):
        fx = CLOSED_X_START + f_idx * FLOOR_H
        k.box((fx - 0.15, -hs - 0.04, 0.0), (fx + 0.15, hs + 0.04, TOWER_SIZE + 0.04), "limestone_dark")
        k.box((fx + 0.8, -hs + 1.5, TOWER_SIZE - 0.02), (fx + 2.7, hs - 1.5, TOWER_SIZE + 0.04), "glass_dark")
        k.box((fx + 0.8, hs - 0.02, 4.0), (fx + 2.7, hs + 0.04, TOWER_SIZE - 1.5), "glass_dark")
        k.box((fx + 0.8, -hs - 0.04, 4.0), (fx + 2.7, -hs + 0.02, TOWER_SIZE - 1.5), "glass_dark")


def build_all():
    ok0 = common.run_model("tw_fallen_section", "buildings", BUDGET_LOD0, build_lod0,
                           grip=(0.0, 0.0, 0.0), top_preview=True, view=(1.0, -0.8, 0.65))  # new: DECISIONS 2026-11-26
    ok1 = common.run_model("tw_fallen_section_lod1", "buildings", BUDGET_LOD1, build_lod1,
                           grip=(0.0, 0.0, 0.0), top_preview=False, view=(1.0, -0.8, 0.65))  # new: DECISIONS 2026-11-26
    return [("tw_fallen_section", ok0), ("tw_fallen_section_lod1", ok1)]
