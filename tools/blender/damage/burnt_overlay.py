"""Burnt facade overlay (`burnt_floor_overlay`) and ground ash patch (`ash_patch`).

- `burnt_floor_overlay` (buildings, <= 2000 tris): 24 m wide x 3.8 m tall sheet at Y +0.02..+0.05,
  facing +Y, origin at bottom centre on the wall (Y=0, Z=0).
- `ash_patch` (props, <= 300 tris): 4 x 4 m flat ash & soot patch at Z=0.02 with charred twigs.
"""
import os
import random
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))

import bmesh  # noqa: E402
from mathutils import Matrix, Vector  # noqa: E402

import common  # noqa: E402

BUDGET_OVERLAY = 2000  # data: budgets.json buildings (burnt_floor_overlay <= 2000)
BUDGET_ASH = 300  # new: DECISIONS 2026-11-26 (ash_patch <= 300)
FACADE_W = 24.0  # new: DECISIONS 2026-11-26 (24 m module width, scaled X by size/24)
FLOOR_H = 3.8  # new: DECISIONS 2026-11-26 (3.8 m floor module height)
BAY_W = 4.0  # new: DECISIONS 2026-11-26 (4 m window bay spacing)


def _box(model, lo, hi, color):
    size = (hi[0] - lo[0], hi[1] - lo[1], hi[2] - lo[2])
    loc = ((lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0, (lo[2] + hi[2]) / 2.0)
    model.add(common.make_box(size, loc=loc, color=color))


def build_overlay(model):
    """Soot sheet, dripping streaks, window char blotches, and rust runs at Y +0.02..+0.05."""
    rng = random.Random(4011)  # new: DECISIONS 2026-11-26
    hs = FACADE_W / 2.0

    # Base dark soot & stain horizontal bands across the facade (Y +0.020..+0.028)
    _box(model, (-hs, 0.020, 0.0), (hs, 0.028, 0.82), "soot")  # lower spandrel soot  # new: DECISIONS 2026-11-26
    _box(model, (-hs, 0.020, 2.75), (hs, 0.028, FLOOR_H), "soot")  # upper lintel soot  # new: DECISIONS 2026-11-26
    # Pier vertical soot strips between bays (at 4 m pier centers)
    for i in range(7):
        px = -hs + i * BAY_W
        x0 = max(-hs, px - 0.48)
        x1 = min(hs, px + 0.48)
        _box(model, (x0, 0.021, 0.80), (x1, 0.030, 2.77), "concrete_stain" if i % 2 else "soot")  # new: DECISIONS 2026-11-26

    # Window char blotches (1.2 m x 1.6 m proud char panels + soot plumes) and rust runs under sills
    for b in range(6):
        uc = -hs + (b + 0.5) * BAY_W
        # 1.2 m x 1.6 m black char blotch around/over window opening (2 cm proud -> Y 0.022..0.042)
        _box(model, (uc - 0.60, 0.022, 1.05), (uc + 0.60, 0.042, 2.65), "soot")  # new: DECISIONS 2026-11-26
        # Scorch plume rising from window top up to module top
        plume_w = rng.uniform(0.75, 1.15)  # new: DECISIONS 2026-11-26
        _box(model, (uc - plume_w, 0.029, 2.60), (uc + plume_w, 0.046, FLOOR_H), "soot")
        _box(model, (uc - plume_w * 0.55, 0.032, 2.95), (uc + plume_w * 0.55, 0.049, FLOOR_H), "concrete_stain")

        # Dripping soot & stain streaks
        for s in range(3):
            sx = uc + rng.uniform(-1.35, 1.35)
            sw = rng.uniform(0.08, 0.22)  # new: DECISIONS 2026-11-26
            sz0 = rng.uniform(0.05, 0.55)
            sz1 = rng.uniform(1.4, 3.65)
            col = "soot" if s != 1 else "concrete_stain"
            _box(model, (sx - sw, 0.030, sz0), (sx + sw, 0.048, sz1), col)

        # Rust runs under sills
        for r in range(2):
            rx = uc + (-0.55 if r == 0 else 0.55) + rng.uniform(-0.2, 0.2)
            _box(model, (rx - 0.06, 0.033, 0.08), (rx + 0.06, 0.050, 0.88), "rust_dark")  # new: DECISIONS 2026-11-26


def _ash_poly(radius_x, radius_y, z0, z1, rot_z, color, seed):
    """Flat irregular 8-gon ash patch (never a sharp rotated square)."""
    import math
    rng = random.Random(seed)
    segs = 8  # new: DECISIONS 2026-11-26
    bm = common.new_bm()
    bot = []
    top = []
    for i in range(segs):
        a = 2.0 * math.pi * i / segs + rot_z
        r = rng.uniform(0.74, 1.04)  # new: DECISIONS 2026-11-26
        x = math.cos(a) * radius_x * r
        y = math.sin(a) * radius_y * r
        bot.append(bm.verts.new((x, y, z0)))
        top.append(bm.verts.new((x * 0.96, y * 0.96, z1)))
    for i in range(segs):
        j = (i + 1) % segs
        bm.faces.new([bot[i], bot[j], top[j], top[i]])
    bm.faces.new(list(reversed(bot)))
    bm.faces.new(top)
    bmesh.ops.recalc_face_normals(bm, faces=list(bm.faces))
    common.paint(bm, color)
    return bm


def build_ash(model):
    """Flat 4 x 4 m ash and soot patch at Z=0.02 with charred twigs (no colliders)."""
    rng = random.Random(4012)  # new: DECISIONS 2026-11-26
    parts = []

    # Flat layered irregular ash patches strictly >= Z=0.02 so zero z-fighting with ground Z=0
    specs = [
        (1.92, 1.75, 0.020, 0.032, 0.15, (0.0, 0.0), "soot", 1),
        (1.55, 1.35, 0.024, 0.038, -0.45, (0.25, -0.20), "concrete_stain", 2),
        (1.25, 1.45, 0.028, 0.044, 0.65, (-0.30, 0.25), "soot", 3),
        (0.85, 0.72, 0.032, 0.050, 0.25, (0.10, 0.05), "concrete_stain", 4),
    ]  # new: DECISIONS 2026-11-26
    for rx, ry, z0, z1, rz, (ox, oy), col, s in specs:
        bm = _ash_poly(rx, ry, z0, z1, rz, col, 4020 + s)
        common.transform(bm, loc=(ox, oy, 0.0))
        parts.append(bm)

    # Charred twigs (`bark_dark`) lying across the ash
    for i in range(9):
        tx = rng.uniform(-1.35, 1.35)
        ty = rng.uniform(-1.35, 1.35)
        tlen = rng.uniform(0.32, 0.72)  # new: DECISIONS 2026-11-26
        twig = common.make_box((tlen, 0.045, 0.038), color="bark_dark")  # new: DECISIONS 2026-11-26
        common.transform(twig, rot=(rng.uniform(-0.08, 0.08), rng.uniform(-0.10, 0.10), rng.uniform(0, 3.14)),
                         loc=(tx, ty, 0.062))
        parts.append(twig)

    model.add(common.combine(parts))


def build_all():
    ok_ov = common.run_model("burnt_floor_overlay", "buildings", BUDGET_OVERLAY, build_overlay,
                             grip=(0.0, 0.0, 0.0), view=(0.6, 1.0, 0.35))  # new: DECISIONS 2026-11-26
    ok_ash = common.run_model("ash_patch", "props", BUDGET_ASH, build_ash,
                              grip=(0.0, 0.0, 0.0), view=(1.0, -1.0, 0.9))  # new: DECISIONS 2026-11-26
    return [("burnt_floor_overlay", ok_ov), ("ash_patch", ok_ash)]
