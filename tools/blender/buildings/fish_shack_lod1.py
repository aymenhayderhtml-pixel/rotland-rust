"""fish_shack_lod1: low LOD for fish_shack (<= 400 tris).

Same 8 x 6 m footprint, gable roof, smoke stack on roof, simplified nets and buoys.
Single body collider, lot_size empty.
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, HERE)

import common  # noqa: E402
import kit  # noqa: E402

NAME, CATEGORY, BUDGET = "fish_shack_lod1", "buildings", 400
TOP = 2.6
RIDGE_Z = 3.8
F = kit.FLOOR_TOP


def build(model):
    k = kit.Kit(model)

    # Main outer box
    k.box((-4.0, -3.0, F), (4.0, 3.0, TOP), "bark_light")

    # Gable roof
    ridge = (0.0, 0.0, RIDGE_Z)
    eave_l = (-4.3, 0.0, 2.4)
    eave_r = (4.3, 0.0, 2.4)
    k.roof(ridge, eave_l, span=(0.0, 1.0, 0.0), width=6.4, color="rock_mid")
    k.roof(ridge, eave_r, span=(0.0, 1.0, 0.0), width=6.4, color="rock_mid")

    # Gable ends
    k.gable([(-4.0, 2.85, TOP), (4.0, 2.85, TOP), (0.0, 2.85, RIDGE_Z)], (0.0, 0.15, 0.0), "bark_dark")
    k.gable([(-4.0, -3.0, TOP), (4.0, -3.0, TOP), (0.0, -3.0, RIDGE_Z)], (0.0, 0.15, 0.0), "bark_dark")

    # Door and window colored panels on exterior
    k.box((-0.6, 3.005, F), (0.6, 3.015, kit.DOOR_H), "gun_black")
    k.box((1.8, 3.005, 1.0), (3.2, 3.015, 1.9), "rock_dark")

    # Smoke stack on roof (simplified 4-sided frustum)
    pipe = common.make_frustum(0.12, 0.11, 2.8, 4.3, segments=4, center=(-1.8, -1.2), rot_z=0.785, color="gun_black")
    pipe_cap = common.make_frustum(0.22, 0.04, 4.3, 4.45, segments=4, center=(-1.8, -1.2), rot_z=0.785, color="gun_black")
    k.model.add(pipe)
    k.model.add(pipe_cap)

    # Simplified net panel on left wall
    k.box((-4.015, -1.8, 0.7), (-4.005, 1.8, 2.2), "canvas_khaki")

    # Simplified buoys panel on front wall
    k.box((-2.5, 3.005, 1.3), (-1.0, 3.015, 1.8), "red_paint")

    # Collider
    k.collider((-4.0, -3.0, 0.0), (4.0, 3.0, RIDGE_Z), "body")

    # Empty
    k.empty("lot_size", (0.0, 0.0, 0.0), scale=(8.0, 6.0, 1.0))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, view=kit.FRONT_VIEW) else 1


if __name__ == "__main__":
    sys.exit(main())
