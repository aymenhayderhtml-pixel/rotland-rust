"""base_floor: 2x2 m player-built wooden floor tile, 0.1 m thick.

Snap: cell. Grid size: 2.0 m. Fits flush with adjacent floors and under walls.
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, os.path.join(HERE, "..", "buildings"))
sys.path.insert(0, HERE)

import common
import kit
import parts_common

NAME = "base_floor"
CATEGORY = parts_common.CATEGORY
BUDGET = parts_common.BUDGET_STRUCTURE


def build(model):
    k = kit.Kit(model)

    # 1. Underside structural joists (visible when looking up at a 2nd floor ceiling)
    joist_color = parts_common.COLOR_FRAME
    k.box((-0.90, -1.0, 0.0), (-0.78, 1.0, 0.055), joist_color)
    k.box((-0.06, -1.0, 0.0), (0.06, 1.0, 0.055), joist_color)
    k.box((0.78, -1.0, 0.0), (0.90, 1.0, 0.055), joist_color)

    # Rim headers at front and back
    k.box((-1.0, -1.0, 0.0), (1.0, -0.92, 0.055), joist_color)
    k.box((-1.0, 0.92, 0.0), (1.0, 1.0, 0.055), joist_color)

    # 2. Subfloor backing board
    k.box((-1.0, -1.0, 0.055), (1.0, 1.0, 0.068), parts_common.COLOR_FRAME)

    # 3. Wooden floor planks on top (Z: 0.068 to 0.10)
    plank_a = parts_common.COLOR_PLANK_A
    plank_b = parts_common.COLOR_PLANK_B

    planks = [
        (-0.995, -0.605, -1.0, 1.0, plank_a),
        (-0.595, -0.205, -1.0, 1.0, plank_b),
        (-0.195, 0.195, -1.0, 0.15, plank_a),
        (-0.195, 0.195, 0.18, 1.0, plank_b),
        (0.205, 0.595, -1.0, 1.0, plank_b),
        (0.605, 0.995, -1.0, 1.0, plank_a),
    ]
    for x0, x1, y0, y1, color in planks:
        k.box((x0, y0, 0.068), (x1, y1, 0.10), color)

    # Full footprint collider: 2.0 x 2.0 x 0.1 m
    k.collider((-1.0, -1.0, 0.0), (1.0, 1.0, 0.10), base="floor")


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
