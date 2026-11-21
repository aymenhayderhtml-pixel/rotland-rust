"""base_stairs: 2x2 m footprint player-built wooden stairs rising 3 m.

Snap: cell. Grid size: 2.0 m. Height: 3.0 m.
Rises from bottom at Y = -1.0 (Z = 0) to top at Y = +1.0 (Z = 3.0),
connecting Floor 0 to Floor 1. Uses a smooth ramp collider for fluid walking.
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

NAME = "base_stairs"
CATEGORY = parts_common.CATEGORY
BUDGET = parts_common.BUDGET_STRUCTURE


def build(model):
    k = kit.Kit(model)
    frame_color = parts_common.COLOR_FRAME
    tread_color = parts_common.COLOR_PLANK_A
    riser_color = parts_common.COLOR_PLANK_B

    num_steps = 10
    step_run = 2.0 / num_steps   # 0.20 m
    step_rise = 3.0 / num_steps  # 0.30 m

    # 1. Steps (treads and risers)
    for i in range(num_steps):
        y0 = -1.0 + i * step_run
        y1 = y0 + step_run
        z_top = (i + 1) * step_rise

        # Tread: horizontal plank (width 1.80 m, slight nosing overhang)
        k.box((-0.90, y0, z_top - 0.04), (0.90, y1 + 0.02, z_top), tread_color)

        # Riser: vertical board underneath tread (skip ground step 0)
        if i > 0:
            k.box((-0.90, y0, z_top - step_rise), (0.90, y0 + 0.03, z_top - 0.04), riser_color)

    # 2. Side stringers (angled carriage beams along left and right)
    stringer_thick = 0.08
    pt_bot_l = (-0.95, -1.0, 0.0)
    pt_top_l = (-0.95, 1.0, 3.0)
    model.add(parts_common.plane_bm(pt_top_l, pt_bot_l, span=(1.0, 0.0, 0.0), width=stringer_thick, thick=0.25, color=frame_color))

    pt_bot_r = (0.95, -1.0, 0.0)
    pt_top_r = (0.95, 1.0, 3.0)
    model.add(parts_common.plane_bm(pt_top_r, pt_bot_r, span=(1.0, 0.0, 0.0), width=stringer_thick, thick=0.25, color=frame_color))

    # 3. Handrail on the right side (X = 0.94)
    # Newel post at bottom
    k.box((0.90, -0.98, 0.0), (0.98, -0.90, 1.15), frame_color)
    # Newel post at top
    k.box((0.90, 0.90, 3.0), (0.98, 0.98, 4.15), frame_color)

    # Angled handrail beam along the posts
    pt_rail_bot = (0.94, -0.94, 1.15)
    pt_rail_top = (0.94, 0.94, 4.15)
    model.add(parts_common.plane_bm(pt_rail_top, pt_rail_bot, span=(1.0, 0.0, 0.0), width=0.07, thick=0.06, color=frame_color))

    # 4. Smooth ramp collider from bottom to top for player movement
    model.add_collider("ramp", parts_common.plane_bm((0.0, 1.0, 3.0), (0.0, -1.0, 0.0),
                                                     span=(1.0, 0.0, 0.0), width=2.0, thick=0.15, color="white"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
