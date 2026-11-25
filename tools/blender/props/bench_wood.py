"""bench_wood: outdoor wooden bench with slatted seat and backrest (<= 260 tris)."""
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402

NAME, CATEGORY, BUDGET = "bench_wood", "props", 260


def build(model):
    # Two leg frames and cross braces
    for sx in (-0.62, 0.62):
        model.add(common.make_box((0.07, 0.07, 0.44), loc=(sx, -0.18, 0.22), color="bark_dark"))
        model.add(common.make_box((0.07, 0.07, 0.44), loc=(sx, 0.18, 0.22), color="bark_dark"))
        model.add(common.make_box((0.06, 0.44, 0.05), loc=(sx, 0.0, 0.42), color="bark_dark"))
        # Backrest upright
        post = common.make_box((0.06, 0.06, 0.48), color="bark_dark")
        common.transform(post, loc=(sx, 0.22, 0.65), rot=(-0.15, 0.0, 0.0))
        model.add(post)

    # Three seat planks
    for sy in (-0.15, 0.0, 0.15):
        model.add(common.make_box((1.50, 0.13, 0.04), loc=(0.0, sy, 0.45), color="wood_crate"))

    # Two backrest planks
    for sz, sy in ((0.64, 0.21), (0.81, 0.235)):
        plank = common.make_box((1.50, 0.035, 0.12), color="wood_crate")
        common.transform(plank, loc=(0.0, sy, sz), rot=(-0.15, 0.0, 0.0))
        model.add(plank)

    model.add_collider("box", common.make_box((1.50, 0.54, 0.88), loc=(0.0, 0.02, 0.44), color="wood_crate"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
