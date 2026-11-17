"""tire_pile: six old tyres stacked in a pyramid (<= 250 tris). Has a collider around the pile."""
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402

NAME, CATEGORY, BUDGET = "tire_pile", "props", 900
TYRE_HEIGHT = 0.22
TYRE_RADIUS = 0.36

# (x, y, z0): three on the ground, two on them, one on top. Each tyre is a solid disc, so it stays cheap.
TYRES = [(-0.40, -0.20, 0.0), (0.40, -0.20, 0.0), (0.0, 0.60, 0.0),
         (-0.20, 0.20, TYRE_HEIGHT), (0.30, 0.45, TYRE_HEIGHT),
         (0.05, 0.10, 2 * TYRE_HEIGHT)]


def build(model):
    for x, y, z0 in TYRES:
        model.add(common.make_frustum(TYRE_RADIUS, TYRE_RADIUS, z0, z0 + TYRE_HEIGHT - 0.04, segments=12,
                                      center=(x, y), color="tire_black"))
        model.add(common.make_frustum(TYRE_RADIUS, TYRE_RADIUS - 0.03, z0 + TYRE_HEIGHT - 0.04, z0 + TYRE_HEIGHT, segments=12,
                                      center=(x, y), color="tire_black"))
        model.add(common.make_frustum(0.2, 0.2, z0 + TYRE_HEIGHT - 0.035, z0 + TYRE_HEIGHT + 0.002, segments=10,
                                      center=(x, y), color="gun_black"))  # dark hollow centre
    model.add_collider("pile", common.make_box((1.6, 1.6, 0.66), loc=(0.0, 0.2, 0.33), color="white"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
