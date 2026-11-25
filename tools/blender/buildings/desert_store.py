"""desert_store: enterable general store with a taller false front and covered porch, 10 x 6.5 m footprint.

Body is 10 x 5 m (y -3.5 to 1.5); porch adds 1.5 m in front (y 1.5 to 3.0). Faded sandstone colors.
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, HERE)

import common  # noqa: E402
import kit  # noqa: E402

NAME, CATEGORY, BUDGET = "desert_store", "buildings", 4500
TOP = 3.0
F = kit.FLOOR_TOP


def build(model):
    k = kit.Kit(model)
    k.solid((-5.0, -3.5, 0.0), (5.0, 1.5, F), "wood_crate", base="floor")
    k.solid((-4.98, 1.51, 0.0), (4.98, 3.0, F), "bark_light", base="deck")

    door = ((-0.6, 1.3, F), (0.6, 1.6, kit.DOOR_H))
    win_l = ((-3.8, 1.3, 0.9), (-1.6, 1.6, 2.3))
    win_r = ((1.6, 1.3, 0.9), (3.8, 1.6, 2.3))
    side_l = ((-5.2, -2.0, 1.0), (-4.7, -0.2, 2.0))
    side_r = ((4.7, -2.0, 1.0), (5.2, -0.2, 2.0))

    k.wall((-5.0, 1.35, F), (5.0, 1.5, TOP), openings=[door, win_l, win_r], color="sandstone", base="wall_front")
    k.wall((-5.0, -3.5, F), (5.0, -3.35, TOP), color="sandstone", base="wall_back")
    k.wall((-5.0, -3.35, F), (-4.85, 1.35, TOP), openings=[side_l], color="sandstone", base="wall_left")
    k.wall((4.85, -3.35, F), (5.0, 1.35, TOP), openings=[side_r], color="sandstone", base="wall_right")

    # Main roof + taller false front facade + cornice trim
    k.box((-4.98, -3.48, TOP), (4.98, 1.48, TOP + 0.18), "rock_dark", roof=True)
    k.collider((-4.98, -3.48, TOP), (4.98, 1.48, TOP + 0.18), "roof")
    k.box((-5.0, 1.35, TOP), (5.0, 1.52, 4.4), "sandstone", roof=True)
    k.box((-3.0, 1.35, 4.4), (3.0, 1.52, 4.85), "sandstone", roof=True)
    k.box((-5.1, 1.3, 4.35), (5.1, 1.6, 4.5), "sandstone_dark", roof=True)
    k.box((-3.1, 1.3, 4.8), (3.1, 1.6, 4.95), "sandstone_dark", roof=True)
    # Sign board on the false front
    k.box((-2.8, 1.52, 3.2), (2.8, 1.60, 4.05), "sand_light", roof=True)
    k.box((-2.65, 1.58, 3.35), (2.65, 1.63, 3.90), "sandstone_dark", roof=True)

    # Porch roof and 4 wooden posts
    k.box((-4.98, 1.52, 2.55), (4.98, 3.05, 2.7), "bark_dark", roof=True)
    k.collider((-4.98, 1.52, 2.55), (4.98, 3.05, 2.7), "porch_roof")
    for px in (-4.7, -1.5, 1.5, 4.7):
        k.solid((px - 0.1, 2.75, F), (px + 0.1, 2.95, 2.55), "bark_dark", base="post")

    # Frames
    k.frame(door, 1.52, 1, "bark_dark", sill=False)
    k.frame(win_l, 1.52, 1, "bark_dark")
    k.frame(win_r, 1.52, 1, "bark_dark")
    k.frame(side_l, -5.02, -1, "bark_dark")
    k.frame(side_r, 5.02, 1, "bark_dark")

    # Interior: counter, 2 display shelves, crates
    k.solid((-2.2, -2.5, F), (2.2, -1.9, 1.05), "bark_dark", base="counter")
    k.box((-2.2, -2.5, 1.05), (2.2, -1.9, 1.1), "wood_crate")
    k.solid((-4.7, -1.2, F), (-4.25, 0.8, 1.9), "bark_light", base="shelf_l")
    k.solid((4.25, -1.2, F), (4.7, 0.8, 1.9), "bark_light", base="shelf_r")
    k.crate(-3.8, 2.2, 0.55, z=F)
    k.crate(3.6, -2.8, 0.6, z=F)

    k.empty("lot_size", (0.0, 0.0, 0.0), scale=(10.0, 6.5, 1.0))
    k.empty("door_1", (0.0, 1.425, F))
    k.empty("loot_1", (0.0, -2.2, 1.1))
    k.empty("loot_2", (-3.7, -0.2, 0.0))
    k.empty("loot_3", (3.7, -0.2, 0.0))
    k.empty("loot_4", (-3.8, 2.2, 0.55 + F))
    k.empty("spawn_1", (2.5, -2.8, F))
    k.empty("spawn_2", (-2.5, 2.2, F))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, top_preview=True, view=kit.FRONT_VIEW) else 1


if __name__ == "__main__":
    sys.exit(main())
