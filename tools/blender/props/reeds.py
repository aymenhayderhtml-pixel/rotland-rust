"""reeds: swamp reed clump with tall tan blades, mossy base and cattail heads (<= 120 tris). No collider."""
import math
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402
import veg  # noqa: E402

NAME, CATEGORY, BUDGET = "reeds", "props", 120
SINK = -0.05


def build(model):
    rng = random.Random(84)

    # Low swamp_moss root mound anchoring the clump below ground (16 tris)
    model.add(common.make_frustum(0.24, 0.14, SINK, 0.12, segments=5, color="swamp_moss"))

    # 5 shorter swamp_moss base blades fanning outward (5 * 4 = 20 tris)
    for i in range(5):
        angle = 2.0 * math.pi * i / 5.0 + rng.uniform(-0.25, 0.25)
        dist = rng.uniform(0.05, 0.13)
        height = rng.uniform(0.38, 0.62)
        lean = rng.uniform(0.24, 0.46)
        yaw = angle - math.pi / 2.0 + rng.uniform(-0.2, 0.2)
        blade = veg.blade(math.cos(angle) * dist, math.sin(angle) * dist, height, lean, yaw, "swamp_moss", width=0.046)
        common.transform(blade, loc=(0.0, 0.0, SINK))
        model.add(blade)

    # 10 tall reed_tan blades (1.25-1.78 m) + 3 swamp_mud cattail heads (10 * 4 + 3 * 12 = 76 tris)
    cattail_indices = {1, 5, 8}
    for i in range(10):
        angle = 2.0 * math.pi * i / 10.0 + rng.uniform(-0.22, 0.22)
        dist = rng.uniform(0.02, 0.14)
        bx, by = math.cos(angle) * dist, math.sin(angle) * dist
        height = rng.uniform(1.50, 1.78) if i in cattail_indices else rng.uniform(1.25, 1.68)
        lean = rng.uniform(0.06, 0.16) if i in cattail_indices else rng.uniform(0.10, 0.28)
        yaw = angle - math.pi / 2.0 + rng.uniform(-0.25, 0.25)
        blade = veg.blade(bx, by, height, lean, yaw, "reed_tan", width=0.038)
        common.transform(blade, loc=(0.0, 0.0, SINK))
        model.add(blade)

        if i in cattail_indices:
            head_z0 = height * 0.68
            head = common.make_frustum(0.034, 0.030, head_z0, head_z0 + 0.22, segments=4, color="swamp_mud")
            common.transform(head, rot=(lean, 0.0, yaw), loc=(bx, by, SINK))
            model.add(head)


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=(0.0, 0.0, 0.0)) else 1


if __name__ == "__main__":
    sys.exit(main())
