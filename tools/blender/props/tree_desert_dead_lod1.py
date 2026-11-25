"""tree_desert_dead_lod1: medium LOD for twisted desert dead tree (<= 150 tris)."""
import math
import os
import sys

import bmesh
from mathutils import Vector

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402
import veg  # noqa: E402

NAME, CATEGORY, BUDGET = "tree_desert_dead_lod1", "props", 150


def _twisted_trunk(stations, segments=5):
    """4-ring 5-sided twisted trunk matching LOD0 silhouette."""
    bm = common.new_bm()
    rings = []
    for center, radius, twist in stations:
        ring = []
        for i in range(segments):
            angle = twist + 2.0 * math.pi * i / segments
            ring.append(bm.verts.new(center + Vector((math.cos(angle) * radius, math.sin(angle) * radius, 0.0))))
        rings.append(ring)

    common.paint_faces(bm, [bm.faces.new(list(reversed(rings[0])))], "bark_light")
    common.paint_faces(bm, [bm.faces.new(rings[-1])], "dry_brush")

    for r_idx in range(len(rings) - 1):
        r0 = rings[r_idx]
        r1 = rings[r_idx + 1]
        color = "bark_light" if r_idx == 0 else "dry_brush"
        for i in range(segments):
            j = (i + 1) % segments
            face = bm.faces.new([r0[i], r0[j], r1[j], r1[i]])
            common.paint_faces(bm, [face], color)

    bmesh.ops.recalc_face_normals(bm, faces=list(bm.faces))
    return bm


def build(model):
    trunk_stations = [
        (Vector((0.00, 0.00, -0.05)), 0.28, 0.00),
        (Vector((0.14, 0.08, 1.15)), 0.18, 0.42),
        (Vector((-0.06, 0.22, 2.25)), 0.13, 0.70),
        (Vector((0.08, 0.14, 3.25)), 0.07, 0.95),
    ]
    model.add(_twisted_trunk(trunk_stations, segments=5))

    # 4 primary bent limbs matching LOD0 (inner bark_light segs=4, outer dry_brush tapered segs=3)
    limbs = [
        (Vector((0.12, 0.08, 1.35)), Vector((0.88, 0.42, 2.05)), Vector((1.55, 0.18, 2.75)), 0.095),
        (Vector((0.04, 0.14, 1.75)), Vector((-0.78, 0.78, 2.45)), Vector((-1.42, 0.98, 3.15)), 0.090),
        (Vector((-0.04, 0.20, 2.15)), Vector((-0.62, -0.68, 2.95)), Vector((-1.10, -1.08, 3.68)), 0.080),
        (Vector((0.02, 0.18, 2.65)), Vector((0.68, -0.64, 3.40)), Vector((1.05, -0.95, 4.10)), 0.070),
    ]
    for p0, p1, p2, r0 in limbs:
        model.add(veg.limb(p0, p1, r0, r0 * 0.62, "bark_light", segs=4))
        model.add(veg.limb(p1, p2, r0 * 0.62, 0.0, "dry_brush", segs=3))

    # Crown limb + 3 silhouette forks
    model.add(veg.limb(Vector((0.07, 0.14, 3.18)), Vector((-0.08, 0.76, 4.30)), 0.060, 0.0, "dry_brush", segs=3))
    model.add(veg.limb(Vector((0.88, 0.42, 2.05)), Vector((1.35, 0.85, 2.45)), 0.032, 0.0, "dry_brush", segs=3))
    model.add(veg.limb(Vector((-0.78, 0.78, 2.45)), Vector((-1.25, 0.36, 2.95)), 0.032, 0.0, "dry_brush", segs=3))
    model.add(veg.limb(Vector((-0.62, -0.68, 2.95)), Vector((-0.20, -1.12, 3.45)), 0.030, 0.0, "dry_brush", segs=3))

    model.add_collider("trunk", common.make_frustum(0.24, 0.10, -0.05, 3.25, segments=6, color="bark_light"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=(0.0, 0.0, 0.0)) else 1


if __name__ == "__main__":
    sys.exit(main())
