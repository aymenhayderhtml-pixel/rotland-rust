"""mushrooms: three small red-capped mushrooms in a cluster (<= 50 tris). No collider."""
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402

NAME, CATEGORY, BUDGET = "mushrooms", "nature", 50

# (x, y, cap radius)
SHROOMS = [(0.0, 0.0, 0.11), (0.20, 0.10, 0.09), (-0.14, 0.18, 0.08)]


def build(model):
    for x, y, cap in SHROOMS:
        model.add(common.make_frustum(0.03, 0.035, 0.0, 0.14, segments=3, center=(x, y), color="white"))
        model.add(common.make_frustum(cap, 0.0, 0.12, 0.20, segments=4, center=(x, y), color="red_paint"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
