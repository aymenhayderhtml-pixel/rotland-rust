"""barrel_rusty: an upright steel barrel with two rust bands (<= 200 tris). Has a collider."""
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402

NAME, CATEGORY, BUDGET = "barrel_rusty", "props", 400
RADIUS = 0.30
HEIGHT = 0.90


def build(model):
    model.add(common.make_frustum(RADIUS, RADIUS, 0.0, HEIGHT, segments=14, color="metal"))
    for z0 in (0.12, 0.42, 0.70):
        model.add(common.make_frustum(RADIUS + 0.012, RADIUS + 0.012, z0, z0 + 0.05, segments=14, color="rust"))
    # Rolled top rim, and a lid bung
    model.add(common.make_frustum(RADIUS + 0.01, RADIUS - 0.01, HEIGHT - 0.01, HEIGHT + 0.025, segments=14, color="rust"))
    model.add(common.make_frustum(0.04, 0.04, HEIGHT, HEIGHT + 0.03, segments=8, center=(0.12, 0.08), color="gun_black"))
    model.add_collider("body", common.make_frustum(RADIUS, RADIUS, 0.0, HEIGHT, segments=8, color="metal"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
