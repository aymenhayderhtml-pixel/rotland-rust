"""police_lod1: low-detail LOD model for the police station (1000 tri budget).

Same footprint, outline, and main colors as police.glb, but no interior,
no window frames, no small details.
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, HERE)

import common  # noqa: E402
import kit  # noqa: E402

NAME, CATEGORY, BUDGET = "police_lod1", "buildings", 1000
F = kit.FLOOR_TOP
TOP1 = 3.0
TOP2 = 5.8
ROOF_TOP = 6.2

WALL_COLOR = "gown_blue"
STRIPE_COLOR = "shirt_blue"
TRIM_COLOR = "rock_dark"


def build(model):
    k = kit.Kit(model)

    # Parking area pad (x -11 to -4, y -8 to 7) - sunken base to prevent Z=0 overlap with fixtures
    k.box((-11.0, -8.0, -0.02), (-4.0, 7.0, 0.04), "rock_dark")

    # Low-poly parking fence (simplified rails)
    # West fence
    k.box((-11.04, -8.04, 0.0), (-10.96, 7.04, 2.0), "metal")
    # South fence (butts against west fence)
    k.box((-10.96, -8.04, 0.0), (-4.0, -7.96, 2.0), "metal")
    # North fence (butts against west fence)
    k.box((-10.96, 6.96, 0.0), (-8.5, 7.04, 2.0), "metal")
    # Barrier housing & arm
    k.box((-4.5, 6.8, 0.0), (-4.1, 7.2, 1.1), "shirt_blue")
    k.box((-8.3, 6.95, 0.83), (-4.1, 7.05, 0.95), "white")
    k.box((-7.5, 6.93, 0.81), (-7.0, 7.07, 0.97), "red_paint")
    k.box((-6.0, 6.93, 0.81), (-5.5, 7.07, 0.97), "red_paint")

    # Dumpster
    k.box((-10.5, -7.2, 0.0), (-8.8, -5.8, 1.3), "shirt_blue")

    # Building main volume (x -4 to 11, y -8 to 7)
    # Floor 1
    k.solid((-4.0, -8.0, 0.0), (11.0, 7.0, TOP1), WALL_COLOR, base="body_f1")
    # Lower blue stripe
    k.box((-4.02, -8.02, 2.65), (11.02, 7.02, 2.95), STRIPE_COLOR)

    # Floor 2
    k.box((-4.0, -8.0, TOP1), (11.0, 7.0, TOP2), WALL_COLOR)
    # Upper blue stripe
    k.box((-4.02, -8.02, 5.4), (11.02, 7.02, 5.65), STRIPE_COLOR)

    # Roof slab and parapet
    k.box((-4.1, -8.1, TOP2), (11.1, 7.1, TOP2 + 0.15), "rock_dark")
    # Parapet rim sitting on the roof slab
    k.box((-4.1, 6.9, TOP2 + 0.15), (11.1, 7.1, ROOF_TOP), TRIM_COLOR)
    k.box((-4.1, -8.1, TOP2 + 0.15), (11.1, -7.9, ROOF_TOP), TRIM_COLOR)
    k.box((-4.1, -7.9, TOP2 + 0.15), (-3.9, 6.9, ROOF_TOP), TRIM_COLOR)
    k.box((10.9, -7.9, TOP2 + 0.15), (11.1, 6.9, ROOF_TOP), TRIM_COLOR)

    # Roof AC unit
    k.box((1.0, -1.0, TOP2 + 0.15), (3.5, 1.5, TOP2 + 1.1), "metal")

    # Front entrance steps
    k.solid((1.5, 7.0, 0.0), (4.5, 8.0, F), "rock_dark", base="steps")

    # Flagpole
    k.model.add(common.make_frustum(0.08, 0.05, 0.0, 7.2, segments=4, center=(7.5, 7.6), color="metal"))

    # Flat painted entrance doors
    k.box((2.0, 6.99, F), (4.0, 7.02, 2.4), "metal")
    k.box((2.2, 7.02, 1.1), (3.8, 7.04, 2.1), "glass_tint")
    # Side door
    k.box((-4.02, 1.0, F), (-3.98, 2.2, kit.DOOR_H), "metal")

    # Flat painted windows (ground floor)
    k.box((-2.8, 7.01, 1.0), (-0.8, 7.02, 2.2), "glass_tint")
    k.box((5.5, 7.01, 1.0), (7.5, 7.02, 2.2), "glass_tint")
    k.box((8.5, 7.01, 1.0), (10.0, 7.02, 2.2), "glass_tint")

    # Flat painted windows (upper floor)
    for wx0, wx1 in ((-2.5, -0.5), (1.5, 3.5), (5.5, 7.5), (8.5, 10.0)):
        k.box((wx0, 7.01, 4.0), (wx1, 7.02, 5.2), "glass_tint")

    k.empty("lot_size", (0.0, 0.0, 0.0), scale=(24.0, 18.0, 1.0))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, view=kit.FRONT_VIEW) else 1


if __name__ == "__main__":
    sys.exit(main())
