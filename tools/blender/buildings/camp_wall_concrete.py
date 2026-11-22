"""camp_wall_concrete: 4 m section of 3.2 m concrete blocks with a slight lean,
rebar showing at one broken top, and a painted stripe.

Features:
- Staggered concrete block construction with chamfered joints.
- Realistic settling lean (slight forward tilt along Y).
- Bold painted hazard stripe across the front face.
- Fractured top corner with exposed bent/twisted steel rebars.
- Tiling along X (-2.0 to +2.0), centered on X, thickness along Y.
- Colliders: solid concrete wall.
"""
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, HERE)

import common  # noqa: E402
import kit  # noqa: E402

NAME, CATEGORY, BUDGET = "camp_wall_concrete", "buildings", 1200
LENGTH = 4.0
HALF_L = LENGTH / 2.0  # 2.0 m
HEIGHT = 3.2


def _make_rebar_segment(k, p0, p1, width=0.022, color="rust"):
    """Helper to add a rectangular rebar segment between two 3D points."""
    x0, y0, z0 = p0
    x1, y1, z1 = p1
    # Bounding box of the segment with padding for thickness
    lo = (min(x0, x1) - width / 2, min(y0, y1) - width / 2, min(z0, z1))
    hi = (max(x0, x1) + width / 2, max(y0, y1) + width / 2, max(z0, z1) + width)
    k.box(lo, hi, color)


def build(model):
    k = kit.Kit(model)

    # 1. Base footing / plinth along ground (z = 0.0 to 0.20)
    k.box((-HALF_L, -0.24, 0.0), (HALF_L, 0.22, 0.20), "rock_dark")

    # 2. Four courses of concrete blocks with a progressive slight lean along Y
    # Course heights:
    # Course 0: 0.20 -> 0.95 (h = 0.75)
    # Course 1: 0.95 -> 1.70 (h = 0.75)
    # Course 2: 1.70 -> 2.45 (h = 0.75)
    # Course 3: 2.45 -> 3.20 (h = 0.75)
    courses = [
        (0.20, 0.95, -0.03, 0.36),
        (0.95, 1.70, 0.00, 0.35),
        (1.70, 2.45, 0.03, 0.34),
        (2.45, 3.20, 0.06, 0.33),
    ]

    colors = ["rock_mid", "rock_light", "rock_mid", "rock_dark"]

    # Course layout (running bond):
    # Even courses: 4 blocks of 1.0 m each
    # Odd courses: 1 half block (0.5m), 3 full blocks (1.0m), 1 half block (0.5m)
    even_splits = [-2.0, -1.0, 0.0, 1.0, 2.0]
    odd_splits = [-2.0, -1.5, -0.5, 0.5, 1.5, 2.0]

    for c_idx, (z0, z1, y_cen, depth) in enumerate(courses):
        splits = even_splits if (c_idx % 2 == 0) else odd_splits
        half_d = depth / 2.0

        for b_idx in range(len(splits) - 1):
            bx0 = splits[b_idx]
            bx1 = splits[b_idx + 1]

            # Small bevel recess gap (0.015 m) between adjacent blocks
            gap_l = 0.01 if bx0 > -HALF_L else 0.0
            gap_r = 0.01 if bx1 < HALF_L else 0.0
            x_lo = bx0 + gap_l
            x_hi = bx1 - gap_r

            # Skip or break Course 3 block at x = 0.5..1.5 for the broken rebar section
            if c_idx == 3 and bx0 == 0.5:
                # Intact part of block (left portion): 0.5..0.75
                k.box((0.51, y_cen - half_d, z0), (0.75, y_cen + half_d, 2.85), colors[(c_idx + b_idx) % 4])
                # Jagged broken crater in the middle (0.75..1.25)
                k.box((0.75, y_cen - half_d, z0), (1.25, y_cen + half_d, 2.65), "rock_dark")
                # Intact right portion: 1.25..1.49
                k.box((1.25, y_cen - half_d, z0), (1.49, y_cen + half_d, 2.95), colors[(c_idx + b_idx) % 4])
                continue

            paint = colors[(c_idx * 2 + b_idx) % 4]
            k.box((x_lo, y_cen - half_d, z0), (x_hi, y_cen + half_d, z1), paint)

    # 3. Painted stripe across Course 1 (z = 1.15 to 1.45) on the outside (+Y face)
    # Hazard red & white stripe
    y_front = 0.00 + 0.35 / 2.0  # front face of Course 1
    stripe_z0 = 1.18
    stripe_z1 = 1.48
    th = 0.015  # slightly proud of concrete face

    # Create alternating red and white hazard stripe blocks across the 4m length
    num_stripes = 10
    s_w = LENGTH / num_stripes  # 0.4 m per stripe
    for s_i in range(num_stripes):
        sx0 = -HALF_L + s_i * s_w
        sx1 = sx0 + s_w
        s_col = "red_paint" if (s_i % 2 == 0) else "white"
        k.box((sx0 + 0.01, y_front, stripe_z0), (sx1 - 0.01, y_front + th, stripe_z1), s_col)

    # 4. Exposed rebar showing at broken top (x ≈ 0.75 to 1.25)
    # Rebar 1: bent upward and hooked forward
    _make_rebar_segment(k, (0.80, 0.05, 2.65), (0.78, 0.06, 3.25), width=0.024, color="rust")
    _make_rebar_segment(k, (0.78, 0.06, 3.25), (0.74, 0.14, 3.42), width=0.022, color="gun_black")

    # Rebar 2: tall vertical spine with slight bend
    _make_rebar_segment(k, (0.95, 0.04, 2.65), (0.96, 0.05, 3.38), width=0.024, color="rust")
    _make_rebar_segment(k, (0.96, 0.05, 3.41), (1.02, 0.03, 3.52), width=0.022, color="gun_black")

    # Rebar 3: curled bent bar
    _make_rebar_segment(k, (1.10, 0.07, 2.65), (1.12, 0.08, 3.12), width=0.024, color="rust")
    _make_rebar_segment(k, (1.12, 0.08, 3.12), (1.18, 0.16, 3.22), width=0.022, color="rust")

    # Rebar 4: short twisted stub
    _make_rebar_segment(k, (1.22, 0.05, 2.65), (1.20, 0.04, 2.92), width=0.022, color="gun_black")

    # Concrete broken rubble chunks lying on the broken ledge
    k.box((0.82, 0.03, 2.65), (0.92, 0.12, 2.75), "rock_light")
    k.box((1.05, -0.05, 2.65), (1.16, 0.04, 2.76), "rock_mid")

    # 5. Colliders:
    k.collider((-HALF_L, -0.22, 0.0), (HALF_L, 0.22, HEIGHT), base="concrete_wall")


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, view=kit.FRONT_VIEW) else 1


if __name__ == "__main__":
    sys.exit(main())
