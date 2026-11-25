"""propane_tank: upright white residential propane cylinder with collar and brass valve (<= 320 tris)."""
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402

NAME, CATEGORY, BUDGET = "propane_tank", "props", 320


def build(model):
    # Base foot ring
    model.add(common.make_frustum(0.18, 0.19, 0.0, 0.08, segments=10, color="rust"))

    # Main cylinder body + domed top
    model.add(common.make_frustum(0.20, 0.20, 0.08, 0.88, segments=12, color="car_white"))
    model.add(common.make_frustum(0.20, 0.10, 0.88, 1.00, segments=12, color="car_white"))
    model.add(common.make_frustum(0.10, 0.0, 1.00, 1.03, segments=12, color="car_white"))

    # Protective top collar ring (open front/back via two curved-style posts + top rim)
    for sx in (-0.11, 0.11):
        model.add(common.make_box((0.03, 0.06, 0.14), loc=(sx, 0.0, 1.05), color="metal"))
    model.add(common.make_frustum(0.12, 0.12, 1.11, 1.15, segments=10, color="metal"))

    # Brass valve and red handwheel
    model.add(common.make_frustum(0.025, 0.025, 1.01, 1.10, segments=6, color="brass"))
    model.add(common.make_frustum(0.045, 0.045, 1.10, 1.12, segments=8, color="red_paint"))

    model.add_collider("body", common.make_frustum(0.20, 0.20, 0.0, 1.15, segments=8, color="car_white"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
