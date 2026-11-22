"""shed: closed 2.4 x 2.4 m plank shell with a gable roof, and a tool rack on the +Y side. Footprint 3 x 3 m.

No door: it is a closed shell, so there is no door marker.
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, HERE)

import common  # noqa: E402
import kit  # noqa: E402

NAME, CATEGORY, BUDGET = "shed", "buildings", 3000
TOP = 2.0
F = kit.FLOOR_TOP
PLANKS = ("wood_crate", "bark_dark")


def build(model):
    k = kit.Kit(model)
    k.solid((-1.2, -1.2, 0.0), (1.2, 1.2, F), "dirt", base="floor")
    k.wall((-1.2, 1.05, F), (1.2, 1.2, TOP), colors=PLANKS, course=0.25, base="wall_front")
    k.wall((-1.2, -1.2, F), (1.2, -1.05, TOP), colors=PLANKS, course=0.25, base="wall_back")
    k.wall((-1.2, -1.05, F), (-1.05, 1.05, TOP), colors=PLANKS, course=0.25, base="wall_left")
    k.wall((1.05, -1.05, F), (1.2, 1.05, TOP), colors=PLANKS, course=0.25, base="wall_right")

    # Gable roof, ridge along X
    ridge = (0.0, 0.0, 2.7)
    k.roof(ridge, (0.0, 1.5, 2.0), span=(1.0, 0.0, 0.0), width=2.85, color="rock_dark")
    k.roof(ridge, (0.0, -1.5, 2.0), span=(1.0, 0.0, 0.0), width=2.85, color="rock_dark")
    for x in (-1.35, 1.2):
        k.gable([(x, -1.38, 2.02), (x, 1.38, 2.02), (x, 0.0, 2.56)], (0.15, 0.0, 0.0), "bark_dark", roof=True)

    # Tool rack outside the front wall: a board on two posts, with a shovel and a rake hung on it (proud of wall y=1.2)
    k.solid((-1.0, 1.22, 0.6), (0.4, 1.34, 1.7), "wood_crate", base="rack")
    k.box((-1.0, 1.22, 0.0), (-0.92, 1.34, 0.6), "bark_dark")
    k.box((0.32, 1.22, 0.0), (0.4, 1.34, 0.6), "bark_dark")
    k.box((-0.2, 1.34, 0.4), (-0.14, 1.38, 1.7), "bark_light")  # shovel handle
    k.box((-0.3, 1.38, 0.2), (-0.04, 1.42, 0.5), "metal")  # shovel blade proud of handle
    k.box((0.1, 1.34, 1.3), (0.3, 1.38, 1.38), "metal")  # rake head

    # Detail (visual plan step 7): corner posts and a ridge cap
    size = 0.14
    for px, py in ((-1.2, -1.2), (1.2, -1.2), (-1.2, 1.2), (1.2, 1.2)):
        k.box((px - size / 2, py - size / 2, 0.02), (px + size / 2, py + size / 2, 2.02), "bark_dark")
    k.ridge_cap((0.0, 0.0, 2.7), 0, 2.89, "bark_dark")

    k.empty("lot_size", (0.0, 0.0, 0.0), scale=(3.0, 3.0, 1.0))
    k.empty("loot_1", (-1.3, 0.0, 0.0))  # behind the shed, on the grass
    k.empty("spawn_1", (1.3, -0.8, 0.0))
    k.empty("spawn_2", (0.0, -1.4, 0.0))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, view=kit.FRONT_VIEW) else 1


if __name__ == "__main__":
    sys.exit(main())
