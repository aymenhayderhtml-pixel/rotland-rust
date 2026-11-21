"""base_half_wall: 2 m wide, 1.2 m tall, 0.15 m thick wooden half wall / railing.

Snap: edge. Grid size: 2.0 m. Height: 1.2 m.
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

NAME = "base_half_wall"
CATEGORY = parts_common.CATEGORY
BUDGET = parts_common.BUDGET_STRUCTURE


def build(model):
    k = kit.Kit(model)
    frame_color = parts_common.COLOR_FRAME
    plank_a = parts_common.COLOR_PLANK_A
    plank_b = parts_common.COLOR_PLANK_B

    # 1. Framing
    # Corner posts (height 1.2 m)
    k.box((-1.0, -0.075, 0.0), (-0.88, 0.075, 1.20), frame_color)
    k.box((0.88, -0.075, 0.0), (1.0, 0.075, 1.20), frame_color)

    # Bottom sill plate
    k.box((-0.88, -0.075, 0.0), (0.88, 0.075, 0.08), frame_color)

    # Center vertical stud
    k.box((-0.05, -0.07, 0.08), (0.05, 0.07, 1.12), frame_color)

    # Top railing / cap (slightly wider for a comfortable handrail feel)
    k.box((-1.0, -0.085, 1.12), (1.0, 0.085, 1.20), frame_color)

    # 2. Horizontal siding planks (Z: 0.08 to 1.12, 3 planks ~0.347 m each)
    step = (1.12 - 0.08) / 3
    for i in range(3):
        color = plank_a if i % 2 == 0 else plank_b
        k.box((-0.88, -0.055, 0.08 + i * step), (0.88, 0.055, 0.08 + (i + 1) * step), color)

    # 3. Collider: full 2.0 x 0.15 x 1.2 m convex box
    k.collider((-1.0, -0.075, 0.0), (1.0, 0.075, 1.20), base="wall")


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
