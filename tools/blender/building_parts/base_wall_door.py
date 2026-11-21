"""base_wall_door: 2 m wide, 3 m tall, 0.15 m thick wooden wall with door.

Doorway opening is walkable (no static collider in opening).
Includes a separate child mesh `door_leaf` hinged at one side, so the game can swing it.
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

NAME = "base_wall_door"
CATEGORY = parts_common.CATEGORY
BUDGET = parts_common.BUDGET_DOOR


def build(model):
    k = kit.Kit(model)
    frame_color = parts_common.COLOR_FRAME
    plank_a = parts_common.COLOR_PLANK_A
    plank_b = parts_common.COLOR_PLANK_B

    # 1. Outer framing
    k.box((-1.0, -0.075, 0.0), (-0.88, 0.075, 3.0), frame_color)
    k.box((0.88, -0.075, 0.0), (1.0, 0.075, 3.0), frame_color)
    k.box((-0.88, -0.075, 2.90), (0.88, 0.075, 3.0), frame_color)

    # Door opening: X in [-0.46, 0.46], Z in [0.0, 2.15]
    # Door jambs and lintel
    k.box((-0.54, -0.08, 0.0), (-0.46, 0.08, 2.23), frame_color)
    k.box((0.46, -0.08, 0.0), (0.54, 0.08, 2.23), frame_color)
    k.box((-0.54, -0.085, 2.15), (0.54, 0.085, 2.23), frame_color)

    # 2. Wall planks left and right of doorway
    # Left of doorway (X -0.88 to -0.54, Z 0.0 to 2.90) - 5 horizontal courses
    step_side = 2.90 / 5
    for i in range(5):
        color = plank_a if i % 2 == 0 else plank_b
        k.box((-0.88, -0.055, i * step_side), (-0.54, 0.055, (i + 1) * step_side), color)

    # Right of doorway (X 0.54 to 0.88, Z 0.0 to 2.90) - 5 horizontal courses
    for i in range(5):
        color = plank_b if i % 2 == 0 else plank_a
        k.box((0.54, -0.055, i * step_side), (0.88, 0.055, (i + 1) * step_side), color)

    # Above doorway (X -0.54 to 0.54, Z 2.23 to 2.90)
    step_top = (2.90 - 2.23) / 2
    for i in range(2):
        color = plank_a if i == 0 else plank_b
        k.box((-0.54, -0.055, 2.23 + i * step_top), (0.54, 0.055, 2.23 + (i + 1) * step_top), color)

    # 3. Frame colliders (leaving doorway opening from X -0.46..0.46, Z 0..2.15 unblocked)
    k.collider((-1.0, -0.075, 0.0), (-0.46, 0.075, 3.0), base="wall_left")
    k.collider((0.46, -0.075, 0.0), (1.0, 0.075, 3.0), base="wall_right")
    k.collider((-0.46, -0.075, 2.15), (0.46, 0.075, 3.0), base="wall_top")

    # 4. Hinge marker empty at left jamb
    model.add_empty("door_hinge", (-0.46, 0.0, 0.0))

    # 5. Separate child mesh: `door_leaf`
    # Positioned inside the opening: X in [-0.44, 0.44], Y in [-0.02, 0.02], Z in [0.03, 2.13]
    door_bms = []

    # Main door panel vertical boards
    door_bms.append(parts_common.box_bm((-0.44, -0.015, 0.03), (-0.15, 0.015, 2.13), plank_a))
    door_bms.append(parts_common.box_bm((-0.15, -0.015, 0.03), (0.15, 0.015, 2.13), plank_b))
    door_bms.append(parts_common.box_bm((0.15, -0.015, 0.03), (0.44, 0.015, 2.13), plank_a))

    # Door framing stiles and rails
    door_bms.append(parts_common.box_bm((-0.44, -0.02, 0.03), (-0.36, 0.02, 2.13), frame_color))  # hinge stile
    door_bms.append(parts_common.box_bm((0.36, -0.02, 0.03), (0.44, 0.02, 2.13), frame_color))    # lock stile
    door_bms.append(parts_common.box_bm((-0.36, -0.02, 0.03), (0.36, 0.02, 0.12), frame_color))   # bottom rail
    door_bms.append(parts_common.box_bm((-0.36, -0.02, 1.04), (0.36, 0.02, 1.14), frame_color))   # mid rail
    door_bms.append(parts_common.box_bm((-0.36, -0.02, 2.04), (0.36, 0.02, 2.13), frame_color))   # top rail

    # Metal strap hinges on left side
    door_bms.append(parts_common.box_bm((-0.46, -0.025, 0.35), (-0.28, 0.025, 0.42), parts_common.COLOR_METAL))
    door_bms.append(parts_common.box_bm((-0.46, -0.025, 1.70), (-0.28, 0.025, 1.77), parts_common.COLOR_METAL))

    # Metal door handle and latch on right side
    door_bms.append(parts_common.box_bm((0.32, -0.035, 1.05), (0.42, 0.035, 1.13), parts_common.COLOR_METAL))
    door_bms.append(parts_common.box_bm((0.28, 0.035, 1.07), (0.38, 0.075, 1.11), parts_common.COLOR_METAL))

    door_leaf = common.combine(door_bms)
    model.add_part("door_leaf", door_leaf)


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
