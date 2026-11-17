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
    k.roof(ridge, (0.0, 1.5, 2.0), span=(1.0, 0.0, 0.0), width=2.7, color="rock_dark")
    k.roof(ridge, (0.0, -1.5, 2.0), span=(1.0, 0.0, 0.0), width=2.7, color="rock_dark")
    for x in (-1.35, 1.2):
        k.gable([(x, -1.5, 2.0), (x, 1.5, 2.0), (x, 0.0, 2.7)], (0.15, 0.0, 0.0), "bark_dark", roof=True)

    # Tool rack outside the front wall: a board on two posts, with a shovel and a rake hung on it
    k.solid((-1.0, 1.2, 0.6), (0.4, 1.32, 1.7), "wood_crate", base="rack")
    k.box((-1.0, 1.2, 0.0), (-0.92, 1.32, 0.6), "bark_dark")
    k.box((0.32, 1.2, 0.0), (0.4, 1.32, 0.6), "bark_dark")
    k.box((-0.2, 1.32, 0.4), (-0.14, 1.36, 1.7), "bark_light")  # shovel handle
    k.box((-0.3, 1.32, 0.2), (-0.04, 1.36, 0.5), "metal")  # shovel blade
    k.box((0.1, 1.32, 1.3), (0.3, 1.36, 1.38), "metal")  # rake head

    # Detail (visual plan step 7): corner posts and a ridge cap
    k.posts([(-1.2, -1.2), (1.2, -1.2), (-1.2, 1.2), (1.2, 1.2)], 2.0, "bark_dark", size=0.14)
    k.ridge_cap((0.0, 0.0, 2.7), 0, 2.85, "bark_dark")

    k.empty("lot_size", (0.0, 0.0, 0.0), scale=(3.0, 3.0, 1.0))
    k.empty("loot_1", (-1.3, 0.0, 0.0))  # behind the shed, on the grass
    k.empty("spawn_1", (1.3, -0.8, 0.0))
    k.empty("spawn_2", (0.0, -1.4, 0.0))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, view=kit.FRONT_VIEW) else 1


if __name__ == "__main__":
    sys.exit(main())
