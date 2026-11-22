"""sign_fallen: a street sign fallen over on the ground with a bent pole (<= 300 tris). Has a collider."""
import math
import os
import sys

from mathutils import Vector

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402

NAME, CATEGORY, BUDGET = "sign_fallen", "props", 300


def _continuous_pole():
    """A continuous bent metal pole lofted along a sequence of spine points so there are no gaps."""
    bm = common.new_bm()
    layer = bm.faces.layers.int[common.LAYER]

    # Concrete collar at the original post hole
    base = common.make_box((0.24, 0.24, 0.06), loc=(0.0, 0.0, 0.03), color="rock_mid")

    # Spine points:
    # 0: base ground stub
    # 1: sharp bend near ground
    # 2: first low bend along ground
    # 3: mid-span resting on ground
    # 4: upper section leading to sign
    spine = [
        Vector((0.0, 0.0, 0.02)),
        Vector((0.01, 0.10, 0.08)),
        Vector((0.05, 0.25, 0.04)),
        Vector((0.15, 1.10, 0.04)),
        Vector((0.26, 1.95, 0.04)),
        Vector((0.36, 2.75, 0.07)),
    ]

    half_w = 0.03
    half_h = 0.03

    rings = []
    for i, pt in enumerate(spine):
        # Calculate forward tangent
        if i < len(spine) - 1:
            tangent = (spine[i + 1] - pt).normalized()
        else:
            tangent = (pt - spine[i - 1]).normalized()

        # Up and right vectors
        up = Vector((0.0, 0.0, 1.0))
        right = tangent.cross(up)
        if right.length < 1e-4:
            right = Vector((1.0, 0.0, 0.0))
        else:
            right.normalize()
        local_up = right.cross(tangent).normalized()

        # 4 corners of square tube: bottom-left, bottom-right, top-right, top-left
        c0 = pt - right * half_w - local_up * half_h
        c1 = pt + right * half_w - local_up * half_h
        c2 = pt + right * half_w + local_up * half_h
        c3 = pt - right * half_w + local_up * half_h

        v0 = bm.verts.new(c0)
        v1 = bm.verts.new(c1)
        v2 = bm.verts.new(c2)
        v3 = bm.verts.new(c3)
        rings.append([v0, v1, v2, v3])

    # End cap at base
    f_start = bm.faces.new(list(reversed(rings[0])))
    f_start[layer] = common.palette.key_of("rust")

    # Connect quads along the spine
    for s in range(len(rings) - 1):
        r1 = rings[s]
        r2 = rings[s + 1]
        for i in range(4):
            j = (i + 1) % 4
            f = bm.faces.new([r1[i], r2[i], r2[j], r1[j]])
            col = "rust" if s < 2 else "metal"
            f[layer] = common.palette.key_of(col)

    # End cap at top
    f_end = bm.faces.new(rings[-1])
    f_end[layer] = common.palette.key_of("metal")

    common.bmesh.ops.recalc_face_normals(bm, faces=list(bm.faces))
    return common.combine([base, bm])


def _sign_plates():
    """Octagonal STOP sign lying flat/tilted on the ground, attached to the upper pole."""
    bms = []

    # Octagonal STOP sign: 8-sided prism, radius 0.38 m, thickness 0.015 m
    # Octagon backing: red
    stop_back = common.make_frustum(0.38, 0.38, 0.0, 0.015, segments=8, rot_z=math.pi / 8, color="red_paint")
    # Inner face: white border ring (proud by 12 mm to prevent coplanar z-fighting with backing)
    stop_white = common.make_frustum(0.34, 0.34, 0.015, 0.028, segments=8, rot_z=math.pi / 8, color="white")
    # Inner red center (proud by another 12 mm to prevent coplanar z-fighting with white ring)
    stop_inner = common.make_frustum(0.30, 0.30, 0.028, 0.042, segments=8, rot_z=math.pi / 8, color="red_paint")

    # Mounting bracket clamp on back of sign
    bracket = common.make_box((0.10, 0.40, 0.04), loc=(0.0, 0.0, -0.025), color="rust")

    # Bullet holes / damage marks
    hole1 = common.make_box((0.03, 0.03, 0.01), loc=(-0.10, 0.08, 0.045), color="gun_black")
    hole2 = common.make_box((0.03, 0.03, 0.01), loc=(0.12, -0.10, 0.045), color="gun_black")

    stop_combined = common.combine([stop_back, stop_white, stop_inner, bracket, hole1, hole2])

    # Position at the pole's upper end (y = 2.45, z = 0.09)
    # Lying face up, tilted gently by 12 degrees
    common.transform(stop_combined, rot=(0.18, 0.12, -0.15), loc=(0.34, 2.45, 0.09))
    bms.append(stop_combined)

    # Street name plate: rectangular green plate sticking past the stop sign
    street_back = common.make_box((0.55, 0.14, 0.02), loc=(0.0, 0.0, 0.01), color="leaf_dark")
    street_white = common.make_box((0.48, 0.08, 0.01), loc=(0.0, 0.0, 0.025), color="white")
    street_sign = common.combine([street_back, street_white])

    # Attach to pole tip near y = 2.85
    common.transform(street_sign, rot=(0.20, -0.15, 0.35), loc=(0.40, 2.85, 0.08))
    bms.append(street_sign)

    return common.combine(bms)


def build(model):
    model.add(_continuous_pole())
    model.add(_sign_plates())
    # Low box collider covering the fallen sign and pole
    model.add_collider("fallen", common.make_box((0.9, 3.1, 0.28), loc=(0.22, 1.45, 0.14), color="white"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
