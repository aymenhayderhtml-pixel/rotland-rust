"""crossbow: hunting crossbow with a short rifle stock, 0.6 m bow arm, string and front stirrup. Two-handed."""
import math
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
from mathutils import Vector  # noqa: E402
import common  # noqa: E402
from parts import box, hull_side, rbox  # noqa: E402

NAME, CATEGORY, BUDGET = "crossbow", "weapons", 600
GRIP = (0.0, -0.12, -0.05)  # pistol grip under the rail


def _cyl_between(p0, p1, r, color, segments=6):
    v0, v1 = Vector(p0), Vector(p1)
    diff = v1 - v0
    length = diff.length
    if length < 1e-5:
        return common.new_bm()
    frust = common.make_frustum(r, r, 0.0, length, segments=segments, color=color)
    rot_quat = Vector((0.0, 0.0, 1.0)).rotation_difference(diff.normalized())
    common.transform(frust, loc=v0, rot=rot_quat.to_euler())
    return frust


def build(model):
    # Short rifle-style wooden stock, butt pad, pistol grip, trigger guard and trigger
    model.add(hull_side([(-0.13, 0.035), (-0.34, 0.045), (-0.34, -0.055), (-0.22, -0.030), (-0.13, -0.020)],
                        0.036, "wood_crate"))
    model.add(rbox((0.038, 0.016, 0.102), (0.0, -0.348, -0.005), "gun_black", 0.3))
    model.add(hull_side([(-0.145, 0.0), (-0.090, 0.0), (-0.100, -0.085), (-0.145, -0.085)], 0.030, "wood_crate"))
    model.add(box((0.009, 0.050, 0.006), (0.0, -0.075, -0.045), "gun_black"))
    model.add(box((0.006, 0.006, 0.020), (0.0, -0.075, -0.030), "metal"))

    # Forend rail, metal bolt track and rear latch / sight bridge
    model.add(rbox((0.040, 0.44, 0.032), (0.0, 0.09, 0.010), "wood_crate", 0.2))
    model.add(box((0.026, 0.42, 0.010), (0.0, 0.10, 0.030), "gun_black"))
    model.add(rbox((0.032, 0.050, 0.024), (0.0, -0.08, 0.044), "metal", 0.25))

    # Front prod riser and 0.6 m wide swept bow limbs
    model.add(rbox((0.060, 0.050, 0.042), (0.0, 0.30, 0.018), "gun_black", 0.2))
    limb_pts = [(0.025, 0.300), (0.170, 0.268), (0.300, 0.210)]
    for side in (-1.0, 1.0):
        for i in range(len(limb_pts) - 1):
            x0, y0 = side * limb_pts[i][0], limb_pts[i][1]
            x1, y1 = side * limb_pts[i + 1][0], limb_pts[i + 1][1]
            length = math.hypot(x1 - x0, y1 - y0) * 1.04
            thick_y = 0.024 * (1.0 - 0.30 * i)
            height_z = 0.032 * (1.0 - 0.25 * i)
            ang = math.atan2(y1 - y0, x1 - x0)
            bm_limb = common.make_box((length, thick_y, height_z), color="gun_black")
            common.transform(bm_limb, loc=((x0 + x1) / 2, (y0 + y1) / 2, 0.022), rot=(0.0, 0.0, ang))
            model.add(bm_limb)
        model.add(box((0.020, 0.020, 0.026), (side * 0.300, 0.210, 0.022), "bark_dark"))

    # Braced bowstring from both limb tips back to the latch
    latch = (0.0, -0.050, 0.036)
    model.add(_cyl_between((-0.300, 0.210, 0.026), latch, 0.004, "white", segments=6))
    model.add(_cyl_between(latch, (0.300, 0.210, 0.026), 0.004, "white", segments=6))

    # Front foot stirrup (~0.75 m total length from -0.35 to +0.40)
    model.add(box((0.008, 0.095, 0.010), (-0.045, 0.355, 0.015), "metal"))
    model.add(box((0.008, 0.095, 0.010), (0.045, 0.355, 0.015), "metal"))
    model.add(box((0.098, 0.010, 0.010), (0.0, 0.400, 0.015), "metal"))

    model.add_empty("grip_L", (0.0, 0.16, -0.01))
    model.add_empty("mount_optic", (0.0, -0.08, 0.056))
    model.add_empty("mount_magazine", (0.020, 0.08, 0.005))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=GRIP) else 1


if __name__ == "__main__":
    sys.exit(main())
