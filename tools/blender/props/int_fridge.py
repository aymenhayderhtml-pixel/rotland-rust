"""int_fridge: refrigerator facing +Y (0.75 x 0.70 x 1.80 m)."""
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402

NAME, CATEGORY, BUDGET = "int_fridge", "props", 300


def build(model):
    # Bottom kickplate
    model.add(common.make_box((0.73, 0.66, 0.08), loc=(0.0, 0.0, 0.04), color="rock_dark"))
    # Main body
    model.add(common.make_box((0.75, 0.68, 1.72), loc=(0.0, -0.01, 0.94), color="white"))
    # Freezer door (upper) & fridge door (lower) on +Y
    model.add(common.make_box((0.72, 0.04, 0.50), loc=(0.0, 0.34, 1.51), color="white"))
    model.add(common.make_box((0.72, 0.04, 1.12), loc=(0.0, 0.34, 0.66), color="white"))
    # Door handles
    model.add(common.make_box((0.04, 0.04, 0.24), loc=(-0.27, 0.37, 1.48), color="metal"))
    model.add(common.make_box((0.04, 0.04, 0.38), loc=(-0.27, 0.37, 0.92), color="metal"))
    # Top trim cap
    model.add(common.make_box((0.75, 0.68, 0.03), loc=(0.0, -0.01, 1.815), color="metal"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
