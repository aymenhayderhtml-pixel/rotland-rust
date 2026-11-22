"""cabin: enterable one-room log cabin, 6 x 5 m walls (roof eaves overhang 0.3 m on each side).

Door on the +Y side in Blender (-Z in Godot). Floor top at 0.1 m, so the door has a small step.
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, HERE)

import common  # noqa: E402
import kit  # noqa: E402

NAME, CATEGORY, BUDGET = "cabin", "buildings", 4000
TOP = 2.4  # wall top
LOG = ("bark_light", "bark_dark")
F = kit.FLOOR_TOP


def build(model):
    k = kit.Kit(model)
    k.solid((-3.0, -2.5, 0.0), (3.0, 2.5, F), "wood_crate", base="floor")

    door = ((-0.55, 2.3, F), (0.55, 2.6, kit.DOOR_H))
    win_left = ((-3.2, -0.9, 1.0), (-2.7, 0.3, 1.9))
    win_right = ((2.7, -0.9, 1.0), (3.2, 0.3, 1.9))
    k.wall((-3.0, 2.35, F), (3.0, 2.5, TOP), openings=[door], colors=LOG, course=0.35, base="wall_front")
    k.wall((-3.0, -2.5, F), (3.0, -2.35, TOP), colors=LOG, course=0.35, base="wall_back")
    k.wall((-3.0, -2.35, F), (-2.85, 2.35, TOP), openings=[win_left], colors=LOG, course=0.35, base="wall_left")
    k.wall((2.85, -2.35, F), (3.0, 2.35, TOP), openings=[win_right], colors=LOG, course=0.35, base="wall_right")

    # Gable roof, ridge along Y: two slabs and a triangle at each end (gables under roof slab)
    ridge = (0.0, 0.0, 3.3)
    k.roof(ridge, (3.3, 0.0, 2.4), span=(0.0, 1.0, 0.0), width=5.0, color="rock_dark")
    k.roof(ridge, (-3.3, 0.0, 2.4), span=(0.0, 1.0, 0.0), width=5.0, color="rock_dark")
    k.gable([(-3.2, 2.36, 2.26), (3.2, 2.36, 2.26), (0.0, 2.36, 3.16)], (0.0, 0.12, 0.0), "bark_dark", roof=True)
    k.gable([(-3.2, -2.48, 2.26), (3.2, -2.48, 2.26), (0.0, -2.48, 3.16)], (0.0, 0.12, 0.0), "bark_dark", roof=True)

    # Inside: table, bed frame with mattress, shelf against the back wall
    k.solid((1.0, -0.4, F), (2.0, 0.4, 0.75), "bark_dark", base="table")
    k.solid((-2.8, -2.2, F), (-1.6, -0.2, 0.45), "bark_dark", base="bed")
    k.box((-2.75, -2.15, 0.45), (-1.65, -0.25, 0.62), "white")
    k.solid((-1.0, -2.35, F), (0.4, -2.0, 1.8), "bark_dark", base="shelf")
    for z in (0.95, 1.45):
        k.box((-1.0, -2.0, z), (0.4, -1.98, z + 0.05), "wood_crate")

    # Detail (visual plan step 7): frames, a boarded window, log-end corner posts, ridge cap, chimney
    k.frame(door, 2.52, 1, "bark_dark")
    k.frame(win_left, -3.02, -1, "bark_dark")
    k.boards(win_left, -3.03, -1)
    k.frame(win_right, 3.02, 1, "bark_dark")
    for px, py in ((-3.0, -2.5), (3.0, -2.5), (-3.0, 2.5), (3.0, 2.5)):
        k.box((px - 0.11, py - 0.11, 0.02), (px + 0.11, py + 0.11, 2.46), "bark_dark")
    k.ridge_cap((0.0, 0.0, 3.3), 1, 5.0, "bark_dark")
    k.chimney(-1.6, -1.2, 2.7, 3.8, "rock_mid")
    k.crate(2.5, -1.9, 0.5)

    k.empty("lot_size", (0.0, 0.0, 0.0), scale=(6.0, 5.0, 1.0))
    k.empty("door_1", (0.0, 2.425, F))
    k.empty("loot_1", (1.5, 0.0, 0.75))  # table
    k.empty("loot_2", (-0.3, -2.2, 1.85))  # shelf top
    k.empty("loot_3", (-2.2, -1.2, 0.62))  # bed
    k.empty("spawn_1", (-1.8, 1.4, F))
    k.empty("spawn_2", (1.8, -1.2, F))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, top_preview=True, view=kit.FRONT_VIEW) else 1


if __name__ == "__main__":
    sys.exit(main())
