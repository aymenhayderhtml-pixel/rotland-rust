"""int_chair: wooden chair facing +Y (seat 0.45 m, back 0.90 m)."""
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402

NAME, CATEGORY, BUDGET = "int_chair", "props", 300


def build(model):
    # Seat: 0.45 x 0.45 x 0.04 m
    model.add(common.make_box((0.45, 0.45, 0.04), loc=(0.0, 0.0, 0.45), color="wood_crate"))
    # Four legs
    for sx in (-0.19, 0.19):
        for sy in (-0.19, 0.19):
            model.add(common.make_box((0.045, 0.045, 0.43), loc=(sx, sy, 0.215), color="bark_dark"))
    # Backrest posts (at -Y)
    for sx in (-0.19, 0.19):
        model.add(common.make_box((0.045, 0.045, 0.43), loc=(sx, -0.19, 0.685), color="bark_dark"))
    # Horizontal backrest slats
    for sz in (0.65, 0.83):
        model.add(common.make_box((0.34, 0.035, 0.08), loc=(0.0, -0.19, sz), color="bark_light"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
