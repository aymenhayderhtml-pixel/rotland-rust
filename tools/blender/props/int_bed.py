"""int_bed: single bed (1.0 x 2.0 m), headboard at -Y, facing +Y."""
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402

NAME, CATEGORY, BUDGET = "int_bed", "props", 300


def build(model):
    # Four corner legs
    for sx in (-0.45, 0.45):
        for sy in (-0.95, 0.95):
            model.add(common.make_box((0.07, 0.07, 0.28), loc=(sx, sy, 0.14), color="bark_dark"))
    # Frame base
    model.add(common.make_box((0.98, 1.98, 0.12), loc=(0.0, 0.0, 0.31), color="bark_dark"))
    # Headboard (-Y) and footboard (+Y)
    model.add(common.make_box((0.98, 0.08, 0.60), loc=(0.0, -0.95, 0.67), color="bark_light"))
    model.add(common.make_box((0.98, 0.06, 0.22), loc=(0.0, 0.96, 0.48), color="bark_light"))
    # Mattress
    model.add(common.make_box((0.90, 1.84, 0.16), loc=(0.0, 0.01, 0.45), color="white"))
    # Blanket over lower two-thirds
    model.add(common.make_box((0.92, 1.18, 0.04), loc=(0.0, 0.34, 0.54), color="shirt_blue"))
    # Pillow at headboard
    model.add(common.make_box((0.60, 0.34, 0.09), loc=(0.0, -0.68, 0.575), color="white"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
