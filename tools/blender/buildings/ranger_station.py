"""ranger_station: 2-storey wooden lookout lodge with wrap-around balcony, stair, and sign board (<= 5000 tris).

Ground floor (8 x 7 m) is walk-in from +Y. Exterior stair along +X leads to the upper wrap-around
balcony (z=2.75) and upper lookout room.
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, HERE)

import common  # noqa: E402
import kit  # noqa: E402

NAME, CATEGORY, BUDGET = "ranger_station", "buildings", 5000
F = kit.FLOOR_TOP
TOP1 = 2.6
F2 = 2.75
TOP2 = 5.0
RIDGE_Z = 6.2
LOG = ("bark_light", "bark_dark")


def build(model):
    k = kit.Kit(model)

    # ---- Ground floor (X: -4.0..4.0, Y: -3.5..3.5) ----
    k.solid((-4.0, -3.5, 0.0), (4.0, 3.5, F), "wood_crate", base="floor1")
    door1 = ((-0.6, 3.3, F), (0.6, 3.6, kit.DOOR_H))
    win_f1 = ((1.5, 3.3, 1.0), (2.9, 3.6, 1.9))
    win_b1 = ((-2.5, -3.6, 1.0), (-0.9, -3.3, 1.9))
    win_l1 = ((-4.1, -1.2, 1.0), (-3.8, 0.8, 1.9))
    k.wall((-4.0, 3.35, F), (4.0, 3.5, TOP1), openings=[door1, win_f1], colors=LOG, course=0.35, base="w1_front")
    k.wall((-4.0, -3.5, F), (4.0, -3.35, TOP1), openings=[win_b1], colors=LOG, course=0.35, base="w1_back")
    k.wall((-4.0, -3.35, F), (-3.85, 3.35, TOP1), openings=[win_l1], colors=LOG, course=0.35, base="w1_left")
    k.wall((3.85, -3.35, F), (4.0, 3.35, TOP1), colors=LOG, course=0.35, base="w1_right")

    k.frame(door1, 3.52, 1, "bark_dark", sill=False)
    k.frame(win_f1, 3.52, 1, "bark_dark")
    k.frame(win_b1, -3.52, -1, "bark_dark")
    k.frame(win_l1, -4.02, -1, "bark_dark")

    # ---- Upper floor / wrap-around balcony deck (X: -4.3..4.3, Y: -3.8..3.8, z=2.6..2.75) ----
    k.solid((-4.3, -3.8, TOP1), (4.3, 3.8, F2), "wood_crate", base="floor2")
    # Balcony corner posts supporting the upper eaves
    for px in (-4.15, 4.15):
        for py in (-3.65, 3.65):
            k.box((px - 0.1, py - 0.1, F2), (px + 0.1, py + 0.1, TOP2), "bark_dark")

    # Balcony railings (solid colliders, leaving 1.2 m stair entry open on +X at Y: -2.4..-1.2)
    rail_z = F2 + 0.9
    k.solid((-4.3, 3.68, F2), (4.3, 3.8, rail_z), "bark_dark", base="rail_f")
    k.solid((-4.3, -3.8, F2), (4.3, -3.68, rail_z), "bark_dark", base="rail_b")
    k.solid((-4.3, -3.68, F2), (-4.18, 3.68, rail_z), "bark_dark", base="rail_l")
    k.solid((4.18, -3.68, F2), (4.3, -2.4, rail_z), "bark_dark", base="rail_r1")
    k.solid((4.18, -1.2, F2), (4.3, 3.68, rail_z), "bark_dark", base="rail_r2")

    # ---- Exterior stair along +X (X: 4.3..5.3, rising from Y=2.4, z=0 to Y=-1.2, z=2.75) ----
    steps = 12
    for s in range(steps):
        t0, t1 = s / steps, (s + 1) / steps
        sz = t1 * F2
        sy1 = 2.4 - t0 * 3.6
        sy0 = 2.4 - t1 * 3.6
        k.solid((4.3, sy0, max(0.0, sz - 0.28)), (5.3, sy1, sz), "bark_light", base=f"step_{s}")
    k.solid((4.3, -2.4, TOP1), (5.3, -1.2, F2), "bark_light", base="stair_landing")
    k.solid((5.22, -2.4, F2), (5.3, 2.4, F2 + 0.9), "bark_dark", base="stair_rail")

    # ---- Upper lookout room (X: -2.8..2.8, Y: -2.3..2.3, z=F2..TOP2) ----
    door2 = ((2.6, -0.5, F2), (2.95, 0.5, F2 + 2.0))
    win_u_f = ((-1.8, 2.1, F2 + 0.8), (1.8, 2.45, F2 + 1.8))
    win_u_b = ((-1.8, -2.45, F2 + 0.8), (1.8, -2.1, F2 + 1.8))
    win_u_l = ((-2.95, -1.5, F2 + 0.8), (-2.6, 1.5, F2 + 1.8))
    k.wall((-2.8, 2.15, F2), (2.8, 2.3, TOP2), openings=[win_u_f], colors=LOG, course=0.35, base="w2_front")
    k.wall((-2.8, -2.3, F2), (2.8, -2.15, TOP2), openings=[win_u_b], colors=LOG, course=0.35, base="w2_back")
    k.wall((-2.8, -2.15, F2), (-2.65, 2.15, TOP2), openings=[win_u_l], colors=LOG, course=0.35, base="w2_left")
    k.wall((2.65, -2.15, F2), (2.8, 2.15, TOP2), openings=[door2], colors=LOG, course=0.35, base="w2_right")

    # Gable roof over upper room (ridge along X)
    ridge = (0.0, 0.0, RIDGE_Z)
    k.roof(ridge, (0.0, -3.8, TOP2), span=(1.0, 0.0, 0.0), width=8.8, color="pine_dark")
    k.roof(ridge, (0.0, 3.8, TOP2), span=(1.0, 0.0, 0.0), width=8.8, color="pine_dark")
    for x, off_x in ((-2.8, 0.12), (2.68, 0.12)):
        k.gable([(x, -2.3, TOP2 - 0.1), (x, 2.3, TOP2 - 0.1), (x, 0.0, RIDGE_Z - 0.12)],
                (off_x, 0.0, 0.0), "bark_dark", roof=True)
    k.ridge_cap((0.0, 0.0, RIDGE_Z), 0, 8.9, "rock_dark")

    # Sign board on the front balcony fascia
    k.box((-1.9, 3.80, F2), (1.9, 3.92, F2 + 0.80), "bark_dark")
    k.box((-1.75, 3.90, F2 + 0.12), (1.75, 3.95, F2 + 0.68), "white")
    k.box((-1.6, 3.94, F2 + 0.34), (1.6, 3.97, F2 + 0.46), "leaf_dark")

    # Interior ground floor: ranger desk, map table, shelf
    k.solid((-3.5, -3.2, F), (-1.2, -2.4, 0.85), "bark_dark", base="desk")
    k.solid((1.0, -1.0, F), (2.6, 0.6, 0.78), "wood_crate", base="map_table")
    k.box((1.2, -0.7, 0.78), (2.4, 0.3, 0.81), "white")
    k.solid((1.5, -3.3, F), (3.6, -2.8, 1.8), "bark_dark", base="shelf")

    # Interior upper floor: bunk + crate
    k.solid((-2.5, -2.0, F2), (-1.3, 0.0, F2 + 0.45), "bark_dark", base="bed")
    k.box((-2.45, -1.95, F2 + 0.45), (-1.35, -0.05, F2 + 0.62), "white")
    k.crate(-2.0, 1.4, 0.55, z=F2)

    k.empty("lot_size", (0.0, 0.0, 0.0), scale=(11.0, 8.0, 1.0))
    k.empty("door_1", (0.0, 3.425, F))
    k.empty("loot_1", (-2.3, -2.8, 0.85))  # ground desk
    k.empty("loot_2", (1.8, -0.2, 0.81))   # map table
    k.empty("loot_3", (-2.0, 1.4, F2 + 0.55))  # upper crate
    k.empty("loot_4", (-1.9, -1.0, F2 + 0.62))  # upper bed
    k.empty("spawn_1", (-1.5, 1.2, F))
    k.empty("spawn_2", (0.8, 0.8, F2))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, top_preview=True, view=kit.FRONT_VIEW) else 1


if __name__ == "__main__":
    sys.exit(main())
