"""cattle_skull: bleached cattle skull with curved horns (~0.6 m across), lying on the ground."""
import math
import os
import sys

import bmesh
from mathutils import Euler, Vector

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402

NAME, CATEGORY, BUDGET = "cattle_skull", "props", 120


def _skull_cranium():
    """5-ring 6-sided sculpted cranium and tapered snout, with recessed dark eye-socket facets."""
    bm = common.new_bm()
    # Cross-section offsets in (x, z) around each y station:
    # 0: top-right, 1: top-left, 2: side-left, 3: bottom-left, 4: bottom-right, 5: side-right
    stations = [
        # Back of cranium
        (-0.18, [(0.055, 0.115), (-0.055, 0.115), (-0.075, 0.055),
                 (-0.048, 0.005), (0.048, 0.005), (0.075, 0.055)]),
        # Wide brow ridge
        (-0.05, [(0.075, 0.135), (-0.075, 0.135), (-0.105, 0.070),
                 (-0.062, 0.000), (0.062, 0.000), (0.105, 0.070)]),
        # Recessed eye sockets / cheek pinch
        (0.04, [(0.048, 0.105), (-0.048, 0.105), (-0.066, 0.052),
                (-0.050, 0.002), (0.050, 0.002), (0.066, 0.052)]),
        # Narrow nasal bridge
        (0.15, [(0.036, 0.074), (-0.036, 0.074), (-0.052, 0.036),
                (-0.040, 0.002), (0.040, 0.002), (0.052, 0.036)]),
        # Tapered snout tip
        (0.24, [(0.024, 0.042), (-0.024, 0.042), (-0.036, 0.020),
                (-0.026, 0.000), (0.026, 0.000), (0.036, 0.020)]),
    ]
    rings = [[bm.verts.new((x, y, z)) for x, z in pts] for y, pts in stations]

    for r_idx in range(len(rings) - 1):
        r0 = rings[r_idx]
        r1 = rings[r_idx + 1]
        for i in range(6):
            j = (i + 1) % 6
            face = bm.faces.new([r0[j], r0[i], r1[i], r1[j]])
            # Upper-side recessed quads between brow (1) and cheek pinch (2) read as dark eye sockets
            is_socket = r_idx == 1 and i in (1, 5)
            common.paint_faces(bm, [face], "sandstone_dark" if is_socket else "bone_white")

    common.paint_faces(bm, [bm.faces.new(rings[0])], "bone_white")
    common.paint_faces(bm, [bm.faces.new(list(reversed(rings[-1])))], "bone_white")
    bmesh.ops.recalc_face_normals(bm, faces=list(bm.faces))
    return bm


def _horn(side):
    """Curved 4-sided horn sweeping outward, forward and upward (`side` is -1 or +1)."""
    bm = common.new_bm()
    pts = [
        (Vector((side * 0.075, -0.095, 0.095)), 0.036),
        (Vector((side * 0.205, -0.065, 0.115)), 0.027),
        (Vector((side * 0.295, 0.015, 0.170)), 0.016),
    ]
    tip = Vector((side * 0.285, 0.095, 0.245))
    centers = [p for p, _ in pts] + [tip]

    rings = []
    for idx, (center, radius) in enumerate(pts):
        tangent = (centers[idx + 1] - centers[max(0, idx - 1)]).normalized()
        ref = Vector((0.0, 0.0, 1.0)) if abs(tangent.z) < 0.9 else Vector((0.0, 1.0, 0.0))
        u_axis = ref.cross(tangent).normalized()
        v_axis = tangent.cross(u_axis).normalized()
        ring = []
        for i in range(4):
            angle = math.pi * 0.25 + 0.5 * math.pi * i
            ring.append(bm.verts.new(center + (u_axis * math.cos(angle) + v_axis * math.sin(angle)) * radius))
        rings.append(ring)

    tip_v = bm.verts.new(tip)
    bm.faces.new(list(reversed(rings[0])))
    for r_idx in range(len(rings) - 1):
        r0 = rings[r_idx]
        r1 = rings[r_idx + 1]
        for i in range(4):
            j = (i + 1) % 4
            bm.faces.new([r0[i], r0[j], r1[j], r1[i]])
    for i in range(4):
        j = (i + 1) % 4
        bm.faces.new([rings[-1][i], rings[-1][j], tip_v])

    bmesh.ops.recalc_face_normals(bm, faces=list(bm.faces))
    common.paint(bm, "bone_white")
    return bm


def build(model):
    skull = common.combine([_skull_cranium(), _horn(-1), _horn(1)])
    # Slight natural rest tilt on the desert ground, sunk ~4.5 cm at the lowest contact
    tilt = Euler((0.08, 0.10, 0.0), "XYZ").to_matrix()
    bmesh.ops.rotate(skull, verts=list(skull.verts), cent=(0.0, 0.0, 0.0), matrix=tilt)
    min_z = min(v.co.z for v in skull.verts)
    bmesh.ops.translate(skull, verts=list(skull.verts), vec=(0.0, 0.0, -min_z - 0.045))
    model.add(skull)


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=(0.0, 0.0, 0.0)) else 1


if __name__ == "__main__":
    sys.exit(main())
