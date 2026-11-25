"""cactus_tall: tall saguaro cactus (~4.1 m) with 2 bent arms and 8-sided alternating ribbed faces."""
import math
import os
import sys

import bmesh
from mathutils import Vector

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402

NAME, CATEGORY, BUDGET = "cactus_tall", "props", 300


def _ribbed_tube(rings, apex, segments=8):
    """Builds a closed 8-sided ribbed tube along (center, u_axis, v_axis, radius) rings to an apex."""
    bm = common.new_bm()
    vert_rings = []
    for center, u_axis, v_axis, radius in rings:
        ring = []
        for i in range(segments):
            angle = 2.0 * math.pi * i / segments
            offset = u_axis * (math.cos(angle) * radius) + v_axis * (math.sin(angle) * radius)
            ring.append(bm.verts.new(center + offset))
        vert_rings.append(ring)

    apex_v = bm.verts.new(apex)
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


def _build_arm(yaw, profile, apex_sz):
    """Builds one bent saguaro arm in the vertical plane at `yaw` (radians).

    `profile` is a list of (s_radial, z, tangent_s, tangent_z, radius).
    """
    fwd = Vector((math.cos(yaw), math.sin(yaw), 0.0))
    up = Vector((0.0, 0.0, 1.0))
    binormal = Vector((-math.sin(yaw), math.cos(yaw), 0.0))

    rings = []
    for s_val, z_val, ts, tz, radius in profile:
        center = fwd * s_val + up * z_val
        tangent = (fwd * ts + up * tz).normalized()
        normal = binormal.cross(tangent).normalized()
        rings.append((center, normal, binormal, radius))

    apex = fwd * apex_sz[0] + up * apex_sz[1]
    return _ribbed_tube(rings, apex, segments=8)


def build(model):
    # Main trunk: 4.10 m tall, sunk 5 cm into the ground, with subtle natural sway
    ux = Vector((1.0, 0.0, 0.0))
    uy = Vector((0.0, 1.0, 0.0))
    trunk_rings = [
        (Vector((0.00, 0.00, -0.05)), ux, uy, 0.24),
        (Vector((0.02, 0.01, 0.55)), ux, uy, 0.27),
        (Vector((0.04, 0.01, 1.65)), ux, uy, 0.28),
        (Vector((0.01, -0.01, 2.85)), ux, uy, 0.25),
        (Vector((-0.02, 0.00, 3.82)), ux, uy, 0.23),
        (Vector((-0.02, 0.00, 3.97)), ux, uy, 0.14),
    ]
    model.add(_ribbed_tube(trunk_rings, Vector((-0.02, 0.00, 4.06)), segments=8))

    # Lower primary arm (+X side, slightly rotated)
    arm_1_profile = [
        (0.08, 1.68, 1.0, 0.0, 0.17),
        (0.48, 1.68, 1.0, 0.0, 0.19),
        (0.735, 1.785, 0.707, 0.707, 0.20),
        (0.84, 2.04, 0.0, 1.0, 0.20),
        (0.84, 3.22, 0.0, 1.0, 0.18),
        (0.84, 3.35, 0.0, 1.0, 0.11),
    ]
    model.add(_build_arm(yaw=0.18, profile=arm_1_profile, apex_sz=(0.84, 3.43)))

    # Higher secondary arm (opposite side, offset yaw for a 3D silhouette)
    arm_2_profile = [
        (0.08, 2.15, 1.0, 0.0, 0.16),
        (0.646, 2.244, 0.707, 0.707, 0.18),
        (0.74, 2.47, 0.0, 1.0, 0.18),
        (0.74, 3.32, 0.0, 1.0, 0.16),
        (0.74, 3.44, 0.0, 1.0, 0.10),
    ]
    model.add(_build_arm(yaw=2.72, profile=arm_2_profile, apex_sz=(0.74, 3.51)))

    # Trunk collider
    model.add_collider("trunk", common.make_frustum(0.28, 0.24, -0.05, 4.0, segments=6, color="cactus_green"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=(0.0, 0.0, 0.0)) else 1


if __name__ == "__main__":
    sys.exit(main())
