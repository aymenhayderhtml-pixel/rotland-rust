"""camp_workshop_1: Tier 1 workbench with vice, tools on a pegboard and interact_1 empty.

Sturdy wooden workbench with lower storage shelf, rear tool pegboard, bench vice and hand tools.
"""
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, HERE)

import common  # noqa: E402
import kit  # noqa: E402

NAME, CATEGORY, BUDGET = "camp_workshop_1", "buildings", 1500


def build(model):
    k = kit.Kit(model)

    # 4 heavy wooden legs
    for lx in (-1.0, 0.88):
        for ly in (-0.38, 0.26):
            k.box((lx, ly, 0.0), (lx + 0.12, ly + 0.12, 0.80), "bark_dark")

    # Lower shelf stretchers and slats (z=0.18 m)
    k.box((-0.98, -0.36, 0.18), (0.98, -0.28, 0.24), "bark_dark")
    k.box((-0.98, 0.28, 0.18), (0.98, 0.36, 0.24), "bark_dark")
    k.box((-0.96, -0.28, 0.20), (0.96, 0.28, 0.24), "wood_crate")

    # Items on lower shelf
    # Wooden parts crate
    k.box((-0.7, -0.2, 0.24), (-0.2, 0.2, 0.52), "wood_crate")
    k.box((-0.68, -0.18, 0.52), (-0.22, 0.18, 0.54), "bark_light")
    # Spare metal pipes
    k.box((0.1, -0.15, 0.24), (0.7, -0.05, 0.30), "metal")
    k.box((0.15, 0.02, 0.24), (0.75, 0.12, 0.30), "metal")

    # Bench tabletop (solid collider)
    k.solid((-1.1, -0.45, 0.80), (1.1, 0.45, 0.88), "wood_crate", base="tabletop")

    # Pegboard uprights and backing (solid collider)
    k.solid((-1.1, -0.44, 0.88), (-1.0, -0.38, 1.85), "bark_dark", base="peg_post_l")
    k.solid((1.0, -0.44, 0.88), (1.1, -0.38, 1.85), "bark_dark", base="peg_post_r")
    k.solid((-1.0, -0.42, 0.92), (1.0, -0.40, 1.85), "bark_light", base="pegboard")
    # Top and bottom framing of pegboard
    k.box((-1.0, -0.43, 1.85), (1.0, -0.37, 1.89), "bark_dark")
    k.box((-1.0, -0.43, 0.88), (1.0, -0.37, 0.92), "bark_dark")

    # Tools hanging on pegboard:
    # 1. Claw hammer (at x = -0.7)
    k.box((-0.72, -0.39, 1.25), (-0.68, -0.37, 1.62), "bark_light")  # handle
    k.box((-0.76, -0.39, 1.62), (-0.64, -0.36, 1.68), "gun_black")   # head
    # 2. Handsaw (at x = -0.35)
    k.box((-0.42, -0.39, 1.20), (-0.32, -0.38, 1.68), "metal")       # blade
    k.box((-0.44, -0.38, 1.60), (-0.30, -0.36, 1.72), "wood_crate")  # handle
    # 3. Two open-ended wrenches (at x = 0.05, 0.25)
    k.box((0.02, -0.39, 1.35), (0.06, -0.37, 1.60), "metal")
    k.box((0.0, -0.39, 1.60), (0.08, -0.37, 1.66), "metal")
    k.box((0.22, -0.39, 1.30), (0.26, -0.37, 1.53), "metal")
    k.box((0.20, -0.39, 1.53), (0.28, -0.37, 1.58), "metal")
    # 4. Hacksaw (at x = 0.6)
    k.box((0.48, -0.39, 1.32), (0.72, -0.37, 1.36), "metal")         # blade
    k.box((0.48, -0.39, 1.36), (0.52, -0.37, 1.54), "gun_black")     # left arch
    k.box((0.48, -0.39, 1.54), (0.72, -0.37, 1.58), "gun_black")     # top arch
    k.box((0.68, -0.39, 1.36), (0.72, -0.37, 1.54), "gun_black")     # right arch
    k.box((0.70, -0.38, 1.28), (0.75, -0.36, 1.40), "wood_crate")    # handle
    # 5. Screwdrivers (at x = -0.15, -0.05)
    k.box((-0.17, -0.38, 1.48), (-0.13, -0.36, 1.60), "red_paint")
    k.box((-0.16, -0.39, 1.36), (-0.14, -0.37, 1.48), "metal")
    k.box((-0.07, -0.38, 1.46), (-0.03, -0.36, 1.58), "shirt_blue")
    k.box((-0.06, -0.39, 1.38), (-0.04, -0.37, 1.46), "metal")

    # Bench vice on the front-left corner (x = -0.75, y = 0.32 to 0.48)
    k.solid((-0.88, 0.28, 0.88), (-0.62, 0.46, 0.94), "gun_black", base="vice_base")
    # Fixed jaw body
    k.box((-0.84, 0.32, 0.94), (-0.66, 0.41, 1.08), "gun_black")
    k.box((-0.86, 0.38, 1.04), (-0.64, 0.41, 1.10), "metal")         # jaw insert
    # Moving jaw body
    k.box((-0.84, 0.45, 0.94), (-0.66, 0.52, 1.08), "gun_black")
    k.box((-0.86, 0.45, 1.04), (-0.64, 0.48, 1.10), "metal")         # jaw insert
    # Screw spindle and T-handle
    k.box((-0.77, 0.30, 0.98), (-0.73, 0.55, 1.02), "metal")
    k.box((-0.86, 0.54, 0.92), (-0.64, 0.56, 1.08), "metal")         # handle crossbar

    # Items on table surface:
    # Oil can with spout
    k.box((0.45, 0.15, 0.88), (0.58, 0.28, 1.02), "red_paint")
    k.box((0.49, 0.19, 1.02), (0.54, 0.24, 1.06), "brass")
    k.box((0.48, 0.20, 1.06), (0.50, 0.28, 1.16), "metal")           # angled spout
    # Small parts organizer tray
    k.box((-0.3, 0.1, 0.88), (0.2, 0.35, 0.94), "wood_crate")
    k.box((-0.28, 0.12, 0.94), (-0.05, 0.33, 0.96), "metal")         # bolts
    k.box((0.0, 0.12, 0.94), (0.18, 0.33, 0.96), "brass")            # brass fittings

    # Empties
    k.empty("interact_1", (0.0, 0.85, 0.0), rot_z=math.pi)
    k.empty("lot_size", (0.0, 0.0, 0.0), scale=(2.6, 2.0, 1.0))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, view=kit.FRONT_VIEW) else 1


if __name__ == "__main__":
    sys.exit(main())
