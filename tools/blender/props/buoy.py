"""buoy: floating navigation marker buoy with red/white body and beacon mast (<= 150 tris). Has a collider."""
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402

NAME, CATEGORY, BUDGET = "buoy", "props", 150


def build(model):
    # Buoy sits at water level / ground (Z=0.0 to 1.55)
    # Lower float body: 6-segment frustum in red paint (Z: 0.0 to 0.45)
    body_lower = common.make_frustum(0.32, 0.42, 0.0, 0.45, segments=6, color="red_paint")
    model.add(body_lower)

    # White middle band (Z: 0.45 to 0.65)
    band_white = common.make_frustum(0.42, 0.38, 0.45, 0.65, segments=6, color="white")
    model.add(band_white)

    # Upper cone body: red paint (Z: 0.65 to 0.88)
    body_upper = common.make_frustum(0.38, 0.24, 0.65, 0.88, segments=6, color="red_paint")
    model.add(body_upper)

    # Black rubber bumper ring around widest waist (Z: 0.42 to 0.48)
    bumper = common.make_frustum(0.44, 0.44, 0.42, 0.48, segments=6, color="tire_black")
    model.add(bumper)

    # Upper metal mast/cage (Z: 0.88 to 1.35)
    mast = common.make_box((0.06, 0.06, 0.50), loc=(0.0, 0.0, 1.12), color="metal")
    cage_ring = common.make_frustum(0.20, 0.20, 1.25, 1.28, segments=5, color="metal")
    model.add(mast)
    model.add(cage_ring)

    # Top beacon / solar reflector (brass / yellow) (Z: 1.35 to 1.52)
    beacon = common.make_frustum(0.10, 0.02, 1.35, 1.52, segments=4, rot_z=0.785, color="brass")
    model.add(beacon)

    # Mooring eye ring at bottom
    eye = common.make_box((0.08, 0.08, 0.06), loc=(0.0, 0.0, -0.03), color="metal")
    model.add(eye)

    # Collider
    model.add_collider("body", common.make_box((0.90, 0.90, 1.55), loc=(0.0, 0.0, 0.77), color="white"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
