"""beach_chair: a wooden/canvas folding beach lounge chair (<= 250 tris). Has a collider."""
import math
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402

NAME, CATEGORY, BUDGET = "beach_chair", "props", 250


def build(model):
    # Width 0.66 m (X from -0.33 to 0.33)
    # Slanted backrest strut: from ground at (y=0.38, z=0.0) up to backrest top at (y=-0.28, z=0.82)
    dy1 = -0.66
    dz1 = 0.82
    len1 = math.hypot(dy1, dz1)
    rot1 = math.atan2(dy1, dz1)  # pitch angle
    mid1_y = (0.38 - 0.28) / 2
    mid1_z = (0.0 + 0.82) / 2

    # Crossing seat strut: from ground at (y=-0.32, z=0.0) up to front seat edge at (y=0.36, z=0.36)
    dy2 = 0.68
    dz2 = 0.36
    len2 = math.hypot(dy2, dz2)
    rot2 = math.atan2(dy2, dz2)
    mid2_y = (-0.32 + 0.36) / 2
    mid2_z = (0.0 + 0.36) / 2

    # Scissor frames on left (-X) and right (+X)
    for x in (-0.30, 0.30):
        # Long backrest upright
        strut1 = common.make_box((0.04, 0.04, len1), color="bark_dark")
        common.transform(strut1, rot=(-rot1, 0.0, 0.0), loc=(x, mid1_y, mid1_z))
        model.add(strut1)

        # Seat support leg
        strut2 = common.make_box((0.04, 0.04, len2), color="bark_dark")
        common.transform(strut2, rot=(-rot2, 0.0, 0.0), loc=(x, mid2_y, mid2_z))
        model.add(strut2)

        # Rear prop strut (supporting the back)
        prop = common.make_box((0.035, 0.035, 0.52), color="bark_dark")
        common.transform(prop, rot=(0.60, 0.0, 0.0), loc=(x, -0.16, 0.22))
        model.add(prop)

    # Cross rails connecting the two sides (along X, width 0.66 m)
    # Top back rail
    r_top = common.make_box((0.66, 0.04, 0.04), loc=(0.0, -0.28, 0.82), color="bark_dark")
    # Front seat rail
    r_seat = common.make_box((0.66, 0.04, 0.04), loc=(0.0, 0.36, 0.36), color="bark_dark")
    # Rear ground rail
    r_rear = common.make_box((0.66, 0.04, 0.04), loc=(0.0, -0.32, 0.03), color="bark_dark")
    # Front ground rail
    r_front = common.make_box((0.66, 0.04, 0.04), loc=(0.0, 0.38, 0.03), color="bark_dark")
    model.add(r_top)
    model.add(r_seat)
    model.add(r_rear)
    model.add(r_front)

    # Canvas fabric sling (blue)
    # Slanted backrest section
    c_back_len = math.hypot(-0.28 - 0.02, 0.82 - 0.28)
    c_back = common.make_box((0.54, 0.02, c_back_len), color="shirt_blue")
    common.transform(c_back, rot=(-rot1, 0.0, 0.0), loc=(0.0, -0.13, 0.55))
    # Seat sagging section
    c_seat_len = math.hypot(0.36 - 0.02, 0.36 - 0.28)
    c_seat = common.make_box((0.54, 0.02, c_seat_len), color="shirt_blue")
    common.transform(c_seat, rot=(-rot2 + 0.2, 0.0, 0.0), loc=(0.0, 0.19, 0.32))
    model.add(c_back)
    model.add(c_seat)

    # Collider
    model.add_collider("chair", common.make_box((0.72, 0.85, 0.85), loc=(0.0, 0.03, 0.42), color="white"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
