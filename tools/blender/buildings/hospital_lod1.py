"""hospital_lod1: low-detail LOD model for the hospital (1200 tri budget).

Same footprint, outline, and main colors as hospital.glb, but no interior,
no window frames, no small details.
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, HERE)

import common  # noqa: E402
import kit  # noqa: E402

NAME, CATEGORY, BUDGET = "hospital_lod1", "buildings", 1200
F = kit.FLOOR_TOP
TOP1 = 3.2
TOP2 = 6.2
TOP3 = 9.2
ROOF_TOP = 9.6

WALL_COLOR = "white"
STRIPE_COLOR = "red_paint"
TRIM_COLOR = "rock_dark"


def build(model):
    k = kit.Kit(model)

    # Ambulance bay pad (x -14 to -7, y -8.5 to 8.5)
    k.box((-14.0, -8.5, 0.0), (-7.0, 8.5, 0.04), "rock_dark")
    k.box((-12.5, -6.5, 0.04), (-8.5, 6.5, 0.06), STRIPE_COLOR)

    # 4 Ambulance bay posts
    posts = [(-13.5, -7.5), (-13.5, 7.5), (-7.5, -7.5), (-7.5, 7.5)]
    for px, py in posts:
        k.solid((px - 0.25, py - 0.25, 0.04), (px + 0.25, py + 0.25, 3.8), WALL_COLOR, base="bay_post")

    # Canopy roof slab
    k.box((-14.2, -8.7, 3.8), (-6.8, 8.7, 4.15), WALL_COLOR)
    # Red fascia edge around canopy roof (sticks out 2 cm)
    for lo, hi in [
        ((-14.18, -8.72, 3.78), (-6.78, -8.68, 4.17)),
        ((-14.18, 8.68, 3.78), (-6.78, 8.72, 4.17)),
        ((-14.22, -8.72, 3.78), (-14.18, 8.72, 4.17)),
    ]:
        k.box(lo, hi, STRIPE_COLOR)

    # Main building body (x -7 to 14, y -9 to 9)
    # Floor 1
    k.solid((-7.0, -9.0, 0.0), (14.0, 9.0, TOP1), WALL_COLOR, base="body_f1")
    # Stripe 1
    k.box((-7.02, -9.02, 3.0), (14.02, 9.02, 3.3), STRIPE_COLOR)

    # Floor 2
    k.box((-7.0, -9.0, TOP1), (14.0, 9.0, TOP2), WALL_COLOR)
    # Stripe 2
    k.box((-7.02, -9.02, 6.0), (14.02, 9.02, 6.3), STRIPE_COLOR)

    # Floor 3
    k.box((-7.0, -9.0, TOP2), (14.0, 9.0, TOP3), WALL_COLOR)

    # Roof slab and parapet
    k.box((-7.1, -9.1, TOP3), (14.1, 9.1, TOP3 + 0.2), "rock_dark")
    # Parapet rim around roof
    k.box((-7.15, 8.95, TOP3 + 0.2), (14.15, 9.15, ROOF_TOP), WALL_COLOR)
    k.box((-7.15, -9.15, TOP3 + 0.2), (14.15, -8.95, ROOF_TOP), WALL_COLOR)
    k.box((-7.15, -8.95, TOP3 + 0.2), (-6.95, 8.95, ROOF_TOP), WALL_COLOR)
    k.box((13.95, -8.95, TOP3 + 0.2), (14.15, 8.95, ROOF_TOP), WALL_COLOR)
    # Parapet red fascia cap
    k.box((-7.2, 8.92, ROOF_TOP - 0.08), (14.2, 9.18, ROOF_TOP + 0.04), STRIPE_COLOR)
    k.box((-7.2, -9.18, ROOF_TOP - 0.08), (14.2, -8.92, ROOF_TOP + 0.04), STRIPE_COLOR)
    k.box((-7.2, -8.92, ROOF_TOP - 0.08), (-6.92, 8.92, ROOF_TOP + 0.04), STRIPE_COLOR)
    k.box((13.92, -8.92, ROOF_TOP - 0.08), (14.2, 8.92, ROOF_TOP + 0.04), STRIPE_COLOR)

    # Elevator penthouse on roof
    k.box((7.5, -6.0, ROOF_TOP), (11.5, -2.0, ROOF_TOP + 1.8), WALL_COLOR)
    # Roof AC unit
    k.box((-3.0, -4.0, ROOF_TOP), (1.0, -1.0, ROOF_TOP + 1.2), "metal")

    # Front entrance awning
    k.solid((1.5, 9.0, 3.2), (6.0, 10.2, 3.35), WALL_COLOR, base="awning")
    k.solid((1.5, 9.02, 0.01), (6.0, 10.0, F), "rock_mid", base="ramp")
    for hx in (2.0, 5.5):
        k.box((hx - 0.08, 9.95, F), (hx + 0.08, 10.11, 3.2), "metal")

    # Red cross sign on front facade (boxes)
    k.box((2.2, 9.01, 6.8), (5.3, 9.06, 8.8), WALL_COLOR)
    k.box((3.3, 9.07, 7.0), (4.2, 9.14, 8.6), STRIPE_COLOR)
    k.box((2.5, 9.06, 7.55), (5.0, 9.15, 8.05), STRIPE_COLOR)

    # Flat painted entrance doors
    k.box((2.5, 9.01, F), (5.0, 9.02, 2.45), "metal")
    k.box((2.65, 9.02, F + 0.2), (4.85, 9.04, 2.35), "glass_tint")
    # Emergency entrance doors
    k.box((-7.02, -0.2, F), (-6.98, 1.8, 2.45), "metal")

    # Flat painted windows (ground floor)
    k.box((-5.5, 9.01, 1.0), (-2.0, 9.02, 2.2), "glass_tint")
    k.box((7.0, 9.01, 1.0), (10.0, 9.02, 2.2), "glass_tint")
    k.box((11.0, 9.01, 1.0), (13.0, 9.02, 2.2), "glass_tint")

    # Flat painted windows (upper floors)
    for z_base in (4.2, 7.2):
        for wx0, wx1 in ((-5.5, -2.0), (-0.5, 2.0), (5.5, 8.0), (9.5, 12.5)):
            k.box((wx0, 9.01, z_base), (wx1, 9.02, z_base + 1.2), "glass_tint")

    k.empty("lot_size", (0.0, 0.0, 0.0), scale=(30.0, 22.0, 1.0))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, view=kit.FRONT_VIEW) else 1


if __name__ == "__main__":
    sys.exit(main())
