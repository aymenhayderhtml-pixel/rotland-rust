"""camp_workshop_2: Tier 2 tool bench adding a grinder, drill press, metal shelf and interact_1 empty.

Upgraded workstation with heavy wooden bench, rear pegboard, bench grinder, drill press,
freestanding metal shelving rack with automotive/mechanical parts, and front interact marker.
"""
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, HERE)

import common  # noqa: E402
import kit  # noqa: E402

NAME, CATEGORY, BUDGET = "camp_workshop_2", "buildings", 2500


def build(model):
    k = kit.Kit(model)

    # ================= WORKBENCH (x: -0.8 to 1.4, y: -0.45 to 0.45) =================
    # 4 heavy wooden legs
    for lx in (-0.75, 1.25):
        for ly in (-0.38, 0.26):
            k.box((lx, ly, 0.0), (lx + 0.12, ly + 0.12, 0.80), "bark_dark")

    # Lower shelf stretchers and slats (z=0.18 m)
    k.box((-0.73, -0.36, 0.18), (1.33, -0.28, 0.24), "bark_dark")
    k.box((-0.73, 0.28, 0.18), (1.33, 0.36, 0.24), "bark_dark")
    k.box((-0.71, -0.28, 0.20), (1.31, 0.28, 0.24), "wood_crate")

    # Bench tabletop (solid collider)
    k.solid((-0.80, -0.45, 0.80), (1.40, 0.45, 0.88), "wood_crate", base="tabletop")

    # Pegboard uprights and backing (solid collider)
    k.solid((-0.80, -0.44, 0.88), (-0.70, -0.38, 1.85), "bark_dark", base="peg_post_l")
    k.solid((1.30, -0.44, 0.88), (1.40, -0.38, 1.85), "bark_dark", base="peg_post_r")
    k.solid((-0.70, -0.42, 0.92), (1.30, -0.40, 1.85), "bark_light", base="pegboard")
    k.box((-0.70, -0.43, 1.85), (1.30, -0.37, 1.89), "bark_dark")
    k.box((-0.70, -0.43, 0.88), (1.30, -0.37, 0.92), "bark_dark")

    # Hanging tools on pegboard:
    # Hammer
    k.box((-0.52, -0.39, 1.25), (-0.48, -0.37, 1.62), "bark_light")
    k.box((-0.56, -0.39, 1.62), (-0.44, -0.36, 1.68), "gun_black")
    # Saw
    k.box((-0.25, -0.39, 1.20), (-0.15, -0.38, 1.68), "metal")
    k.box((-0.27, -0.38, 1.60), (-0.13, -0.36, 1.72), "wood_crate")
    # Wrenches
    k.box((0.10, -0.39, 1.35), (0.14, -0.37, 1.60), "metal")
    k.box((0.08, -0.39, 1.60), (0.16, -0.37, 1.66), "metal")
    k.box((0.25, -0.39, 1.30), (0.29, -0.37, 1.55), "metal")
    # Screwdrivers
    k.box((0.45, -0.38, 1.48), (0.49, -0.36, 1.60), "red_paint")
    k.box((0.46, -0.39, 1.36), (0.48, -0.37, 1.48), "metal")
    k.box((0.55, -0.38, 1.46), (0.59, -0.36, 1.58), "shirt_blue")
    k.box((0.56, -0.39, 1.38), (0.58, -0.37, 1.46), "metal")

    # ================= 1. BENCH VICE (front-right corner x = 1.05, y = 0.35) =================
    k.solid((0.92, 0.26, 0.88), (1.18, 0.44, 0.94), "gun_black", base="vice_base")
    k.box((0.96, 0.30, 0.94), (1.14, 0.39, 1.08), "gun_black")
    k.box((0.94, 0.36, 1.04), (1.16, 0.39, 1.10), "metal")
    k.box((0.96, 0.43, 0.94), (1.14, 0.50, 1.08), "gun_black")
    k.box((0.94, 0.43, 1.04), (1.16, 0.46, 1.10), "metal")
    k.box((1.02, 0.28, 0.98), (1.06, 0.53, 1.02), "metal")
    k.box((0.94, 0.52, 0.92), (1.14, 0.54, 1.08), "metal")

    # ================= 2. BENCH DRILL PRESS (center x = 0.35, y = 0.05) =================
    k.solid((0.22, -0.08, 0.88), (0.48, 0.18, 0.93), "gun_black", base="drill_base")
    # Vertical column
    k.box((0.32, -0.06, 0.93), (0.38, 0.0, 1.62), "metal")
    # Work table on column
    k.box((0.25, 0.0, 1.12), (0.45, 0.16, 1.16), "gun_black")
    # Drill head housing at top
    k.box((0.22, -0.12, 1.48), (0.48, 0.18, 1.68), "red_paint")
    # Motor rear box
    k.box((0.26, -0.18, 1.50), (0.44, -0.12, 1.64), "gun_black")
    # Chuck and drill bit
    k.box((0.33, 0.06, 1.36), (0.37, 0.10, 1.48), "metal")
    k.box((0.34, 0.07, 1.22), (0.36, 0.09, 1.36), "metal")
    # 3-spoke handle
    k.box((0.48, 0.01, 1.55), (0.58, 0.03, 1.57), "metal")
    k.box((0.48, 0.07, 1.51), (0.56, 0.09, 1.63), "metal")

    # ================= 3. BENCH GRINDER (left side of bench x = -0.40, y = 0.10) =================
    k.solid((-0.58, 0.0, 0.88), (-0.22, 0.24, 0.92), "gun_black", base="grinder_base")
    # Center motor housing
    k.box((-0.48, 0.02, 0.92), (-0.32, 0.22, 1.08), "shirt_blue")
    # Left abrasive wheel and guard
    k.box((-0.58, -0.01, 0.90), (-0.48, 0.25, 1.12), "gun_black")
    k.box((-0.56, 0.01, 0.92), (-0.50, 0.23, 1.10), "rock_dark")     # coarse stone
    # Right wire/buffing wheel and guard
    k.box((-0.32, -0.01, 0.90), (-0.22, 0.25, 1.12), "gun_black")
    k.box((-0.30, 0.01, 0.92), (-0.24, 0.23, 1.10), "rock_light")    # buffing wheel
    # Eye shields
    k.box((-0.56, 0.14, 1.12), (-0.24, 0.20, 1.16), "metal")

    # ================= 4. METAL SHELVING UNIT (x: -1.75 to -1.00, y: -0.45 to 0.45) =================
    for sx in (-1.75, -1.05):
        for sy in (-0.45, 0.40):
            k.box((sx, sy, 0.0), (sx + 0.05, sy + 0.05, 1.85), "metal")

    # 4 metal shelf tiers
    for sz in (0.15, 0.65, 1.15, 1.65):
        k.solid((-1.74, -0.44, sz), (-1.01, 0.44, sz + 0.04), "metal", base="shelf_tier")

    # Shelf items:
    # Bottom shelf (z=0.19): Car battery and heavy gear
    k.box((-1.65, -0.30, 0.19), (-1.35, -0.05, 0.42), "gun_black")
    k.box((-1.62, -0.28, 0.42), (-1.56, -0.22, 0.46), "brass")
    k.box((-1.44, -0.28, 0.42), (-1.38, -0.22, 0.46), "metal")
    k.box((-1.60, 0.10, 0.19), (-1.25, 0.35, 0.32), "metal")

    # Second shelf (z=0.69): Red metal toolbox and spray cans
    k.box((-1.68, -0.35, 0.69), (-1.22, 0.05, 0.95), "red_paint")
    k.box((-1.66, -0.33, 0.95), (-1.24, 0.03, 0.98), "metal")
    k.box((-1.52, -0.36, 0.82), (-1.38, -0.34, 0.88), "metal")
    k.box((-1.62, 0.15, 0.69), (-1.52, 0.25, 0.92), "shirt_blue")
    k.box((-1.48, 0.18, 0.69), (-1.38, 0.28, 0.92), "white")

    # Third shelf (z=1.19): Parts bins
    k.box((-1.68, -0.35, 1.19), (-1.40, 0.0, 1.32), "shirt_blue")
    k.box((-1.65, -0.32, 1.25), (-1.43, -0.03, 1.30), "brass")
    k.box((-1.38, -0.35, 1.19), (-1.10, 0.0, 1.32), "shirt_blue")
    k.box((-1.35, -0.32, 1.25), (-1.13, -0.03, 1.30), "metal")

    # Top shelf (z=1.69): Wire spool
    k.box((-1.65, -0.25, 1.69), (-1.35, 0.05, 1.84), "bark_dark")
    k.box((-1.62, -0.22, 1.72), (-1.38, 0.02, 1.81), "brass")

    # Empties
    k.empty("interact_1", (0.30, 0.95, 0.0), rot_z=math.pi)
    k.empty("lot_size", (0.0, 0.0, 0.0), scale=(3.6, 2.2, 1.0))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, view=kit.FRONT_VIEW) else 1


if __name__ == "__main__":
    sys.exit(main())
