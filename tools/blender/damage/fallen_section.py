"""Fallen tower (`tw_fallen_section` and `_lod1`): a 24 m tower that toppled across a street and broke
into three segments on impact.

X is the long axis (about 72 m, ends near -36 and +36), Y the width (24 m), Z up with the ground at Z=0.
Origin at bottom centre. The tower lies on its side, so its floors are vertical slabs (YZ planes) along X.
Each segment is rolled and yawed a little and sunk into the ground. The +X segment carries the crown
(parapet, roof machinery, and a toppled water tank on the street). Rubble ramps fill the gaps and climb
the outer ends.
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
from bmesh.types import BMVert  # noqa: E402
from mathutils import Matrix, Vector  # noqa: E402

import common  # noqa: E402
import kit  # noqa: E402

BUDGET_LOD0 = 16000  # new: DECISIONS 2026-11-27 (R41-10 fallen section <= 16k tris)
BUDGET_LOD1 = 3000  # new: DECISIONS 2026-11-26 (10% LOD1 budget for fallen section)
TOWER_SIZE = 24.0  # new: DECISIONS 2026-11-26 (24 m tower width)
HW = TOWER_SIZE / 2.0
FLOOR_H = 3.8  # new: DECISIONS 2026-11-26 (standard tower floor height)
BAY_W = 4.0  # new: DECISIONS 2026-11-26 (standard 4 m facade bay)
END_TRIM = 3.5  # new: DECISIONS 2026-11-26 (broken end bands reach up to 3.5 m past the main block)
RAMP_BASE_Z = -0.3  # new: DECISIONS 2026-11-26 (ramp base sunk below the ground)
MAX_SLOPE_DEG = 33.0  # new: DECISIONS 2026-11-26 (rubble ramps stay under 35 deg so players can walk up)
SEED = 4040  # new: DECISIONS 2026-11-26

# (x start, x end, roll deg, yaw deg, sink m, local height m). Gaps between segments are 4 m.
SEGMENTS = [
    (-32.0, -12.0, 7.0, 3.0, 1.5, 19.5),  # new: DECISIONS 2026-11-26
    (-8.0, 12.0, -8.0, -4.0, 2.5, 19.5),  # new: DECISIONS 2026-11-26
    (16.0, 32.0, 6.0, 2.0, 1.0, 19.5),  # new: DECISIONS 2026-11-26 (+X crown segment)
]
# (base x, top x, rise m, half width m, colour): rubble wedges in the gaps and at both outer ends
RAMPS = [
    (-12.6, -8.1, 2.6, 10.5, "rock_mid"),  # new: DECISIONS 2026-11-26
    (15.8, 12.2, 2.4, 10.5, "concrete_grey"),  # new: DECISIONS 2026-11-26
    (-36.0, -31.8, 2.6, 9.0, "rock_mid"),  # new: DECISIONS 2026-11-26
    (36.0, 31.8, 2.6, 9.0, "concrete_grey"),  # new: DECISIONS 2026-11-26
]


class _Bag:
    """Collects parts in local coordinates. `_place` moves them into the Model."""

    def __init__(self):
        self.parts = []
        self.cols = []

    def add(self, bm, roof=False):
        self.parts.append(bm)

    def add_collider(self, label, bm):
        self.cols.append((label, bm))


def _place(model, bag, prefix, loc, rot, colliders=True):
    for bm in bag.parts:
        common.transform(bm, loc=loc, rot=rot)
        model.add(bm)
    if colliders:
        for label, bm in bag.cols:
            common.transform(bm, loc=loc, rot=rot)
            model.add_collider(f"{prefix}_{label}", bm)


def _seg_loc(x0, x1, hloc, sink):
    return ((x0 + x1) / 2.0, 0.0, hloc / 2.0 - sink)


def _seg_rot(roll_deg, yaw_deg):
    return (math.radians(roll_deg), 0.0, math.radians(yaw_deg))


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


def _rbox(bag, center, size, rot, color):
    """Box of `size` centred on `center`, turned by `rot` (roll, pitch, yaw in radians)."""
    bm = common.make_box(size, color=color)
    common.transform(bm, loc=center, rot=rot)
    bag.add(bm)


def _hull(bag, points, color, collide=False):
    """Convex rubble piece: the hull of `points`. Wedges get a convex collider so the player can walk up."""
    bm = common.new_bm()
    verts = [bm.verts.new(p) for p in points]
    bm.verts.ensure_lookup_table()
    res = bmesh.ops.convex_hull(bm, input=verts)
    inner = [g for key in ("geom_interior", "geom_unused") for g in res.get(key, []) if isinstance(g, BMVert)]
    bmesh.ops.delete(bm, geom=inner, context="VERTS")
    bmesh.ops.recalc_face_normals(bm, faces=list(bm.faces))
    if collide:
        bag.add_collider(f"ramp{len(bag.cols)}", bm.copy())
    common.paint(bm, color)
    bag.add(bm)


def _ramp(bag, x_base, x_top, rise, half_w, color):
    """Rubble wedge from the ground at x_base up to height `rise` at x_top, never steeper than MAX_SLOPE_DEG."""
    run = abs(x_top - x_base)
    h = min(rise, run * math.tan(math.radians(MAX_SLOPE_DEG)) + RAMP_BASE_Z)
    pts = [(x_base, -half_w, RAMP_BASE_Z), (x_base, half_w, RAMP_BASE_Z),
           (x_top, -half_w, RAMP_BASE_Z), (x_top, half_w, RAMP_BASE_Z),
           (x_top, -half_w + 0.8, h), (x_top, half_w - 0.8, h)]  # new: DECISIONS 2026-11-26
    _hull(bag, pts, color, collide=True)


def _lump(bag, c, r, h, rng, color):
    """Irregular rubble heap at c: a jittered ring on the ground and a smaller cap."""
    pts = []
    for i in range(6):  # new: DECISIONS 2026-11-26
        a = 2 * math.pi * i / 6
        rr = r * rng.uniform(0.8, 1.0)  # new: DECISIONS 2026-11-26
        pts.append((c[0] + math.cos(a) * rr, c[1] + math.sin(a) * rr, RAMP_BASE_Z))
    for i in range(4):  # new: DECISIONS 2026-11-26
        a = 2 * math.pi * i / 4 + 0.4  # new: DECISIONS 2026-11-26
        rr = r * 0.45 * rng.uniform(0.7, 1.0)  # new: DECISIONS 2026-11-26
        pts.append((c[0] + math.cos(a) * rr, c[1] + math.sin(a) * rr, h * rng.uniform(0.7, 1.0)))  # new: DECISIONS 2026-11-26
    _hull(bag, pts, color)


def _face(k, sgn, x0, x1, z0, z1, depth, color):
    """Box stuck on the +Y (sgn=1) or -Y face of a segment, `depth` metres proud of it."""
    y0, y1 = sorted((sgn * HW, sgn * (HW + depth)))
    k.box((x0, y0, z0), (x1, y1, z1), color)


def _broken_end(bag, k, s, hl, hz, rng):
    """One broken end (s = -1 or +1): wall bands reaching different distances out, a notch with soot,
    vertical floor slabs sticking out at different lengths, hanging rebar, cables and a pipe."""
    face = s * (hl - END_TRIM)
    z_edges = (-hz, -hz + 5.0, -hz + 10.5, -hz + 14.0, hz)  # new: DECISIONS 2026-11-26
    reach = (END_TRIM, 1.2, 0.0, 2.2)  # new: DECISIONS 2026-11-26 (0 = missing chunk, shows the notch)
    colors = ("limestone", "limestone_dark", "concrete_stain", "limestone")
    for j, r_out in enumerate(reach):
        z0, z1 = z_edges[j], z_edges[j + 1]
        if r_out > 0:
            xa, xb = sorted((face, s * (hl - END_TRIM + r_out)))
            k.box((xa, -HW, z0), (xb, HW, z1), colors[j])
        else:
            xa, xb = sorted((face, face + s * 0.06))  # new: DECISIONS 2026-11-26 (soot skin on the notch wall)
            k.box((xa, -HW + 0.8, z0 + 0.6), (xb, HW - 0.8, z1 - 0.6), "soot")  # new: DECISIONS 2026-11-26
    # Floor slabs (YZ planes, 0.26 m thick) at different lengths, with jagged tops
    for e, drop, col in ((1.0, 0.0, "concrete_grey"), (3.0, 4.5, "concrete_stain"),  # new: DECISIONS 2026-11-26
                         (4.2, 2.0, "concrete_grey"), (5.2, 9.5, "concrete_stain")):  # new: DECISIONS 2026-11-26
        xc = s * (hl - END_TRIM + e)
        xa, xb = sorted((xc - 0.13, xc + 0.13))  # new: DECISIONS 2026-11-26 (floor slab thickness)
        k.box((xa, -HW + 0.7, -hz), (xb, HW - 0.7, hz - drop), col)  # new: DECISIONS 2026-11-26
    # Hanging rebar, drooping cables and a pipe at the broken end
    for _ in range(3):  # new: DECISIONS 2026-11-26
        y = rng.uniform(-HW + 2.0, HW - 2.0)  # new: DECISIONS 2026-11-26
        z = rng.uniform(-hz + 6.0, hz - 2.0)  # new: DECISIONS 2026-11-26
        _angled_bar(bag, (s * (hl - 0.3), y, z), (s * (hl + 1.0), y + 0.3, z - 1.8), 0.06, 0.06, "rust_dark")  # new: DECISIONS 2026-11-26
    for y in (-5.0, 4.0):  # new: DECISIONS 2026-11-26
        _angled_bar(bag, (s * (hl - 0.4), y, hz - 1.0), (s * (hl + 0.5), y + 0.4, hz - 5.0), 0.07, 0.07, "gun_black")  # new: DECISIONS 2026-11-26
        _angled_bar(bag, (s * (hl + 0.5), y + 0.4, hz - 5.0), (s * (hl + 0.2), y + 0.5, hz - 8.5), 0.06, 0.06, "gun_black")  # new: DECISIONS 2026-11-26
    _pipe(bag, (s * (hl - 1.0), 6.0, -hz + 4.0), (s * (hl + 1.4), 6.2, -hz + 4.6), 0.14, "metal")  # new: DECISIONS 2026-11-26


def _floor_bands(k, hl, hz):
    """Floor lines: vertical slabs across the segment at each floor boundary (floors stand on their edge)."""
    n = int(2 * hl // FLOOR_H)
    for i in range(n + 1):
        xf = -hl + i * FLOOR_H
        if abs(xf) < hl - END_TRIM:
            k.box((xf - 0.15, -HW - 0.1, -hz), (xf + 0.15, HW + 0.1, hz), "limestone_dark")  # new: DECISIONS 2026-11-26


def _top_teeth(k, hl, hz, rng):
    """Broken top edges on both sides: uneven teeth of wall, so the silhouette is jagged."""
    x = -hl + 0.4  # new: DECISIONS 2026-11-26
    while x < hl - 1.0:  # new: DECISIONS 2026-11-26
        w = rng.uniform(1.0, 2.2)  # new: DECISIONS 2026-11-26
        h = rng.uniform(0.6, 3.2)  # new: DECISIONS 2026-11-26
        if rng.random() > 0.3:  # new: DECISIONS 2026-11-26 (about 70% of gaps have a tooth)
            for sgn in (1.0, -1.0):
                _face(k, sgn, x, min(x + w, hl - 0.4), hz - 0.5, hz + h, 0.2, "limestone_dark")  # new: DECISIONS 2026-11-26
        x += w + rng.uniform(0.6, 2.0)  # new: DECISIONS 2026-11-26


def _pane_colour(rng):
    """Glass for one bay: None for a missing pane (dark cavity), else dark glass or soot-black glass."""
    roll = rng.random()
    if roll < 0.15:  # new: DECISIONS 2026-11-26 (15% missing panes)
        return None
    return "glass_dark" if roll < 0.7 else "soot"  # new: DECISIONS 2026-11-26


def _side_bay(k, sgn, xc, zc, rng):
    """One 3 m x 3.2 m pane in a 3.8 m x 4 m bay on a long side, with piers and head/sill bars."""
    col = _pane_colour(rng)
    if col is None:
        _face(k, sgn, xc - 1.5, xc + 1.5, zc - 1.6, zc + 1.6, 0.02, "soot")  # new: DECISIONS 2026-11-26
        return
    _face(k, sgn, xc - 1.5, xc + 1.5, zc - 1.6, zc + 1.6, 0.06, col)  # new: DECISIONS 2026-11-26
    _face(k, sgn, xc - 1.7, xc - 1.5, zc - 1.9, zc + 1.9, 0.12, "limestone_dark")  # new: DECISIONS 2026-11-26 (pier)
    _face(k, sgn, xc + 1.5, xc + 1.7, zc - 1.9, zc + 1.9, 0.12, "limestone_dark")  # new: DECISIONS 2026-11-26 (pier)
    _face(k, sgn, xc - 1.7, xc + 1.7, zc + 1.6, zc + 1.8, 0.12, "limestone_dark")  # new: DECISIONS 2026-11-26 (head)
    _face(k, sgn, xc - 1.7, xc + 1.7, zc - 1.8, zc - 1.6, 0.12, "limestone_dark")  # new: DECISIONS 2026-11-26 (sill)


def _top_bay(k, xc, yc, hz, rng):
    """One pane on the top face: the upward-facing side of the fallen floor."""
    col = _pane_colour(rng)
    if col is None:
        k.box((xc - 1.5, yc - 1.6, hz), (xc + 1.5, yc + 1.6, hz + 0.02), "soot")  # new: DECISIONS 2026-11-26
        return
    k.box((xc - 1.5, yc - 1.6, hz), (xc + 1.5, yc + 1.6, hz + 0.06), col)  # new: DECISIONS 2026-11-26
    k.box((xc - 1.7, yc - 1.7, hz), (xc - 1.5, yc + 1.7, hz + 0.12), "limestone_dark")  # new: DECISIONS 2026-11-26
    k.box((xc + 1.5, yc - 1.7, hz), (xc + 1.7, yc + 1.7, hz + 0.12), "limestone_dark")  # new: DECISIONS 2026-11-26
    k.box((xc - 1.7, yc - 1.7, hz), (xc + 1.7, yc - 1.6, hz + 0.12), "limestone_dark")  # new: DECISIONS 2026-11-26
    k.box((xc - 1.7, yc + 1.6, hz), (xc + 1.7, yc + 1.7, hz + 0.12), "limestone_dark")  # new: DECISIONS 2026-11-26


def _windows(k, hl, hz, rng):
    """Facade bays on both long sides and the top. Floors are vertical, so columns run along X every
    FLOOR_H; side rows go up Z every BAY_W, top rows go across Y every BAY_W."""
    n_cols = int(2 * hl // FLOOR_H)
    n_rows = int(2 * hz // BAY_W)
    n_across = int(TOWER_SIZE // BAY_W)  # new: DECISIONS 2026-11-26 (6 top bays across the 24 m width)
    for i in range(n_cols):
        xc = -hl + (i + 0.5) * FLOOR_H
        for sgn in (1.0, -1.0):
            for r in range(n_rows):
                _side_bay(k, sgn, xc, -hz + (r + 0.5) * BAY_W, rng)
        for j in range(n_across):
            _top_bay(k, xc, -HW + (j + 0.5) * BAY_W, hz, rng)


def _crushed_bottom(bag, hl, hz, rng):
    """The side that hit the ground is crushed: tilted broken slabs along its bottom edge."""
    step = 2 * hl / 4.0  # new: DECISIONS 2026-11-26 (four pieces per segment)
    for j in range(4):  # new: DECISIONS 2026-11-26
        xc = -hl + (j + 0.5) * step
        yc = rng.uniform(-8.0, 8.0)  # new: DECISIONS 2026-11-26
        roll = rng.uniform(-0.45, 0.45)  # new: DECISIONS 2026-11-26 (about 26 deg tilt)
        yaw = rng.uniform(-0.2, 0.2)  # new: DECISIONS 2026-11-26
        _rbox(bag, (xc, yc, -hz + 0.4), (step * 1.4, 4.5, 0.5), (roll, 0.0, yaw), "concrete_stain")  # new: DECISIONS 2026-11-26


def _crown(k, hl, hz):
    """Crown on the +X segment: broken parapet on both top edges, an AC unit and an elevator bulkhead on the roof."""
    for sgn in (-1.0, 1.0):
        for a, b in ((-7.5, -5.0), (-3.5, -1.0), (0.5, 2.2), (4.0, 7.5)):  # new: DECISIONS 2026-11-26
            k.box((a, sgn * HW - 0.25, hz), (b, sgn * HW + 0.25, hz + 1.2), "limestone_dark")  # new: DECISIONS 2026-11-26
    k.box((1.0, -4.0, hz), (3.0, -1.5, hz + 1.5), "metal")  # new: DECISIONS 2026-11-26
    k.box((-5.0, 3.0, hz), (-2.0, 7.0, hz + 2.5), "limestone")  # new: DECISIONS 2026-11-26


def _segment(x0, x1, hloc, crown, rng):
    """One broken segment in local coordinates (centred on its middle, bottom at -hloc/2)."""
    bag = _Bag()
    k = kit.Kit(bag)
    hl, hz = (x1 - x0) / 2.0, hloc / 2.0
    core = hl - END_TRIM
    k.box((-core, -HW, -hz), (core, HW, hz), "limestone")
    k.collider((-core, -HW, -hz), (core, HW, hz), "core")
    k.collider((-hl, -HW, hz - 1.5), (hl, HW, hz), "top_deck")  # new: DECISIONS 2026-11-27 (full walkable top across broken ends)
    for s in (-1.0, 1.0):
        _broken_end(bag, k, s, hl, hz, rng)
    _floor_bands(k, hl, hz)
    _windows(k, hl, hz, rng)
    _crushed_bottom(bag, hl, hz, rng)
    if crown:
        _crown(k, hl, hz)
    else:
        _top_teeth(k, hl, hz, rng)
    return bag


def _gap_litter(bag, xg, rng):
    """Rubble in a gap between segments: heaps, rebar, slabs and a pipe lying on the ground."""
    for y in (-9.0, -4.5, 0.0, 4.5, 9.0):  # new: DECISIONS 2026-11-26
        colour = rng.choice(("rock_mid", "concrete_grey", "limestone_dark"))
        _lump(bag, (xg + rng.uniform(-1.2, 1.2), y, 0.0), 2.0, 1.6, rng, colour)  # new: DECISIONS 2026-11-26
    for _ in range(3):  # new: DECISIONS 2026-11-26
        y = rng.uniform(-9.0, 9.0)  # new: DECISIONS 2026-11-26
        _angled_bar(bag, (xg - 1.6, y, 0.9), (xg + 1.6, y + 0.4, 2.2), 0.06, 0.06, "rust_dark")  # new: DECISIONS 2026-11-26
    for _ in range(2):  # new: DECISIONS 2026-11-26
        y = rng.uniform(-9.0, 9.0)  # new: DECISIONS 2026-11-26
        _rbox(bag, (xg + rng.uniform(-1.0, 1.0), y, 0.5), (1.8, 2.8, 0.25),  # new: DECISIONS 2026-11-26
              (rng.uniform(-0.3, 0.3), 0.0, rng.uniform(0.0, math.pi)), "concrete_grey")
    _pipe(bag, (xg - 1.0, -5.0, 0.8), (xg + 1.2, -4.2, 1.2), 0.12, "metal")  # new: DECISIONS 2026-11-26


def _crown_ground(bag):
    """Toppled water tank and roof machinery lying on the street beside the crown."""
    tank = common.make_frustum(1.8, 1.8, -2.0, 2.0, segments=10, color="bark_dark")  # new: DECISIONS 2026-11-26
    common.transform(tank, loc=(24.0, -15.5, 1.8), rot=(0.0, math.pi / 2, 0.0))  # new: DECISIONS 2026-11-26
    bag.add(tank)
    for dx in (-1.2, 1.2):  # new: DECISIONS 2026-11-26
        band = common.make_frustum(1.86, 1.86, -0.2, 0.2, segments=10, color="metal")  # new: DECISIONS 2026-11-26
        common.transform(band, loc=(24.0 + dx, -15.5, 1.8), rot=(0.0, math.pi / 2, 0.0))
        bag.add(band)
    _rbox(bag, (30.0, 14.0, 0.9), (2.4, 1.8, 1.6), (0.15, 0.1, 0.4), "metal")  # new: DECISIONS 2026-11-26
    _rbox(bag, (22.0, 17.5, 1.2), (3.0, 3.0, 3.0), (0.2, -0.1, 0.3), "limestone_dark")  # new: DECISIONS 2026-11-26


def _ground_debris(bag, rng):
    """Panels, glass and window frames scattered on the street beside the fallen section."""
    kinds = (("concrete_grey", (2.4, 1.6, 0.12)), ("glass_tint", (0.9, 0.7, 0.03)),  # new: DECISIONS 2026-11-26
             ("metal", (1.2, 1.0, 0.08)))  # new: DECISIONS 2026-11-26
    for _ in range(16):  # new: DECISIONS 2026-11-26
        x = rng.uniform(-30.0, 30.0)  # new: DECISIONS 2026-11-26
        y = rng.choice((-1.0, 1.0)) * rng.uniform(13.5, 19.0)  # new: DECISIONS 2026-11-26
        colour, size = rng.choice(kinds)
        tilt = (rng.uniform(-0.2, 0.2), rng.uniform(-0.2, 0.2), rng.uniform(0.0, math.pi))  # new: DECISIONS 2026-11-26
        _rbox(bag, (x, y, 0.15), size, tilt, colour)  # new: DECISIONS 2026-11-26


def _gap_steps(bag, x0, z0, x1, z1, y_half=5.5, prefix="gap"):
    """Broken floor slabs as walkable steps (<= 0.35 m step) bridging between two segment tops."""
    k = kit.Kit(bag)
    dz = z1 - z0
    n = max(4, int(math.ceil(abs(dz) / 0.30)))
    dx = (x1 - x0) / n
    for i in range(n):
        t = (i + 1) / (n + 1) if abs(dz) < 0.1 else (i + 1) / n
        z_top = z0 + dz * (i + 0.5) / n if dz < 0 else z0 + dz * (i + 1) / n
        if dz < 0:
            z_top = z0 + dz * (i + 1) / (n + 1)
        xa = x0 + i * dx - 0.25
        xb = x0 + (i + 1) * dx + 0.25
        col = "concrete_grey" if i % 2 == 0 else "concrete_stain"
        k.box((xa, -y_half, z_top - 0.38), (xb, y_half, z_top), col)
        k.collider((xa, -y_half, z_top - 0.45), (xb, y_half, z_top), f"{prefix}_step_{i}")


def _world(bag, rng):
    """Gap ramps, outer-end ramps, gap floor-slab steps, gap litter, crown street debris and scattered debris."""
    for x_base, x_top, rise, half_w, colour in RAMPS:
        _ramp(bag, x_base, x_top, rise, half_w, colour)
    _gap_steps(bag, -12.0, 18.0, -8.0, 17.0, prefix="gap0")
    _gap_steps(bag, 12.0, 17.0, 16.0, 18.5, prefix="gap1")
    _gap_litter(bag, -10.0, rng)
    _gap_litter(bag, 14.0, rng)
    _crown_ground(bag)
    _ground_debris(bag, rng)


def build_lod0(model):
    rng = random.Random(SEED)
    for idx, (x0, x1, roll_d, yaw_d, sink, hloc) in enumerate(SEGMENTS):
        bag = _segment(x0, x1, hloc, crown=idx == len(SEGMENTS) - 1, rng=rng)
        _place(model, bag, f"seg{idx}", _seg_loc(x0, x1, hloc, sink), _seg_rot(roll_d, yaw_d))
    world = _Bag()
    _world(world, rng)
    _place(model, world, "world", (0.0, 0.0, 0.0), (0.0, 0.0, 0.0))


def build_lod1(model):
    """Silhouette only: three tilted boxes in the same places as lod0, plus the rubble wedges."""
    for idx, (x0, x1, roll_d, yaw_d, sink, hloc) in enumerate(SEGMENTS):
        bag = _Bag()
        bag.add(common.make_box((x1 - x0, TOWER_SIZE, hloc), color="limestone"))
        _place(model, bag, f"seg{idx}", _seg_loc(x0, x1, hloc, sink), _seg_rot(roll_d, yaw_d), colliders=False)
    world = _Bag()
    for x_base, x_top, rise, half_w, colour in RAMPS:
        _ramp(world, x_base, x_top, rise, half_w, colour)
    _place(model, world, "world", (0.0, 0.0, 0.0), (0.0, 0.0, 0.0), colliders=False)


def build_all():
    ok0 = common.run_model("tw_fallen_section", "buildings", BUDGET_LOD0, build_lod0,
                           grip=(0.0, 0.0, 0.0), top_preview=True, view=(1.0, -0.8, 0.65))  # new: DECISIONS 2026-11-26
    ok1 = common.run_model("tw_fallen_section_lod1", "buildings", BUDGET_LOD1, build_lod1,
                           grip=(0.0, 0.0, 0.0), top_preview=False, view=(1.0, -0.8, 0.65))  # new: DECISIONS 2026-11-26
    return [("tw_fallen_section", ok0), ("tw_fallen_section_lod1", ok1)]


if __name__ == "__main__":
    res = build_all()
    failed = [n for n, ok in res if not ok]
    for n, ok in res:
        print(f"FALLEN: {n} {'OK' if ok else 'FAILED'}")
    sys.exit(1 if failed else 0)
