"""int_shelf: wooden storage shelf facing +Y (1.1 x 0.38 x 1.8 m)."""
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402

NAME, CATEGORY, BUDGET = "int_shelf", "props", 300


def build(model):
    # Side panels
    for sx in (-0.525, 0.525):
        model.add(common.make_box((0.05, 0.38, 1.80), loc=(sx, 0.0, 0.90), color="bark_dark"))
    # Back panel
    model.add(common.make_box((1.00, 0.03, 1.80), loc=(0.0, -0.175, 0.90), color="bark_dark"))
    # Five shelves
    for sz in (0.06, 0.48, 0.90, 1.32, 1.77):
        model.add(common.make_box((1.00, 0.35, 0.04), loc=(0.0, 0.015, sz), color="wood_crate"))
    # Small items on shelves
    model.add(common.make_box((0.24, 0.20, 0.22), loc=(-0.25, 0.02, 0.61), color="red_paint"))
    model.add(common.make_box((0.18, 0.18, 0.26), loc=(0.22, 0.02, 1.05), color="shirt_blue"))
    model.add(common.make_box((0.28, 0.22, 0.18), loc=(-0.10, 0.02, 1.43), color="metal"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
