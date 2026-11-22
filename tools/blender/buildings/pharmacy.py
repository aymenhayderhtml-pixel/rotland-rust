"""pharmacy: enterable small one-floor pharmacy, 10 x 8 m footprint on a 12 x 10 m lot.

White and green walls, green cross sign (made of boxes) above the door, front window,
inside a counter and 3 shelf rows. 3 loot, 2 spawns.
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, HERE)

import common  # noqa: E402
import kit  # noqa: E402

NAME, CATEGORY, BUDGET = "pharmacy", "buildings", 3000
TOP = 3.3
F = kit.FLOOR_TOP
WALL_COLORS = ("leaf_mid", "white", "white")  # green base wainscot, white upper


def build(model):
    k = kit.Kit(model)

    # Floor and front curb
    k.solid((-5.0, -4.0, 0.0), (5.0, 3.4, F), "white", base="floor")
    k.solid((-5.0, 3.4, 0.0), (5.0, 4.0, F), "rock_mid", base="curb_front")

    # Front door (1.2 m wide) and front display window
    door = ((2.0, 3.25, F), (3.2, 3.45, kit.DOOR_H))
    window = ((-4.2, 3.25, 0.8), (1.4, 3.45, 2.4))
    back_window = ((-2.0, -4.1, 1.4), (0.0, -3.85, 2.2))

    # Exterior walls
    k.wall((-5.0, 3.25, F), (5.0, 3.4, TOP), openings=[door, window],
           colors=WALL_COLORS, course=1.1, base="wall_front")
    k.wall((-5.0, -4.0, F), (5.0, -3.85, TOP), openings=[back_window],
           colors=WALL_COLORS, course=1.1, base="wall_back")
    k.wall((-5.0, -3.85, F), (-4.85, 3.25, TOP), colors=WALL_COLORS, course=1.1, base="wall_left")
    k.wall((4.85, -3.85, F), (5.0, 3.25, TOP), colors=WALL_COLORS, course=1.1, base="wall_right")

    # Flat roof slab and colliders
    k.box((-5.0, -4.0, TOP), (5.0, 3.4, TOP + 0.15), "rock_dark", roof=True)
    k.collider((-5.0, -4.0, TOP), (5.0, 3.4, TOP + 0.15), "roof_slab")

    # Green roof parapets
    k.solid((-5.0, -4.0, TOP + 0.15), (5.0, -3.85, TOP + 0.35), "leaf_mid", base="parapet_back")
    k.solid((-5.0, 3.25, TOP + 0.15), (5.0, 3.4, TOP + 0.45), "leaf_mid", base="parapet_front")
    k.solid((-5.0, -3.85, TOP + 0.15), (-4.85, 3.25, TOP + 0.35), "leaf_mid", base="parapet_left")
    k.solid((4.85, -3.85, TOP + 0.15), (5.0, 3.25, TOP + 0.35), "leaf_mid", base="parapet_right")

    # Rooftop ventilation unit
    k.box((1.5, -1.8, TOP + 0.15), (3.0, -0.4, TOP + 0.75), "metal", roof=True)
    k.box((1.7, -1.6, TOP + 0.75), (2.8, -0.6, TOP + 0.82), "rock_dark", roof=True)
    k.box((-3.5, -2.0, TOP + 0.15), (-3.2, -1.7, TOP + 0.8), "metal", roof=True)

    # Green cross sign (made of boxes) above the door
    # White mounting square
    k.box((2.15, 3.41, 2.35), (3.05, 3.45, 3.15), "white")
    # Green cross bars (non-overlapping)
    k.box((2.45, 3.45, 2.45), (2.75, 3.51, 3.05), "leaf_mid")
    k.box((2.25, 3.45, 2.65), (2.45, 3.51, 2.85), "leaf_mid")
    k.box((2.75, 3.45, 2.65), (2.95, 3.51, 2.85), "leaf_mid")

    # Projecting green cross blade sign on corner bracket (visible along street)
    k.box((4.25, 3.4, 2.78), (4.35, 3.9, 2.84), "metal")
    k.box((4.27, 3.55, 2.50), (4.33, 3.85, 2.80), "white")
    k.box((4.26, 3.65, 2.53), (4.34, 3.75, 2.77), "leaf_mid")
    k.box((4.26, 3.57, 2.62), (4.34, 3.65, 2.70), "leaf_mid")
    k.box((4.26, 3.75, 2.62), (4.34, 3.83, 2.70), "leaf_mid")

    # Frames for door and windows
    k.frame(door, 3.4, 1, "metal")
    k.frame(window, 3.4, 1, "metal")
    k.frame(back_window, -4.0, -1, "metal")

    # Window mullions and display
    k.box((-2.83, 3.32, 0.82), (-2.77, 3.44, 2.38), "metal")
    k.box((-1.43, 3.32, 0.82), (-1.37, 3.44, 2.38), "metal")
    k.box((-0.03, 3.32, 0.82), (0.03, 3.44, 2.38), "metal")
    k.box((-4.18, 3.31, 1.57), (1.38, 3.45, 1.63), "metal")

    # Window boards over left pane
    k.boards(((-4.2, 3.25, 0.8), (-2.8, 3.45, 2.4)), 3.4, 1, "bark_light", rows=(0.3, 0.7))

    # ---- Pharmacy Interior ----
    # Prescription counter (L-shaped)
    k.solid((0.2, 1.0, F), (3.8, 1.6, 1.05), "leaf_mid", base="counter_main")
    k.box((0.15, 0.95, 1.05), (3.85, 1.65, 1.10), "white")
    k.solid((3.2, -1.0, F), (3.8, 1.0, 1.05), "leaf_mid", base="counter_side")
    k.box((3.15, -1.05, 1.05), (3.85, 0.95, 1.10), "white")
    # Cash register
    k.box((1.0, 1.25, 1.10), (1.3, 1.55, 1.25), "metal")
    k.box((1.05, 1.28, 1.25), (1.25, 1.52, 1.28), "gun_black")

    # Back wall medicine cabinets (behind counter)
    k.solid((0.5, -3.85, F), (4.5, -3.3, 2.2), "white", base="cabinet_back")
    for z in (0.5, 0.9, 1.3, 1.7, 2.1):
        k.box((0.52, -3.83, z), (4.48, -3.28, z + 0.04), "leaf_mid")
    # Medicine boxes and bottles on back shelves
    for x, col in ((0.8, "red_paint"), (1.6, "water_blue"), (2.4, "white"), (3.2, "leaf_dark"), (4.0, "red_paint")):
        k.box((x - 0.25, -3.75, 1.34), (x + 0.25, -3.4, 1.68), col)

    # 3 Shelf Rows (parallel aisles on customer side)
    shelf_xs = [(-4.4, -3.8), (-2.8, -2.2), (-1.2, -0.6)]
    for i, (x0, x1) in enumerate(shelf_xs, 1):
        k.solid((x0, -2.75, F), (x1, 1.75, 1.7), "white", base=f"shelf_row_{i}")
        # End caps in medical green
        k.box((x0 - 0.02, 1.75, F), (x1 + 0.02, 1.82, 1.72), "leaf_mid")
        k.box((x0 - 0.02, -2.82, F), (x1 + 0.02, -2.75, 1.72), "leaf_mid")
        # Top cap
        k.box((x0 - 0.02, -2.75, 1.7), (x1 + 0.02, 1.75, 1.75), "leaf_mid")
        # Medicine packages on shelves
        for y, col in ((-2.0, "red_paint"), (-1.0, "water_blue"), (0.0, "leaf_dark"), (1.0, "white")):
            k.box((x0 - 0.03, y - 0.2, 0.9), (x1 + 0.03, y + 0.2, 1.15), col)

    # Empties
    k.empty("lot_size", (0.0, 0.0, 0.0), scale=(12.0, 10.0, 1.0))
    k.empty("door_1", (2.6, 3.4, F))
    k.empty("loot_1", (2.2, 1.3, 1.15))    # on prescription counter
    k.empty("loot_2", (-2.0, -0.5, F))     # floor in customer aisle
    k.empty("loot_3", (2.0, -3.0, F))      # floor in front of back medicine cabinets
    k.empty("spawn_1", (-3.3, -0.8, F))    # in customer aisle
    k.empty("spawn_2", (2.0, -1.2, F))     # behind counter


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, top_preview=True, view=kit.FRONT_VIEW) else 1


if __name__ == "__main__":
    sys.exit(main())
