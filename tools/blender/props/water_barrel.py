"""water_barrel: wooden rain-water barrel with metal hoops, lid, and brass spigot (<= 320 tris)."""
import math
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402

NAME, CATEGORY, BUDGET = "water_barrel", "props", 320


def build(model):
    # Bulged barrel body in two halves
    model.add(common.make_frustum(0.28, 0.33, 0.0, 0.47, segments=10, color="wood_crate"))
    model.add(common.make_frustum(0.33, 0.28, 0.47, 0.94, segments=10, color="wood_crate"))

    # Metal hoops
    model.add(common.make_frustum(0.305, 0.315, 0.14, 0.20, segments=10, color="metal"))
    model.add(common.make_frustum(0.315, 0.305, 0.74, 0.80, segments=10, color="metal"))

    # Lid
    model.add(common.make_frustum(0.27, 0.27, 0.92, 0.96, segments=10, color="bark_dark"))

    # Brass spigot near the bottom front
    spout = common.make_frustum(0.028, 0.024, 0.0, 0.10, segments=6, color="brass")
    common.transform(spout, loc=(0.0, -0.30, 0.18), rot=(math.pi * 0.5, 0.0, 0.0))
    model.add(spout)

    model.add_collider("body", common.make_frustum(0.33, 0.33, 0.0, 0.96, segments=8, color="wood_crate"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
