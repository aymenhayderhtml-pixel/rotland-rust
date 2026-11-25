"""int_rug: woven floor rug (1.8 x 1.2 m, 0.02 m thick)."""
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402

NAME, CATEGORY, BUDGET = "int_rug", "props", 300


def build(model):
    # Outer border
    model.add(common.make_box((1.80, 1.20, 0.016), loc=(0.0, 0.0, 0.008), color="red_paint"))
    # Inner field
    model.add(common.make_box((1.48, 0.90, 0.018), loc=(0.0, 0.0, 0.009), color="canvas_khaki"))
    # Center accent band
    model.add(common.make_box((0.90, 0.36, 0.020), loc=(0.0, 0.0, 0.010), color="shirt_blue"))
    # End fringe strips
    for sx in (-0.93, 0.93):
        model.add(common.make_box((0.06, 1.14, 0.012), loc=(sx, 0.0, 0.006), color="white"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
