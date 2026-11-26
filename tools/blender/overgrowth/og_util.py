"""Shared geometry helpers for R40 overgrowth models."""
import math
import os
import sys

import bmesh
from mathutils import Vector

_HERE = os.path.dirname(os.path.abspath(__file__))
_BLENDER_DIR = os.path.abspath(os.path.join(_HERE, ".."))
if _BLENDER_DIR not in sys.path:
    sys.path.insert(0, _BLENDER_DIR)

import common  # noqa: E402

# data: budgets.json nature
NATURE_MAX_BUDGET = 15000

# new: DECISIONS 2026-11-26 (Z lift for ground decals/mats to prevent z-fighting)
GROUND_Z_LIFT = 0.015


def make_leaf_diamond(width, length, thickness, color="leaf_mid"):
    """Manifold 6-vertex, 8-triangle diamond leaf centered at origin, lying in XZ."""
    bm = common.new_bm()
    hw = width * 0.5  # new: DECISIONS 2026-11-26
    hl = length * 0.5  # new: DECISIONS 2026-11-26
    ht = thickness * 0.5  # new: DECISIONS 2026-11-26

    # Kite-diamond proportions: widest slightly below middle
    v_top = bm.verts.new((0.0, 0.0, hl))
    v_bot = bm.verts.new((0.0, 0.0, -hl))
    v_left = bm.verts.new((-hw, 0.0, -hl * 0.15))  # new: DECISIONS 2026-11-26
    v_right = bm.verts.new((hw, 0.0, -hl * 0.15))  # new: DECISIONS 2026-11-26
    v_front = bm.verts.new((0.0, ht, 0.0))
    v_back = bm.verts.new((0.0, -ht, 0.0))

    faces = [
        (v_top, v_right, v_front),
        (v_right, v_bot, v_front),
        (v_bot, v_left, v_front),
        (v_left, v_top, v_front),
        (v_right, v_top, v_back),
        (v_bot, v_right, v_back),
        (v_left, v_bot, v_back),
        (v_top, v_left, v_back),
    ]
    for verts in faces:
        bm.faces.new(verts)
    bmesh.ops.recalc_face_normals(bm, faces=list(bm.faces))
    common.paint(bm, color)
    return bm


def make_twig(start, end, r0, r1, color="bark_dark", segs=5):
    """Tapered manifold branch/stem from start to end."""
    s = Vector(start)
    e = Vector(end)
    d = e - s
    if d.length < 1e-4:  # new: DECISIONS 2026-11-26
        d = Vector((0.0, 0.0, 0.01))  # new: DECISIONS 2026-11-26
    bm = common.make_frustum(r0, r1, 0.0, d.length, segments=segs, color=color)
    rot = Vector((0.0, 0.0, 1.0)).rotation_difference(d.normalized()).to_matrix()
    bmesh.ops.rotate(bm, verts=list(bm.verts), cent=(0.0, 0.0, 0.0), matrix=rot)
    bmesh.ops.translate(bm, verts=list(bm.verts), vec=s)
    return bm


def make_blade(x, y, z, height, lean, yaw, color="grass", width=0.035):
    """Leaning 3-sided grass blade anchored at (x, y, z)."""
    bm = common.make_frustum(width, 0.0, 0.0, height, segments=3, color=color)
    common.transform(bm, rot=(lean, 0.0, yaw), loc=(x, y, z))
    return bm


def make_paper_scrap(w, length, thickness, bend_z, color="white"):
    """Subdivided bent thin quad prism for crumpled paper scraps (manifold)."""
    bm = common.new_bm()
    hw = w * 0.5  # new: DECISIONS 2026-11-26
    hl = length * 0.5  # new: DECISIONS 2026-11-26
    ht = thickness * 0.5  # new: DECISIONS 2026-11-26

    # 3x2 grid top and bottom so the scrap folds slightly off the ground
    cols = (-hw, 0.0, hw)
    rows = (-hl, hl)
    top = []
    bot = []
    for r in rows:
        t_row = []
        b_row = []
        for idx, c in enumerate(cols):
            dz = bend_z if idx != 1 else 0.0
            if r > 0 and idx == 2:
                dz *= 1.4  # new: DECISIONS 2026-11-26
            t_row.append(bm.verts.new((c, r, ht + dz)))
            b_row.append(bm.verts.new((c, r, -ht + dz)))
        top.append(t_row)
        bot.append(b_row)

    for i in range(2):
        bm.faces.new((top[0][i], top[0][i + 1], top[1][i + 1], top[1][i]))
        bm.faces.new((bot[1][i], bot[1][i + 1], bot[0][i + 1], bot[0][i]))
        bm.faces.new((bot[0][i], bot[0][i + 1], top[0][i + 1], top[0][i]))
        bm.faces.new((top[1][i], top[1][i + 1], bot[1][i + 1], bot[1][i]))
    bm.faces.new((top[0][0], top[1][0], bot[1][0], bot[0][0]))
    bm.faces.new((bot[0][2], bot[1][2], top[1][2], top[0][2]))
    bmesh.ops.recalc_face_normals(bm, faces=list(bm.faces))
    common.paint(bm, color)
    return bm


def add_pavement_rim(model, radius, pieces, seed, z_base=0.0):
    """Broken dark asphalt rim slabs around a dirt hole."""
    import random
    rng = random.Random(seed)
    for i in range(pieces):
        ang = 2.0 * math.pi * i / pieces + rng.uniform(-0.28, 0.28)  # new: DECISIONS 2026-11-26
        dist = radius * rng.uniform(0.78, 0.98)  # new: DECISIONS 2026-11-26
        sx = rng.uniform(0.22, 0.34)  # new: DECISIONS 2026-11-26
        sy = rng.uniform(0.18, 0.28)  # new: DECISIONS 2026-11-26
        sz = rng.uniform(0.05, 0.08)  # new: DECISIONS 2026-11-26
        tilt = rng.uniform(0.08, 0.20)  # new: DECISIONS 2026-11-26
        slab = common.make_box((sx, sy, sz), color="rock_dark")
        common.transform(
            slab,
            rot=(tilt * math.sin(ang), -tilt * math.cos(ang), ang + rng.uniform(-0.4, 0.4)),
            loc=(math.cos(ang) * dist, math.sin(ang) * dist, z_base + sz * 0.42),
        )
        model.add(slab)
