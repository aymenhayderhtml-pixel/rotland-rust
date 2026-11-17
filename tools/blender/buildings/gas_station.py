"""gas_station: enterable shop (10 x 5 m, y -4 to 1) with a 3 m deep fuel canopy in front (y 1 to 4).

Footprint 10 x 8 m. Shop door and big window on the +Y side (-Z in Godot). Two fuel pumps under the canopy.
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, HERE)

import common  # noqa: E402
import kit  # noqa: E402

NAME, CATEGORY, BUDGET = "gas_station", "buildings", 4000
TOP = 3.0  # shop wall top
F = kit.FLOOR_TOP


def build(model):
    k = kit.Kit(model)
    k.solid((-5.0, -4.0, 0.0), (5.0, 1.0, F), "rock_mid", base="floor")

    big_window = ((-3.5, 0.7, 0.9), (2.5, 1.2, 2.6))
    door = ((3.2, 0.7, F), (4.2, 1.2, kit.DOOR_H))
    left_window = ((-5.2, -3.0, 1.0), (-4.7, -1.0, 2.2))
    right_window = ((4.7, -3.0, 1.0), (5.2, -1.0, 2.2))

    k.wall((-5.0, -4.0, F), (5.0, -3.85, TOP), color="white", base="wall_back")
    k.wall((-5.0, -3.85, F), (-4.85, 1.0, TOP), openings=[left_window], color="white", base="wall_left")
    k.wall((4.85, -3.85, F), (5.0, 1.0, TOP), openings=[right_window], color="white", base="wall_right")
    k.wall((-5.0, 0.85, F), (5.0, 1.0, TOP), openings=[big_window, door], color="white", base="wall_front")

    # Flat shop roof, red fascia on the front, and the canopy roof over the pumps
    k.box((-5.0, -4.0, TOP), (5.0, 1.0, TOP + 0.2), "rock_dark", roof=True)
    k.collider((-5.0, -4.0, TOP), (5.0, 1.0, TOP + 0.2), "shop_roof")
    k.box((-4.5, 1.0, 2.55), (4.5, 1.14, 2.95), "red_paint")
    k.box((-5.0, 1.0, 4.2), (5.0, 4.0, 4.35), "red_paint", roof=True)
    k.collider((-5.0, 1.0, 4.2), (5.0, 4.0, 4.35), "canopy_roof")
    for x in (-4.6, 4.6):
        for y in (1.4, 3.6):
            k.solid((x - 0.1, y - 0.1, F), (x + 0.1, y + 0.1, 4.2), "metal", base="post")

    # Two fuel pumps under the canopy
    for x in (-2.0, 2.0):
        k.solid((x - 0.35, 2.25, F), (x + 0.35, 2.75, 1.3), "red_paint", base="pump")
        k.box((x - 0.3, 2.3, 1.3), (x + 0.3, 2.7, 1.45), "gun_black")

    # Inside the shop: counter, two shelves on the side walls
    k.solid((-2.0, -1.8, F), (2.0, -1.2, 1.1), "white", base="counter")
    k.box((-2.0, -1.8, 1.1), (2.0, -1.2, 1.15), "wood_crate")
    k.solid((-4.8, -0.8, F), (-4.45, 0.6, 2.0), "metal", base="shelf_left")
    k.solid((4.45, -0.8, F), (4.8, 0.6, 2.0), "metal", base="shelf_right")

    # Detail (visual plan step 7): metal frames, boarded side windows, a roof AC unit and a vending machine
    k.frame(big_window, 1.0, 1, "metal")
    k.frame(door, 1.0, 1, "metal")
    k.frame(left_window, -5.0, -1, "metal")
    k.boards(left_window, -5.0, -1)
    k.frame(right_window, 5.0, 1, "metal")
    k.boards(right_window, 5.0, 1, rows=(0.25, 0.75))
    k.box((1.5, -3.0, TOP + 0.2), (2.6, -2.0, TOP + 0.75), "metal", roof=True)
    k.box((1.6, -2.02, TOP + 0.3), (2.5, -1.98, TOP + 0.65), "rock_dark", roof=True)
    k.solid((-4.45, 1.0, F), (-3.75, 1.6, 1.9), "shirt_blue", base="vending")
    k.box((-4.35, 1.6, 0.9), (-3.85, 1.63, 1.75), "white")

    k.empty("lot_size", (0.0, 0.0, 0.0), scale=(10.0, 8.0, 1.0))
    k.empty("door_1", (3.7, 0.925, F))
    k.empty("loot_1", (0.0, -1.5, 1.15))  # counter
    # Floor level in front of the shelves; loot_3 sits further back so it stays clear of the doorway
    k.empty("loot_2", (-3.9, -0.1, 0.0))  # in front of the left shelf
    k.empty("loot_3", (3.9, -1.4, 0.0))  # in front of the right shelf, beside the door's inside path
    k.empty("spawn_1", (2.5, -3.0, F))  # inside the shop
    k.empty("spawn_2", (-3.0, 3.2, 0.0))  # by the pumps, outside


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, top_preview=True, view=kit.FRONT_VIEW) else 1


if __name__ == "__main__":
    sys.exit(main())
