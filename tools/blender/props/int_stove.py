"""int_stove: kitchen stove/range facing +Y (0.75 x 0.65 x 0.90 m + backsplash)."""
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402

NAME, CATEGORY, BUDGET = "int_stove", "props", 300


def build(model):
    # Main body
    model.add(common.make_box((0.75, 0.65, 0.88), loc=(0.0, 0.0, 0.44), color="white"))
    # Cooktop plate
    model.add(common.make_box((0.75, 0.65, 0.03), loc=(0.0, 0.0, 0.895), color="gun_black"))
    # Four burners
    for bx in (-0.18, 0.18):
        for by in (-0.14, 0.14):
            model.add(common.make_frustum(0.085, 0.085, 0.91, 0.93, segments=6, center=(bx, by), color="rock_dark"))
    # Backsplash at -Y
    model.add(common.make_box((0.75, 0.06, 0.18), loc=(0.0, -0.295, 0.99), color="white"))
    # Oven door and handle on +Y
    model.add(common.make_box((0.58, 0.03, 0.48), loc=(0.0, 0.33, 0.38), color="rock_dark"))
    model.add(common.make_box((0.48, 0.04, 0.035), loc=(0.0, 0.355, 0.67), color="metal"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
