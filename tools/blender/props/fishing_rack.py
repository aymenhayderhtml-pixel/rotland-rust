"""fishing_rack: wooden drying rack with drying fish hung on horizontal poles (<= 400 tris). Has a collider."""
import math
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402

NAME, CATEGORY, BUDGET = "fishing_rack", "props", 400


def build(model):
    # Dimensions: length along X: 2.8 m (-1.4 to 1.4), base depth along Y: 1.0 m (-0.5 to 0.5), apex height 1.80 m
    dy = 0.50
    dz = 1.80
    leg_len = math.hypot(dy, dz)
    leg_rot = math.atan2(dy, dz)

    # End A-frames at X = -1.30 and X = +1.30
    for x in (-1.30, 1.30):
        # Front leg (slants from y=0.50, z=0.0 to y=0.0, z=1.80)
        leg_fwd = common.make_box((0.08, 0.08, leg_len), color="bark_dark")
        common.transform(leg_fwd, rot=(leg_rot, 0.0, 0.0), loc=(x, 0.25, 0.90))
        # Back leg (slants from y=-0.50, z=0.0 to y=0.0, z=1.80)
        leg_back = common.make_box((0.08, 0.08, leg_len), color="bark_dark")
        common.transform(leg_back, rot=(-leg_rot, 0.0, 0.0), loc=(x, -0.25, 0.90))
        # Horizontal cross-tie halfway up
        cross_tie = common.make_box((0.07, 0.68, 0.07), loc=(x, 0.0, 0.68), color="bark_dark")
        model.add(leg_fwd)
        model.add(leg_back)
        model.add(cross_tie)

    # Long horizontal drying poles spanning across along X (length 2.85 m)
    # Top ridge pole
    pole_top = common.make_box((2.85, 0.07, 0.07), loc=(0.0, 0.0, 1.82), color="bark_light")
    # Front mid drying pole
    pole_fwd = common.make_box((2.85, 0.06, 0.06), loc=(0.0, 0.23, 1.15), color="bark_light")
    # Back mid drying pole
    pole_back = common.make_box((2.85, 0.06, 0.06), loc=(0.0, -0.23, 1.15), color="bark_light")
    model.add(pole_top)
    model.add(pole_fwd)
    model.add(pole_back)

    # Diagonal stability brace
    diag = common.make_box((2.80, 0.05, 0.05), color="bark_dark")
    common.transform(diag, rot=(0.0, 0.18, 0.0), loc=(0.0, 0.0, 0.85))
    model.add(diag)

    # 6 dried fish hanging down from mid poles
    fish_positions = [
        (-0.90, 0.23, 0.88),
        (-0.45, -0.23, 0.86),
        (-0.10, 0.23, 0.90),
        (0.35, -0.23, 0.87),
        (0.70, 0.23, 0.85),
        (1.05, -0.23, 0.89),
    ]
    fish_colors = ("canvas_khaki", "rock_light", "bark_light", "canvas_khaki", "rock_light", "bark_light")
    for idx, (fx, fy, fz) in enumerate(fish_positions):
        # Hanging twine
        string = common.make_box((0.015, 0.015, 0.22), loc=(fx, fy, fz + 0.18), color="wheat_gold")
        # Fish body and tail
        col = fish_colors[idx % len(fish_colors)]
        body = common.make_box((0.08, 0.04, 0.26), loc=(fx, fy, fz), color=col)
        tail = common.make_box((0.14, 0.02, 0.08), loc=(fx, fy, fz - 0.12), color=col)
        model.add(string)
        model.add(body)
        model.add(tail)

    # Wooden salting crate on the ground
    crate = common.make_box((0.48, 0.48, 0.36), loc=(-0.30, 0.0, 0.18), color="wood_crate")
    model.add(crate)

    # Collider
    model.add_collider("rack", common.make_box((2.85, 1.15, 1.85), loc=(0.0, 0.0, 0.92), color="white"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
