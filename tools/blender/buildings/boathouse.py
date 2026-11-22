"""boathouse: enterable 14 x 10 m boathouse with central boat slip and big door to water (<= 4000 tris).

Half over the water edge (+Y side). Big boat door opens to the water (+Y),
player entrance on the land side (-Y). Origin at center on ground/water level (z=0).
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, HERE)

import common  # noqa: E402
import kit  # noqa: E402

NAME, CATEGORY, BUDGET = "boathouse", "buildings", 4000
TOP = 3.4
RIDGE_Z = 5.2
F = kit.FLOOR_TOP
WALL_COLORS = ("bark_dark", "bark_light", "wood_crate")
GRIP = (0.0, 0.0, 0.0)


def build(model):
    k = kit.Kit(model)

    # Footprint: 14 m wide (X: -7.0 to 7.0), 10 m deep (Y: -5.0 to 5.0)
    # Land half (-Y, Y: -5.0 to 0.0): solid wooden floor
    k.solid((-7.0, -5.0, 0.0), (7.0, 0.0, F), "wood_crate", base="floor_land")

    # Water half (+Y, Y: 0.0 to 5.0): walkways on left and right flanking open boat slip
    # Left walkway (X: -7.0 to -2.2)
    k.solid((-7.0, 0.01, 0.0), (-2.2, 5.0, F), "wood_crate", base="walkway_left")
    # Right walkway (X: 2.2 to 7.0)
    k.solid((2.2, 0.01, 0.0), (7.0, 5.0, F), "wood_crate", base="walkway_right")

    # Stilts / pilings under the water half (+Y) extending down to -2.5 m
    piling_spots = [
        (-6.8, 1.8), (-6.8, 3.5), (-6.8, 4.9),
        (6.8, 1.8), (6.8, 3.5), (6.8, 4.9),
        (-2.25, 1.8), (-2.25, 3.5), (-2.25, 4.9),
        (2.25, 1.8), (2.25, 3.5), (2.25, 4.9),
    ]
    for px, py in piling_spots:
        post = common.make_frustum(0.14, 0.14, -2.5, 0.08, segments=6, center=(px, py), color="bark_dark")
        k.model.add(post)

    # Water side bumper rub-rails along boat slip inner edges
    k.box((-2.24, 0.02, F - 0.05), (-2.14, 4.98, F + 0.10), "bark_dark")
    k.box((2.14, 0.02, F - 0.05), (2.24, 4.98, F + 0.10), "bark_dark")

    # Wall Openings:
    # 1. Big boat door on +Y (water side): 4.4 m wide, 2.9 m tall
    boat_door = ((-2.2, 4.85, F), (2.2, 5.15, 2.95))
    # 2. Player entry door on -Y (land side): 1.2 m wide, 2.2 m tall
    # Top aligned to wall course (F + 6 * 0.35) to prevent floating-point coplanar slicing
    DOOR_H = F + 6 * 0.35
    player_door = ((-0.6, -5.15, F), (0.6, -4.85, DOOR_H))
    # Windows on land side
    win_land_l = ((-5.0, -5.15, 1.2), (-3.2, -4.85, DOOR_H))
    win_land_r = ((3.2, -5.15, 1.2), (5.0, -4.85, DOOR_H))
    # Side windows
    win_side_l = ((-7.15, -1.0, 1.2), (-6.85, 0.8, DOOR_H))
    win_side_r = ((6.85, -1.0, 1.2), (7.15, 0.8, DOOR_H))

    # Exterior walls
    # Front (+Y, water side)
    k.wall((-7.0, 4.85, F), (7.0, 5.0, TOP), openings=[boat_door],
           colors=WALL_COLORS, course=0.35, base="wall_water")
    # Back (-Y, land side)
    k.wall((-7.0, -5.0, F), (7.0, -4.85, TOP), openings=[player_door, win_land_l, win_land_r],
           colors=WALL_COLORS, course=0.35, base="wall_land")
    # Left (-X) & Right (+X)
    k.wall((-7.0, -4.85, F), (-6.85, 4.85, TOP), openings=[win_side_l],
           colors=WALL_COLORS, course=0.35, base="wall_left")
    k.wall((6.85, -4.85, F), (7.0, 4.85, TOP), openings=[win_side_r],
           colors=WALL_COLORS, course=0.35, base="wall_right")

    # Heavy timber frame for the big boat door (proud of front wall at y = 5.0)
    # Starts at 0.08 (inside floor slab) to avoid coplanar bottom face with wall at F=0.1
    k.box((-2.35, 4.82, 0.08), (-2.15, 5.05, 3.15), "bark_dark")
    k.box((2.15, 4.82, 0.08), (2.35, 5.05, 3.15), "bark_dark")
    k.box((-2.15, 4.82, 2.93), (2.15, 5.05, 3.15), "bark_dark")

    # Gable roof: ridge along Y (X = 0, Z = RIDGE_Z)
    ridge = (0.0, 0.0, RIDGE_Z)
    eave_l = (-7.3, 0.0, 3.2)
    eave_r = (7.3, 0.0, 3.2)
    k.roof(ridge, eave_l, span=(0.0, 1.0, 0.0), width=10.4, color="rock_dark")
    k.roof(ridge, eave_r, span=(0.0, 1.0, 0.0), width=10.4, color="rock_dark")

    # Gable triangle walls at +Y and -Y (gables under roof slab, inset inside outer walls)
    k.gable([(-6.95, 4.86, 3.06), (6.95, 4.86, 3.06), (0.0, 4.86, 5.06)], (0.0, 0.12, 0.0), "bark_dark", roof=True)
    k.gable([(-6.95, -4.98, 3.06), (6.95, -4.98, 3.06), (0.0, -4.98, 5.06)], (0.0, 0.12, 0.0), "bark_dark", roof=True)

    # Ridge cap along Y
    k.ridge_cap((0.0, 0.0, RIDGE_Z), 1, 10.5, "bark_dark")

    # Door and window trims
    def frame(opening, face, sign, color="bark_dark", t=0.08, depth=0.05, sill=True):
        lo, hi = opening
        axis = 0 if (hi[0] - lo[0]) < (hi[1] - lo[1]) else 1
        u = 1 - axis
        a, b = sorted((face, face + sign * depth))

        def strip(u0, u1, z0, z1, extra=0.0):
            p0, p1 = [0.0, 0.0, z0], [0.0, 0.0, z1]
            p0[u], p1[u] = u0, u1
            p0[axis], p1[axis] = (a - extra, b) if sign < 0 else (a, b + extra)
            k.box(tuple(p0), tuple(p1), color)

        window = lo[2] > F + 0.05
        strip(lo[u] - t, lo[u], lo[2], hi[2])
        strip(hi[u], hi[u] + t, lo[2], hi[2])
        strip(lo[u] - t, hi[u] + t, hi[2], hi[2] + t)
        if window and sill:
            strip(lo[u] - t * 1.5, hi[u] + t * 1.5, lo[2] - t, lo[2], extra=0.04)

    frame(player_door, -5.02, -1, "bark_dark")
    frame(win_land_l, -5.02, -1, "bark_dark")
    k.boards(win_land_l, -5.03, -1)
    frame(win_land_r, -5.02, -1, "bark_dark")
    frame(win_side_l, -7.02, -1, "bark_dark")
    frame(win_side_r, 7.02, 1, "bark_dark")

    # Corner timber posts
    for px, py in ((-7.0, -5.0), (7.0, -5.0), (-7.0, 5.0), (7.0, 5.0)):
        k.box((px - 0.13, py - 0.13, 0.02), (px + 0.13, py + 0.13, TOP + 0.02), "bark_dark")

    # Ceiling cross-tie beams / roof trusses (inside walls, x=-6.84..6.84)
    for ty in (-3.5, 0.0, 3.5):
        k.box((-6.84, ty - 0.08, TOP - 0.10), (6.84, ty + 0.08, TOP + 0.05), "bark_dark", roof=True)

    # Boat hoist winch hanging from center rafter over the boat slip
    k.box((-0.25, 2.5 - 0.25, TOP - 0.60), (0.25, 2.5 + 0.25, TOP), "metal", roof=True)
    k.box((-0.03, 2.5 - 0.03, TOP - 1.20), (0.03, 2.5 + 0.03, TOP - 0.60), "metal", roof=True)

    # Interior clutter & props:
    # 1. Long workbench along land side wall
    k.solid((2.2, -4.8, F), (5.5, -4.0, 0.88), "wood_crate", base="workbench")
    k.box((2.15, -4.83, 0.88), (5.55, -3.95, 0.94), "bark_dark")
    # Tackle box and lantern on workbench (sitting proud at z=0.95)
    k.box((2.8, -4.5, 0.95), (3.3, -4.2, 1.14), "metal")
    k.box((4.2, -4.5, 0.95), (4.5, -4.2, 1.25), "brass")

    # 2. Shelves on left wall
    k.collider((-6.85, -4.0, F), (-6.0, -1.5, 2.1), "shelf_left")
    k.box((-6.83, -4.0, F), (-6.78, -3.95, 2.1), "bark_dark")
    k.box((-6.05, -4.0, F), (-6.0, -3.95, 2.1), "bark_dark")
    k.box((-6.83, -1.55, F), (-6.78, -1.5, 2.1), "bark_dark")
    k.box((-6.05, -1.55, F), (-6.0, -1.5, 2.1), "bark_dark")
    for sz in (0.6, 1.1, 1.6):
        k.box((-6.80, -3.98, sz), (-6.02, -1.52, sz + 0.05), "wood_crate")
    # Some supplies on shelves (sitting proud at z=1.16)
    k.box((-6.7, -3.5, 1.16), (-6.2, -3.0, 1.45), "red_paint")
    k.box((-6.7, -2.6, 1.16), (-6.1, -1.8, 1.50), "metal")

    # 3. Crates and barrels on right walkway
    k.crate(5.5, 2.0, 0.6, "wood_crate", z=F)
    k.crate(5.5, 2.7, 0.5, "wood_crate", z=F)
    k.model.add(common.make_frustum(0.3, 0.3, F, F + 0.9, segments=10, center=(5.6, 3.8), color="red_paint"))
    for bz in (F + 0.15, F + 0.7):
        k.model.add(common.make_frustum(0.32, 0.32, bz, bz + 0.05, segments=10, center=(5.6, 3.8), color="rock_dark"))
    k.collider((5.3, 3.5, F), (5.9, 4.1, F + 0.9), "barrel")

    # 4. Life preserver ring on interior wall
    ring = common.make_frustum(0.25, 0.25, -0.04, 0.04, segments=6, color="red_paint")
    common.transform(ring, rot=(1.57, 0.0, 0.0), loc=(-1.5, -4.82, 1.8))
    k.model.add(ring)

    # 5. Oars leaning against wall
    oar1 = common.make_box((0.05, 0.05, 2.10), loc=(-5.8, -4.75, 1.05), color="bark_light")
    common.transform(oar1, rot=(0.10, 0.08, 0.0))
    k.model.add(oar1)

    # Empties
    k.empty("lot_size", (0.0, 0.0, 0.0), scale=(14.0, 10.0, 1.0))
    k.empty("door_1", (0.0, -4.95, F))
    k.empty("door_2", (0.0, 4.95, F))
    k.empty("loot_1", (3.8, -4.4, 0.95))   # workbench
    k.empty("loot_2", (-6.3, -2.5, 1.20))  # shelf
    k.empty("loot_3", (5.5, 2.0, F + 0.65)) # crate top
    k.empty("loot_4", (-4.5, 2.5, F))      # left walkway
    k.empty("spawn_1", (-3.5, -2.5, F))
    k.empty("spawn_2", (4.5, 1.0, F))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=GRIP, top_preview=True, view=kit.FRONT_VIEW) else 1


if __name__ == "__main__":
    sys.exit(main())
