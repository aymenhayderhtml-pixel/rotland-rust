"""cactus_round: barrel cactus (~0.72 m tall, ~0.82 m wide) with ribs and 6 small bone_white spines."""
import math
import os
import sys

import bmesh
from mathutils import Vector

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402

NAME, CATEGORY, BUDGET = "cactus_round", "props", 200


def _barrel_body(segments=10):
    """Squashed ribbed barrel cactus sunk 5 cm into the ground."""
    bm = common.new_bm()
    profile = [
        (-0.05, 0.24),
        (0.14, 0.39),
        (0.35, 0.42),
        (0.54, 0.35),
        (0.65, 0.19),
    ]
    vert_rings = []
    for z_val, radius in profile:
        ring = []
        for i in range(segments):
            angle = 2.0 * math.pi * i / segments
            flute = 1.04 if i % 2 == 0 else 0.96
            rr = radius * flute
            ring.append(bm.verts.new((math.cos(angle) * rr, math.sin(angle) * rr, z_val)))
        vert_rings.append(ring)

    apex_v = bm.verts.new((0.0, 0.0, 0.67))
    base_face = bm.faces.new(list(reversed(vert_rings[0])))
    common.paint_faces(bm, [base_face], "cactus_dark")

    for r_idx in range(len(vert_rings) - 1):
        r0 = vert_rings[r_idx]
        r1 = vert_rings[r_idx + 1]
        for i in range(segments):
            j = (i + 1) % segments
            face = bm.faces.new([r0[i], r0[j], r1[j], r1[i]])
            color = "cactus_green" if i % 2 == 0 else "cactus_dark"
            common.paint_faces(bm, [face], color)

    top_ring = vert_rings[-1]
    for i in range(segments):
        j = (i + 1) % segments
        face = bm.faces.new([top_ring[i], top_ring[j], apex_v])
        color = "cactus_green" if i % 2 == 0 else "cactus_dark"
        common.paint_faces(bm, [face], color)

    bmesh.ops.recalc_face_normals(bm, faces=list(bm.faces))
    return bm


def _spine(origin, direction, base_r=0.022, length=0.11):
    """Tiny 4-sided pyramid spine pointing along `direction`."""
    bm = common.make_frustum(base_r, 0.0, 0.0, length, segments=4, color="bone_white")
    rot = Vector((0.0, 0.0, 1.0)).rotation_difference(direction.normalized()).to_matrix()
    bmesh.ops.rotate(bm, verts=list(bm.verts), cent=(0.0, 0.0, 0.0), matrix=rot)
    bmesh.ops.translate(bm, verts=list(bm.verts), vec=origin)
    return bm


def build(model):
    model.add(_barrel_body(segments=10))

    # 6 small pyramid spines distributed around the upper barrel and crown
    spine_specs = [
        (0.0, 0.36, 0.36, 0.55),
        (1.05, 0.34, 0.50, 0.85),
        (2.15, 0.37, 0.34, 0.45),
        (3.20, 0.33, 0.52, 0.90),
        (4.30, 0.36, 0.38, 0.50),
        (5.35, 0.25, 0.60, 1.25),
    ]
    for angle, radial, z_pos, z_dir in spine_specs:
        origin = Vector((math.cos(angle) * radial, math.sin(angle) * radial, z_pos))
        direction = Vector((math.cos(angle), math.sin(angle), z_dir))
        model.add(_spine(origin, direction))

    model.add_collider(
        "hull",
        common.make_icosphere(subdiv=1, radius=(0.41, 0.41, 0.35), loc=(0.0, 0.0, 0.31), color="cactus_green"),
    )


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=(0.0, 0.0, 0.0)) else 1


if __name__ == "__main__":
    sys.exit(main())
