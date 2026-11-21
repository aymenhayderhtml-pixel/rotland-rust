"""pharmacy_lod1: low LOD for pharmacy, 10 x 8 m footprint on a 12 x 10 m lot.

Same footprint, outline, roof shape and main colors as pharmacy, with no interior
or small detail parts.
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, HERE)

import common  # noqa: E402
import kit  # noqa: E402

NAME, CATEGORY, BUDGET = "pharmacy_lod1", "buildings", 400
TOP = 3.3
F = kit.FLOOR_TOP


def build(model):
    k = kit.Kit(model)

    # Floor / curb base
    k.box((-5.0, -4.0, 0.0), (5.0, 3.4, F), "white")
    k.box((-5.0, 3.4, 0.0), (5.0, 4.0, F), "rock_mid")

    # Walls (green lower band, white upper)
    k.box((-5.0, -4.0, F), (5.0, 3.4, 1.1), "leaf_mid")
    k.box((-5.0, -4.0, 1.1), (5.0, 3.4, TOP), "white")

    # Roof slab and green parapets
    k.box((-5.0, -4.0, TOP), (5.0, 3.4, TOP + 0.15), "rock_dark")
    k.box((-5.0, -4.0, TOP + 0.15), (5.0, -3.85, TOP + 0.35), "leaf_mid")
    k.box((-5.0, 3.25, TOP + 0.15), (5.0, 3.4, TOP + 0.45), "leaf_mid")
    k.box((-5.0, -3.85, TOP + 0.15), (-4.85, 3.25, TOP + 0.35), "leaf_mid")
    k.box((4.85, -3.85, TOP + 0.15), (5.0, 3.25, TOP + 0.35), "leaf_mid")

    # Rooftop HVAC unit
    k.box((1.5, -1.8, TOP + 0.15), (3.0, -0.4, TOP + 0.75), "metal")

    # Green cross sign above door (made of boxes)
    k.box((2.15, 3.41, 2.35), (3.05, 3.44, 3.15), "white")
    k.box((2.45, 3.44, 2.45), (2.75, 3.48, 3.05), "leaf_mid")
    k.box((2.25, 3.44, 2.65), (2.95, 3.48, 2.85), "leaf_mid")

    # Projecting green cross blade sign on corner bracket
    k.box((4.25, 3.4, 2.78), (4.35, 3.9, 2.84), "metal")
    k.box((4.27, 3.55, 2.50), (4.33, 3.85, 2.80), "white")
    k.box((4.26, 3.65, 2.53), (4.34, 3.75, 2.77), "leaf_mid")

    # Door and window colored panels on front face
    k.box((2.0, 3.405, F), (3.2, 3.415, kit.DOOR_H), "gun_black")
    k.box((-4.2, 3.405, 0.8), (1.4, 3.415, 2.4), "rock_dark")

    # Collider
    k.collider((-5.0, -4.0, 0.0), (5.0, 3.4, TOP + 0.45), "body")

    # Lot size
    k.empty("lot_size", (0.0, 0.0, 0.0), scale=(12.0, 10.0, 1.0))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, view=kit.FRONT_VIEW) else 1


if __name__ == "__main__":
    sys.exit(main())
