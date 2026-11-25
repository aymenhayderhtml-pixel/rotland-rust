"""motel: long one-storey desert motel with 5 doors in a row, covered walkway, and tall pole sign.

Body is 15 x 5 m (y -3.5 to 1.5); covered walkway adds 1.7 m in front (y 1.5 to 3.2).
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, HERE)

import common  # noqa: E402
import kit  # noqa: E402

NAME, CATEGORY, BUDGET = "motel", "buildings", 5500
TOP = 2.7
F = kit.FLOOR_TOP
ROOM_XS = (-6.0, -3.0, 0.0, 3.0, 6.0)


def build(model):
    k = kit.Kit(model)
    k.solid((-7.5, -3.5, 0.0), (7.5, 1.5, F), "rock_mid", base="floor")
    k.solid((-7.48, 1.51, 0.0), (7.48, 3.2, F), "sand_dark", base="walkway")

    front_openings = []
    for xc in ROOM_XS:
        door = ((xc - 0.85, 1.3, F), (xc + 0.15, 1.6, kit.DOOR_H))
        win = ((xc + 0.45, 1.3, 1.0), (xc + 1.15, 1.6, 1.85))
        front_openings.extend([door, win])

    k.wall((-7.5, 1.35, F), (7.5, 1.5, TOP), openings=front_openings, color="sand_light", base="wall_front")
    k.wall((-7.5, -3.5, F), (7.5, -3.35, TOP), color="sand_light", base="wall_back")
    k.wall((-7.5, -3.35, F), (-7.35, 1.35, TOP), color="sand_light", base="wall_left")
    k.wall((7.35, -3.35, F), (7.5, 1.35, TOP), color="sand_light", base="wall_right")

    # Partition walls between the 5 rooms
    for px in (-4.5, -1.5, 1.5, 4.5):
        k.wall((px - 0.075, -3.35, F), (px + 0.075, 1.35, TOP), color="sand_light", base="wall_part")

    # Roof over body + sandstone fascia + covered walkway canopy on posts
    k.box((-7.5, -3.5, TOP), (7.5, 1.5, TOP + 0.2), "rock_dark", roof=True)
    k.collider((-7.5, -3.5, TOP), (7.5, 1.5, TOP + 0.2), "roof")
    k.box((-7.55, 1.48, TOP - 0.1), (7.55, 1.6, TOP + 0.28), "sandstone", roof=True)
    k.box((-7.5, 1.5, 2.45), (7.5, 3.25, 2.6), "sandstone_dark", roof=True)
    k.collider((-7.5, 1.5, 2.45), (7.5, 3.25, 2.6), "canopy_roof")

    for px in (-7.3, -4.5, -1.5, 1.5, 4.5, 7.3):
        k.solid((px - 0.09, 2.95, F), (px + 0.09, 3.13, 2.45), "bark_dark", base="post")

    # Frames on all 5 doors and windows
    for xc in ROOM_XS:
        door = ((xc - 0.85, 1.3, F), (xc + 0.15, 1.6, kit.DOOR_H))
        win = ((xc + 0.45, 1.3, 1.0), (xc + 1.15, 1.6, 1.85))
        k.frame(door, 1.52, 1, "bark_dark", sill=False)
        k.frame(win, 1.52, 1, "bark_dark")

    # Simple bed and nightstand in each room
    for idx, xc in enumerate(ROOM_XS):
        k.solid((xc - 1.2, -3.1, F), (xc - 0.1, -1.3, 0.45), "bark_dark", base="bed")
        k.box((xc - 1.15, -3.05, 0.45), (xc - 0.15, -1.35, 0.6), "white")
        k.solid((xc + 0.3, -3.2, F), (xc + 1.0, -2.5, 0.65), "wood_crate", base="stand")
        k.empty(f"door_{idx + 1}", (xc - 0.35, 1.425, F))
        k.empty(f"loot_{idx + 1}", (xc + 0.65, -2.85, 0.65))

    # Tall pole sign on the front-left corner
    sx, sy = -6.8, 3.55
    k.solid((sx - 0.12, sy - 0.12, 0.0), (sx + 0.12, sy + 0.12, 5.4), "metal", base="sign_pole")
    k.box((sx - 1.3, sy - 0.15, 4.1), (sx + 1.3, sy + 0.15, 5.3), "sandstone_dark")
    k.box((sx - 1.22, sy + 0.14, 4.18), (sx + 1.22, sy + 0.19, 5.22), "sand_light")
    # Raised bar-letters suggesting "MOTEL"
    for bx in (-0.95, -0.45, 0.0, 0.45, 0.95):
        k.box((sx + bx - 0.14, sy + 0.19, 4.45), (sx + bx + 0.14, sy + 0.25, 4.95), "red_paint")

    k.empty("lot_size", (0.0, 0.0, 0.0), scale=(15.2, 7.2, 1.0))
    k.empty("spawn_1", (-3.0, 2.4, F))
    k.empty("spawn_2", (3.0, -0.5, F))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, top_preview=True, view=kit.FRONT_VIEW) else 1


if __name__ == "__main__":
    sys.exit(main())
