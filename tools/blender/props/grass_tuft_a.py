"""grass_tuft_a: green grass tuft: a dozen leaning blades in two greens (visual plan step 4)."""
import math
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402
import veg  # noqa: E402
from mathutils import Euler, Vector  # noqa: E402,F401

NAME, CATEGORY, BUDGET = "grass_tuft_a", "nature", 80

def build(model):
    rng = random.Random(11)
    for i in range(13):
        a, r = rng.uniform(0.0, 6.28), rng.uniform(0.0, 0.09)
        color = "grass" if i % 3 else "leaf_light"
        model.add(veg.blade(math.cos(a) * r, math.sin(a) * r, rng.uniform(0.2, 0.46), rng.uniform(0.1, 0.55),
                            rng.uniform(0.0, 6.28), color))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
