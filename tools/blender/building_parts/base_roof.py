"""base_roof: 2x2 m player-built flat roof with slight slope and overhang.

Snap: cell. Grid size: 2.0 m. Fits on top of 3 m walls (world height: floor*3 + 3).
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

NAME = "base_roof"
CATEGORY = parts_common.CATEGORY
BUDGET = parts_common.BUDGET_STRUCTURE


def build(model):
    k = kit.Kit(model)
    frame_color = parts_common.COLOR_FRAME

    # 1. Underside ceiling rafters (visible from inside room)
    k.box((-0.85, -1.0, 0.0), (-0.73, 1.0, 0.045), frame_color)
    k.box((-0.06, -1.0, 0.0), (0.06, 1.0, 0.045), frame_color)
    k.box((0.73, -1.0, 0.0), (0.85, 1.0, 0.045), frame_color)

    # Rim headers at front and back
    k.box((-1.0, -1.0, 0.0), (1.0, -0.92, 0.045), frame_color)
    k.box((-1.0, 0.92, 0.0), (1.0, 1.0, 0.045), frame_color)

    # 2. Subdeck backing slab
    k.box((-1.0, -1.0, 0.045), (1.0, 1.0, 0.06), frame_color)

    # 3. Sloped roof surface:
    # High edge at back (Y = 1.0, Z = 0.12), low edge at front (Y = -1.0, Z = 0.06)
    # Front eave has a 0.06 m overhang (extends to Y = -1.06) with a drip edge fascia
    roof_color_a = parts_common.COLOR_ROOF
    roof_color_b = parts_common.COLOR_FRAME

    # Divide into 5 sloped planks across X
    xs = [-1.0, -0.60, -0.20, 0.20, 0.60, 1.0]
    for i in range(len(xs) - 1):
        x0, x1 = xs[i], xs[i + 1]
        width = x1 - x0 - 0.005
        mid_x = (x0 + x1) / 2
        color = roof_color_a if i % 2 == 0 else roof_color_b

        # Plane slab from high edge (Y = 1.0, Z = 0.12) to low eave (Y = -1.06, Z = 0.06)
        pt_high = (mid_x, 1.0, 0.12)
        pt_low = (mid_x, -1.06, 0.06)
        model.add(parts_common.plane_bm(pt_high, pt_low, span=(1.0, 0.0, 0.0), width=width, thick=0.035, color=color))

    # Front fascia drip board along Y = -1.06
    k.box((-1.0, -1.06, 0.02), (1.0, -1.03, 0.065), frame_color)

    # Side fascia trims along X = -1.0 and X = 1.0
    k.box((-1.0, -1.06, 0.03), (-0.97, 1.0, 0.125), frame_color)
    k.box((0.97, -1.06, 0.03), (1.0, 1.0, 0.125), frame_color)

    # 4. Collider: sloped slab covering the 2x2 footprint
    model.add_collider("roof_1", parts_common.plane_bm((0.0, 1.0, 0.12), (0.0, -1.0, 0.06),
                                                       span=(1.0, 0.0, 0.0), width=2.0, thick=0.08, color="white"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
