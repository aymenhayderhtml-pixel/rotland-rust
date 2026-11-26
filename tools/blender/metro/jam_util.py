"""Shared helpers for the R38 jam models: bevelled panels, tyres and rims, stroke lettering, origin offset.

Build coordinates are metres: +X right, +Y forward, +Z up. `Model.finalize_and_export` moves each model so
its bounding box bottom centre is the origin. `offset_for` gives that shift before export, so interior parts
placed by a seat table use `to_build`.
"""
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, os.path.join(HERE, "..", "vehicles"))

import bmesh  # noqa: E402
import common  # noqa: E402
import car_parts  # noqa: E402

WHEEL_R = car_parts.WHEEL_RADIUS  # the car wheel mesh radius (0.34 m); kept for old callers
TYRE_SEG = 24  # new: DECISIONS 2026-11-26 (tyre segments, rounder tread)


def box(lo, hi, color):
    """Axis-aligned box between two opposite corners."""
    size = [hi[i] - lo[i] for i in range(3)]
    loc = [(hi[i] + lo[i]) / 2 for i in range(3)]
    return common.make_box(size, loc=loc, color=color)


def bevel_bm(bm, offset, color):
    """Bevels every edge of a box-like bmesh (two segments, rounded) and repaints it `color`."""
    bmesh.ops.bevel(bm, geom=list(bm.edges), offset=offset, offset_type="OFFSET", segments=2,
                    profile=0.5, affect="EDGES")
    common.paint(bm, color)
    return bm


def pbox(lo, hi, color, bevel=0.015):
    """Box with bevelled edges (a shaped panel, not a plain block). Bevel is capped to a third of the thinnest side."""
    bm = box(lo, hi, color)
    if bevel > 0:
        thinnest = min(hi[i] - lo[i] for i in range(3))
        bevel_bm(bm, min(bevel, thinnest / 3.0), color)
    return bm


def bar(x0, z0, x1, z1, y, thick, depth, color, bevel=0.0):
    """Flat bar on a face in the XZ plane (at height y), from (x0, z0) to (x1, z1). `bevel` rounds its edges."""
    length = math.hypot(x1 - x0, z1 - z0)
    angle = math.atan2(-(z1 - z0), x1 - x0)
    bm = common.make_box((length, depth, thick), color=color)
    if bevel > 0:
        bevel_bm(bm, min(bevel, depth / 3.0), color)
    common.transform(bm, rot=(0.0, angle, 0.0), loc=((x0 + x1) / 2, y, (z0 + z1) / 2))
    return bm


def tyre_bm(radius, half_w, color="tire_black", seg=TYRE_SEG):
    """Tyre with rounded shoulders and a tread band, axis along local Z (turned later)."""
    prof = [(-half_w, radius - 0.07), (-half_w * 0.82, radius - 0.01), (-half_w * 0.45, radius),
            (half_w * 0.45, radius), (half_w * 0.82, radius - 0.01), (half_w, radius - 0.07)]
    bm = common.new_bm()
    rings = []
    for z, r in prof:
        rings.append([bm.verts.new((math.cos(a) * r * (1.0 + (0.012 if k % 2 else 0.0)),
                                    math.sin(a) * r * (1.0 + (0.012 if k % 2 else 0.0)), z))
                      for k, a in ((k, 2 * math.pi * k / seg) for k in range(seg))])
    for k in range(len(rings) - 1):
        r0, r1 = rings[k], rings[k + 1]
        for i in range(seg):
            j = (i + 1) % seg
            bm.faces.new([r0[i], r0[j], r1[j], r1[i]])
    bm.faces.new(list(reversed(rings[0])))
    bm.faces.new(rings[-1])
    bmesh.ops.recalc_face_normals(bm, faces=list(bm.faces))
    common.paint(bm, color)
    return bm


def rim_bm(radius, half_w, spokes=5):
    """Steel rim with a barrel, five spokes and a hub cap, axis along local Z."""
    parts = [common.make_frustum(radius * 0.62, radius * 0.62, -half_w * 0.6, half_w * 0.6, segments=20,
                                 color="metal"),
             common.make_frustum(radius * 0.5, radius * 0.5, half_w * 0.6, half_w * 0.6 + 0.02, segments=20,
                                 color="metal")]
    for k in range(spokes):
        spoke = common.make_box((radius * 0.95, 0.05, 0.05), color="chrome")
        common.transform(spoke, rot=(0.0, 0.0, 2 * math.pi * k / spokes), loc=(0.0, 0.0, half_w * 0.6))
        parts.append(spoke)
    parts.append(common.make_frustum(0.07, 0.07, half_w * 0.6, half_w * 0.6 + 0.05, segments=12, color="chrome"))
    return common.combine(parts)


def wheel_parts(x, y, radius, half_w, flat=1.0, tyre=True, rim_color="metal"):
    """Tyre plus rim for a wheel at (x, y) on the ground, axle along X. `flat` squashes the tyre (0.72 = flat)."""
    out = []
    if tyre:
        out.append(tyre_bm(radius, half_w))
    out.append(rim_bm(radius, half_w))
    for bm in out:
        common.transform(bm, rot=(0.0, math.pi / 2.0, 0.0))
        common.transform(bm, scale=(1.0, 1.0, flat), loc=(x, y, radius * flat))
    return out


def ground_rim(x, y, radius, half_w, yaw=0.0):
    """A wheel rim lying flat on the ground (the tyre is gone)."""
    bm = rim_bm(radius, half_w)
    common.transform(bm, rot=(0.0, 0.0, yaw), loc=(x, y, half_w * 0.6 + 0.02))
    return bm


def offset_for(bms):
    """The shift Model.finalize_and_export adds to build coordinates (bbox bottom centre becomes origin)."""
    xs, ys, zs = [], [], []
    for bm in bms:
        for v in bm.verts:
            xs.append(v.co.x)
            ys.append(v.co.y)
            zs.append(v.co.z)
    return (-(min(xs) + max(xs)) / 2, -(min(ys) + max(ys)) / 2, -min(zs))


def to_build(point, offset):
    """Build coordinates for a point given in exported (origin-shifted) coordinates."""
    return (point[0] - offset[0], point[1] - offset[1], point[2] - offset[2])


def extrude_sections(sections, color):
    """Solid from closed profiles: sections = [(x, [(y, z), ...]), ...], same point count per profile."""
    bm = common.new_bm()
    rings = [[bm.verts.new((x, y, z)) for (y, z) in prof] for x, prof in sections]
    n = len(sections[0][1])
    for r in range(len(rings) - 1):
        a, b = rings[r], rings[r + 1]
        for k in range(n):
            j = (k + 1) % n
            bm.faces.new([a[k], a[j], b[j], b[k]])
    bm.faces.new(list(reversed(rings[0])))
    bm.faces.new(rings[-1])
    bmesh.ops.recalc_face_normals(bm, faces=list(bm.faces))
    common.paint(bm, color)
    return bm


def add_bbox_collider(model, bms, inset=0.05, label="shell"):
    """One solid box collider around the given parts (build coordinates), shrunk by `inset` metres
    on each side so the player can stand right against the vehicle. Added by the lead (R38): the jam
    vehicles must block the player and the car like the old car_wreck does."""
    xs, ys, zs = [], [], []
    for bm in bms:
        for v in bm.verts:
            xs.append(v.co.x)
            ys.append(v.co.y)
            zs.append(v.co.z)
    lo = (min(xs) + inset, min(ys) + inset, min(zs))
    hi = (max(xs) - inset, max(ys) - inset, max(zs) - inset)
    size = (hi[0] - lo[0], hi[1] - lo[1], hi[2] - lo[2])
    loc = ((lo[0] + hi[0]) / 2, (lo[1] + hi[1]) / 2, (lo[2] + hi[2]) / 2)
    model.add_collider(label, common.make_box(size, loc=loc, color="white"))
