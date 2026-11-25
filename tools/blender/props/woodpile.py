"""woodpile: stacked firewood logs on two ground rails (<= 350 tris)."""
import math
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402

NAME, CATEGORY, BUDGET = "woodpile", "props", 350


def build(model):
    # Two bottom runner rails
    for sy in (-0.18, 0.18):
        model.add(common.make_box((1.05, 0.07, 0.06), loc=(0.0, sy, 0.03), color="bark_dark"))

    # Stacked logs lying along Y
    rows = [
        # (z, xs, color)
        (0.14, (-0.38, -0.19, 0.0, 0.19, 0.38), "bark_dark"),
        (0.29, (-0.29, -0.10, 0.10, 0.29), "bark_light"),
        (0.44, (-0.19, 0.0, 0.19), "wood_crate"),
        (0.58, (-0.09, 0.09), "bark_dark"),
    ]
    for row_idx, (z, xs, default_col) in enumerate(rows):
        for i, x in enumerate(xs):
            col = "wood_crate" if (row_idx + i) % 3 == 2 else default_col
            length = 0.52 + 0.04 * ((i % 2) - 0.5)
            log = common.make_frustum(0.082, 0.078, -length * 0.5, length * 0.5, segments=6, color=col)
            common.transform(log, loc=(x, 0.0, z), rot=(math.pi * 0.5, 0.0, 0.03 * ((i % 3) - 1)))
            model.add(log)

    model.add_collider("box", common.make_box((1.05, 0.56, 0.64), loc=(0.0, 0.0, 0.32), color="bark_dark"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
