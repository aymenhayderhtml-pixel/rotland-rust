"""adobe_house: enterable one-storey flat-roof desert mud house, 7 x 5 m footprint.

Sand_light/sand_dark walls with beveled rounded corners, wooden roof beams (vigas) sticking out,
parapet rim, a doorway on +Y, and two small windows.
"""
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, HERE)

import common  # noqa: E402
import kit  # noqa: E402

NAME, CATEGORY, BUDGET = "adobe_house", "buildings", 4000
TOP = 2.6
F = kit.FLOOR_TOP


def build(model):
    k = kit.Kit(model)
    k.solid((-3.5, -2.5, 0.0), (3.5, 2.5, F), "sand_dark", base="floor")

    door = ((-0.55, 2.3, F), (0.55, 2.6, kit.DOOR_H))
    win_front = ((1.3, 2.3, 1.0), (2.3, 2.6, 1.85))
    win_left = ((-3.7, -0.6, 1.0), (-3.3, 0.6, 1.85))

    k.wall((-3.5, 2.35, F), (3.5, 2.5, TOP), openings=[door, win_front], color="sand_light", base="wall_front")
    k.wall((-3.5, -2.5, F), (3.5, -2.35, TOP), color="sand_light", base="wall_back")
    k.wall((-3.5, -2.35, F), (-3.35, 2.35, TOP), openings=[win_left], color="sand_light", base="wall_left")
    k.wall((3.35, -2.35, F), (3.5, 2.35, TOP), color="sand_light", base="wall_right")

    # Rounded adobe corners (8-sided pillars at the 4 corners, matching parapet height)
    for cx in (-3.45, 3.45):
        for cy in (-2.45, 2.45):
            model.add(common.make_frustum(0.22, 0.20, 0.0, TOP + 0.42, segments=8, center=(cx, cy), color="sand_light"))

    # Flat roof + low parapet rim
    k.box((-3.45, -2.45, TOP), (3.45, 2.45, TOP + 0.16), "sand_dark", roof=True)
    k.collider((-3.45, -2.45, TOP), (3.45, 2.45, TOP + 0.16), "roof")
    k.box((-3.5, 2.32, TOP + 0.16), (3.5, 2.5, TOP + 0.42), "sand_light", roof=True)
    k.box((-3.5, -2.5, TOP + 0.16), (3.5, -2.32, TOP + 0.42), "sand_light", roof=True)
    k.box((-3.5, -2.32, TOP + 0.16), (-3.32, 2.32, TOP + 0.42), "sand_light", roof=True)
    k.box((3.32, -2.32, TOP + 0.16), (3.5, 2.32, TOP + 0.42), "sand_light", roof=True)

    # Wooden roof beams (vigas) sticking out along +Y and -Y
    for vx in (-2.6, -1.3, 0.0, 1.3, 2.6):
        beam = common.make_frustum(0.08, 0.08, -2.85, 2.85, segments=6, color="bark_dark")
        common.transform(beam, loc=(vx, 0.0, TOP + 0.04), rot=(math.pi / 2, 0.0, 0.0))
        model.add(beam, roof=True)

    # Trim frames
    k.frame(door, 2.52, 1, "bark_dark", sill=False)
    k.frame(win_front, 2.52, 1, "bark_dark")
    k.frame(win_left, -3.52, -1, "bark_dark")

    # Interior: bed, table, shelf, crate
    k.solid((-3.2, -2.2, F), (-1.9, -0.3, 0.45), "bark_dark", base="bed")
    k.box((-3.15, -2.15, 0.45), (-1.95, -0.35, 0.62), "sand_light")
    k.solid((1.2, -0.8, F), (2.4, 0.2, 0.75), "bark_light", base="table")
    k.solid((-0.4, -2.3, F), (1.0, -1.9, 1.7), "bark_dark", base="shelf")
    k.crate(2.8, -1.9, 0.55, z=F)

    k.empty("lot_size", (0.0, 0.0, 0.0), scale=(7.0, 5.0, 1.0))
    k.empty("door_1", (0.0, 2.425, F))
    k.empty("loot_1", (1.8, -0.3, 0.75))  # table
    k.empty("loot_2", (-2.55, -1.25, 0.62))  # bed
    k.empty("loot_3", (0.3, -2.1, 1.75))  # shelf
    k.empty("spawn_1", (-1.5, 1.0, F))
    k.empty("spawn_2", (2.0, 1.2, F))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, top_preview=True, view=kit.FRONT_VIEW) else 1


if __name__ == "__main__":
    sys.exit(main())
