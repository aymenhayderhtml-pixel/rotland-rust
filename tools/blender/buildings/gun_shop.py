"""gun_shop: neutral-palette walk-in town gun shop with crossed-guns sign, barred windows, counter & wall racks.

Body is 8.0 x 5.8 m (x: -4.0..4.0, y: -3.5..2.3), front porch is 1.2 m (y: 2.31..3.5).
Walk-in parity with house_small: floor F=0.1, door top DOOR_H=2.2, 0.15 m walls.
"""
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, HERE)

import common  # noqa: E402
import kit  # noqa: E402

NAME, CATEGORY, BUDGET = "gun_shop", "buildings", 3000
TOP = 2.6
F = kit.FLOOR_TOP
SIDING = ("rock_light", "white")


def _sign_rifle(model, center, rot_y):
    """Large stylized rifle silhouette for the front sign, rotated in the X-Z plane."""
    parts = [
        common.make_box((0.72, 0.03, 0.14), loc=(-0.72, 0.0, -0.03), color="bark_dark"),
        common.make_box((0.56, 0.035, 0.14), loc=(-0.10, 0.0, 0.02), color="metal"),
        common.make_box((0.12, 0.028, 0.24), loc=(-0.05, 0.0, -0.14), color="gun_black"),
        common.make_box((0.92, 0.024, 0.06), loc=(0.64, 0.0, 0.05), color="gun_black"),
        common.make_box((0.46, 0.028, 0.07), loc=(0.04, 0.0, 0.14), color="gun_black"),
        common.make_box((0.40, 0.028, 0.08), loc=(0.38, 0.0, -0.02), color="bark_dark"),
    ]
    bm = common.combine(parts)
    common.transform(bm, loc=center, rot=(0.0, rot_y, 0.0))
    model.add(bm, roof=True)


def _rack_rifle(k, x0, x1, y_mid, z_mid, stock_col, axis="x"):
    """Horizontal rifle silhouette on an interior wall rack."""
    if axis == "x":
        xm = (x0 + x1) / 2
        k.box((x0, y_mid - 0.02, z_mid - 0.045), (x0 + 0.34, y_mid + 0.02, z_mid + 0.025), stock_col)
        k.box((x0 + 0.34, y_mid - 0.025, z_mid - 0.015), (xm + 0.10, y_mid + 0.025, z_mid + 0.04), "metal")
        k.box((xm + 0.10, y_mid - 0.012, z_mid + 0.012), (x1, y_mid + 0.012, z_mid + 0.035), "gun_black")
        k.box((xm - 0.04, y_mid - 0.015, z_mid - 0.10), (xm + 0.03, y_mid + 0.015, z_mid - 0.015), "gun_black")
    else:
        ym = (x0 + x1) / 2
        k.box((y_mid - 0.02, x0, z_mid - 0.045), (y_mid + 0.02, x0 + 0.34, z_mid + 0.025), stock_col)
        k.box((y_mid - 0.025, x0 + 0.34, z_mid - 0.015), (y_mid + 0.025, ym + 0.10, z_mid + 0.04), "metal")
        k.box((y_mid - 0.012, ym + 0.10, z_mid + 0.012), (y_mid + 0.012, x1, z_mid + 0.035), "gun_black")
        k.box((y_mid - 0.015, ym - 0.04, z_mid - 0.10), (y_mid + 0.015, ym + 0.03, z_mid - 0.015), "gun_black")


def build(model):
    k = kit.Kit(model)

    # Floor & front porch deck (1 cm seam at y=2.30..2.31, same as house_small)
    k.solid((-4.0, -3.5, 0.0), (4.0, 2.30, F), "rock_mid", base="floor")
    k.solid((-3.98, 2.31, 0.0), (3.98, 3.50, F), "rock_dark", base="deck")

    # Door & barred window openings
    WIN_Z0 = 0.90
    WIN_Z1 = 2.00
    door = ((-2.0, 2.05, F), (-0.8, 2.45, kit.DOOR_H))
    win_front = ((0.6, 2.05, WIN_Z0), (2.8, 2.45, WIN_Z1))
    win_left = ((-4.15, -1.5, WIN_Z0), (-3.70, 0.3, WIN_Z1))

    k.wall((-4.0, 2.15, F), (4.0, 2.30, TOP), openings=[door, win_front], colors=SIDING, course=0.45, base="wall_front")
    k.wall((-4.0, -3.50, F), (4.0, -3.35, TOP), colors=SIDING, course=0.45, base="wall_back")
    k.wall((-4.0, -3.35, F), (-3.85, 2.15, TOP), openings=[win_left], colors=SIDING, course=0.45, base="wall_left")
    k.wall((3.85, -3.35, F), (4.0, 2.15, TOP), colors=SIDING, course=0.45, base="wall_right")

    # Door & window trim (2 cm proud of outer wall faces)
    k.frame(door, 2.32, 1, "metal", sill=False)
    k.frame(win_front, 2.32, 1, "metal")
    k.frame(win_left, -4.02, -1, "metal")

    # Security window bars (offset in Y/X so zero z-fighting)
    for bx in (0.95, 1.32, 1.70, 2.08, 2.45):
        k.box((bx - 0.015, 2.20, WIN_Z0), (bx + 0.015, 2.23, WIN_Z1), "gun_black")
    for bz in (1.25, 1.65):
        k.box((0.60, 2.24, bz - 0.015), (2.80, 2.27, bz + 0.015), "gun_black")

    for by in (-1.15, -0.78, -0.42, -0.05):
        k.box((-3.94, by - 0.015, WIN_Z0), (-3.91, by + 0.015, WIN_Z1), "gun_black")
    for bz in (1.25, 1.65):
        k.box((-3.98, -1.50, bz - 0.015), (-3.95, 0.30, bz + 0.015), "gun_black")

    # Steel corner trim strips
    for cx in (-4.02, 3.92):
        for cy in (-3.52, 2.22):
            k.box((cx, cy, F), (cx + 0.10, cy + 0.10, TOP), "metal")

    # Gable roof (symmetric Y around y=-0.60; back eave at -3.65 balances front sign at +3.65)
    ridge = (0.0, -0.60, 3.85)
    k.roof(ridge, (0.0, -3.65, TOP), span=(1.0, 0.0, 0.0), width=8.12, color="rock_dark")
    k.roof(ridge, (0.0, 2.45, TOP), span=(1.0, 0.0, 0.0), width=8.12, color="rock_dark")
    for x, off_x in ((-3.98, 0.12), (3.86, 0.12)):
        k.gable([(x, -3.50, TOP - 0.12), (x, 2.30, TOP - 0.12), (x, -0.60, 3.72)],
                (off_x, 0.0, 0.0), "rock_light", roof=True)
    k.ridge_cap((0.0, -0.60, 3.85), 0, 8.20, "metal")

    # Porch roof and 3 posts (clear of doorway x=-2.0..-0.8)
    k.solid((-4.00, 2.32, 2.44), (4.00, 3.55, 2.56), "rock_dark", base="porch_roof")
    for px in (-3.88, -0.25, 3.73):
        k.solid((px, 3.30, F), (px + 0.15, 3.45, 2.44), "metal", base="post")

    # Bold crossed-guns sign on front porch roof (max Y = 3.65 matches back eave -3.65)
    k.box((-2.50, 3.42, 2.58), (2.50, 3.50, 3.78), "gun_black", roof=True)
    k.box((-2.35, 3.52, 2.70), (2.35, 3.56, 3.66), "wood_crate", roof=True)
    _sign_rifle(model, (0.0, 3.59, 3.18), 0.42)
    _sign_rifle(model, (0.0, 3.63, 3.18), -0.42)

    # Interior counter (front-facing +Y; shopkeeper stands behind at (-0.90, -2.45, F))
    k.solid((-3.0, -1.80, F), (1.2, -1.10, 1.05), "bark_dark", base="counter")
    k.box((-3.02, -1.82, 1.05), (1.22, -1.08, 1.11), "wood_crate")
    # Cash register & ammo boxes on counter
    k.box((-2.4, -1.55, 1.11), (-2.0, -1.25, 1.28), "metal")
    k.box((0.4, -1.55, 1.11), (0.85, -1.25, 1.24), "red_paint")
    k.box((0.39, -1.56, 1.22), (0.86, -1.24, 1.25), "brass")

    # Back wall gun rack (behind counter, 2 cm off back wall y=-3.35)
    k.solid((-2.8, -3.33, F), (1.2, -2.93, 0.85), "bark_dark", base="back_rack")
    k.box((-2.75, -3.32, 0.85), (1.15, -3.26, 2.20), "bark_light")
    for rx in (-2.4, -0.8, 0.8):
        k.box((rx - 0.03, -3.26, 0.90), (rx + 0.03, -3.16, 2.15), "bark_dark")
    _rack_rifle(k, -2.2, -0.9, -3.14, 1.22, "wood_crate", axis="x")
    _rack_rifle(k, -2.2, -0.9, -3.14, 1.62, "bark_light", axis="x")
    _rack_rifle(k, -0.5, 0.8, -3.14, 1.42, "gun_black", axis="x")
    _rack_rifle(k, -0.5, 0.8, -3.14, 1.88, "wood_crate", axis="x")

    # Right wall gun rack (2 cm off right wall x=3.85)
    k.solid((3.40, -2.5, F), (3.83, 1.1, 0.82), "bark_dark", base="right_rack")
    k.box((3.77, -2.4, 0.82), (3.82, 1.0, 2.12), "bark_light")
    for ry in (-2.0, -0.7, 0.6):
        k.box((3.66, ry - 0.03, 0.88), (3.77, ry + 0.03, 2.08), "bark_dark")
    _rack_rifle(k, -2.1, -0.8, 3.65, 1.20, "wood_crate", axis="y")
    _rack_rifle(k, -2.1, -0.8, 3.65, 1.65, "gun_black", axis="y")
    _rack_rifle(k, -0.4, 0.9, 3.65, 1.40, "bark_light", axis="y")

    # Left back corner ammo shelf (clear of left window y=-1.5..0.3)
    k.solid((-3.83, -3.2, F), (-3.35, -1.9, 1.65), "bark_dark", base="ammo_shelf")
    for sz in (0.55, 1.05):
        k.box((-3.80, -3.15, sz), (-3.38, -1.95, sz + 0.04), "wood_crate")
    k.box((-3.74, -2.95, 0.59), (-3.44, -2.45, 0.82), "pine_dark")
    k.box((-3.74, -2.40, 1.09), (-3.44, -2.05, 1.28), "red_paint")

    # Crates
    k.crate(2.8, -2.8, 0.60, z=F)
    k.crate(3.1, 2.85, 0.55, z=F)

    # Empties
    k.empty("lot_size", (0.0, 0.0, 0.0), scale=(8.0, 7.0, 1.0))
    k.empty("door_1", (-1.4, 2.225, F))
    k.empty("npc_1", (-0.90, -2.45, F), rot_z=0.0)
    k.empty("loot_1", (-0.9, -1.45, 1.11))
    k.empty("loot_2", (2.8, -2.8, F + 0.60))
    k.empty("loot_3", (3.1, 2.85, F + 0.55))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, top_preview=True, view=kit.FRONT_VIEW) else 1


if __name__ == "__main__":
    sys.exit(main())
