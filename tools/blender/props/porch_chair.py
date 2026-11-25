"""porch_chair: rustic wooden porch chair with slatted back and armrests (<= 260 tris)."""
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402

NAME, CATEGORY, BUDGET = "porch_chair", "props", 260


def build(model):
    # Four legs
    for sx in (-0.24, 0.24):
        for sy in (-0.22, 0.22):
            model.add(common.make_box((0.05, 0.05, 0.44), loc=(sx, sy, 0.22), color="bark_dark"))

    # Seat planks
    for sy in (-0.15, 0.0, 0.15):
        model.add(common.make_box((0.54, 0.13, 0.04), loc=(0.0, sy, 0.44), color="wood_crate"))

    # Armrests and front supports
    for sx in (-0.25, 0.25):
        model.add(common.make_box((0.05, 0.05, 0.22), loc=(sx, -0.20, 0.55), color="bark_dark"))
        model.add(common.make_box((0.07, 0.50, 0.04), loc=(sx, 0.01, 0.67), color="wood_crate"))

    # Tilted backrest uprights and horizontal slats
    for sx in (-0.24, 0.24):
        post = common.make_box((0.05, 0.05, 0.52), color="bark_dark")
        common.transform(post, loc=(sx, 0.25, 0.68), rot=(-0.14, 0.0, 0.0))
        model.add(post)

    for sz, sy in ((0.60, 0.24), (0.75, 0.26), (0.90, 0.28)):
        slat = common.make_box((0.46, 0.035, 0.09), color="wood_crate")
        common.transform(slat, loc=(0.0, sy, sz), rot=(-0.14, 0.0, 0.0))
        model.add(slat)

    model.add_collider("box", common.make_box((0.58, 0.56, 0.94), loc=(0.0, 0.02, 0.47), color="wood_crate"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
