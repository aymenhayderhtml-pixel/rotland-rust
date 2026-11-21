"""boathouse_lod1: low LOD for boathouse (<= 500 tris).

Same 14 x 10 m footprint, gable roof, big door opening on +Y, simplified stilts on +Y.
Single body collider, lot_size empty.
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, HERE)

import common  # noqa: E402
import kit  # noqa: E402

NAME, CATEGORY, BUDGET = "boathouse_lod1", "buildings", 500
TOP = 3.4
RIDGE_Z = 5.2
F = kit.FLOOR_TOP
GRIP = (0.0, 0.0, 0.0)


def build(model):
    k = kit.Kit(model)

    # Main building box (outer shell)
    # Left wall section
    k.box((-7.0, -5.0, F), (-2.2, 5.0, TOP), "bark_dark")
    # Right wall section
    k.box((2.2, -5.0, F), (7.0, 5.0, TOP), "bark_dark")
    # Back wall center section (-Y)
    k.box((-2.2, -5.0, F), (2.2, -4.85, TOP), "bark_dark")
    # Front header beam above big boat door (+Y)
    k.box((-2.2, 4.85, 2.95), (2.2, 5.0, TOP), "bark_dark")

    # Front door opening representation (dark interior panel)
    k.box((-2.2, 0.0, F), (2.2, 0.05, 2.95), "gun_black")
    # Back player door panel
    k.box((-0.6, -5.02, F), (0.6, -4.98, kit.DOOR_H), "gun_black")

    # Stilts under +Y water half
    for px in (-6.8, -2.3, 2.3, 6.8):
        for py in (2.5, 4.9):
            post = common.make_frustum(0.14, 0.14, -2.5, 0.1, segments=4, center=(px, py), color="bark_dark")
            k.model.add(post)

    # Gable roof
    ridge = (0.0, 0.0, RIDGE_Z)
    eave_l = (-7.3, 0.0, 3.2)
    eave_r = (7.3, 0.0, 3.2)
    k.roof(ridge, eave_l, span=(0.0, 1.0, 0.0), width=10.4, color="rock_dark")
    k.roof(ridge, eave_r, span=(0.0, 1.0, 0.0), width=10.4, color="rock_dark")

    # Gable ends
    k.gable([(-7.0, 4.85, TOP), (7.0, 4.85, TOP), (0.0, 4.85, RIDGE_Z)], (0.0, 0.15, 0.0), "bark_dark")
    k.gable([(-7.0, -5.0, TOP), (7.0, -5.0, TOP), (0.0, -5.0, RIDGE_Z)], (0.0, 0.15, 0.0), "bark_dark")

    # Collider
    k.collider((-7.0, -5.0, 0.0), (7.0, 5.0, RIDGE_Z), "body")

    # Empty
    k.empty("lot_size", (0.0, 0.0, 0.0), scale=(14.0, 10.0, 1.0))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=GRIP, view=kit.FRONT_VIEW) else 1


if __name__ == "__main__":
    sys.exit(main())
