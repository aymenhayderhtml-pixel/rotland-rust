"""fish_shack: enterable 8 x 6 m fishing shack with nets, wall buoys, and a smoke stack (<= 2500 tris).

Enterable wooden shack with gable roof. Features exterior fishing nets, hanging buoys,
and a metal smoke stack pipe. Interior has fish cleaning table, stove, and clutter.
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, HERE)

import common  # noqa: E402
import kit  # noqa: E402

NAME, CATEGORY, BUDGET = "fish_shack", "buildings", 2500
TOP = 2.6
RIDGE_Z = 3.8
F = kit.FLOOR_TOP
WALL_COLORS = ("bark_light", "wood_crate", "bark_dark")


def build(model):
    k = kit.Kit(model)

    # Floor: 8.0 x 6.0 m (X: -4.0 to 4.0, Y: -3.0 to 3.0)
    k.solid((-4.0, -3.0, 0.0), (4.0, 3.0, F), "wood_crate", base="floor")

    # Openings:
    # Front door (+Y): X: -0.6 to 0.6, Z: F to kit.DOOR_H (2.2 m)
    door = ((-0.6, 2.85, F), (0.6, 3.15, kit.DOOR_H))
    # Front window (+Y, to the right of door)
    win_front = ((1.8, 2.85, 1.0), (3.2, 3.15, 1.9))
    # Back window (-Y)
    win_back = ((-2.2, -3.15, 1.0), (-0.8, -2.85, 1.9))
    # Right side window (+X)
    win_side = ((3.85, -0.6, 1.0), (4.15, 0.8, 1.9))

    # Exterior walls
    k.wall((-4.0, 2.85, F), (4.0, 3.0, TOP), openings=[door, win_front],
           colors=WALL_COLORS, course=0.36, base="wall_front")
    k.wall((-4.0, -3.0, F), (4.0, -2.85, TOP), openings=[win_back],
           colors=WALL_COLORS, course=0.36, base="wall_back")
    k.wall((-4.0, -2.85, F), (-3.85, 2.85, TOP),
           colors=WALL_COLORS, course=0.36, base="wall_left")
    k.wall((3.85, -2.85, F), (4.0, 2.85, TOP), openings=[win_side],
           colors=WALL_COLORS, course=0.36, base="wall_right")

    # Gable roof: ridge along Y (X = 0, Z = RIDGE_Z), eaves at X = ±4.3 m, Z = 2.4 m
    ridge = (0.0, 0.0, RIDGE_Z)
    eave_l = (-4.3, 0.0, 2.4)
    eave_r = (4.3, 0.0, 2.4)
    k.roof(ridge, eave_l, span=(0.0, 1.0, 0.0), width=6.4, color="rock_mid")
    k.roof(ridge, eave_r, span=(0.0, 1.0, 0.0), width=6.4, color="rock_mid")

    # Gable ends
    k.gable([(-4.0, 2.85, TOP), (4.0, 2.85, TOP), (0.0, 2.85, RIDGE_Z)], (0.0, 0.15, 0.0), "bark_dark", roof=True)
    k.gable([(-4.0, -3.0, TOP), (4.0, -3.0, TOP), (0.0, -3.0, RIDGE_Z)], (0.0, 0.15, 0.0), "bark_dark", roof=True)
    k.ridge_cap((0.0, 0.0, RIDGE_Z), 1, 6.5, "bark_dark")

    # Door and window trims
    def clean_frame(opening, face, sign, color="bark_dark", t=0.08, depth=0.05, sill=True):
        lo, hi = opening
        axis = 0 if (hi[0] - lo[0]) < (hi[1] - lo[1]) else 1
        u = 1 - axis
        a, b = sorted((face, face + sign * depth))
        window = lo[2] > F + 0.05
        z_bot = lo[2]
        if window and sill:
            p0, p1 = [0.0, 0.0, lo[2] - t], [0.0, 0.0, lo[2]]
            p0[u], p1[u] = lo[u] - t * 1.5, hi[u] + t * 1.5
            p0[axis], p1[axis] = (a - 0.04, b) if sign < 0 else (a, b + 0.04)
            k.box(tuple(p0), tuple(p1), color)
        z_top = hi[2]
        for u0, u1 in ((lo[u] - t, lo[u]), (hi[u], hi[u] + t)):
            p0, p1 = [0.0, 0.0, z_bot], [0.0, 0.0, z_top]
            p0[u], p1[u] = u0, u1
            p0[axis], p1[axis] = a, b
            k.box(tuple(p0), tuple(p1), color)
        p0, p1 = [0.0, 0.0, z_top], [0.0, 0.0, z_top + t]
        p0[u], p1[u] = lo[u] - t, hi[u] + t
        p0[axis], p1[axis] = a, b
        k.box(tuple(p0), tuple(p1), color)

    clean_frame(door, 3.02, 1, "bark_dark", sill=False)
    clean_frame(win_front, 3.02, 1, "bark_dark")
    clean_frame(win_back, -3.02, -1, "bark_dark")
    k.boards(win_back, -3.03, -1)
    clean_frame(win_side, 4.02, 1, "bark_dark")

    # Corner posts (lifted 0.02 m above ground to avoid floor bottom coplanarity)
    for px, py in ((-4.0, -3.0), (4.0, -3.0), (-4.0, 3.0), (4.0, 3.0)):
        k.box((px - 0.11, py - 0.11, 0.02), (px + 0.11, py + 0.11, TOP + 0.02), "bark_dark")

    # ---- Signature Lake Detail 1: Smoke stack ----
    # Metal stove pipe penetrating roof at X = -1.8, Y = -1.2, rising to Z = 4.3 m
    pipe = common.make_frustum(0.12, 0.11, 2.8, 4.3, segments=6, center=(-1.8, -1.2), color="gun_black")
    pipe_cap = common.make_frustum(0.24, 0.04, 4.3, 4.45, segments=5, center=(-1.8, -1.2), color="gun_black")
    pipe_collar = common.make_frustum(0.18, 0.18, 3.3, 3.42, segments=6, center=(-1.8, -1.2), color="metal")
    k.model.add(pipe, roof=True)
    k.model.add(pipe_cap, roof=True)
    k.model.add(pipe_collar, roof=True)

    # ---- Signature Lake Detail 2: Fishing nets on exterior left wall (-X) ----
    # Hanging wooden rack bar (proud of wall x = -4.0)
    k.box((-4.14, -2.0, 2.15), (-4.02, 2.0, 2.22), "bark_dark")
    # Net strips forming a draped mesh look
    for ny in (-1.5, -0.75, 0.0, 0.75, 1.5):
        k.box((-4.08, ny - 0.28, 0.6), (-4.04, ny + 0.28, 2.15), "canvas_khaki")
    # Horizontal cord lines
    k.box((-4.10, -1.9, 1.0), (-4.05, 1.9, 1.04), "cloth_olive")
    k.box((-4.10, -1.9, 1.5), (-4.05, 1.9, 1.54), "cloth_olive")
    # Cork/wood floats along top of net
    for fy in (-1.4, -0.7, 0.0, 0.7, 1.4):
        k.box((-4.15, fy - 0.06, 2.08), (-4.03, fy + 0.06, 2.20), "wheat_gold")

    # ---- Signature Lake Detail 3: Cluster of buoys hanging on front wall beside door ----
    # Hanging peg rail
    k.box((-2.8, 3.01, 1.85), (-1.0, 3.08, 1.92), "bark_dark")
    # 4 colorful buoys hanging from pegs
    buoy_specs = [
        (-2.5, "red_paint", "white"),
        (-2.0, "shirt_blue", "white"),
        (-1.5, "wheat_gold", "red_paint"),
        (-1.1, "water_blue", "white"),
    ]
    for bx, c_body, c_tip in buoy_specs:
        peg = common.make_box((0.03, 0.10, 0.03), loc=(bx, 3.08, 1.88), color="bark_light")
        cord = common.make_box((0.015, 0.015, 0.22), loc=(bx, 3.12, 1.75), color="wheat_gold")
        b_body = common.make_frustum(0.08, 0.06, 1.35, 1.65, segments=5, center=(bx, 3.12), color=c_body)
        b_tip = common.make_frustum(0.06, 0.02, 1.65, 1.74, segments=4, center=(bx, 3.12), color=c_tip)
        k.model.add(peg)
        k.model.add(cord)
        k.model.add(b_body)
        k.model.add(b_tip)

    # ---- Interior: Fish cleaning table, potbelly stove, crates, barrel ----
    # 1. Fish cleaning table on right wall
    k.solid((1.4, -1.8, F), (3.6, -0.4, 0.88), "wood_crate", base="table")
    k.box((1.35, -1.85, 0.88), (3.65, -0.35, 0.94), "bark_dark")
    # Cutting board and two fish on table
    k.box((1.8, -1.4, 0.94), (2.5, -0.8, 0.97), "bark_light")
    k.box((1.9, -1.3, 0.97), (2.4, -1.1, 1.02), "rock_light")
    k.box((2.0, -1.0, 0.97), (2.4, -0.85, 1.02), "canvas_khaki")

    # 2. Wood-burning potbelly stove under smoke stack
    k.solid((-2.3, -1.7, F), (-1.3, -0.7, 0.85), "gun_black", base="stove")
    stove_top = common.make_frustum(0.36, 0.28, 0.85, 1.10, segments=6, center=(-1.8, -1.2), color="gun_black")
    stove_pipe_lower = common.make_frustum(0.12, 0.12, 1.10, 2.80, segments=6, center=(-1.8, -1.2), color="gun_black")
    k.model.add(stove_top)
    k.model.add(stove_pipe_lower)

    # 3. Stack of fish crates and barrel (barrel sits at floor level z=F)
    k.crate(-3.2, 1.8, 0.55, "wood_crate", z=F)
    k.crate(-3.2, 1.0, 0.45, "wood_crate", z=F)
    k.model.add(common.make_frustum(0.3, 0.3, F, F + 0.9, segments=10, center=(3.0, 2.0), color="bark_dark"))
    for zb in (F + 0.15, F + 0.7):
        k.model.add(common.make_frustum(0.32, 0.32, zb, zb + 0.05, segments=10, center=(3.0, 2.0), color="rock_dark"))
    k.collider((2.7, 1.7, F), (3.3, 2.3, F + 0.9), "barrel")

    # 4. Storage shelf on back wall (proud of back wall inner face y = -2.85)
    k.solid((0.8, -2.83, F), (3.5, -2.3, 1.8), "bark_dark", base="shelf")
    for z in (0.6, 1.1, 1.6):
        k.box((0.82, -2.81, z), (3.48, -2.25, z + 0.05), "wood_crate")
    k.box((1.2, -2.6, 1.15), (1.6, -2.4, 1.45), "metal")
    k.box((2.2, -2.6, 1.15), (2.5, -2.4, 1.40), "brass")

    # Empties
    k.empty("lot_size", (0.0, 0.0, 0.0), scale=(8.0, 6.0, 1.0))
    k.empty("door_1", (0.0, 2.95, F))
    k.empty("loot_1", (2.5, -1.1, 0.95))   # table
    k.empty("loot_2", (1.8, -2.5, 1.20))   # shelf
    k.empty("loot_3", (-3.2, 1.8, F + 0.60)) # crate
    k.empty("spawn_1", (-1.2, 1.0, F))
    k.empty("spawn_2", (1.2, 1.0, F))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, top_preview=True, view=kit.FRONT_VIEW) else 1


if __name__ == "__main__":
    sys.exit(main())
