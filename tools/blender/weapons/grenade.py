"""grenade: round green frag grenade with a ridged body, spoon lever and brass pin ring (~0.1 m). One hand."""
import math
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402
from parts import box, loft_y  # noqa: E402

NAME, CATEGORY, BUDGET = "grenade", "weapons", 600
GRIP = (0.0, 0.0, 0.0)  # center of the grenade body


def _loft_z(profile, color, segments=10):
    bm = loft_y(profile, color, segments=segments)
    common.transform(bm, rot=(math.pi / 2, 0.0, 0.0))
    return bm


def build(model):
    # Round green body
    model.add(_loft_z([(-0.045, 0.016), (-0.032, 0.028), (0.0, 0.032), (0.030, 0.026), (0.042, 0.015)],
                      "leaf_dark", segments=10))

    # Horizontal and vertical frag ridges
    for z, r in ((-0.016, 0.0315), (0.0, 0.0335), (0.016, 0.0315)):
        model.add(common.make_frustum(r, r, z - 0.003, z + 0.003, segments=10, color="leaf_mid"))
    for deg in (0, 45, 90, 135):
        bm_rib = common.make_box((0.066, 0.007, 0.056), color="leaf_mid")
        common.transform(bm_rib, rot=(0.0, 0.0, math.radians(deg)))
        model.add(bm_rib)

    # Metal fuse neck and top cap
    model.add(common.make_frustum(0.014, 0.014, 0.040, 0.062, segments=8, color="metal"))
    model.add(common.make_frustum(0.016, 0.016, 0.062, 0.070, segments=8, color="gun_black"))

    # Spoon lever on the side
    model.add(box((0.014, 0.024, 0.005), (0.0, -0.014, 0.068), "metal"))
    bm_spoon = common.make_box((0.014, 0.005, 0.060), color="metal")
    common.transform(bm_spoon, rot=(0.14, 0.0, 0.0), loc=(0.0, -0.032, 0.036))
    model.add(bm_spoon)

    # Pull pin and brass ring
    model.add(box((0.018, 0.004, 0.004), (0.010, 0.0, 0.058), "brass"))
    for i in range(6):
        ang = i * math.pi / 3.0
        bm_seg = common.make_box((0.010, 0.003, 0.003), color="brass")
        common.transform(
            bm_seg,
            rot=(0.0, ang + math.pi / 2.0, 0.0),
            loc=(0.022 + math.cos(ang) * 0.011, 0.0, 0.058 + math.sin(ang) * 0.011),
        )
        model.add(bm_seg)


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=GRIP) else 1


if __name__ == "__main__":
    sys.exit(main())
