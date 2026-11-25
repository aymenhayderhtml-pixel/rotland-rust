"""bait_shop: walk-in swamp bait shop with big sign board, porch, crates, and ice fridge out front (<= 3500 tris).

Body is 7 x 5.5 m (Y -3.0..2.5) with a 1.0 m front porch (Y 2.5..3.5). Door on +Y.
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, HERE)

import common  # noqa: E402
import kit  # noqa: E402

NAME, CATEGORY, BUDGET = "bait_shop", "buildings", 3500
F = kit.FLOOR_TOP
TOP = 2.6
WALL_COLORS = ("bark_light", "wood_crate")


def build(model):
    k = kit.Kit(model)

    # Floor + porch
    k.solid((-3.5, -3.0, 0.0), (3.5, 3.5, F), "wood_crate", base="floor")

    door = ((-1.4, 2.3, F), (-0.2, 2.65, kit.DOOR_H))
    win_f = ((0.7, 2.3, 1.0), (2.4, 2.65, 1.9))
    win_l = ((-3.6, -1.2, 1.0), (-3.3, 0.4, 1.9))
    win_r = ((3.3, -1.2, 1.0), (3.6, 0.4, 1.9))
    k.wall((-3.5, 2.35, F), (3.5, 2.5, TOP), openings=[door, win_f], colors=WALL_COLORS, course=0.35, base="w_front")
    k.wall((-3.5, -3.0, F), (3.5, -2.85, TOP), colors=WALL_COLORS, course=0.35, base="w_back")
    k.wall((-3.5, -2.85, F), (-3.35, 2.35, TOP), openings=[win_l], colors=WALL_COLORS, course=0.35, base="w_left")
    k.wall((3.35, -2.85, F), (3.5, 2.35, TOP), openings=[win_r], colors=WALL_COLORS, course=0.35, base="w_right")

    k.frame(door, 2.52, 1, "bark_dark", sill=False)
    k.frame(win_f, 2.52, 1, "bark_dark")
    k.frame(win_l, -3.52, -1, "bark_dark")
    k.frame(win_r, 3.52, 1, "bark_dark")

    # Gable roof (ridge along X)
    ridge = (0.0, -0.25, 3.8)
    k.roof(ridge, (0.0, -3.2, TOP), span=(1.0, 0.0, 0.0), width=7.2, color="rust")
    k.roof(ridge, (0.0, 2.7, TOP), span=(1.0, 0.0, 0.0), width=7.2, color="rust")
    for x, off_x in ((-3.5, 0.12), (3.38, 0.12)):
        k.gable([(x, -3.0, TOP - 0.1), (x, 2.5, TOP - 0.1), (x, -0.25, 3.68)], (off_x, 0.0, 0.0), "bark_dark", roof=True)
    k.ridge_cap((0.0, -0.25, 3.8), 0, 7.3, "metal")

    # Porch roof + posts
    k.solid((-3.55, 2.5, 2.45), (3.55, 3.55, 2.58), "metal", base="porch_roof")
    for px in (-3.35, 3.35):
        k.solid((px - 0.08, 3.3, F), (px + 0.08, 3.46, 2.45), "bark_dark", base="porch_post")

    # Big sign board on porch roof front
    k.box((-2.2, 3.48, 2.58), (2.2, 3.60, 3.45), "bark_dark")
    k.box((-2.05, 3.58, 2.70), (2.05, 3.63, 3.33), "white")
    k.box((-1.9, 3.62, 2.95), (1.9, 3.65, 3.12), "water_blue")
    k.box((-1.9, 3.62, 2.75), (0.6, 3.65, 2.86), "red_paint")

    # Out front on porch: fridge/ice box and crates
    k.solid((2.0, 2.65, F), (3.0, 3.30, 1.15), "white", base="ice_box")
    k.box((1.98, 2.63, 1.15), (3.02, 3.32, 1.24), "water_blue")
    k.box((2.35, 3.30, 0.90), (2.65, 3.35, 0.96), "metal")
    k.crate(-2.8, 2.95, 0.55, z=F)
    k.crate(-2.1, 3.0, 0.45, z=F)

    # Interior: shop counter, bait shelf, barrel
    k.solid((-3.1, -1.8, F), (-0.4, -1.0, 0.95), "bark_dark", base="counter")
    k.box((-3.1, -1.8, 0.95), (-0.4, -1.0, 1.02), "wood_crate")
    k.solid((1.0, -2.8, F), (3.1, -2.3, 1.8), "bark_dark", base="shelf")
    k.barrel(2.8, 1.4, color="water_blue")

    k.empty("lot_size", (0.0, 0.0, 0.0), scale=(7.0, 6.5, 1.0))
    k.empty("door_1", (-0.8, 2.425, F))
    k.empty("loot_1", (-1.8, -1.4, 1.02))  # shop counter
    k.empty("loot_2", (2.0, -2.55, 1.80))  # bait shelf
    k.empty("loot_3", (-2.8, 2.95, F + 0.55))  # porch crate
    k.empty("spawn_1", (0.2, 0.4, F))
    k.empty("spawn_2", (1.6, -1.0, F))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, top_preview=True, view=kit.FRONT_VIEW) else 1


if __name__ == "__main__":
    sys.exit(main())
