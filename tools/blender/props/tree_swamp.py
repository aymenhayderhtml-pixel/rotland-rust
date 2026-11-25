"""tree_swamp: cypress/mangrove swamp tree with flared base, arching roots, twisted trunk and drooping moss canopy (<= 500 tris)."""
import math
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402
import veg  # noqa: E402
from mathutils import Vector  # noqa: E402

NAME, CATEGORY, BUDGET = "tree_swamp", "props", 500


def build(model):
    rng = random.Random(81)

    # Wide flared buttress base sunk 6 cm below ground (24 tris)
    model.add(common.make_frustum(0.92, 0.44, -0.06, 1.60, segments=7, color="swamp_bark"))

    # 5 arching mangrove/cypress roots plunging into the mud (5 * 24 = 120 tris)
    for i in range(5):
        angle = 2.0 * math.pi * i / 5.0 + rng.uniform(-0.22, 0.22)
        dx, dy = math.cos(angle), math.sin(angle)
        start = Vector((dx * 0.42, dy * 0.42, 1.15 + rng.uniform(-0.12, 0.14)))
        knee_r = 1.18 + rng.uniform(-0.10, 0.12)
        knee = Vector((dx * knee_r, dy * knee_r, 0.62 + rng.uniform(-0.08, 0.10)))
        end_r = 1.65 + rng.uniform(-0.12, 0.15)
        ground = Vector((dx * end_r, dy * end_r, -0.08))
        knee_overlap = knee + (start - knee).normalized() * 0.04
        model.add(veg.limb(start, knee, 0.17, 0.13, "swamp_bark", segs=4))
        model.add(veg.limb(knee_overlap, ground, 0.13, 0.07, "swamp_bark", segs=4))

    # 3-segment twisted trunk reaching ~6.5 m (3 * 20 = 60 tris)
    model.add(veg.limb((0.0, 0.0, 1.40), (0.24, -0.15, 3.15), 0.46, 0.34, "swamp_bark", segs=6))
    model.add(veg.limb((0.22, -0.14, 3.00), (-0.20, 0.20, 4.85), 0.35, 0.25, "swamp_bark", segs=6))
    model.add(veg.limb((-0.18, 0.18, 4.70), (0.08, 0.04, 6.50), 0.26, 0.17, "swamp_bark", segs=6))

    # 4 spreading upper boughs supporting the flat canopy (4 * 12 = 48 tris)
    for start, end in [
        ((-0.04, 0.10, 5.65), (2.25, -0.75, 6.85)),
        ((-0.02, 0.12, 5.80), (-2.10, -1.25, 6.80)),
        ((0.02, 0.08, 5.95), (-0.55, 2.25, 6.95)),
        ((0.06, 0.05, 6.10), (1.45, 1.75, 7.00)),
    ]:
        model.add(veg.limb(start, end, 0.15, 0.07, "swamp_bark", segs=4))

    # Flat drooping cypress canopy (~8.7 m tall, 9 * 20 = 180 tris)
    model.add(common.make_icosphere(
        subdiv=1, jitter=0.14, seed=81, radius=(2.50, 2.30, 0.80),
        loc=(0.05, 0.0, 7.65), color="swamp_moss",
    ))
    model.add(common.make_icosphere(
        subdiv=1, jitter=0.14, seed=82, radius=(1.85, 1.70, 0.55),
        loc=(0.0, 0.05, 8.15), color="swamp_moss",
    ))
    perimeter_blobs = [
        (2.15, -0.70, 7.05, (1.55, 1.40, 0.68), (0.08, 0.14, 0.0), "swamp_moss"),
        (-1.95, -1.20, 6.98, (1.60, 1.45, 0.70), (0.10, -0.12, 0.2), "leaf_dark"),
        (-0.45, 2.15, 7.12, (1.50, 1.40, 0.65), (-0.14, -0.04, 0.1), "swamp_moss"),
        (1.45, 1.75, 7.18, (1.45, 1.35, 0.65), (-0.10, 0.10, -0.2), "leaf_dark"),
        (-2.20, 0.75, 7.08, (1.45, 1.35, 0.66), (-0.06, -0.14, 0.3), "swamp_moss"),
        (0.55, -2.10, 6.95, (1.50, 1.40, 0.68), (0.14, 0.05, -0.1), "leaf_dark"),
        (2.15, 0.65, 7.22, (1.35, 1.25, 0.62), (-0.04, 0.12, 0.2), "swamp_moss"),
    ]
    for idx, (x, y, z, rad, rot, col) in enumerate(perimeter_blobs):
        model.add(common.make_icosphere(
            subdiv=1, jitter=0.16, seed=90 + idx, radius=rad,
            loc=(x, y, z), rot=rot, color=col,
        ))

    # 3 hanging Spanish moss strands under outer canopy edges (3 * 12 = 36 tris)
    for size, rot, loc in [
        ((0.13, 0.09, 2.30), (0.04, -0.05, 0.30), (2.05, -0.68, 5.75)),
        ((0.11, 0.08, 2.00), (-0.03, 0.04, 1.10), (-1.85, -1.15, 5.85)),
        ((0.12, 0.08, 1.85), (0.05, 0.03, 2.20), (-0.35, 2.05, 6.02)),
    ]:
        strand = common.make_box(size, color="swamp_moss")
        common.transform(strand, rot=rot, loc=loc)
        model.add(strand)

    model.add_collider("trunk", common.make_frustum(0.60, 0.25, -0.05, 6.20, segments=6, color="swamp_bark"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=(0.0, 0.0, 0.0)) else 1


if __name__ == "__main__":
    sys.exit(main())
