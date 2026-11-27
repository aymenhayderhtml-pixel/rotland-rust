"""R42-06 army line vehicles: `army_tank`, `army_truck_burnt`, `heli_crashed`, `heli_rotor_blade`
(+ `_lod1` for the three vehicles). Build coordinates: metres, +X right, +Y forward, +Z up.

Everything here reads as "the army tried to hold the street here and lost": the tank stands burned-out
with its hatch open and a thrown track, the truck is a bare-hooped black shell on its rims, and the
helicopter lies nose-down in the road with a broken rotor. No insignia, no stars: plain stencil numbers.
"""
import math
import os
import random
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, os.path.join(HERE, "..", "metro"))
sys.path.insert(0, os.path.join(HERE, "..", "vehicles"))

import bmesh  # noqa: E402
from mathutils import Vector  # noqa: E402

import common  # noqa: E402
import jam_truck  # noqa: E402  (the R38 army truck this burnt shell mirrors)
from army_line import (DARK, KHAKI, METAL, OLIVE, box, pbox, pbox_at, pivot,  # noqa: E402
                       sandbag, stencil_on_side, stencil_text)
from car_parts import _beam  # noqa: E402
from jam_util import ground_rim, rim_bm, tyre_bm  # noqa: E402

# new: DECISIONS 2026-11-26 (R42 budgets: vehicles 12000 LOD0, _lod1 1200; heli_rotor_blade 4000)
VEHICLE_BUDGET = 12000
VEHICLE_LOD1_BUDGET = 1200
BLADE_BUDGET = 4000

RUST = "rust"
SOOT = "soot"
SOIL = "soil_dark"
BURNT = "gun_black"


# --- shared geometry helpers -------------------------------------------------------------------
def loft(rings, color, cap=True):
    """Closed loft between rings of the same point count (list of (x, y, z) points each)."""
    bm = common.new_bm()
    ring_v = [[bm.verts.new(p) for p in ring] for ring in rings]
    n = len(rings[0])
    for a, b in zip(ring_v, ring_v[1:]):
        for i in range(n):
            j = (i + 1) % n
            bm.faces.new([a[i], a[j], b[j], b[i]])
    if cap:
        bm.faces.new(list(reversed(ring_v[0])))
        bm.faces.new(ring_v[-1])
    common.paint(bm, color)
    bmesh.ops.recalc_face_normals(bm, faces=list(bm.faces))
    return bm


def ring_xz(y, half_w, z_lo, z_hi, cut=0.2):
    """8-point rounded-rectangle ring in the XZ plane at depth `y` (bottom corners chamfered)."""
    w, lo, hi, c = half_w, z_lo, z_hi, min(cut, half_w * 0.4, (z_hi - z_lo) * 0.3)
    return [(w, y, lo + c), (w, y, hi - c), (w - c, y, hi), (-w + c, y, hi),
            (-w, y, hi - c), (-w, y, lo + c), (-w + c, y, lo), (w - c, y, lo)]


def ring_xy(z, half_x, half_y, colour_cut=0.3):
    """8-point turret ring in the XY plane at height `z`, pointed at the front (+Y)."""
    cx, cy, k = half_x, half_y, colour_cut
    return [(cx, cy * 0.35, z), (cx * 0.62, cy - k, z), (-cx * 0.62, cy - k, z), (-cx, cy * 0.35, z),
            (-cx, -cy * 0.55, z), (-cx * 0.55, -cy, z), (cx * 0.55, -cy, z), (cx, -cy * 0.55, z)]


def grime(parts, size, count, seed, colour=RUST, z=(0.2, 1.2), x=(0.0, 1.0), y=(0.0, 1.0), thick=0.02):
    """Scatter thin patches (rust / soot / mud) over the given ranges on both flanks."""
    rng = random.Random(seed)
    for _ in range(count):
        sx = rng.uniform(*x)
        for side in (-1, 1):
            px = side * sx
            py = rng.uniform(*y)
            pz = rng.uniform(*z)
            parts.append(pbox((px, py, pz), (px + side * thick, py + size[0], pz + size[1]),
                              colour if rng.random() < 0.7 else SOOT, bevel=0.0))


# --- army_tank: wheels, tracks, hull ------------------------------------------------------------
HULL_Y0, HULL_Y1 = -4.35, 3.25  # hull runs 7.6 m; the gun adds the rest of the 9.5 m length
HULL_HALF_W = 1.12
TRACK_X0, TRACK_X1 = 1.14, 1.80  # track band outer edge -> total width 3.6 m
TRACK_Z0, TRACK_Z1 = 0.02, 0.92
TRACK_Y0, TRACK_Y1 = -3.85, 2.85


def _road_wheels(side, missing=False):
    out = []
    x = side * (HULL_HALF_W + 0.36)
    for k in range(6):  # 6 road wheels per side
        y = -3.0 + k * 1.14
        if missing and k in (3, 4):  # the thrown side has two wheels stripped
            continue
        out.append(common.make_frustum(0.355, 0.355, -0.17, 0.17, segments=12, color="tire_black"))
        common.transform(out[-1], rot=(0.0, math.pi / 2.0, 0.0), loc=(x, y, 0.38))
        rim = common.make_frustum(0.19, 0.19, -0.2, 0.2, segments=10, color="rock_dark")
        common.transform(rim, rot=(0.0, math.pi / 2.0, 0.0), loc=(x, y, 0.38))
        out.append(rim)
        for a in (0.0, math.pi / 3.0, 2 * math.pi / 3.0):  # lightening holes / bolts
            hole = common.make_frustum(0.055, 0.055, -0.21, 0.21, segments=6, color=DARK)
            common.transform(hole, rot=(0.0, math.pi / 2.0, 0.0),
                             loc=(x, y + math.cos(a) * 0.11, 0.38 + math.sin(a) * 0.11))
            out.append(hole)
    # drive sprocket (rear) and idler (front), slightly larger and toothed
    for y, seg in ((-3.78, 14), (2.8, 14)):
        sp = common.make_frustum(0.44, 0.44, -0.14, 0.14, segments=seg, color=METAL)
        common.transform(sp, rot=(0.0, math.pi / 2.0, 0.0), loc=(x, y, 0.47))
        out.append(sp)
        hub = common.make_frustum(0.2, 0.2, -0.19, 0.19, segments=8, color=DARK)
        common.transform(hub, rot=(0.0, math.pi / 2.0, 0.0), loc=(x, y, 0.47))
        out.append(hub)
    # suspension arms to the hull
    for k in range(6):
        y = -3.0 + k * 1.14
        out.append(_beam((side * HULL_HALF_W, y, 0.62), (x, y, 0.38), 0.055, color=METAL, segments=5))
    return out


def _track_links(side, pitch=0.27, only_arcs=None, colour=BURNT):
    """Track of separate links swept round the road-wheel run; `only_arcs` keeps a bent loose piece."""
    x = side * (HULL_HALF_W + 0.33)
    r = (TRACK_Z1 - TRACK_Z0) / 2
    zc = (TRACK_Z0 + TRACK_Z1) / 2
    path = []
    span_a, span_b = TRACK_Y0 + r, TRACK_Y1 - r

    def add(y, z, ang):
        path.append((y, z, ang))

    steps = int((span_b - span_a) / pitch)
    for i in range(steps + 1):  # bottom run
        add(span_a + (span_b - span_a) * i / steps, TRACK_Z0, 0.0)
    for i in range(1, 8):  # front arc
        a = -math.pi / 2 + math.pi * i / 8.0
        add(span_b + math.cos(a) * r, zc + math.sin(a) * r, a + math.pi / 2)
    for i in range(1, steps + 1):  # top run back
        add(span_b - (span_b - span_a) * i / steps, TRACK_Z1, math.pi)
    for i in range(1, 8):  # rear arc
        a = math.pi / 2 + math.pi * i / 8.0
        add(span_a + math.cos(a) * r, zc + math.sin(a) * r, a + math.pi / 2)
    out = []
    for y, z, ang in path:
        link = pbox((-0.24, -pitch * 0.55, -0.055), (0.24, pitch * 0.55, 0.055), colour, bevel=0.0)
        common.transform(link, rot=(ang, 0.0, 0.0), loc=(x, y, z))
        out.append(link)
        pad = box((-0.1, -pitch * 0.3, 0.055), (0.1, pitch * 0.3, 0.11), "rock_dark")
        common.transform(pad, rot=(ang, 0.0, 0.0), loc=(x, y, z))
        out.append(pad)
    return out


def _thrown_track(seed=7701):
    """The right-hand track, off the wheels and lying in a loose S on the road beside the tank."""
    rng = random.Random(seed)
    out = []
    x0 = TRACK_X1 + 0.75
    for i in range(22):
        t = i / 21.0
        y = -3.0 + t * 5.6
        x = x0 + math.sin(t * 3.4) * 0.34 + (0.28 if t > 0.72 else 0.0)
        ang = math.cos(t * 3.4) * 0.55
        link = pbox((-0.24, -0.14, -0.055), (0.24, 0.14, 0.055), BURNT, bevel=0.0)
        common.transform(link, rot=(0.0, 0.0, ang), loc=(x, y, 0.07 + 0.12 * (1.0 - t) * (t > 0.8)))
        out.append(link)
    for k in range(4):  # a few loose links and a broken pin
        x = x0 + rng.uniform(-0.9, 0.9)
        y = rng.uniform(-4.4, 3.4)
        link = pbox((-0.22, -0.13, -0.05), (0.22, 0.13, 0.05), BURNT, bevel=0.0)
        common.transform(link, rot=(0.0, rng.uniform(-0.1, 0.1), rng.uniform(0, 3.1)), loc=(x, y, 0.06))
        out.append(link)
    out.append(_beam((x0 - 0.2, -3.4, 0.05), (x0 + 0.5, -3.2, 0.05), 0.03, color=METAL, segments=5))
    return out


def _tank_hull(lod=False):
    parts = []
    # belly and lower hull
    parts.append(loft([ring_xz(HULL_Y0, HULL_HALF_W, 0.42, 1.52, 0.26),
                       ring_xz(HULL_Y0 + 0.5, HULL_HALF_W, 0.3, 1.56, 0.26),
                       ring_xz(HULL_Y1 - 0.9, HULL_HALF_W, 0.3, 1.56, 0.26),
                       ring_xz(HULL_Y1, HULL_HALF_W - 0.02, 0.72, 1.5, 0.26)], OLIVE))
    if lod:
        for side in (-1, 1):
            parts.append(box((side * TRACK_X0, TRACK_Y0, 0.0), (side * TRACK_X1, TRACK_Y1, 0.92), BURNT))
            parts.append(box((side * (HULL_HALF_W - 0.02), -1.6, 1.5), (side * 1.55, 1.3, 2.25), OLIVE))
            parts.append(_beam((0.0, 1.7, 1.9), (0.0, 5.2, 1.9), 0.1, color=DARK, segments=6))
        return parts
    # upper hull deck with bevels, engine deck at the rear
    parts.append(loft([ring_xz(HULL_Y0 + 0.15, HULL_HALF_W - 0.04, 1.5, 1.62, 0.16),
                       ring_xz(-1.3, HULL_HALF_W - 0.04, 1.5, 1.7, 0.16),
                       ring_xz(1.0, HULL_HALF_W - 0.06, 1.5, 1.64, 0.16),
                       ring_xz(HULL_Y1 - 0.05, HULL_HALF_W - 0.1, 1.42, 1.55, 0.16)], OLIVE))
    # engine deck grilles (two banks of slats) and the rear plate with exhausts
    for side in (-1, 1):
        for k in range(7):
            x = side * (0.28 + k * 0.13)
            parts.append(box((x - 0.035, HULL_Y0 + 0.35, 1.68), (x + 0.035, -1.9, 1.74), DARK))
    parts.append(pbox((-HULL_HALF_W + 0.1, HULL_Y0 + 0.2, 1.6), (HULL_HALF_W - 0.1, HULL_Y0 + 0.65, 1.76),
                      METAL, bevel=0.02))
    for side in (-1, 1):
        parts.append(pbox((side * (HULL_HALF_W - 0.3) - 0.14, HULL_Y0 - 0.02, 1.0),
                          (side * (HULL_HALF_W - 0.3) + 0.14, HULL_Y0 + 0.04, 1.3), SOOT, bevel=0.01))
        parts.append(_beam((side * 0.7, HULL_Y0 + 0.02, 1.5), (side * 0.7, HULL_Y0 - 0.1, 1.6), 0.07,
                           color="rust_dark", segments=6))
    # sloped glacis over the front of the hull: a big angled upper plate with side cheeks
    parts.append(pbox_at((-HULL_HALF_W - 0.02, 1.85, 0.0), (HULL_HALF_W + 0.02, 3.3, 0.12), OLIVE,
                         rot=(0.28, 0.0, 0.0), loc=(0.0, 0.0, 1.36), bevel=0.02))
    parts.append(pbox((-0.34, 2.9, 1.0), (0.34, 3.32, 1.32), METAL, bevel=0.02))  # lower nose plate
    for side in (-1, 1):
        parts.append(pbox_at((side * 1.0 - 0.16, 2.2, 0.9), (side * 1.0 + 0.16, 3.1, 1.36), OLIVE,
                             rot=(0.0, side * 0.2, 0.0), bevel=0.02))
    # driver's hatch, periscopes and the front tow lugs
    parts.append(common.make_frustum(0.34, 0.34, 0.0, 0.07, segments=10, center=(-0.42, 1.55), color=OLIVE))
    parts.append(common.make_frustum(0.29, 0.29, 0.07, 0.1, segments=10, center=(-0.42, 1.55), color=DARK))
    for k in range(3):
        parts.append(box((-0.62 + k * 0.2, 1.86, 1.56), (-0.52 + k * 0.2, 1.94, 1.64), "glass_dark"))
    parts.append(pbox((0.15, 1.3, 1.55), (0.75, 1.8, 1.66), METAL, bevel=0.02))  # closed loader hatch
    parts.append(common.make_frustum(0.24, 0.24, 0.0, 0.06, segments=10, center=(0.45, 1.55), color=OLIVE))
    for side in (-1, 1):
        parts.append(_beam((side * 0.55, HULL_Y1 - 0.02, 1.2), (side * 0.55, HULL_Y1 + 0.18, 1.26), 0.06,
                           color=METAL, segments=5))
        # headlights with brush guards
        parts.append(pbox((side * 0.78 - 0.11, HULL_Y1 - 0.02, 1.25), (side * 0.78 + 0.11, HULL_Y1 + 0.06, 1.45),
                          "white", bevel=0.01))
        for a in (-1, 0, 1):
            parts.append(_beam((side * 0.78 + a * 0.1, HULL_Y1 + 0.02, 1.2), (side * 0.78 + a * 0.1,
                                                                           HULL_Y1 + 0.1, 1.5), 0.012,
                               color=DARK, segments=4))
    # side skirts over the tracks (left full, right partly torn off) with a rubber flap line
    for side in (-1, 1):
        if side > 0:  # right side skirt is half gone: two short pieces only
            spans = [(1.2, 2.9), (-0.4, 0.7)]
        else:
            spans = [(-3.9, 2.95)]
        for y0, y1 in spans:
            parts.append(pbox((side * TRACK_X0 - 0.02, y0, 0.95), (side * (TRACK_X1 - 0.06), y1, 1.44), OLIVE,
                              bevel=0.02))
            parts.append(box((side * (TRACK_X1 - 0.1), y0, 0.95), (side * (TRACK_X1 - 0.06), y1, 1.02), DARK))
        for k in range(6):  # skirt hanger blocks
            y = -3.4 + k * 1.2
            parts.append(box((side * (HULL_HALF_W - 0.02), y, 1.24), (side * TRACK_X0, y + 0.16, 1.4), METAL))
    # spare track links and a stowed tow cable on the hull sides
    for k in range(4):
        parts.append(pbox((-TRACK_X0, -2.4 + k * 0.3, 1.46), (-TRACK_X0 + 0.16, -2.2 + k * 0.3, 1.62), BURNT,
                          bevel=0.0))
    parts.append(_beam((TRACK_X0 - 0.06, -1.4, 1.5), (TRACK_X0 - 0.06, 1.6, 1.5), 0.035, color=METAL, segments=5))
    # tools along the left deck: a shovel, a sledgehammer and two stowage bins
    parts.append(_beam((-0.9, 0.2, 1.62), (-0.9, 1.5, 1.62), 0.026, color="bark_light", segments=4))
    parts.append(pbox((-1.0, 1.5, 1.6), (-0.8, 1.72, 1.66), METAL, bevel=0.01))
    parts.append(pbox((0.55, -0.5, 1.64), (1.0, 0.25, 1.82), OLIVE, bevel=0.02))  # bin
    parts.append(pbox((-1.0, -0.9, 1.64), (-0.55, -0.2, 1.82), OLIVE, bevel=0.02))  # bin
    # mud / soot on the lower hull
    grime(parts, (0.34, 0.22), 9, 5101, colour=SOIL, z=(0.4, 0.95), x=(HULL_HALF_W - 0.02, HULL_HALF_W))
    grime(parts, (0.3, 0.34), 6, 5102, colour=SOOT, z=(0.9, 1.45), x=(HULL_HALF_W - 0.02, HULL_HALF_W))
    return parts


def _tank_turret(lod=False):
    parts = []
    if lod:
        return parts
    # turret body: 3 stacked octagon rings, wider and more pointed at the front
    parts.append(loft([ring_xy(1.56, 1.46, 1.88),
                       ring_xy(1.9, 1.5, 1.78),
                       ring_xy(2.22, 1.28, 1.44),
                       ring_xy(2.34, 0.95, 1.05)], OLIVE))
    # turret side stowage baskets (frame + bags) over the rear quarters
    for side in (-1, 1):
        fy = -0.5
        parts.append(pbox((side * 1.3, fy, 1.72), (side * 1.56, fy + 1.5, 1.78), METAL, bevel=0.0))
        parts.append(pbox((side * 1.3, fy, 2.02), (side * 1.56, fy + 1.5, 2.08), METAL, bevel=0.0))
        for k in range(4):
            y = fy + 0.12 + k * 0.44
            parts.append(box((side * 1.3, y, 1.72), (side * 1.56, y + 0.05, 2.08), METAL))
        for k in range(2):
            y = fy + 0.35 + k * 0.66
            bag = common.make_icosphere(subdiv=1, jitter=0.12, seed=60 + k + (3 if side > 0 else 0),
                                        radius=(0.13, 0.28, 0.16), loc=(side * 1.44, y, 1.9), color=KHAKI)
            parts.append(bag)
        parts.append(pbox((side * 1.32, fy + 0.15, 1.62), (side * 1.54, fy + 0.75, 1.72), OLIVE, bevel=0.01))
    # rear turret bustle rack with a rolled tarp and two ammo boxes
    parts.append(pbox((-1.1, -2.1, 1.72), (1.1, -1.7, 1.78), METAL, bevel=0.0))
    parts.append(pbox((-1.1, -2.1, 2.0), (1.1, -1.7, 2.06), METAL, bevel=0.0))
    for x in (-1.05, -0.35, 0.35, 1.05):
        parts.append(box((x - 0.03, -2.1, 1.72), (x + 0.03, -1.7, 2.06), METAL))
    roll = common.make_frustum(0.13, 0.13, -1.0, 1.0, segments=8, color=KHAKI)
    common.transform(roll, rot=(0.0, math.pi / 2.0, 0.0), loc=(0.0, -1.9, 1.96))
    parts.append(roll)
    for x in (-0.62, 0.62):
        parts.append(pbox((x - 0.26, -2.08, 1.78), (x + 0.26, -1.72, 1.96), OLIVE, bevel=0.02))
    # gun mantlet and the long gun with a thermal sleeve and a muzzle brake
    parts.append(loft([ring_xz(1.25, 0.72, 1.62, 2.2, 0.18), ring_xz(1.95, 0.5, 1.66, 2.12, 0.16)], OLIVE))
    parts.append(_beam((0.0, 1.9, 1.9), (0.0, 5.05, 1.9), 0.095, color=DARK, segments=10))
    for k in range(3):  # thermal sleeve segments
        y = 2.35 + k * 0.72
        parts.append(_beam((0.0, y, 1.9), (0.0, y + 0.5, 1.9), 0.145, color=OLIVE, segments=10))
    parts.append(_beam((0.0, 4.95, 1.9), (0.0, 5.3, 1.9), 0.15, color=DARK, segments=8))  # muzzle brake
    parts.append(box((-0.06, 5.28, 1.84), (0.06, 5.34, 1.96), BURNT))
    parts.append(_beam((0.0, 2.1, 2.16), (0.0, 2.9, 2.16), 0.035, color=METAL, segments=5))  # recoil cylinder
    # commander's cupola with the hatch OPEN: collar, dark interior, lid tilted up on its hinge
    cup_y, cup_x = -0.35, -0.4
    parts.append(common.make_frustum(0.42, 0.42, 0.0, 0.16, segments=12, center=(cup_x, cup_y), color=OLIVE))
    common.transform(parts[-1], loc=(0.0, 0.0, 2.3))
    parts.append(common.make_frustum(0.36, 0.36, 0.0, 0.04, segments=12, center=(cup_x, cup_y), color=SOOT))
    common.transform(parts[-1], loc=(0.0, 0.0, 2.3))
    lid = common.make_frustum(0.42, 0.4, 0.0, 0.06, segments=12, center=(0.0, 0.0), color=OLIVE)
    common.transform(lid, rot=(0.0, -0.95, 0.0), loc=(cup_x - 0.36, cup_y, 2.52))
    parts.append(lid)
    parts.append(_beam((cup_x + 0.3, cup_y, 2.44), (cup_x - 0.44, cup_y, 2.56), 0.02, color=METAL, segments=4))
    # roof machine gun on the hatch ring (pintle, receiver, barrel, ammo box)
    gx, gy = cup_x + 0.42, cup_y - 0.1
    parts.append(_beam((gx, gy, 2.42), (gx, gy, 2.62), 0.045, color=METAL, segments=6))
    parts.append(pbox((gx - 0.07, gy - 0.28, 2.6), (gx + 0.07, gy + 0.22, 2.74), DARK, bevel=0.015))
    parts.append(_beam((gx, gy + 0.22, 2.68), (gx, gy + 0.82, 2.68), 0.028, color=DARK, segments=6))
    parts.append(pbox((gx - 0.1, gy - 0.5, 2.58), (gx + 0.1, gy - 0.3, 2.72), OLIVE, bevel=0.01))
    parts.append(box((gx - 0.04, gy - 0.16, 2.74), (gx + 0.04, gy + 0.1, 2.78), METAL))
    # gunner's sight box and the two smoke-launcher clusters
    parts.append(pbox((0.62, 0.95, 2.2), (0.92, 1.35, 2.42), DARK, bevel=0.02))
    for side in (-1, 1):
        for k in range(4):
            tube = common.make_frustum(0.055, 0.055, 0.0, 0.3, segments=6, color=DARK)
            common.transform(tube, rot=(-0.45, 0.0, side * 0.35),
                             loc=(side * 1.16, 0.55 + k * 0.0, 2.02 + k * 0.09))
            parts.append(tube)
        parts.append(pbox((side * 1.24 - 0.09, 0.42, 1.92), (side * 1.24 + 0.09, 0.72, 2.02), METAL, bevel=0.0))
    # radio antennae (one bent) and the loader's hatch (closed)
    parts.append(_beam((-0.95, -1.6, 2.2), (-1.02, -1.72, 3.5), 0.018, color=DARK, segments=4))
    parts.append(_beam((-1.02, -1.72, 3.5), (-1.05, -2.6, 3.66), 0.016, color=DARK, segments=4))
    parts.append(_beam((0.95, -1.6, 2.2), (0.98, -1.66, 3.2), 0.016, color=DARK, segments=4))
    parts.append(common.make_frustum(0.3, 0.3, 0.0, 0.06, segments=10, center=(0.62, -0.9), color=OLIVE))
    common.transform(parts[-1], loc=(0.0, 0.0, 2.34))
    # stencilled unit numbers, plain digits only
    for side in (-1, 1):
        parts.append(stencil_on_side("07", 0.17, 0.24, side * 1.47 + side * 0.006, 1.85, -0.3, "soot",
                                     outward=side))
    parts.append(stencil_on_side("07", 0.22, 0.32, -HULL_HALF_W - 0.006, 0.95, 1.6, "soot", outward=-1))
    # soot bleeding up from the engine deck onto the turret rear
    grime(parts, (0.5, 0.3), 4, 5103, colour=SOOT, z=(1.6, 2.1), x=(0.9, 1.45), y=(-1.9, -0.4))
    return parts


def _build_army_tank(model, lod=False):
    parts = _tank_hull(lod) + _tank_turret(lod)
    if not lod:
        parts += _road_wheels(-1)
        parts += _road_wheels(1, missing=True)
        parts += _track_links(-1)
        parts += _thrown_track()  # right track lies in the road
    for bm in parts:
        model.add(bm)
    if lod:
        model.add_collider("hull", box((-HULL_HALF_W - 1.0, HULL_Y0, 0.0), (TRACK_X1, HULL_Y1, 1.62), "white"))
        return
    # hull box (walkable deck, top at 1.7 m) + a rear step so it can be climbed from behind
    model.add_collider("hull", box((-TRACK_X1, HULL_Y0 - 0.05, 0.0), (TRACK_X1, HULL_Y1, 1.62), "white"))
    model.add_collider("deck", box((-HULL_HALF_W, HULL_Y0, 1.55), (HULL_HALF_W, HULL_Y1 - 0.1, 1.74), "white"))
    model.add_collider("step", box((-1.4, HULL_Y0 - 0.55, 0.0), (1.4, HULL_Y0 + 0.05, 0.62), "white"))
    model.add_collider("turret", box((-1.5, -2.1, 1.56), (1.5, 1.4, 2.36), "white"))
    model.add_collider("gun", box((-0.2, 1.9, 1.78), (0.2, 5.34, 2.02), "white"))
    model.add_collider("track_off", box((1.9, -3.2, 0.0), (3.1, 2.7, 0.2), "white"))


# --- army_truck_burnt --------------------------------------------------------------------------
def _build_army_truck_burnt(model, lod=False):
    """The R38 army truck shape, burnt out: no canvas (bare hoops), no tyres, black and rust."""
    hl, hw = jam_truck.HALF_L, jam_truck.HALF_W
    cargo_front = jam_truck.CARGO_FRONT
    top = jam_truck.CANVAS_TOP
    base = jam_truck.CANVAS_BASE
    parts = []
    if lod:
        parts.append(box((-hw, -hl, 0.3), (hw, hl, 0.9), BURNT))
        parts.append(box((-hw + 0.1, 1.5, 0.9), (hw - 0.1, hl, 2.0), BURNT))
        for side in (-1, 1):
            for y in (2.5, -1.8, -2.8):
                parts.append(box((side * (hw - 0.1) - 0.15, y - 0.4, 0.0), (side * (hw + 0.1), y + 0.4, 0.6),
                                 SOOT))
        for bm in parts:
            model.add(bm)
        model.add_collider("shell", box((-hw, -hl, 0.0), (hw, hl, 2.0), "white"))
        return
    # chassis rails, burnt floor and the frame of the cargo bed
    for x in (-0.6, 0.6):
        parts.append(pbox((x - 0.07, -hl, 0.28), (x + 0.07, hl - 0.1, 0.52), BURNT, bevel=0.01))
    parts.append(pbox((-hw + 0.2, -hl + 0.05, 0.28), (hw - 0.2, cargo_front, base - 0.06), SOOT, bevel=0.02))
    parts.append(pbox((-hw + 0.1, -hl, base - 0.06), (hw - 0.1, -hl + 0.09, base), BURNT, bevel=0.02))
    for side in (-1, 1):  # bed side rails, half fallen off on the right
        if side < 0:
            parts.append(pbox((side * hw - 0.05, -hl, base), (side * hw + 0.0, cargo_front, base + 0.42), BURNT,
                              bevel=0.01))
        else:
            parts.append(pbox((side * hw - 0.05, -hl, base), (side * hw, -0.4, base + 0.42), BURNT, bevel=0.01))
    # cab, burnt: hollowed shell with the roof caved in and the glass gone
    cab = loft([ring_xz(1.5, hw, 0.5, 0.98, 0.12), ring_xz(3.0, hw, 0.5, 0.98, 0.12)], BURNT)
    parts.append(cab)
    parts.append(loft([ring_xz(1.6, hw - 0.1, 0.98, 1.95, 0.14),
                       ring_xz(2.75, hw - 0.12, 0.98, 1.86, 0.14)], BURNT))
    parts.append(pbox((-hw + 0.1, 1.55, 1.86), (0.2, 2.8, 1.94), BURNT, bevel=0.02))  # sagging roof
    for side in (-1, 1):  # empty window openings (dark), doors rusted shut
        parts.append(pbox((side * (hw - 0.02), 1.95, 1.15), (side * hw + side * 0.01, 2.6, 1.78), SOOT, bevel=0.0))
        parts.append(pbox((side * (hw - 0.01), 1.6, 0.55), (side * hw + side * 0.02, 2.75, 1.12), BURNT, bevel=0.02))
    parts.append(pbox((-0.85, 2.96, 1.15), (0.85, 3.02, 1.78), SOOT, bevel=0.0))  # windscreen frame, glass gone
    parts.append(pbox((-hw - 0.04, hl - 0.3, 0.3), (hw + 0.04, hl + 0.02, 0.5), BURNT, bevel=0.03))
    for k in range(6):  # grille slats, black
        x = -0.6 + k * 0.24
        parts.append(box((x - 0.04, hl - 0.02, 0.55), (x + 0.04, hl + 0.02, 0.9), SOOT))
    for side in (-1, 1):  # bare mirror arms, one bent
        parts.append(_beam((side * hw, 2.9, 1.2), (side * (hw + 0.24), 2.9, 1.5 if side < 0 else 1.34), 0.025,
                           color="rust_dark", segments=4))
    # bare canvas hoops: the frame is all that is left of the tilt (arched over the bed, two bent)
    ribs = jam_truck.RIBS
    for k in range(ribs):
        y = -hl + 0.2 + k * (cargo_front + hl - 0.4) / (ribs - 1)
        bent = k in (3, 6)  # two hoops collapsed inward by the fire
        for side in (-1, 1):
            parts.append(_beam((side * (hw - 0.16), y, base),
                               (side * (hw - 0.4), y, base + (top - base) * 0.55), 0.034,
                               color="rust_dark", segments=5))
            parts.append(_beam((side * (hw - 0.4), y, base + (top - base) * 0.55),
                               (side * (0.34 if bent else 0.14), y, base + (top - base) * (0.62 if bent else 1.0)),
                               0.03, color="rust_dark", segments=5))
        parts.append(_beam((-0.14, y, base + (top - base) * 0.98),
                           (0.14, y, base + (top - base) * 0.98), 0.028, color="rust_dark", segments=5))
    for side in (-1, 1):  # the long rails the hoops hang from
        parts.append(_beam((side * (hw - 0.2), -hl, base), (side * (hw - 0.2), cargo_front, base), 0.035,
                           color=METAL, segments=5))
    # shredded canvas scraps still hanging from two hoops
    for y, z, w in ((-1.6, 1.5, 0.5), (0.4, 1.75, 0.42), (-2.6, 1.2, 0.34)):
        parts.append(pbox((-w / 2, y - 0.02, z - 0.35), (w / 2, y + 0.02, z), KHAKI, bevel=0.0))
    # no tyres: the truck sits on its bare rims
    for x, y in ((-(hw - 0.1), 2.5), (hw - 0.1, 2.5), (-(hw - 0.1), -1.8), (hw - 0.1, -1.8),
                 (-(hw - 0.1), -2.8), (hw - 0.1, -2.8)):
        parts.append(ground_rim(x, y, 0.42, 0.18, yaw=0.0))
        parts.append(common.make_frustum(0.1, 0.1, 0.0, 0.12, segments=6, center=(x, y), color=DARK))
    # fire damage: soot wash over the cab roof, rust bloom on the bed sides, ash under the shell
    rng = random.Random(7811)
    for _ in range(16):
        x = rng.uniform(-hw + 0.05, hw - 0.05)
        y = rng.uniform(-hl + 0.2, hl - 0.2)
        parts.append(pbox((x, y, 0.5 if y > cargo_front else 0.02), (x + rng.uniform(0.2, 0.5),
                                                                    y + rng.uniform(0.2, 0.5), 0.52 if
                                                                    y > cargo_front else 0.04),
                          SOOT if rng.random() < 0.5 else RUST, bevel=0.0))
    grime(parts, (0.4, 0.26), 6, 7812, colour=RUST, z=(0.6, 1.6), x=(hw - 0.02, hw))
    for _ in range(10):  # charred spars and ash on the ground
        x = rng.uniform(-hw - 0.4, hw + 0.4)
        y = rng.uniform(-hl - 0.5, 0.6)
        parts.append(box((x, y, 0.0), (x + rng.uniform(0.2, 0.5), y + rng.uniform(0.15, 0.4), 0.035), SOOT))
    parts.append(stencil_on_side("07", 0.16, 0.22, hw + 0.006, 0.62, 2.3, "white", outward=1))
    for bm in parts:
        model.add(bm)
    model.add_collider("shell", box((-hw - 0.05, -hl, 0.0), (hw + 0.05, hl, 2.0), "white"))


# --- heli_crashed ------------------------------------------------------------------------------
# Generic medium transport helicopter, ~15 m long, built upright then tipped nose-down 20 degrees.
HELI_NOSE = 6.6
HELI_TAIL = -7.3
TILT = math.radians(20.0)  # new: DECISIONS 2026-11-26 (nose crushed into the road, tilted 20 deg)
ROLL = math.radians(7.0)   # new: DECISIONS 2026-11-26 (the wreck also lies over on its left side)


def _heli_body(lod=False):
    parts = []   # airframe: tips with the wreck
    ground = []  # debris on the road: stays flat
    if lod:
        parts.append(box((-1.25, -2.8, 0.9), (1.25, 3.4, 3.1), OLIVE))   # cabin
        parts.append(box((-0.45, -7.2, 1.0), (0.45, -2.8, 2.1), OLIVE))  # boom
        parts.append(box((-0.9, -7.4, 1.1), (0.9, -6.6, 3.2), OLIVE))    # fin
        parts.append(box((-0.12, 2.6, 3.1), (0.12, 6.6, 3.3), DARK))     # rotor mast + stub blades
        parts.append(box((-1.3, -1.9, 0.6), (1.3, 3.0, 0.8), METAL))     # skids
        return parts, ground
    hw, zc = 1.26, 1.95  # cabin half width, cabin centre height
    hh = 1.18
    # Fuselage: crushed nose, cockpit, cabin, boat tail, and a boom that sags down after the break.
    parts.append(loft([
        ring_xz(HELI_NOSE, 0.44, zc - 0.44, zc + 0.02, 0.14),   # crushed nose tip, driven into the road
        ring_xz(5.5, 0.82, zc - 0.8, zc + 0.46, 0.2),
        ring_xz(3.9, 1.02, zc - 1.02, zc + 0.92, 0.22),         # cockpit shoulders
        ring_xz(2.6, hw, zc - hh, zc + hh, 0.26),
        ring_xz(-1.4, hw, zc - hh, zc + hh, 0.26),              # cabin
        ring_xz(-2.9, hw - 0.14, zc - 1.0, zc + 1.06, 0.22),
        ring_xz(-3.5, 0.56, zc - 0.44, zc + 0.5, 0.14),         # boom root
        ring_xz(-5.4, 0.44, zc - 0.66, zc + 0.24, 0.12),        # boom sags: centre drops
        ring_xz(HELI_TAIL, 0.36, zc - 1.5, zc - 0.75, 0.1),     # boom end, bent right down
    ], OLIVE))
    # Crushed nose: three folded plates and the torn chin, so the tip reads as squashed into the road.
    for k, (y, w, z, tilt) in enumerate(((HELI_NOSE - 0.15, 0.5, zc + 0.06, -0.55),
                                         (HELI_NOSE - 0.55, 0.72, zc - 0.16, -0.3),
                                         (HELI_NOSE - 0.95, 0.9, zc - 0.42, -0.12))):
        parts.append(pbox_at((-w, y - 0.34, z - 0.07), (w, y, z + 0.07), OLIVE,
                             rot=(tilt, 0.0, 0.1 * (k - 1)), bevel=0.02))
    parts.append(pbox((-0.3, HELI_NOSE - 0.55, zc - 0.7), (0.3, HELI_NOSE, zc - 0.5), "rock_dark", bevel=0.02))
    # Cockpit: shattered dark glass, the frame bars and the door post.
    for side in (-1, 1):
        parts.append(pbox((side * (hw - 0.06), 3.9, zc - 0.5), (side * (hw - 0.01), 5.35, zc + 0.7),
                          "glass_dark", bevel=0.0))
        parts.append(_beam((side * (hw - 0.05), 3.9, zc + 0.72), (side * (hw - 0.62), 5.3, zc + 0.34), 0.03,
                           color=OLIVE, segments=4))
        parts.append(_beam((side * (hw - 0.05), 3.9, zc), (side * (hw - 0.4), 5.4, zc - 0.2), 0.026,
                           color=OLIVE, segments=4))
    parts.append(pbox((-0.92, 5.15, zc + 0.34), (0.92, 5.5, zc + 0.86), "glass_dark", bevel=0.0))
    parts.append(box((-0.95, 5.4, zc + 0.8), (0.95, 5.56, zc + 0.9), OLIVE))  # windscreen header
    # Cabin door slid open on the right flank: a dark opening with rails, the door hung aft.
    z0, z1 = zc - 0.92, zc + 0.86
    parts.append(pbox((hw - 0.1, 0.0, z0), (hw + 0.01, 1.18, z1), SOOT, bevel=0.0))
    for y0, zz0, y1, zz1 in ((0.0, z0, 0.06, z1), (1.12, z0, 1.18, z1), (0.0, z0, 1.18, z0 + 0.07),
                             (0.0, z1 - 0.07, 1.18, z1)):
        parts.append(pbox((hw - 0.02, y0, zz0), (hw + 0.05, y1, zz1), METAL, bevel=0.0))
    parts.append(box((hw + 0.05, -1.4, z0 + 0.06), (hw + 0.11, 1.2, z0 + 0.12), METAL))  # slide rail
    parts.append(pbox_at((hw + 0.02, -1.45, z0 + 0.1), (hw + 0.1, -0.2, z1 - 0.08), OLIVE,
                         rot=(0.0, 0.0, -0.04), bevel=0.02))
    parts.append(box((hw + 0.1, -1.2, zc + 0.5), (hw + 0.13, -0.3, zc + 0.62), "glass_dark"))
    parts.append(pbox((hw + 0.08, -1.5, z0 + 0.06), (hw + 0.14, -1.35, z1 - 0.06), METAL, bevel=0.0))
    # Seats inside the cabin, visible through the open door, plus a trooper bench and loose kit.
    for y in (0.55, -0.15, -0.85):
        parts.append(pbox((hw - 0.72, y - 0.24, z0 + 0.02), (hw - 0.24, y + 0.24, z0 + 0.3), "cloth_olive",
                          bevel=0.02))
        parts.append(pbox((hw - 0.26, y - 0.24, z0 + 0.24), (hw - 0.18, y + 0.24, z0 + 0.96), "cloth_olive",
                          bevel=0.02))
    parts.append(pbox((-hw + 0.12, -2.6, z0 - 0.06), (hw - 0.12, 2.4, z0), METAL, bevel=0.0))  # cabin floor
    parts.append(pbox((-hw + 0.12, 2.35, z0 - 0.02), (hw - 0.12, 2.48, zc + 0.5), METAL, bevel=0.0))
    parts.append(pbox((-hw + 0.15, -1.5, z0 + 0.02), (hw - 0.4, -0.9, z0 + 0.46), KHAKI, bevel=0.04))  # packs
    parts.append(pbox((-0.2, -2.2, z0 + 0.02), (0.5, -1.6, z0 + 0.42), KHAKI, bevel=0.04))
    parts.append(_beam((-0.3, 1.3, z0 + 1.4), (0.4, 1.3, z0 + 0.9), 0.02, color="leather", segments=4))
    # Engine deck, exhausts and the soot ring around them.
    parts.append(pbox((-0.88, -3.3, zc + 1.0), (0.88, -1.5, zc + 1.32), METAL, bevel=0.02))
    for k in range(5):
        parts.append(box((-0.72 + k * 0.34, -3.2, zc + 1.32), (-0.52 + k * 0.34, -1.75, zc + 1.36), DARK))
    for side in (-1, 1):
        parts.append(_beam((side * 0.55, -2.1, zc + 1.26), (side * 0.8, -3.2, zc + 1.5), 0.085, color=SOOT,
                           segments=6))
    parts.append(common.make_frustum(0.7, 0.32, 0.0, 0.14, segments=12, center=(0.0, -2.4), color=SOOT))
    common.transform(parts[-1], loc=(0.0, 0.0, zc + 1.32))
    parts.append(pbox((-0.5, -1.5, zc + 1.3), (0.5, -0.4, zc + 1.42), METAL, bevel=0.02))  # transmission hump
    # Rotor mast and hub on top, with two bent stub blades (the other two are gone).
    parts.append(common.make_frustum(0.16, 0.14, 0.0, 0.62, segments=10, center=(0.0, -0.55), color=METAL))
    common.transform(parts[-1], loc=(0.0, 0.0, zc + 1.36))
    hub = common.make_frustum(0.36, 0.3, 0.0, 0.24, segments=10, center=(0.0, -0.55), color=METAL)
    common.transform(hub, loc=(0.0, 0.0, zc + 1.94))
    parts.append(hub)
    for k in range(4):  # four blade grips, two still holding a bent stub
        a = k * math.pi / 2.0 + 0.4
        grip = (math.cos(a) * 0.42, -0.55 + math.sin(a) * 0.42, zc + 2.08)
        parts.append(_beam((math.cos(a) * 0.3, -0.55 + math.sin(a) * 0.3, zc + 2.02), grip, 0.07,
                           color=DARK, segments=5))
        if k in (1, 2):
            droop = -0.55 if k == 1 else -0.95
            mid = (math.cos(a) * 1.7, -0.55 + math.sin(a) * 1.7, zc + 2.08 + droop * 0.4)
            tip = (math.cos(a) * 2.6, -0.55 + math.sin(a) * 2.6, zc + 2.08 + droop * 1.4)
            parts.append(_beam(grip, mid, 0.085, color=DARK, segments=6))
            parts.append(_beam(mid, tip, 0.07, color=DARK, segments=6))
            parts.append(pbox((-0.1, -0.2, -0.02), (0.1, 0.2, 0.02), METAL, bevel=0.0))
            common.transform(parts[-1], rot=(0.0, 0.0, a), loc=grip)
    # Boom break: a kinked joint with torn skins, then the fin and the bent tail rotor.
    parts.append(_beam((0.0, -3.6, zc + 0.05), (0.0, -4.2, zc - 0.28), 0.2, color=OLIVE, segments=8))
    parts.append(pbox((-0.52, -4.3, zc - 0.5), (0.52, -3.8, zc + 0.3), RUST, bevel=0.02))  # torn collar
    parts.append(pbox_at((-0.09, -7.5, zc - 1.6), (0.09, -6.5, zc - 0.9), OLIVE,
                         rot=(-0.45, 0.0, 0.0), bevel=0.03))
    parts.append(_beam((-0.05, -7.15, zc - 1.2), (0.05, -7.0, zc - 1.05), 0.05, color=METAL, segments=4))
    for a, ln in ((0.0, 0.5), (2.4, 0.42)):  # two bent tail-rotor blades
        parts.append(_beam((0.0, -7.0, zc - 1.0), (math.cos(a) * ln, -7.0, zc - 1.0 + math.sin(a) * ln * 0.5),
                           0.028, color=DARK, segments=4))
    parts.append(_beam((-0.4, -6.9, zc - 0.75), (0.4, -6.9, zc - 0.75), 0.05, color=OLIVE, segments=5))
    # Skids, collapsed: the left leg splayed out flat on the road, the right one folded under.
    for side in (-1, 1):
        zt = zc - 0.95 if side < 0 else zc - 1.15
        parts.append(_beam((side * 1.32, -1.9, zt), (side * 1.32, 3.0, zt), 0.075, color=METAL, segments=6))
        for y in (-1.6, 2.5):
            parts.append(_beam((side * 1.32, y, zt), (side * 1.06, y, zt + 0.8), 0.05, color=METAL, segments=5))
        parts.append(_beam((side * 1.32, -1.85, zt), (side * (1.62 if side < 0 else 1.15), -2.1, 0.06), 0.05,
                           color=METAL, segments=5))
        parts.append(box((side * 1.3 - 0.05, -2.2, 0.02), (side * 1.3 + 0.05, -1.8, 0.1), METAL))
    parts.append(_beam((-1.32, -1.9, zc - 0.95), (1.32, -1.9, zc - 1.15), 0.05, color=METAL, segments=5))
    # Debris field: torn skin, a rotor fragment and a scorched patch behind the engine.
    rng = random.Random(7911)
    for _ in range(12):
        x = rng.uniform(-3.2, 3.2)
        y = rng.uniform(-6.0, 3.4)
        ground.append(pbox((x, y, 0.0), (x + rng.uniform(0.2, 0.62), y + rng.uniform(0.2, 0.62),
                                         rng.uniform(0.03, 0.14)), rng.choice((SOOT, RUST, METAL)), bevel=0.0))
    ground.append(pbox((2.0, -2.6, 0.0), (2.9, -1.9, 0.04), RUST, bevel=0.0))  # a bent tail blade on the road
    parts.append(stencil_on_side("42", 0.14, 0.2, hw + 0.012, zc - 0.45, -2.1, "soot", outward=1))
    parts.append(stencil_on_side("ARMY", 0.13, 0.18, -hw - 0.012, zc - 0.45, -1.6, "soot", outward=-1))
    return parts, ground


def _build_heli_crashed(model, lod=False):
    parts, ground = _heli_body(lod)
    # Tip the airframe: -20 deg about X drops the nose (+Y) into the road and lifts the tail.
    # The debris and the flattened skid tips stay flat on the road (they are built on it, not tipping).
    combined = common.combine(parts)
    common.transform(combined, rot=(0.0, ROLL, 0.0))
    common.transform(combined, rot=(-TILT, 0.0, 0.0), loc=(0.0, 0.0, 0.0))
    model.add(combined)
    if ground:
        model.add(common.combine(ground))
    if lod:
        return
    for label, lo, hi in (("body", (-1.35, -3.2, 0.3), (1.35, 6.7, 3.4)),
                          ("boom", (-0.6, -7.6, 0.2), (0.6, -3.2, 2.6))):
        bm = box(lo, hi, "white")
        common.transform(bm, rot=(0.0, ROLL, 0.0))
        common.transform(bm, rot=(-TILT, 0.0, 0.0))
        model.add_collider(label, bm)


# --- heli_rotor_blade --------------------------------------------------------------------------
def _build_heli_rotor_blade(model):
    """One long main-rotor blade, 7 m, bent about 4 m out, with a root fitting and a twisted tip."""
    length = 7.0  # new: DECISIONS 2026-11-26 (a single rotor blade is 7 m)
    sections = []
    n = 9
    for i in range(n + 1):
        t = i / n
        x = t * length
        # the blade kinks down and back from about 55% of its length
        bend = max(0.0, (t - 0.55) / 0.45)
        y = -bend * bend * 0.85
        z = -bend * bend * 0.55
        chord = 0.36 * (1.0 - 0.55 * t)
        thick = 0.07 * (1.0 - 0.4 * t)
        sections.append((x, [(y - chord / 2, z - thick), (y + chord / 2, z - thick),
                             (y + chord / 2, z + thick), (y - chord / 2, z + thick)]))
    blade = loft([[(sx, py, pz) for py, pz in prof] for sx, prof in sections], DARK)
    parts = [blade]
    # root fitting and the bolt holes, plus one torn chunk of the tip
    parts.append(pbox((-0.02, -0.2, -0.12), (0.42, 0.2, 0.12), METAL, bevel=0.02))
    for k in range(2):
        parts.append(common.make_frustum(0.05, 0.05, 0.0, 0.05, segments=6,
                                         center=(0.14 + k * 0.16, 0.0), color="chrome"))
        common.transform(parts[-1], loc=(0.0, 0.0, 0.12))
    parts.append(pbox((3.2, -0.22, -0.05), (3.9, 0.22, 0.05), METAL, bevel=0.0))  # balancing strip
    parts.append(pbox((5.7, -0.16, -0.04), (6.2, 0.14, 0.04), RUST, bevel=0.0))  # bent scraped patch
    model.add(common.combine(parts))
    model.add_collider("blade", box((0.0, -0.24, -0.1), (length, 0.24, 0.12), "white"))


# --- group -------------------------------------------------------------------------------------
def build_all():
    """Builds the army line vehicles. Returns [(name, ok)]."""
    return [
        ("army_tank", common.run_model("army_tank", "vehicles", VEHICLE_BUDGET, _build_army_tank,
                                       view=(0.9, -1.0, 0.55))),
        ("army_tank_lod1", common.run_model("army_tank_lod1", "vehicles", VEHICLE_LOD1_BUDGET,
                                            lambda m: _build_army_tank(m, lod=True))),
        ("army_truck_burnt", common.run_model("army_truck_burnt", "vehicles", VEHICLE_BUDGET,
                                              _build_army_truck_burnt, view=(0.9, -1.0, 0.6))),
        ("army_truck_burnt_lod1", common.run_model("army_truck_burnt_lod1", "vehicles", VEHICLE_LOD1_BUDGET,
                                                   lambda m: _build_army_truck_burnt(m, lod=True))),
        ("heli_crashed", common.run_model("heli_crashed", "vehicles", VEHICLE_BUDGET, _build_heli_crashed,
                                          view=(1.0, -0.75, 0.5))),
        ("heli_crashed_lod1", common.run_model("heli_crashed_lod1", "vehicles", VEHICLE_LOD1_BUDGET,
                                               lambda m: _build_heli_crashed(m, lod=True))),
        ("heli_rotor_blade", common.run_model("heli_rotor_blade", "props", BLADE_BUDGET,
                                              _build_heli_rotor_blade, view=(0.6, -1.0, 0.8))),
    ]
