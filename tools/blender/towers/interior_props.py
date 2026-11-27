"""Shared furniture, barricade, and clutter builders for R43 stop-off floor interiors."""
import math
import os
import random
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
BLENDER = os.path.dirname(HERE)
for _p in (BLENDER, os.path.join(BLENDER, "metro"), os.path.join(BLENDER, "traces")):
    if _p not in sys.path:
        sys.path.insert(0, _p)

import common  # noqa: E402
from graffiti_font import FONT, quad_bar  # noqa: E402
from jam_util import pbox  # noqa: E402


def _place(model, parts, x, y, z, rot_z=0.0):
    bm = common.combine(parts)
    common.transform(bm, loc=(x, y, z), rot=(0.0, 0.0, rot_z))
    model.add(bm)


def _box_col(model, label, size, x, y, z, rot_z=0.0):
    bm = common.make_box(size, color="white")
    common.transform(bm, loc=(x, y, z), rot=(0.0, 0.0, rot_z))
    model.add_collider(label, bm)


def clutter_papers(model, cx, cy, z, count=5, spread=0.8, seed=0):
    rng = random.Random(seed)
    parts = []
    for _ in range(count):
        px = cx + rng.uniform(-spread, spread)
        py = cy + rng.uniform(-spread, spread)
        col = rng.choice(("white", "bone_white"))
        s = pbox((-0.11, -0.14, 0.0), (0.11, 0.14, 0.008), col, bevel=0.001)
        common.transform(s, loc=(px, py, z + 0.02), rot=(0.0, 0.0, rng.uniform(0.0, math.tau)))
        parts.append(s)
    model.add(common.combine(parts))


def desk_station(model, label, x, y, rot_z=0.0, tipped_chair=False, seed=0):
    """1.6 x 0.8 m office desk with monitor, keyboard, papers, and chair."""
    parts = [
        pbox((-0.80, -0.40, 0.71), (0.80, 0.40, 0.76), "bark_light", bevel=0.01),
        pbox((-0.76, -0.36, 0.0), (-0.70, 0.36, 0.71), "metal", bevel=0.006),
        pbox((0.70, -0.36, 0.0), (0.76, 0.36, 0.71), "metal", bevel=0.006),
        # Monitor + stand + keyboard
        pbox((-0.24, 0.08, 0.76), (0.24, 0.14, 1.12), "gun_black", bevel=0.008),
        pbox((-0.21, 0.07, 0.79), (0.21, 0.08, 1.09), "soot", bevel=0.002),
        pbox((-0.06, 0.12, 0.76), (0.06, 0.22, 0.84), "gun_black", bevel=0.004),
        pbox((-0.18, -0.18, 0.76), (0.18, -0.04, 0.785), "gun_black", bevel=0.003),
    ]
    # Papers on desk (2 cm above surface)
    rng = random.Random(seed)
    for dx in (-0.50, 0.48):
        s = pbox((-0.10, -0.13, 0.0), (0.10, 0.13, 0.006), "white", bevel=0.001)
        common.transform(s, loc=(dx, rng.uniform(-0.1, 0.1), 0.78), rot=(0.0, 0.0, rng.uniform(-0.4, 0.4)))
        parts.append(s)
    # Office chair
    chair = [
        pbox((-0.22, -0.22, 0.44), (0.22, 0.22, 0.49), "pants_dark", bevel=0.01),
        pbox((-0.20, -0.22, 0.49), (0.20, -0.16, 0.92), "pants_dark", bevel=0.01),
        pbox((-0.03, -0.03, 0.05), (0.03, 0.03, 0.44), "metal", bevel=0.004),
        pbox((-0.22, -0.03, 0.0), (0.22, 0.03, 0.05), "gun_black", bevel=0.004),
        pbox((-0.03, -0.22, 0.0), (0.03, 0.22, 0.05), "gun_black", bevel=0.004),
    ]
    c_bm = common.combine(chair)
    if tipped_chair:
        common.transform(c_bm, loc=(0.1, -0.78, 0.24), rot=(1.35, 0.2, 0.4))
    else:
        common.transform(c_bm, loc=(0.0, -0.58, 0.0), rot=(0.0, 0.0, rng.uniform(-0.3, 0.3)))
    parts.append(c_bm)
    _place(model, parts, x, y, 0.0, rot_z)
    _box_col(model, label, (1.62, 0.85, 1.12), x, y, 0.56, rot_z)


def cubicle_wall(model, label, x0, y0, x1, y1, h=1.35, color="tarp_blue"):
    """Low office partition wall with metal frame + collider."""
    dx, dy = x1 - x0, y1 - y0
    length = math.hypot(dx, dy)
    ang = math.atan2(dy, dx)
    cx, cy = (x0 + x1) * 0.5, (y0 + y1) * 0.5
    parts = [
        pbox((-length * 0.5, -0.04, 0.04), (length * 0.5, 0.04, h - 0.04), color, bevel=0.005),
        pbox((-length * 0.5, -0.05, h - 0.04), (length * 0.5, 0.05, h), "metal", bevel=0.004),
        pbox((-length * 0.5, -0.05, 0.0), (length * 0.5, 0.05, 0.04), "metal", bevel=0.004),
    ]
    _place(model, parts, cx, cy, 0.0, ang)
    _box_col(model, label, (length, 0.12, h), cx, cy, h * 0.5, ang)


def filing_cabinet(model, label, x, y, rot_z=0.0, color="metal"):
    parts = [pbox((-0.26, -0.30, 0.0), (0.26, 0.30, 1.32), color, bevel=0.01)]
    for dz in (0.18, 0.48, 0.78, 1.08):
        parts.append(pbox((-0.22, -0.32, dz - 0.10), (0.22, -0.29, dz + 0.10), "concrete_grey", bevel=0.004))
        parts.append(pbox((-0.06, -0.34, dz - 0.015), (0.06, -0.31, dz + 0.015), "gun_black", bevel=0.002))
    _place(model, parts, x, y, 0.0, rot_z)
    _box_col(model, label, (0.54, 0.62, 1.32), x, y, 0.66, rot_z)


def copier(model, label, x, y, rot_z=0.0):
    parts = [
        pbox((-0.45, -0.35, 0.0), (0.45, 0.35, 0.92), "white", bevel=0.012),
        pbox((-0.42, -0.32, 0.92), (0.42, 0.32, 1.06), "rock_light", bevel=0.008),
        pbox((-0.30, -0.38, 0.85), (0.05, -0.33, 0.96), "gun_black", bevel=0.005),
    ]
    _place(model, parts, x, y, 0.0, rot_z)
    _box_col(model, label, (0.92, 0.72, 1.06), x, y, 0.53, rot_z)


def meeting_table(model, label, x, y, rot_z=0.0, length=3.2):
    hl = length * 0.5
    parts = [
        pbox((-hl, -0.60, 0.72), (hl, 0.60, 0.78), "bark_dark", bevel=0.012),
        pbox((-hl + 0.3, -0.40, 0.0), (-hl + 0.45, 0.40, 0.72), "metal", bevel=0.006),
        pbox((hl - 0.45, -0.40, 0.0), (hl - 0.3, 0.40, 0.72), "metal", bevel=0.006),
    ]
    for cx in (-hl + 0.55, 0.0, hl - 0.55):
        for sy in (-0.75, 0.75):
            parts.append(pbox((cx - 0.20, sy - 0.20, 0.42), (cx + 0.20, sy + 0.20, 0.47), "pants_dark", bevel=0.008))
            parts.append(pbox((cx - 0.03, sy - 0.03, 0.0), (cx + 0.03, sy + 0.03, 0.42), "metal", bevel=0.003))
    _place(model, parts, x, y, 0.0, rot_z)
    _box_col(model, label, (length, 1.30, 0.80), x, y, 0.40, rot_z)


def kitchenette(model, label, x, y, rot_z=0.0):
    """2.4 x 0.65 m counter + sink + fridge."""
    parts = [
        pbox((-1.20, -0.32, 0.0), (0.45, 0.32, 0.88), "rock_light", bevel=0.01),
        pbox((-1.22, -0.34, 0.88), (0.47, 0.34, 0.94), "concrete_grey", bevel=0.008),
        pbox((-0.60, -0.20, 0.93), (-0.10, 0.20, 0.96), "metal", bevel=0.004),
        pbox((0.50, -0.34, 0.0), (1.20, 0.34, 1.85), "white", bevel=0.015),
        pbox((0.52, -0.36, 1.24), (1.18, -0.33, 1.27), "metal", bevel=0.003),
    ]
    _place(model, parts, x, y, 0.0, rot_z)
    _box_col(model, label, (2.44, 0.72, 1.85), x, y, 0.925, rot_z)


def bed(model, label, x, y, rot_z=0.0, blanket="tarp_blue"):
    parts = [
        pbox((-0.56, -1.00, 0.14), (0.56, 1.00, 0.28), "bark_dark", bevel=0.01),
        pbox((-0.56, 0.92, 0.28), (0.56, 1.00, 0.86), "bark_dark", bevel=0.01),
        pbox((-0.52, -0.96, 0.28), (0.52, 0.92, 0.46), "white", bevel=0.02),
        pbox((-0.53, -0.97, 0.30), (0.53, 0.35, 0.48), blanket, bevel=0.015),
        pbox((-0.36, 0.55, 0.46), (0.36, 0.86, 0.56), "bone_white", bevel=0.02),
    ]
    _place(model, parts, x, y, 0.0, rot_z)
    _box_col(model, label, (1.14, 2.02, 0.86), x, y, 0.43, rot_z)


def sofa(model, label, x, y, rot_z=0.0, color="brownstone"):
    parts = [
        pbox((-1.00, -0.42, 0.10), (1.00, 0.42, 0.42), color, bevel=0.02),
        pbox((-1.00, 0.24, 0.42), (1.00, 0.42, 0.84), color, bevel=0.02),
        pbox((-1.00, -0.42, 0.42), (-0.82, 0.42, 0.64), color, bevel=0.02),
        pbox((0.82, -0.42, 0.42), (1.00, 0.42, 0.64), color, bevel=0.02),
    ]
    _place(model, parts, x, y, 0.0, rot_z)
    _box_col(model, label, (2.02, 0.86, 0.84), x, y, 0.42, rot_z)


def coffee_table(model, label, x, y, rot_z=0.0):
    parts = [
        pbox((-0.55, -0.30, 0.38), (0.55, 0.30, 0.43), "wood_crate", bevel=0.01),
        pbox((-0.50, -0.25, 0.0), (-0.44, -0.19, 0.38), "bark_dark", bevel=0.005),
        pbox((0.44, -0.25, 0.0), (0.50, -0.19, 0.38), "bark_dark", bevel=0.005),
        pbox((-0.50, 0.19, 0.0), (-0.44, 0.25, 0.38), "bark_dark", bevel=0.005),
        pbox((0.44, 0.19, 0.0), (0.50, 0.25, 0.38), "bark_dark", bevel=0.005),
    ]
    _place(model, parts, x, y, 0.0, rot_z)
    _box_col(model, label, (1.12, 0.62, 0.45), x, y, 0.225, rot_z)


def crib_and_toys(model, label, x, y, rot_z=0.0):
    parts = [
        pbox((-0.65, -0.38, 0.15), (0.65, 0.38, 0.24), "bark_light", bevel=0.01),
        pbox((-0.60, -0.34, 0.24), (0.60, 0.34, 0.34), "bone_white", bevel=0.015),
        pbox((-0.65, -0.38, 0.82), (0.65, 0.38, 0.88), "bark_light", bevel=0.008),
    ]
    for sx in (-0.60, -0.30, 0.0, 0.30, 0.60):
        parts.append(pbox((sx - 0.02, -0.38, 0.24), (sx + 0.02, -0.34, 0.82), "bark_light", bevel=0.003))
        parts.append(pbox((sx - 0.02, 0.34, 0.24), (sx + 0.02, 0.38, 0.82), "bark_light", bevel=0.003))
    # Toy blocks nearby
    for dx, dy, col in ((-0.9, 0.2, "red_paint"), (-0.8, 0.35, "water_blue"), (-0.95, 0.38, "brass")):
        parts.append(pbox((dx - 0.07, dy - 0.07, 0.0), (dx + 0.07, dy + 0.07, 0.14), col, bevel=0.008))
    _place(model, parts, x, y, 0.0, rot_z)
    _box_col(model, label, (1.32, 0.78, 0.88), x, y, 0.44, rot_z)


def bathroom_set(model, label, x, y, rot_z=0.0):
    parts = [
        pbox((-0.85, -0.35, 0.0), (0.85, 0.35, 0.44), "white", bevel=0.02),
        pbox((-0.75, -0.25, 0.40), (0.75, 0.25, 0.46), "water_lake", bevel=0.005),
        pbox((1.10, -0.22, 0.0), (1.50, 0.22, 0.42), "white", bevel=0.02),
        pbox((1.12, 0.05, 0.42), (1.48, 0.24, 0.82), "white", bevel=0.015),
    ]
    _place(model, parts, x, y, 0.0, rot_z)
    _box_col(model, label, (2.40, 0.72, 0.82), x, y, 0.41, rot_z)


def mattress_bed(model, x, y, rot_z=0.0, bag_col="cloth_olive"):
    parts = [
        pbox((-0.48, -0.95, 0.0), (0.48, 0.95, 0.14), "bone_white", bevel=0.03),
        pbox((-0.40, -0.88, 0.12), (0.40, 0.45, 0.22), bag_col, bevel=0.025),
        pbox((-0.28, 0.55, 0.14), (0.28, 0.85, 0.24), "white", bevel=0.02),
    ]
    _place(model, parts, x, y, 0.0, rot_z)


def camp_barricade(model, label, x, y, rot_z=0.0, length=2.4):
    """Overturned table + stacked boxes + sandbags blocking a corridor."""
    hl = length * 0.5
    tbl = pbox((-hl * 0.7, -0.45, 0.0), (hl * 0.7, 0.45, 0.07), "bark_dark", bevel=0.01)
    common.transform(tbl, loc=(0.0, 0.0, 0.55), rot=(1.35, 0.04, 0.0))
    parts = [
        tbl,
        pbox((-hl, -0.35, 0.0), (-hl + 0.65, 0.35, 0.60), "wood_crate", bevel=0.01),
        pbox((hl - 0.65, -0.35, 0.0), (hl, 0.35, 0.55), "canvas_khaki", bevel=0.02),
        pbox((hl - 0.55, -0.25, 0.55), (hl - 0.05, 0.25, 0.85), "wood_crate", bevel=0.01),
    ]
    _place(model, parts, x, y, 0.0, rot_z)
    _box_col(model, label, (length, 0.75, 1.05), x, y, 0.525, rot_z)


def supply_pile(model, label, x, y, rot_z=0.0):
    parts = [
        pbox((-0.65, -0.45, 0.0), (0.15, 0.35, 0.55), "wood_crate", bevel=0.01),
        pbox((-0.55, -0.35, 0.55), (0.05, 0.25, 0.95), "cloth_olive", bevel=0.01),
        pbox((0.22, -0.38, 0.0), (0.58, -0.02, 0.48), "water_blue", bevel=0.02),
        pbox((0.24, 0.04, 0.0), (0.60, 0.40, 0.48), "water_blue", bevel=0.02),
        pbox((-0.20, 0.42, 0.0), (0.16, 0.72, 0.24), "red_paint", bevel=0.01),  # camp stove
        common.make_frustum(0.11, 0.11, 0.24, 0.38, segments=8, center=(-0.02, 0.57), color="metal"),
    ]
    _place(model, parts, x, y, 0.0, rot_z)
    _box_col(model, label, (1.30, 1.20, 0.95), x, y, 0.475, rot_z)


def radio_table(model, label, x, y, rot_z=0.0):
    parts = [
        pbox((-0.75, -0.40, 0.70), (0.75, 0.40, 0.76), "wood_crate", bevel=0.01),
        pbox((-0.70, -0.35, 0.0), (-0.64, 0.35, 0.70), "bark_dark", bevel=0.005),
        pbox((0.64, -0.35, 0.0), (0.70, 0.35, 0.70), "bark_dark", bevel=0.005),
        pbox((-0.40, -0.15, 0.76), (0.10, 0.15, 1.02), "gun_black", bevel=0.01),
        pbox((0.02, 0.05, 1.02), (0.04, 0.07, 1.55), "metal", bevel=0.002),  # antenna
        pbox((0.25, -0.20, 0.78), (0.58, 0.18, 0.79), "bone_white", bevel=0.002),  # map
    ]
    _place(model, parts, x, y, 0.0, rot_z)
    _box_col(model, label, (1.52, 0.82, 1.05), x, y, 0.525, rot_z)


def body_under_sheet(model, x, y, rot_z=0.0):
    parts = [
        pbox((-0.32, -0.88, 0.0), (0.32, 0.88, 0.22), "bone_white", bevel=0.06),
        pbox((-0.18, 0.55, 0.0), (0.18, 0.86, 0.30), "bone_white", bevel=0.08),
        pbox((-0.18, -0.20, 0.21), (0.16, 0.30, 0.24), "blood_dark", bevel=0.01),
    ]
    _place(model, parts, x, y, 0.0, rot_z)


def vine_cluster(model, x, y, z, rot_z=0.0, seed=0):
    rng = random.Random(seed)
    parts = []
    for i in range(4):
        vx = (i - 1.5) * 0.22 + rng.uniform(-0.05, 0.05)
        h = rng.uniform(0.55, 1.15)
        col = rng.choice(("leaf_dark", "leaf_mid", "grime_green"))
        parts.append(pbox((vx - 0.06, -0.05, -h), (vx + 0.06, 0.05, 0.0), col, bevel=0.01))
    _place(model, parts, x, y, z, rot_z)


def help_sign(model, x, y, z, rot_z=0.0, color="red_paint"):
    """Draws 'HELP' in stroke bars facing +Y in local space (2 cm thick)."""
    font = dict(FONT)
    font["P"] = [(0, 0, 0, 1), (0, 1, 0.72, 1), (0.72, 1, 1, 0.74), (1, 0.74, 0.72, 0.5), (0.72, 0.5, 0, 0.5)]
    lw, lh, gap = 0.32, 0.48, 0.10
    total = 4 * lw + 3 * gap
    ox = -total * 0.5
    parts = []
    for ch in "HELP":
        for u0, v0, u1, v1 in font[ch]:
            parts.append(quad_bar(-(ox + u0 * lw), v0 * lh, -(ox + u1 * lw), v1 * lh, 0.0, 0.04, color))
        ox += lw + gap
    _place(model, parts, x, y, z, rot_z)
