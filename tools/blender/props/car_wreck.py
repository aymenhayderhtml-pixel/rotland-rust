"""car_wreck: a rusted hatchback shell on flat tyres (<= 600 tris). Has a collider.

Standalone, not the V11-08 car script: the wreck has no glass, no doors that work and no
steering, so it only shares the car's shape (sizes and the cabin loft from car_parts).
"""
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, os.path.join(HERE, "..", "vehicles"))
import common  # noqa: E402
import car_parts  # noqa: E402  (the cabin loft is the same shape as the living car)

NAME, CATEGORY, BUDGET = "car_wreck", "props", 600
BODY_HALF_WIDTH = 0.80
BODY_HALF_LENGTH = 1.90
SILL_Z = 0.30  # wreck sits a little higher: the tyres are flat
BELT_Z = 0.85
ROOF_Z = 1.42
WHEEL_RADIUS = 0.22  # flat tyres
WHEEL_HALF_WIDTH = 0.06
WHEEL_X = 0.72
WHEEL_Y = 1.22


def _body():
    """Rusted lower body, glassless cabin, one dangling bumper and a faded red door panel."""
    lower = common.make_box((2 * BODY_HALF_WIDTH, 2 * BODY_HALF_LENGTH, BELT_Z - SILL_Z),
                            loc=(0.0, 0.0, (SILL_Z + BELT_Z) / 2), color="rust")
    cabin = common.new_bm()
    car_parts._loft(cabin,
                    bottom=[(-0.70, 0.90, BELT_Z), (0.70, 0.90, BELT_Z), (0.66, -1.85, BELT_Z), (-0.66, -1.85, BELT_Z)],
                    top=[(-0.62, 0.30, ROOF_Z), (0.62, 0.30, ROOF_Z), (0.60, -1.20, ROOF_Z), (-0.60, -1.20, ROOF_Z)],
                    side_colors=["glass_tint", "rust", "glass_tint", "rust"], top_color="rust")
    bumper = common.make_box((1.7, 0.06, 0.14), loc=(0.0, 1.93, 0.36), color="metal")
    common.transform(bumper, rot=(0.35, 0.0, 0.0))  # hanging off one side
    door = common.make_box((0.03, 0.95, 0.55), loc=(-BODY_HALF_WIDTH - 0.015, 0.5, 0.6), color="car_red")
    return common.combine([lower, cabin, bumper, door])


def _tyre():
    """Flat tyre, origin at the wheel centre, axle along X."""
    tyre = common.make_frustum(WHEEL_RADIUS, WHEEL_RADIUS, -WHEEL_HALF_WIDTH, WHEEL_HALF_WIDTH,
                               segments=6, color="tire_black")
    common.transform(tyre, rot=(0.0, math.pi / 2, 0.0))
    return tyre


def build(model):
    model.add(_body())
    for x, y in [(-WHEEL_X, WHEEL_Y), (WHEEL_X, WHEEL_Y), (-WHEEL_X, -WHEEL_Y), (WHEEL_X, -WHEEL_Y)]:
        tyre = _tyre()
        common.transform(tyre, loc=(x, y, WHEEL_RADIUS))
        model.add(tyre)
    model.add_collider("shell", common.make_box((2 * (BODY_HALF_WIDTH - 0.02), 2 * (BODY_HALF_LENGTH - 0.02),
                                                 ROOF_Z - SILL_Z),
                                                loc=(0.0, 0.0, (SILL_Z + ROOF_Z) / 2), color="white"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
