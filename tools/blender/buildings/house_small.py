"""house_small: enterable two-room house with a front porch, 9 x 7 m footprint.

Body is 9 x 6.5 m (y -3.5 to 3.0); the porch adds 0.5 m in front (+Y, -Z in Godot), so the
footprint is 7 m deep. Front door on the porch side, one interior doorway between the rooms.
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, HERE)

import common  # noqa: E402
import kit  # noqa: E402

NAME, CATEGORY, BUDGET = "house_small", "buildings", 4000
TOP = 2.4
F = kit.FLOOR_TOP
SIDING = ("white", "rock_light")


def build(model):
    k = kit.Kit(model)
    # Floor and porch deck share one height, so the only step is at the threshold
    k.solid((-4.5, -3.5, 0.0), (4.5, 3.0, F), "wood_crate", base="floor")
    k.solid((-4.48, 3.01, 0.0), (4.48, 3.51, F), "bark_light", base="deck")

    door = ((-2.5, 2.8, F), (-1.5, 3.1, kit.DOOR_H))  # 1.0 m wide: wider than the 0.9 m clearance check
    front_window = ((1.5, 2.8, 1.0), (3.0, 3.1, 1.9))
    back_window = ((1.5, -3.6, 1.0), (3.0, -3.2, 1.9))
    left_window = ((-4.7, -2.0, 1.0), (-4.2, -0.5, 1.9))
    right_window = ((4.2, -1.0, 1.0), (4.7, 0.5, 1.9))
    inner_door = ((-0.2, -0.5, F), (0.2, 0.5, kit.DOOR_H))

    k.wall((-4.5, 2.85, F), (4.5, 3.0, TOP), openings=[door, front_window], colors=SIDING, course=0.3, base="wall_front")
    k.wall((-4.5, -3.5, F), (4.5, -3.35, TOP), openings=[back_window], colors=SIDING, course=0.3, base="wall_back")
    k.wall((-4.5, -3.35, F), (-4.35, 2.85, TOP), openings=[left_window], colors=SIDING, course=0.3, base="wall_left")
    k.wall((4.35, -3.35, F), (4.5, 2.85, TOP), openings=[right_window], colors=SIDING, course=0.3, base="wall_right")
    k.wall((-0.075, -3.35, F), (0.075, 2.85, TOP), openings=[inner_door], colors=SIDING, course=0.3, base="wall_inner")

    # Gable roof with the ridge along X. The ridge sits over the body's centre (y = -0.25)
    ridge = (0.0, -0.25, 4.2)
    k.roof(ridge, (0.0, -3.5, 2.4), span=(1.0, 0.0, 0.0), width=9.0, color="rock_dark")
    k.roof(ridge, (0.0, 3.0, 2.4), span=(1.0, 0.0, 0.0), width=9.0, color="rock_dark")
    # Inset gables in X, lower ridge/eaves under roof slab
    for x, off_x in ((-4.48, 0.12), (4.36, 0.12)):
        k.gable([(x, -3.5, 2.26), (x, 3.0, 2.26), (x, -0.25, 4.06)], (off_x, 0.0, 0.0), "white", roof=True)

    # Porch: flat roof on two posts (proud in Y of front wall)
    k.box((-4.48, 3.02, 2.40), (4.48, 3.52, 2.52), "rock_dark", roof=True)
    k.collider((-4.48, 3.02, 2.40), (4.48, 3.52, 2.52), "porch_roof")
    for x in (-4.3, 4.2):
        k.solid((x, 3.2, F), (x + 0.15, 3.35, 2.4), "bark_dark", base="post")

    # Kitchen (left room)
    k.solid((-4.3, -3.35, F), (-1.5, -2.75, 0.95), "white", base="counter")
    k.box((-4.3, -3.35, 0.95), (-1.5, -2.75, 1.0), "wood_crate")
    k.solid((-4.3, -1.8, F), (-3.6, -1.2, 1.9), "metal", base="fridge")
    k.solid((-2.6, 0.2, F), (-1.4, 1.0, 0.75), "bark_dark", base="table")

    # Living room and bedroom (right room)
    k.solid((1.2, -2.9, F), (3.4, -2.2, 0.45), "shirt_blue", base="couch")
    k.solid((1.2, -2.9, 0.45), (3.4, -2.7, 0.95), "shirt_blue", base="couch_back")
    k.solid((2.0, 0.2, F), (3.4, 2.4, 0.45), "bark_dark", base="bed")
    k.box((2.05, 0.25, 0.45), (3.35, 2.35, 0.62), "white")
    k.box((2.3, 2.0, 0.62), (3.1, 2.3, 0.72), "white")

    # Detail (visual plan step 7): white trim, boarded windows, ridge cap, chimney, porch rail
    k.frame(door, 3.02, 1)
    k.frame(front_window, 3.02, 1)
    k.boards(front_window, 3.03, 1)
    k.frame(back_window, -3.52, -1)
    k.frame(left_window, -4.52, -1)
    k.boards(left_window, -4.53, -1, rows=(0.3, 0.7))
    k.frame(right_window, 4.52, 1)
    k.ridge_cap((0.0, -0.25, 4.2), 0, 9.3)
    k.chimney(2.5, -1.5, 3.3, 4.6)
    for x0, x1 in ((-4.3, -2.7), (-1.3, 4.2)):
        k.box((x0, 3.4, 0.85), (x1, 3.47, 0.92), "white")
        x = x0
        while x <= x1 + 0.01:
            k.box((x - 0.03, 3.41, F), (x + 0.03, 3.46, 0.85), "white")
            x += 0.4

    k.empty("lot_size", (0.0, 0.0, 0.0), scale=(9.0, 7.0, 1.0))
    k.empty("door_1", (-2.0, 2.925, F))
    k.empty("loot_1", (-2.8, -3.0, 1.0))  # kitchen counter
    k.empty("loot_2", (-2.0, 0.6, 0.75))  # kitchen table
    k.empty("loot_3", (2.7, 1.2, 0.62))  # bed
    k.empty("loot_4", (2.0, -2.5, 0.45))  # couch
    k.empty("spawn_1", (-3.0, -1.5, F))
    k.empty("spawn_2", (0.8, 1.5, F))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, top_preview=True, view=kit.FRONT_VIEW) else 1


if __name__ == "__main__":
    sys.exit(main())
