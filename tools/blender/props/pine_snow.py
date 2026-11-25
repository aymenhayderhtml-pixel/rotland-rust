"""pine_snow: snow-laden alpine pine (~9.7 m) with pine_snow_dark boughs and snow_white caps (<= 450 tris)."""
import math
import os
import random
import sys

import bmesh
from mathutils import Vector

_DIR = os.path.dirname(os.path.abspath(__file__))
_PARENT = os.path.join(_DIR, "..")
for _p in (_DIR, _PARENT):
    if _p not in sys.path:
        sys.path.insert(0, _p)

import common  # noqa: E402
import veg  # noqa: E402

NAME, CATEGORY, BUDGET = "pine_snow", "props", 450
SINK_Z = -0.05


def _rooted_trunk(model, r0, r1, height, color, seed, roots=4, segs=6):
    """Tapered trunk with a flared base and ground-sunk root wedges (64 tris)."""
    rng = random.Random(seed)
    model.add(common.make_frustum(r0, r1, SINK_Z, height, segments=segs, color=color))
    model.add(common.make_frustum(r0 * 1.4, r0, SINK_Z, 0.45, segments=segs, color=color))
    for i in range(roots):
        a = 2.0 * math.pi * i / roots + rng.uniform(-0.28, 0.28)
        length = r0 * rng.uniform(1.25, 1.85)
        d = Vector((math.cos(a), math.sin(a), 0.0))
        side = Vector((-d.y, d.x, 0.0))
        pts = [
            d * (r0 * 0.7) + side * (r0 * 0.35) + Vector((0.0, 0.0, SINK_Z)),
            d * (r0 * 0.7) - side * (r0 * 0.35) + Vector((0.0, 0.0, SINK_Z)),
            d * (r0 * 0.8) + Vector((0.0, 0.0, r0 * 1.35)),
            d * (r0 + length) + Vector((0.0, 0.0, SINK_Z)),
            d * (r0 + length * 0.45) + Vector((0.0, 0.0, r0 * 0.55)),
        ]
        hull = common.new_bm()
        verts = [hull.verts.new(p) for p in pts]
        bmesh.ops.convex_hull(hull, input=verts)
        bmesh.ops.recalc_face_normals(hull, faces=list(hull.faces))
        common.paint(hull, color)
        model.add(hull)


def _snow_cap(r, z0, height, seed, points=8, droop=0.14, inner=0.70, center=(0.0, 0.0)):
    """Thin jagged snow blanket sitting on the upper half of a pine tier (same seed aligns points)."""
    rng = random.Random(seed)
    bm = common.new_bm()
    cx, cy = center
    rot = rng.uniform(0.0, 2.0 * math.pi)
    apex = bm.verts.new((cx, cy, z0 + height))
    rim = []
    for i in range(points * 2):
        a = rot + math.pi * i / points
        if i % 2 == 0:
            rr = r * rng.uniform(0.92, 1.06)
            z = z0 - droop * rng.uniform(0.7, 1.15)
        else:
            rr = r * inner
            z = z0 + 0.08 * height
        rim.append(bm.verts.new((cx + math.cos(a) * rr, cy + math.sin(a) * rr, z)))
    under = bm.verts.new((cx, cy, z0 + 0.22 * height))
    n = len(rim)
    top_faces = []
    under_faces = []
    for i in range(n):
        top_faces.append(bm.faces.new([rim[i], rim[(i + 1) % n], apex]))
        under_faces.append(bm.faces.new([rim[(i + 1) % n], rim[i], under]))
    bmesh.ops.recalc_face_normals(bm, faces=list(bm.faces))
    common.paint_faces(bm, top_faces, "snow_white")
    common.paint_faces(bm, under_faces, "snow_shadow")
    return bm


def build(model):
    rng = random.Random(3107)
    _rooted_trunk(model, 0.25, 0.14, 3.2, "bark_dark", seed=3107)
    model.add_collider("trunk", common.make_frustum(0.24, 0.20, SINK_Z, 2.8, segments=6, color="bark_dark"))

    # 5 jagged pine tiers with matching snow caps draped over the upper half of each layer
    tiers = [
        (2.60, 1.80, 2.40),
        (2.15, 3.10, 2.20),
        (1.70, 4.40, 2.00),
        (1.25, 5.60, 1.80),
        (0.82, 6.75, 1.70),
    ]
    for i, (radius, z0, height) in enumerate(tiers):
        off = (rng.uniform(-0.08, 0.08), rng.uniform(-0.08, 0.08))
        seed = 100 + 13 * i
        model.add(veg.tier(radius, z0, height, "pine_snow_dark", seed=seed,
                           points=8, droop=0.32, inner=0.64, center=off))
        cap_z0 = z0 + 0.36 * height
        cap_h = 0.64 * height + 0.14
        model.add(_snow_cap(radius * 0.72, cap_z0, cap_h, seed=seed,
                            points=8, droop=0.14, inner=0.70, center=off))

    # Top pine spire and snowy tip (~9.7 m total height)
    model.add(common.make_frustum(0.50, 0.0, 7.80, 9.35, segments=6, color="pine_snow_dark"))
    model.add(common.make_frustum(0.38, 0.0, 8.35, 9.75, segments=6, rot_z=0.3, color="snow_white"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=(0.0, 0.0, 0.0)) else 1


if __name__ == "__main__":
    sys.exit(main())
