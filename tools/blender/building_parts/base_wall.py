"""base_wall: 2 m wide, 3 m tall, 0.15 m thick player-built wooden wall.

Snap: edge. Grid size: 2.0 m. Height: 3.0 m. Sits on cell boundary.
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

NAME = "base_wall"
CATEGORY = parts_common.CATEGORY
BUDGET = parts_common.BUDGET_STRUCTURE


def build(model):
    k = kit.Kit(model)
    frame_color = parts_common.COLOR_FRAME
    plank_a = parts_common.COLOR_PLANK_A
    plank_b = parts_common.COLOR_PLANK_B

    # 1. Structural framing (0.15 m deep along Y: -0.075 to +0.075)
    # Left and right corner posts
    k.box((-1.0, -0.075, 0.0), (-0.88, 0.075, 3.0), frame_color)
    k.box((0.88, -0.075, 0.0), (1.0, 0.075, 3.0), frame_color)

    # Bottom sill and top plate
    k.box((-0.88, -0.075, 0.0), (0.88, 0.075, 0.10), frame_color)
    k.box((-0.88, -0.075, 2.90), (0.88, 0.075, 3.0), frame_color)

    # Center vertical stud
    k.box((-0.05, -0.07, 0.10), (0.05, 0.07, 2.90), frame_color)

    # 2. Horizontal siding planks (0.11 m deep along Y: -0.055 to +0.055)
    # 7 stacked horizontal boards between Z 0.10 and 2.90 (height 0.40 m each)
    plank_height = 0.40
    colors = [plank_a, plank_b, plank_a, plank_b, plank_a, plank_b, plank_a]
    for i, color in enumerate(colors):
        z0 = 0.10 + i * plank_height
        z1 = z0 + plank_height
        # Planks running across X between corner posts
        k.box((-0.88, -0.055, z0), (0.88, 0.055, z1), color)

    # 3. Collider: full 2.0 x 0.15 x 3.0 m convex box
    k.collider((-1.0, -0.075, 0.0), (1.0, 0.075, 3.0), base="wall")


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
