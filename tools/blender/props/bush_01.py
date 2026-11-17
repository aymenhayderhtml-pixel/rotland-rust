"""bush_01: bush: a mound of small leaf clusters in three greens, with a few red berries (visual plan step 4)."""
import math
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402
import veg  # noqa: E402
from mathutils import Euler, Vector  # noqa: E402,F401

NAME, CATEGORY, BUDGET = "bush_01", "nature", 800

# (x, y, z, radius, color)
BLOBS = [(0.0, 0.0, 0.42, 0.42, "leaf_mid"), (0.36, 0.10, 0.30, 0.30, "leaf_dark"),
         (-0.32, 0.20, 0.28, 0.32, "leaf_dark"), (0.12, -0.36, 0.30, 0.32, "leaf_mid"),
         (-0.22, -0.26, 0.45, 0.28, "leaf_dark"), (0.26, 0.36, 0.48, 0.27, "leaf_light"),
         (-0.05, 0.08, 0.72, 0.27, "leaf_light"), (0.30, -0.12, 0.62, 0.22, "leaf_light")]


def build(model):
    rng = random.Random(9)
    for seed, (x, y, z, r, color) in enumerate(BLOBS):
        model.add(common.make_icosphere(subdiv=2, jitter=0.2, seed=seed + 20, radius=(r, r, r * 0.8),
                                        loc=(x, y, z), color=color))
    for _ in range(7):
        a, h = rng.uniform(0.0, 6.28), rng.uniform(0.25, 0.75)
        r = 0.5 * math.sqrt(max(0.05, 1.0 - ((h - 0.35) / 0.55) ** 2))
        model.add(common.make_box((0.05, 0.05, 0.05), loc=(math.cos(a) * r, math.sin(a) * r, h), color="red_paint"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
