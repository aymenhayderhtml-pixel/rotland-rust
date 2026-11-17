"""grass_tuft_b: dry grass tuft: straw-coloured blades of mixed height (visual plan step 4)."""
import math
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402
import veg  # noqa: E402
from mathutils import Euler, Vector  # noqa: E402,F401

NAME, CATEGORY, BUDGET = "grass_tuft_b", "nature", 80

def build(model):
    rng = random.Random(12)
    for i in range(13):
        a, r = rng.uniform(0.0, 6.28), rng.uniform(0.0, 0.09)
        color = "dirt" if i % 3 else "bark_light"
        model.add(veg.blade(math.cos(a) * r, math.sin(a) * r, rng.uniform(0.2, 0.5), rng.uniform(0.15, 0.6),
                            rng.uniform(0.0, 6.28), color))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
