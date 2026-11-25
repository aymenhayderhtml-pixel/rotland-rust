"""tree_desert_dead_lod2: far low-poly LOD for twisted desert dead tree (<= 40 tris)."""
import math
import os
import sys

import bmesh
from mathutils import Vector

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402
import veg  # noqa: E402

NAME, CATEGORY, BUDGET = "tree_desert_dead_lod2", "props", 40


def _bent_trunk():
    """3-ring 4-sided bent trunk (20 tris): bark_light base, dry_brush upper."""
    bm = common.new_bm()
    stations = [
        (Vector((0.00, 0.00, -0.05)), 0.25, 0.00),
        (Vector((0.10, 0.12, 1.45)), 0.16, 0.45),
        (Vector((0.06, 0.14, 3.25)), 0.07, 0.85),
    ]
    rings = []
    for center, radius, twist in stations:
        ring = []
        for i in range(4):
            angle = twist + 0.5 * math.pi * i
            ring.append(bm.verts.new(center + Vector((math.cos(angle) * radius, math.sin(angle) * radius, 0.0))))
        rings.append(ring)

    common.paint_faces(bm, [bm.faces.new(list(reversed(rings[0])))], "bark_light")
    common.paint_faces(bm, [bm.faces.new(rings[-1])], "dry_brush")

    for r_idx in range(len(rings) - 1):
        r0 = rings[r_idx]
        r1 = rings[r_idx + 1]
        color = "bark_light" if r_idx == 0 else "dry_brush"
        for i in range(4):
            j = (i + 1) % 4
            face = bm.faces.new([r0[i], r0[j], r1[j], r1[i]])
            common.paint_faces(bm, [face], color)

    bmesh.ops.recalc_face_normals(bm, faces=list(bm.faces))
    return bm


def build(model):
    model.add(_bent_trunk())  # 20 tris

    # 4 tapered main limbs matching LOD0/LOD1 silhouette (4 tris each = 16 tris; total 36 <= 40)
    for p0, p1, r0 in [
        (Vector((0.10, 0.10, 1.40)), Vector((1.55, 0.18, 2.75)), 0.085),
        (Vector((0.06, 0.14, 1.80)), Vector((-1.42, 0.98, 3.15)), 0.080),
        (Vector((0.04, 0.14, 2.25)), Vector((-1.10, -1.08, 3.68)), 0.075),
        (Vector((0.06, 0.14, 2.80)), Vector((1.05, -0.95, 4.10)), 0.065),
    ]:
        model.add(veg.limb(p0, p1, r0, 0.0, "dry_brush", segs=3))

    model.add_collider("trunk", common.make_frustum(0.24, 0.10, -0.05, 3.25, segments=6, color="bark_light"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=(0.0, 0.0, 0.0)) else 1


if __name__ == "__main__":
    sys.exit(main())
