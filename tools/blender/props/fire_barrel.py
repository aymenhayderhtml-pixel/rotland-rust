"""fire_barrel: rusted burn barrel with soot rim, charred wood, and glowing embers (<= 350 tris)."""
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402

NAME, CATEGORY, BUDGET = "fire_barrel", "props", 350


def build(model):
    # Main rusted barrel body
    model.add(common.make_frustum(0.30, 0.30, 0.0, 0.72, segments=12, color="rust"))
    model.add(common.make_frustum(0.30, 0.30, 0.72, 0.90, segments=12, color="gun_black"))

    # Metal hoops
    for z0 in (0.16, 0.52):
        model.add(common.make_frustum(0.312, 0.312, z0, z0 + 0.04, segments=12, color="metal"))

    # Ash bed at top
    model.add(common.make_frustum(0.26, 0.26, 0.80, 0.84, segments=8, color="rock_dark"))

    # Crossed charred logs sticking out of the top
    for rot_z, tilt_x in ((0.4, 0.65), (2.1, 0.55), (-1.1, 0.70)):
        log = common.make_box((0.07, 0.07, 0.38), color="bark_dark")
        common.transform(log, loc=(0.0, 0.0, 0.88), rot=(tilt_x, 0.0, rot_z))
        model.add(log)

    # Glowing ember chunks
    for x, y, z, col in (
        (0.06, -0.05, 0.86, "carrot_orange"),
        (-0.07, 0.04, 0.85, "red_paint"),
        (0.01, 0.06, 0.87, "carrot_orange"),
    ):
        model.add(common.make_box((0.09, 0.08, 0.06), loc=(x, y, z), color=col))

    model.add_collider("body", common.make_frustum(0.30, 0.30, 0.0, 0.90, segments=8, color="rust"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
