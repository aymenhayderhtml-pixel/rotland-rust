"""base_wall_window: 2 m wide, 3 m tall, 0.15 m thick wooden wall with centered window.

Snap: edge. Grid size: 2.0 m. Height: 3.0 m. Window opening is unblocked for visibility and shooting.
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

NAME = "base_wall_window"
CATEGORY = parts_common.CATEGORY
BUDGET = parts_common.BUDGET_STRUCTURE


def build(model):
    k = kit.Kit(model)
    frame_color = parts_common.COLOR_FRAME
    plank_a = parts_common.COLOR_PLANK_A
    plank_b = parts_common.COLOR_PLANK_B

    # 1. Outer framing
    k.box((-1.0, -0.075, 0.0), (-0.88, 0.075, 3.0), frame_color)
    k.box((0.88, -0.075, 0.0), (1.0, 0.075, 3.0), frame_color)
    k.box((-0.88, -0.075, 0.0), (0.88, 0.075, 0.10), frame_color)
    k.box((-0.88, -0.075, 2.90), (0.88, 0.075, 3.0), frame_color)

    # 2. Window frame and trim (window opening: X -0.45..0.45, Z 1.05..2.05)
    # Sill shelf
    k.box((-0.50, -0.09, 0.98), (0.50, 0.09, 1.05), frame_color)
    # Top header
    k.box((-0.50, -0.085, 2.05), (0.50, 0.085, 2.12), frame_color)
    # Side jambs
    k.box((-0.50, -0.08, 1.05), (-0.45, 0.08, 2.05), frame_color)
    k.box((0.45, -0.08, 1.05), (0.50, 0.08, 2.05), frame_color)

    # Wooden cross mullion dividers inside the opening
    k.box((-0.02, -0.02, 1.05), (0.02, 0.02, 2.05), frame_color)
    k.box((-0.45, -0.02, 1.53), (0.45, 0.02, 1.57), frame_color)

    # 3. Horizontal planks filling around window
    # Below window (Z 0.10 to 0.98): 3 planks
    step_b = (0.98 - 0.10) / 3
    for i in range(3):
        color = plank_a if i % 2 == 0 else plank_b
        k.box((-0.88, -0.055, 0.10 + i * step_b), (0.88, 0.055, 0.10 + (i + 1) * step_b), color)

    # Above window (Z 2.12 to 2.90): 2 planks
    step_t = (2.90 - 2.12) / 2
    for i in range(2):
        color = plank_b if i % 2 == 0 else plank_a
        k.box((-0.88, -0.055, 2.12 + i * step_t), (0.88, 0.055, 2.12 + (i + 1) * step_t), color)

    # Left of window (X -0.88 to -0.50, Z 0.98 to 2.12): 3 planks
    step_s = (2.12 - 0.98) / 3
    for i in range(3):
        color = plank_a if i % 2 == 0 else plank_b
        k.box((-0.88, -0.055, 0.98 + i * step_s), (-0.50, 0.055, 0.98 + (i + 1) * step_s), color)

    # Right of window (X 0.50 to 0.88, Z 0.98 to 2.12): 3 planks
    for i in range(3):
        color = plank_b if i % 2 == 0 else plank_a
        k.box((0.50, -0.055, 0.98 + i * step_s), (0.88, 0.055, 0.98 + (i + 1) * step_s), color)

    # 4. Colliders on solid sections around window opening (leaving doorway/window open)
    k.collider((-1.0, -0.075, 0.0), (-0.45, 0.075, 3.0), base="wall_left")
    k.collider((0.45, -0.075, 0.0), (1.0, 0.075, 3.0), base="wall_right")
    k.collider((-0.45, -0.075, 0.0), (0.45, 0.075, 1.05), base="wall_bottom")
    k.collider((-0.45, -0.075, 2.05), (0.45, 0.075, 3.0), base="wall_top")


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
