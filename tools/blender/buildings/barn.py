"""barn: closed red barn shell, 10.8 x 8 m, with big closed double doors on the +Y side and two hay bales either side.

Footprint 12 x 8 m including the bales. No doorway, so no door marker. Ridge runs along X.
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, HERE)

import common  # noqa: E402
import kit  # noqa: E402

NAME, CATEGORY, BUDGET = "barn", "buildings", 3000
TOP = 4.0
F = kit.FLOOR_TOP


def build(model):
    k = kit.Kit(model)
    k.solid((-5.4, -4.0, 0.0), (5.4, 4.0, F), "dirt", base="floor")
    k.wall((-5.4, 3.85, F), (5.4, 4.0, TOP), color="red_paint", base="wall_front")
    k.wall((-5.4, -4.0, F), (5.4, -3.85, TOP), color="red_paint", base="wall_back")
    k.wall((-5.4, -3.85, F), (-5.25, 3.85, TOP), color="red_paint", base="wall_left")
    k.wall((5.25, -3.85, F), (5.4, 3.85, TOP), color="red_paint", base="wall_right")

    # Closed double doors: two dark panels with a white trim bar, sitting just proud of the front wall
    k.box((-2.6, 4.0, F), (-0.1, 4.03, 3.6), "bark_dark")
    k.box((0.1, 4.0, F), (2.6, 4.03, 3.6), "bark_dark")
    k.box((-2.6, 4.0, 3.6), (2.6, 4.03, 3.7), "white")

    # Gable roof, ridge along X (the long way)
    ridge = (0.0, 0.0, 6.0)
    k.roof(ridge, (0.0, 4.0, 4.0), span=(1.0, 0.0, 0.0), width=11.4, color="rock_dark")
    k.roof(ridge, (0.0, -4.0, 4.0), span=(1.0, 0.0, 0.0), width=11.4, color="rock_dark")
    for x in (5.4, -5.55):
        k.gable([(x, -4.0, 4.0), (x, 4.0, 4.0), (x, 0.0, 6.0)], (0.15, 0.0, 0.0), "white", roof=True)

    # Hay bales outside both ends: two on the ground and one stacked
    for x0, x1 in ((5.4, 6.0), (-6.0, -5.4)):
        k.solid((x0, -2.4, 0.0), (x1, -1.2, 0.7), "wood_crate", base="bale")
        k.solid((x0, -1.2, 0.0), (x1, 0.0, 0.7), "wood_crate", base="bale")
        k.solid((x0, -2.4, 0.7), (x1, -1.2, 1.4), "wood_crate", base="bale")

    # Detail (visual plan step 7): white door trim and cross bars, white corner boards, darker board
    # lines on the long walls, a hayloft door on each gable and a ridge cap
    k.frame(((-2.6, 3.9, F), (2.6, 4.1, 3.6)), 4.03, 1, "white", t=0.12)
    for x0, x1 in ((-2.6, -0.1), (0.1, 2.6)):
        k.box((x0, 4.03, 1.75), (x1, 4.07, 1.87), "white")
    k.posts([(-5.4, -4.0), (5.4, -4.0), (-5.4, 4.0), (5.4, 4.0)], 4.0, "white", size=0.2)
    for y, sign in ((4.0, 1), (-4.0, -1)):
        x = -5.0
        while x < 5.1:
            if not (y > 0 and -2.75 < x < 2.75):
                a, b = sorted((y, y + sign * 0.02))
                k.box((x - 0.03, a, F), (x + 0.03, b, 4.0), "blood_dark")
            x += 0.8
    for x, sign in ((5.4, 1), (-5.4, -1)):
        a, b = sorted((x, x + sign * 0.18))
        k.box((a, -0.8, 4.3), (b, 0.8, 5.3), "bark_dark")
    k.ridge_cap((0.0, 0.0, 6.0), 0, 11.7)

    k.empty("lot_size", (0.0, 0.0, 0.0), scale=(12.0, 8.0, 1.0))
    k.empty("loot_1", (-5.7, 2.6, 0.0))  # outside, by the bales
    k.empty("spawn_1", (0.0, -5.0, 0.0))
    k.empty("spawn_2", (4.0, 5.0, 0.0))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, view=kit.FRONT_VIEW) else 1


if __name__ == "__main__":
    sys.exit(main())
