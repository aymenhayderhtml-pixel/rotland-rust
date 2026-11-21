"""shop: enterable row of 2 shop fronts under one flat roof, 16 x 8 m footprint on an 18 x 10 m lot.

Two shop fronts under one flat roof: big front windows (some broken, one boarded),
colored awnings over each, blank sign boards above each. Inside: shelves and counters,
a dividing wall between the two shops with one doorway. 4 loot (2 per shop), 2 spawns.
"""
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, HERE)

import common  # noqa: E402
import kit  # noqa: E402

NAME, CATEGORY, BUDGET = "shop", "buildings", 4000
TOP = 3.3  # flat roof wall top
F = kit.FLOOR_TOP
WALL_COLORS = ("rock_light", "white")


def _make_awning(k, x0, x1, color, valance_color=None):
    width = x1 - x0
    x_mid = (x0 + x1) / 2
    dy = 0.8
    dz = -0.4
    length = math.hypot(dy, dz)
    rot_x = math.atan2(dz, dy)

    # Sloped canvas slab
    slab = common.make_box((width, length, 0.05), color=color)
    common.transform(slab, loc=(x_mid, 3.6, 2.65), rot=(rot_x, 0.0, 0.0))
    k.model.add(slab)

    # Vertical front valance strip
    v_col = valance_color or color
    k.box((x0, 3.96, 2.30), (x1, 4.0, 2.45), v_col)

    # Metal support brackets at both ends and center
    for x in (x0 + 0.06, (x0 + x1) / 2, x1 - 0.06):
        k.box((x - 0.02, 3.2, 2.43), (x + 0.02, 3.96, 2.47), "metal")
        diag_len = math.hypot(0.6, 0.35)
        diag = common.make_box((0.03, diag_len, 0.03), color="metal")
        common.transform(diag, loc=(x, 3.5, 2.25), rot=(math.atan2(0.35, 0.6), 0.0, 0.0))
        k.model.add(diag)


def build(model):
    k = kit.Kit(model)

    # Floors: inside shops and front curb/step under awnings
    k.solid((-8.0, -4.0, 0.0), (0.0, 3.2, F), "wood_crate", base="floor_left")
    k.solid((0.0, -4.0, 0.0), (8.0, 3.2, F), "rock_light", base="floor_right")
    k.solid((-8.0, 3.2, 0.0), (8.0, 4.0, F), "rock_mid", base="curb_front")

    # Front wall openings (doors 1.2 m wide, big display windows)
    door_1 = ((-2.0, 3.05, F), (-0.8, 3.25, kit.DOOR_H))
    window_1 = ((-6.8, 3.05, 0.8), (-2.6, 3.25, 2.4))
    door_2 = ((0.8, 3.05, F), (2.0, 3.25, kit.DOOR_H))
    window_2 = ((2.6, 3.05, 0.8), (6.8, 3.25, 2.4))

    # Back wall windows for natural lighting
    back_window_1 = ((-5.5, -4.1, 1.4), (-3.5, -3.85, 2.2))
    back_window_2 = ((3.5, -4.1, 1.4), (5.5, -3.85, 2.2))

    # Dividing wall inner doorway between the two shops
    inner_door = ((-0.1, -0.6, F), (0.1, 0.6, kit.DOOR_H))

    # Exterior walls
    k.wall((-8.0, 3.05, F), (8.0, 3.2, TOP), openings=[door_1, window_1, door_2, window_2],
           colors=WALL_COLORS, course=0.3, base="wall_front")
    k.wall((-8.0, -4.0, F), (8.0, -3.85, TOP), openings=[back_window_1, back_window_2],
           colors=WALL_COLORS, course=0.3, base="wall_back")
    k.wall((-8.0, -3.85, F), (-7.85, 3.05, TOP), colors=WALL_COLORS, course=0.3, base="wall_left")
    k.wall((7.85, -3.85, F), (8.0, 3.05, TOP), colors=WALL_COLORS, course=0.3, base="wall_right")

    # Interior dividing wall between the two shops
    k.wall((-0.075, -3.85, F), (0.075, 3.05, TOP), openings=[inner_door],
           colors=WALL_COLORS, course=0.3, base="wall_divider")

    # Flat roof slab and colliders
    k.box((-8.0, -4.0, TOP), (8.0, 3.2, TOP + 0.15), "rock_dark", roof=True)
    k.collider((-8.0, -4.0, TOP), (8.0, 3.2, TOP + 0.15), "roof_slab")

    # Roof parapets
    k.solid((-8.0, -4.0, TOP + 0.15), (8.0, -3.85, TOP + 0.35), "rock_mid", base="parapet_back")
    k.solid((-8.0, 3.05, TOP + 0.15), (8.0, 3.2, TOP + 0.45), "rock_mid", base="parapet_front")
    k.solid((-8.0, -3.85, TOP + 0.15), (-7.85, 3.05, TOP + 0.35), "rock_mid", base="parapet_left")
    k.solid((7.85, -3.85, TOP + 0.15), (8.0, 3.05, TOP + 0.35), "rock_mid", base="parapet_right")

    # Rooftop details (AC units, vent pipes)
    k.box((-4.5, -2.0, TOP + 0.15), (-3.0, -0.6, TOP + 0.8), "metal", roof=True)
    k.box((-4.3, -1.8, TOP + 0.8), (-3.2, -0.8, TOP + 0.86), "gun_black", roof=True)
    k.box((2.8, -2.2, TOP + 0.15), (4.2, -0.8, TOP + 0.8), "metal", roof=True)
    k.box((3.0, -2.0, TOP + 0.8), (4.0, -1.0, TOP + 0.86), "gun_black", roof=True)
    k.box((5.5, 0.5, TOP + 0.15), (5.7, 0.7, TOP + 0.7), "metal", roof=True)
    k.box((-6.0, 0.5, TOP + 0.15), (-5.8, 0.7, TOP + 0.7), "metal", roof=True)

    # Awnings over each shop front
    _make_awning(k, -7.2, -0.6, "red_paint", valance_color="white")
    _make_awning(k, 0.6, 7.2, "shirt_blue", valance_color="white")

    # Sign boards above each shop front (blank colored panels, no text)
    # Shop 1 sign
    k.box((-7.0, 3.21, 2.88), (-1.0, 3.26, 3.25), "bark_dark")
    k.box((-6.85, 3.23, 2.92), (-1.15, 3.28, 3.21), "canvas_khaki")
    # Shop 2 sign
    k.box((1.0, 3.21, 2.88), (7.0, 3.26, 3.25), "rock_dark")
    k.box((1.15, 3.23, 2.92), (6.85, 3.28, 3.21), "cloth_olive")

    # Window frames and trim
    k.frame(door_1, 3.2, 1, "metal")
    k.frame(window_1, 3.2, 1, "metal")
    k.frame(door_2, 3.2, 1, "metal")
    k.frame(window_2, 3.2, 1, "metal")
    k.frame(back_window_1, -4.0, -1, "metal")
    k.frame(back_window_2, -4.0, -1, "metal")
    k.frame(inner_door, 0.0, 1, "wood_crate")

    # Shop 1 big window: boarded up
    k.box((-4.73, 3.12, 0.8), (-4.67, 3.24, 2.4), "metal")
    k.box((-6.8, 3.12, 1.57), (-2.6, 3.24, 1.63), "metal")
    k.boards(window_1, 3.2, 1, color="bark_light", rows=(0.25, 0.5, 0.75))

    # Shop 2 big window: broken panes with divider mullions
    k.box((4.67, 3.12, 0.8), (4.73, 3.24, 2.4), "metal")
    k.box((2.6, 3.12, 1.57), (6.8, 3.24, 1.63), "metal")
    # Remaining broken glass shard in corner
    shard = common.make_box((0.8, 0.02, 0.6), color="rock_dark")
    common.transform(shard, loc=(2.8, 3.18, 2.0), rot=(0.0, 0.0, 0.1))
    k.model.add(shard)
    # One emergency board nailed diagonally across crack
    board = common.make_box((1.6, 0.04, 0.12), color="bark_dark")
    common.transform(board, loc=(3.4, 3.26, 1.2), rot=(0.0, 0.0, 0.35))
    k.model.add(board)

    # ---- Shop 1 Interior (left) ----
    # Counter
    k.solid((-3.6, 1.2, F), (-1.4, 1.8, 1.05), "wood_crate", base="counter_1")
    k.box((-3.65, 1.15, 1.05), (-1.35, 1.85, 1.10), "rock_light")
    # Cash register
    k.box((-1.8, 1.45, 1.10), (-1.5, 1.75, 1.25), "metal")
    k.box((-1.75, 1.48, 1.25), (-1.55, 1.72, 1.28), "gun_black")

    # Left wall shelves
    k.solid((-7.85, -3.4, F), (-7.25, 1.5, 2.0), "bark_dark", base="shelf_wall_1")
    for z in (0.5, 0.9, 1.3, 1.7):
        k.box((-7.85, -3.4, z), (-7.2, 1.5, z + 0.04), "bark_light")
    # Some grocery boxes on wall shelves
    for y, col in ((-2.8, "red_paint"), (-1.8, "white"), (-0.8, "soda_purple"), (0.5, "brass")):
        k.box((-7.7, y - 0.2, 0.94), (-7.3, y + 0.2, 1.25), col)

    # Center gondola shelf
    k.solid((-5.6, -2.8, F), (-4.6, 0.4, 1.5), "metal", base="gondola_1")
    k.box((-5.65, -2.8, 0.6), (-4.55, 0.4, 0.64), "rock_dark")
    k.box((-5.65, -2.8, 1.05), (-4.55, 0.4, 1.09), "rock_dark")
    # Supplies on gondola shelf
    for y, col in ((-2.2, "wood_crate"), (-1.2, "shirt_blue"), (-0.2, "red_paint")):
        k.box((-5.4, y - 0.25, 1.09), (-4.8, y + 0.25, 1.35), col)

    # ---- Shop 2 Interior (right) ----
    # Counter
    k.solid((1.4, 1.2, F), (3.6, 1.8, 1.05), "bark_dark", base="counter_2")
    k.box((1.35, 1.15, 1.05), (3.65, 1.85, 1.10), "wood_crate")
    # Cash register
    k.box((1.5, 1.45, 1.10), (1.8, 1.75, 1.25), "metal")
    k.box((1.55, 1.48, 1.25), (1.75, 1.72, 1.28), "gun_black")

    # Right wall shelving
    k.solid((7.25, -3.4, F), (7.85, 1.5, 2.0), "metal", base="shelf_wall_2")
    for z in (0.5, 0.9, 1.3, 1.7):
        k.box((7.2, -3.4, z), (7.85, 1.5, z + 0.04), "rock_mid")

    # Back wall shelving
    k.solid((1.0, -3.85, F), (6.8, -3.3, 1.8), "wood_crate", base="shelf_back_2")
    for x, col in ((1.5, "metal"), (3.0, "red_paint"), (4.5, "rock_dark"), (5.8, "wood_crate")):
        k.box((x - 0.3, -3.75, 1.8), (x + 0.3, -3.4, 2.15), col)

    # Floor crates in Shop 2
    k.crate(5.5, -1.0, 0.5, "wood_crate", z=F)
    k.crate(5.5, -0.4, 0.4, "wood_crate", z=F)

    # Empties
    k.empty("lot_size", (0.0, 0.0, 0.0), scale=(18.0, 10.0, 1.0))
    k.empty("door_1", (-1.4, 3.2, F))
    k.empty("door_2", (1.4, 3.2, F))
    k.empty("loot_1", (-2.5, 1.5, 1.15))   # counter 1
    k.empty("loot_2", (-6.0, -1.2, F))     # floor in front of wall shelf 1
    k.empty("loot_3", (2.5, 1.5, 1.15))    # counter 2
    k.empty("loot_4", (4.0, -2.8, F))      # floor in front of back shelf 2
    k.empty("spawn_1", (-4.5, -1.2, F))    # shop 1 aisle
    k.empty("spawn_2", (4.5, -1.2, F))     # shop 2 aisle


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, top_preview=True, view=kit.FRONT_VIEW) else 1


if __name__ == "__main__":
    sys.exit(main())
