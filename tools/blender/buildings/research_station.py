"""research_station: 2 connected arctic metal modules on short stilts, with steps, antenna, and dish.

Main lab module (6 x 4.5 m) + corridor + side bunk module (3.5 x 3.5 m). Walk-in via front steps.
"""
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, HERE)

import common  # noqa: E402
import kit  # noqa: E402

NAME, CATEGORY, BUDGET = "research_station", "buildings", 5000
STILT = 0.5
F = STILT + kit.FLOOR_TOP  # 0.6 m floor top
TOP = 3.1
PANELS = ("white", "rock_light")


def build(model):
    k = kit.Kit(model)

    # Short metal stilts under both modules and corridor
    stilt_pts = [
        (-4.3, -2.3), (-1.5, -2.3), (1.3, -2.3),
        (-4.3, 1.8), (-1.5, 1.8), (1.3, 1.8),
        (2.7, -1.8), (5.8, -1.8), (2.7, 1.3), (5.8, 1.3),
    ]
    for sx, sy in stilt_pts:
        k.solid((sx - 0.1, sy - 0.1, 0.0), (sx + 0.1, sy + 0.1, STILT), "metal", base="stilt")

    # Elevated floors
    k.solid((-4.5, -2.5, STILT), (1.5, 2.0, F), "rock_mid", base="floor_main")
    k.solid((1.5, -1.0, STILT), (2.5, 0.8, F), "rock_mid", base="floor_link")
    k.solid((2.5, -2.0, STILT), (6.0, 1.5, F), "rock_mid", base="floor_side")

    # Front steps up to main door (x -2.2..-1.1, y 2.0..2.9)
    k.solid((-2.3, 2.6, 0.0), (-1.0, 2.9, 0.2), "metal", base="step_1")
    k.solid((-2.3, 2.3, 0.0), (-1.0, 2.6, 0.4), "metal", base="step_2")
    k.solid((-2.3, 2.0, 0.0), (-1.0, 2.3, F), "metal", base="step_3")

    # Openings
    door = ((-2.2, 1.8, F), (-1.1, 2.1, F + 2.1))
    win_front = ((0.0, 1.8, F + 0.8), (1.0, 2.1, F + 1.6))
    win_left = ((-4.7, -0.8, F + 0.8), (-4.3, 0.6, F + 1.6))
    link_door_l = ((1.3, -0.6, F), (1.7, 0.4, F + 2.0))
    link_door_r = ((2.3, -0.6, F), (2.7, 0.4, F + 2.0))

    # Main module walls
    k.wall((-4.5, 1.85, F), (1.5, 2.0, TOP), openings=[door, win_front], colors=PANELS, course=0.5, base="m_front")
    k.wall((-4.5, -2.5, F), (1.5, -2.35, TOP), colors=PANELS, course=0.5, base="m_back")
    k.wall((-4.5, -2.35, F), (-4.35, 1.85, TOP), openings=[win_left], colors=PANELS, course=0.5, base="m_left")
    k.wall((1.35, -2.35, F), (1.5, 1.85, TOP), openings=[link_door_l], colors=PANELS, course=0.5, base="m_right")

    # Corridor walls
    k.wall((1.5, 0.65, F), (2.5, 0.8, TOP - 0.2), colors=PANELS, course=0.5, base="c_front")
    k.wall((1.5, -1.0, F), (2.5, -0.85, TOP - 0.2), colors=PANELS, course=0.5, base="c_back")

    # Side module walls
    k.wall((2.5, 1.35, F), (6.0, 1.5, TOP), colors=PANELS, course=0.5, base="s_front")
    k.wall((2.5, -2.0, F), (6.0, -1.85, TOP), colors=PANELS, course=0.5, base="s_back")
    k.wall((2.5, -1.85, F), (2.65, 1.35, TOP), openings=[link_door_r], colors=PANELS, course=0.5, base="s_left")
    k.wall((5.85, -1.85, F), (6.0, 1.35, TOP), colors=PANELS, course=0.5, base="s_right")

    # Roofs (metal + snow_white trim cap)
    k.box((-4.55, -2.55, TOP), (1.55, 2.05, TOP + 0.18), "metal", roof=True)
    k.collider((-4.55, -2.55, TOP), (1.55, 2.05, TOP + 0.18), "roof_main")
    k.box((1.5, -1.02, TOP - 0.2), (2.5, 0.82, TOP - 0.05), "metal", roof=True)
    k.box((2.45, -2.05, TOP), (6.05, 1.55, TOP + 0.18), "metal", roof=True)
    k.collider((2.45, -2.05, TOP), (6.05, 1.55, TOP + 0.18), "roof_side")

    # Antenna mast + satellite dish on roof
    k.box((-3.6, -1.6, TOP + 0.18), (-3.48, -1.48, TOP + 2.4), "metal", roof=True)
    k.box((-3.9, -1.56, TOP + 2.0), (-3.18, -1.52, TOP + 2.04), "metal", roof=True)
    dish = common.make_frustum(0.65, 0.15, 0.0, 0.28, segments=10, color="white")
    common.transform(dish, loc=(4.6, -0.5, TOP + 0.6), rot=(math.radians(55), 0.0, math.radians(-30)))
    model.add(dish, roof=True)
    k.box((4.52, -0.58, TOP + 0.18), (4.68, -0.42, TOP + 0.65), "metal", roof=True)

    # Window/door frames
    k.frame(door, 2.02, 1, "metal", sill=False)
    k.frame(win_front, 2.02, 1, "metal")
    k.frame(win_left, -4.52, -1, "metal")

    # Interior: lab desk, server rack, bunk in side module
    k.solid((-4.2, -2.2, F), (-2.2, -1.5, F + 0.8), "metal", base="desk")
    k.solid((-0.8, -2.2, F), (0.2, -1.6, F + 1.8), "gun_black", base="rack")
    k.box((-0.7, -1.6, F + 1.1), (0.1, -1.58, F + 1.4), "water_blue")
    k.solid((3.8, -1.7, F), (5.6, -0.7, F + 0.45), "metal", base="bunk")
    k.box((3.85, -1.65, F + 0.45), (5.55, -0.75, F + 0.6), "white")

    k.empty("lot_size", (0.0, 0.0, 0.0), scale=(11.0, 5.5, 1.0))
    k.empty("door_1", (-1.65, 1.925, F))
    k.empty("loot_1", (-3.2, -1.85, F + 0.8))
    k.empty("loot_2", (4.7, -1.2, F + 0.6))
    k.empty("loot_3", (0.8, 0.0, F))
    k.empty("spawn_1", (-2.0, -0.5, F))
    k.empty("spawn_2", (3.5, 0.5, F))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, top_preview=True, view=kit.FRONT_VIEW) else 1


if __name__ == "__main__":
    sys.exit(main())
