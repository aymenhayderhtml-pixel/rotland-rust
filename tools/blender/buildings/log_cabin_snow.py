"""log_cabin_snow: enterable snow-biome log cabin, 7 x 5.5 m walls, steep roof with thick snow.

Alternating bark_light/bark_dark log courses, steep gable roof under snow_white slab + snow overhang,
and a stone chimney.
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, HERE)

import common  # noqa: E402
import kit  # noqa: E402

NAME, CATEGORY, BUDGET = "log_cabin_snow", "buildings", 4000
TOP = 2.4
LOG = ("bark_light", "bark_dark")
F = kit.FLOOR_TOP


def build(model):
    k = kit.Kit(model)
    k.solid((-3.5, -2.75, 0.0), (3.5, 2.75, F), "wood_crate", base="floor")

    door = ((-0.55, 2.55, F), (0.55, 2.85, F + 6 * 0.35))
    win_l = ((-3.7, -0.9, 1.0), (-3.2, 0.3, 1.9))
    win_r = ((3.2, -0.9, 1.0), (3.7, 0.3, 1.9))
    k.wall((-3.5, 2.6, F), (3.5, 2.75, TOP), openings=[door], colors=LOG, course=0.35, base="wall_front")
    k.wall((-3.5, -2.75, F), (3.5, -2.6, TOP), colors=LOG, course=0.35, base="wall_back")
    k.wall((-3.5, -2.6, F), (-3.35, 2.6, TOP), openings=[win_l], colors=LOG, course=0.35, base="wall_left")
    k.wall((3.35, -2.6, F), (3.5, 2.6, TOP), openings=[win_r], colors=LOG, course=0.35, base="wall_right")

    # Steep gable roof (ridge along Y at z=4.1), dark timber underneath + thick snow_white slabs on top
    ridge_timber = (0.0, 0.0, 4.0)
    k.roof(ridge_timber, (3.85, 0.0, 2.25), span=(0.0, 1.0, 0.0), width=5.8, color="bark_dark", thick=0.12)
    k.roof(ridge_timber, (-3.85, 0.0, 2.25), span=(0.0, 1.0, 0.0), width=5.8, color="bark_dark", thick=0.12)

    ridge_snow = (0.0, 0.0, 4.24)
    k.roof(ridge_snow, (3.98, 0.0, 2.43), span=(0.0, 1.0, 0.0), width=6.05, color="snow_white", thick=0.22)
    k.roof(ridge_snow, (-3.98, 0.0, 2.43), span=(0.0, 1.0, 0.0), width=6.05, color="snow_white", thick=0.22)

    # Snow edge overhang along both eaves
    for ex in (-3.98, 3.82):
        k.box((ex, -3.02, 2.15), (ex + 0.16, 3.02, 2.38), "snow_shadow", roof=True)

    # Gable ends
    k.gable([(-3.7, 2.61, 2.26), (3.7, 2.61, 2.26), (0.0, 2.61, 3.92)], (0.0, 0.12, 0.0), "bark_dark", roof=True)
    k.gable([(-3.7, -2.73, 2.26), (3.7, -2.73, 2.26), (0.0, -2.73, 3.92)], (0.0, 0.12, 0.0), "bark_dark", roof=True)

    # Corner log posts
    for px, py in ((-3.5, -2.75), (3.5, -2.75), (-3.5, 2.75), (3.5, 2.75)):
        k.box((px - 0.12, py - 0.12, 0.02), (px + 0.12, py + 0.12, 2.46), "bark_dark")

    # Frames and stone chimney with snow cap
    k.frame(door, 2.77, 1, "bark_dark", sill=False)
    k.frame(win_l, -3.52, -1, "bark_dark")
    k.frame(win_r, 3.52, 1, "bark_dark")
    k.chimney(-1.8, -1.3, 3.0, 4.5, "rock_mid")
    k.box((-2.12, -1.62, 4.6), (-1.48, -0.98, 4.72), "snow_white", roof=True)

    # Interior: bed, table, shelf, crate
    k.solid((-3.2, -2.4, F), (-1.9, -0.4, 0.45), "bark_dark", base="bed")
    k.box((-3.15, -2.35, 0.45), (-1.95, -0.45, 0.62), "white")
    k.solid((1.2, -0.5, F), (2.3, 0.5, 0.75), "bark_dark", base="table")
    k.solid((-1.0, -2.55, F), (0.6, -2.15, 1.8), "bark_dark", base="shelf")
    k.crate(2.8, -2.1, 0.55, z=F)

    k.empty("lot_size", (0.0, 0.0, 0.0), scale=(8.0, 6.1, 1.0))
    k.empty("door_1", (0.0, 2.675, F))
    k.empty("loot_1", (1.75, 0.0, 0.75))
    k.empty("loot_2", (-0.2, -2.35, 1.85))
    k.empty("loot_3", (-2.55, -1.4, 0.62))
    k.empty("spawn_1", (-1.8, 1.4, F))
    k.empty("spawn_2", (1.8, -1.4, F))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, top_preview=True, view=kit.FRONT_VIEW) else 1


if __name__ == "__main__":
    sys.exit(main())
