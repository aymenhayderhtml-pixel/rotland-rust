"""Part builders shared by the weapon models.

Build coordinates: +Y is forward (the business end), +Z is up, 1 unit = 1 m.
Each weapon script picks its grip point; the exporter moves that point to the origin.
"""
import math
import os
import sys

import bmesh

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402


def cyl_y(r0, r1, y0, y1, color, x=0.0, z=0.0, segments=8):
    """Round part from y0 to y1 (r0 at y0, r1 at y1). Use r0 = 0 for a cone point."""
    bm = common.make_frustum(r0, r1, 0.0, y1 - y0, segments=segments, color=color)
    common.transform(bm, rot=(-math.pi / 2, 0.0, 0.0), loc=(x, y0, z))
    return bm


def box(size, center, color):
    """Box with size (x, y, z) centered at `center`."""
    return common.make_box(size, loc=center, color=color)


def convex(points, color):
    """Solid convex hull of a point cloud. Wedge heads use it: a hull corner is a sharp edge."""
    bm = common.new_bm()
    verts = [bm.verts.new(p) for p in points]
    result = bmesh.ops.convex_hull(bm, input=verts)
    # Points inside the hull are not part of the surface, so drop them
    inner = list({g for g in result["geom_interior"] + result["geom_unused"] if isinstance(g, bmesh.types.BMVert)})
    bmesh.ops.delete(bm, geom=inner, context="VERTS")
    bmesh.ops.recalc_face_normals(bm, faces=list(bm.faces))  # outward, so callers can read normals
    bm.normal_update()
    common.paint(bm, color)
    return bm


def prism_yz(points, thickness, color):
    """Flat blade: a (y, z) outline pushed out along X by `thickness` (convex or simple polygon)."""
    bm = common.new_bm()
    half = thickness / 2
    front = [bm.verts.new((half, y, z)) for y, z in points]
    back = [bm.verts.new((-half, y, z)) for y, z in points]
    bm.faces.new(front)
    bm.faces.new(list(reversed(back)))
    count = len(points)
    for i in range(count):
        j = (i + 1) % count
        bm.faces.new([front[i], front[j], back[j], back[i]])
    common.paint(bm, color)
    return bm


def rbox(size, center, color, bevel=0.2):
    """Box with its four long (Y) edges chamfered: an 8-sided bar, the base shape of most gun parts.
    `bevel` is the chamfer as a fraction of the smaller of width and height."""
    sx, sy, sz = size[0] / 2, size[1] / 2, size[2] / 2
    b = bevel * min(size[0], size[2])
    cx, cy, cz = center
    pts = []
    for y in (-sy, sy):
        for x, z in ((sx - b, sz), (sx, sz - b), (sx, -sz + b), (sx - b, -sz),
                     (-sx + b, -sz), (-sx, -sz + b), (-sx, sz - b), (-sx + b, sz)):
            pts.append((cx + x, cy + y, cz + z))
    return convex(pts, color)


def loft_y(profile, color, x=0.0, z=0.0, segments=12):
    """Round part along +Y through (y, radius) stations: tapers, knobs and bulges in one piece."""
    bm = common.new_bm()
    rings = []
    for y, r in profile:
        r = max(r, 0.0005)
        rings.append([bm.verts.new((x + math.cos(a) * r, y, z + math.sin(a) * r))
                      for a in (2 * math.pi * (i + 0.5) / segments for i in range(segments))])
    for r0, r1 in zip(rings, rings[1:]):
        for i in range(segments):
            j = (i + 1) % segments
            bm.faces.new([r0[i], r0[j], r1[j], r1[i]])
    bm.faces.new(list(reversed(rings[0])))
    bm.faces.new(rings[-1])
    bmesh.ops.recalc_face_normals(bm, faces=list(bm.faces))
    common.paint(bm, color)
    return bm


def hull_side(outline_yz, width, color):
    """Solid from a convex (y, z) outline, `width` thick along X (grips, stocks, wedges)."""
    half = width / 2
    return convex([(s * half, y, z) for y, z in outline_yz for s in (-1, 1)], color)
