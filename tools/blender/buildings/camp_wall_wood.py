"""camp_wall_wood: 4 m section of sharpened log palisade (2.6 m high) with an inside walkway plank.

Wall SECTIONS that tile in a row along X: 4 m long (-2.0 to +2.0), centered on X, thickness along Y.
+Y is the outside face, -Y is the inside camp face (with walkway).
Colliders: palisade wall and walkway deck.
"""
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, HERE)

import common  # noqa: E402
import kit  # noqa: E402

NAME, CATEGORY, BUDGET = "camp_wall_wood", "buildings", 1200
LENGTH = 4.0
HALF_L = LENGTH / 2.0  # 2.0 m
AVG_HEIGHT = 2.6
NUM_LOGS = 16
LOG_R = 0.13
LOG_DIAMETER = 0.25  # spacing between logs: 4.0 / 16 = 0.25 m


def build(model):
    k = kit.Kit(model)

    # 1. Vertical sharpened logs along X (-2.0 to +2.0)
    # Varied heights around 2.6 m with sharpened conical tops
    heights = [
        2.58, 2.64, 2.55, 2.68, 2.60, 2.53, 2.66, 2.57,
        2.63, 2.59, 2.70, 2.54, 2.65, 2.58, 2.67, 2.56
    ]
    colors = ["bark_light", "bark_dark"]

    for i in range(NUM_LOGS):
        cx = -HALF_L + 0.125 + i * LOG_DIAMETER
        cy = 0.08  # slightly forward so walkway fits on -Y inside
        h = heights[i]
        tip_h = 0.45
        body_h = h - tip_h
        color = colors[i % 2]
        rot_z = (i * 0.72) % math.pi

        # 6-sided cylinder for trunk
        body = common.make_frustum(LOG_R, LOG_R, 0.0, body_h, segments=6,
                                   center=(cx, cy), rot_z=rot_z, color=color)
        model.add(body)

        # 6-sided sharpened cone tip
        tip = common.make_frustum(LOG_R, 0.0, body_h, h, segments=6,
                                  center=(cx, cy), rot_z=rot_z, color=color)
        model.add(tip)

    # 2. Horizontal lashing cross-rails on inside face of logs (cy - LOG_R)
    rail_y0 = cy - LOG_R - 0.08
    rail_y1 = cy - LOG_R
    # Lower rail
    k.box((-HALF_L + 0.01, rail_y0, 0.65), (HALF_L - 0.01, rail_y1, 0.77), "bark_dark")
    # Upper rail
    k.box((-HALF_L + 0.01, rail_y0, 1.95), (HALF_L - 0.01, rail_y1, 2.07), "bark_dark")

    # 3. Outside ground sleeper / anchor log (+Y side)
    sill_y0 = cy + LOG_R
    sill_y1 = sill_y0 + 0.14
    k.box((-HALF_L + 0.01, sill_y0, 0.0), (HALF_L - 0.01, sill_y1, 0.22), "bark_dark")

    # 4. Inside walkway plank (at z = 1.3 m, allowing guard to see over 2.6m wall)
    walk_z = 1.30
    walk_th = 0.08
    walk_y0 = rail_y0 - 0.44  # walkway width 0.44 m
    walk_y1 = rail_y0 + 0.02  # snug against cross-rail
    # Deck planks
    k.box((-HALF_L + 0.01, walk_y0, walk_z - walk_th), (HALF_L - 0.01, walk_y1, walk_z), "wood_crate")
    # Outer longitudinal support beam under the walkway
    k.box((-HALF_L + 0.01, walk_y0, walk_z - walk_th - 0.08),
          (HALF_L - 0.01, walk_y0 + 0.10, walk_z - walk_th), "bark_dark")

    # Vertical support posts under walkway at x = -1.5, 0.0, +1.5
    for px in (-1.5, 0.0, 1.5):
        k.box((px - 0.06, walk_y0 + 0.01, 0.0), (px + 0.06, walk_y0 + 0.09, walk_z - walk_th - 0.08), "bark_dark")
        # Diagonal strut from wall base to walkway outer beam
        k.box((px - 0.04, walk_y0 + 0.10, walk_z - walk_th - 0.15),
              (px + 0.04, rail_y0, 0.45), "bark_light")

    # Small wooden access ladder on the inside at x = -0.8
    k.ladder(-0.8, walk_y0 - 0.02, walk_z + 0.04, color="bark_light", half_w=0.22, rung_gap=0.28)

    # 5. Colliders:
    # Palisade wall solid:
    k.collider((-HALF_L, -0.08, 0.0), (HALF_L, 0.24, AVG_HEIGHT), base="palisade")
    # Walkway solid:
    k.collider((-HALF_L, walk_y0, 0.0), (HALF_L, rail_y1, walk_z), base="walkway")


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, view=kit.FRONT_VIEW) else 1


if __name__ == "__main__":
    sys.exit(main())
