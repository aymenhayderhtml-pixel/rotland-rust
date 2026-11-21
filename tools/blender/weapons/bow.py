"""bow: recurve hunting bow with a wrapped riser, laminated limbs and a braced string. Two-handed."""
import math
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
from mathutils import Vector  # noqa: E402
import common  # noqa: E402
from parts import box, cyl_y, loft_y, prism_yz, rbox  # noqa: E402,F401

NAME, CATEGORY, BUDGET = "bow", "weapons", 800
GRIP = (0.0, 0.0, 0.0)  # center of the riser grip


def _cyl_between(p0, p1, r, color, segments=6):
    v0, v1 = Vector(p0), Vector(p1)
    diff = v1 - v0
    length = diff.length
    if length < 1e-5:
        return common.new_bm()
    direction = diff.normalized()
    frust = common.make_frustum(r, r, 0.0, length, segments=segments, color=color)
    rot_quat = Vector((0.0, 0.0, 1.0)).rotation_difference(direction)
    common.transform(frust, loc=v0, rot=rot_quat.to_euler())
    return frust


def build(model):
    # Central riser: sculpted wood with arrow pass and leather grip wrap
    model.add(loft_y([(-0.016, 0.014), (-0.008, 0.016), (0.008, 0.016), (0.016, 0.014)],
                     "wood_crate", x=0.0, z=0.0, segments=8))
    # Ergonomic leather grip
    model.add(rbox((0.026, 0.034, 0.12), (0.0, -0.008, 0.0), "leather", 0.3))
    for z in (-0.04, -0.02, 0.0, 0.02, 0.04):
        model.add(box((0.028, 0.036, 0.008), (0.0, -0.008, z), "bark_dark"))
    # Arrow shelf / rest on the side
    model.add(box((0.014, 0.025, 0.008), (-0.016, 0.002, 0.035), "metal"))

    # Upper and lower recurve limbs
    # Recurve profile: sweeps back (-Y) then curves forward (+Y) at the tips
    upper_profile = [
        (0.000, 0.060),
        (-0.024, 0.160),
        (-0.060, 0.280),
        (-0.095, 0.400),
        (-0.090, 0.490),
        (-0.055, 0.550),
        (-0.010, 0.585),
    ]
    # Build upper limb (laminated wood belly and dark backing)
    for i in range(len(upper_profile) - 1):
        y0, z0 = upper_profile[i]
        y1, z1 = upper_profile[i + 1]
        t = i / (len(upper_profile) - 1)
        w = 0.034 * (1.0 - 0.40 * t)
        th = 0.014 * (1.0 - 0.35 * t)
        my = (y0 + y1) / 2
        mz = (z0 + z1) / 2
        ang = math.atan2(y1 - y0, z1 - z0)
        bm_belly = common.make_box((w, th, math.hypot(y1 - y0, z1 - z0) * 1.05), color="wood_crate")
        common.transform(bm_belly, loc=(0.0, my - 0.002, mz), rot=(-ang, 0.0, 0.0))
        model.add(bm_belly)
        bm_back = common.make_box((w * 1.02, 0.003, math.hypot(y1 - y0, z1 - z0) * 1.05), color="bark_dark")
        common.transform(bm_back, loc=(0.0, my + th * 0.48, mz), rot=(-ang, 0.0, 0.0))
        model.add(bm_back)

    # Upper tip nock
    model.add(box((0.020, 0.020, 0.025), (0.0, -0.005, 0.590), "bark_dark"))

    # Lower limb (mirror of upper limb along Z)
    lower_profile = [(y, -z) for y, z in upper_profile]
    for i in range(len(lower_profile) - 1):
        y0, z0 = lower_profile[i]
        y1, z1 = lower_profile[i + 1]
        t = i / (len(lower_profile) - 1)
        w = 0.034 * (1.0 - 0.40 * t)
        th = 0.014 * (1.0 - 0.35 * t)
        my = (y0 + y1) / 2
        mz = (z0 + z1) / 2
        ang = math.atan2(y1 - y0, z1 - z0)
        bm_belly = common.make_box((w, th, math.hypot(y1 - y0, z1 - z0) * 1.05), color="wood_crate")
        common.transform(bm_belly, loc=(0.0, my - 0.002, mz), rot=(-ang, 0.0, 0.0))
        model.add(bm_belly)
        bm_back = common.make_box((w * 1.02, 0.003, math.hypot(y1 - y0, z1 - z0) * 1.05), color="bark_dark")
        common.transform(bm_back, loc=(0.0, my + th * 0.48, mz), rot=(-ang, 0.0, 0.0))
        model.add(bm_back)

    # Lower tip nock
    model.add(box((0.020, 0.020, 0.025), (0.0, -0.005, -0.590), "bark_dark"))

    # Bowstring under tension: top tip to nock point to bottom tip
    top_nock = (0.0, -0.008, 0.585)
    center_nock = (0.0, -0.150, 0.0)
    bottom_nock = (0.0, -0.008, -0.585)
    model.add(_cyl_between(top_nock, center_nock, 0.0045, "white", segments=6))
    model.add(_cyl_between(center_nock, bottom_nock, 0.0045, "white", segments=6))
    # Serving / brass nocking point
    model.add(_cyl_between((0.0, -0.150, -0.035), (0.0, -0.150, 0.035), 0.007, "gun_black", segments=6))
    model.add(box((0.012, 0.012, 0.010), (0.0, -0.150, 0.015), "brass"))

    # Off-hand grip / draw point
    model.add_empty("grip_L", (0.0, -0.150, 0.0))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=GRIP) else 1


if __name__ == "__main__":
    sys.exit(main())
