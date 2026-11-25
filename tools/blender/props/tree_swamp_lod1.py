"""tree_swamp_lod1: medium LOD for cypress/mangrove swamp tree (<= 180 tris)."""
import math
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402
import veg  # noqa: E402

NAME, CATEGORY, BUDGET = "tree_swamp_lod1", "props", 180


def build(model):
    # Flared base sunk 6 cm below ground (16 tris)
    model.add(common.make_frustum(0.90, 0.42, -0.06, 1.60, segments=5, color="swamp_bark"))

    # 4 simplified angled roots (4 * 8 = 32 tris)
    for angle in (0.15, 1.72, 3.25, 4.82):
        dx, dy = math.cos(angle), math.sin(angle)
        model.add(veg.limb((dx * 0.45, dy * 0.45, 1.05), (dx * 1.55, dy * 1.55, -0.08),
                           0.16, 0.07, "swamp_bark", segs=3))

    # 2-segment slightly twisted trunk (2 * 16 = 32 tris)
    model.add(veg.limb((0.0, 0.0, 1.45), (0.16, -0.10, 4.00), 0.43, 0.30, "swamp_bark", segs=5))
    model.add(veg.limb((0.15, -0.09, 3.88), (0.05, 0.04, 6.45), 0.30, 0.17, "swamp_bark", segs=5))

    # Wide flat drooping canopy (3 * 20 = 60 tris)
    model.add(common.make_icosphere(
        subdiv=1, jitter=0.14, seed=81, radius=(3.10, 2.80, 0.95),
        loc=(0.05, 0.0, 7.65), color="swamp_moss",
    ))
    model.add(common.make_icosphere(
        subdiv=1, jitter=0.14, seed=82, radius=(1.80, 1.60, 0.72),
        loc=(-1.50, -1.00, 7.05), color="leaf_dark",
    ))
    model.add(common.make_icosphere(
        subdiv=1, jitter=0.14, seed=83, radius=(1.80, 1.60, 0.72),
        loc=(1.55, 0.95, 7.10), color="swamp_moss",
    ))

    # 2 hanging moss strands (2 * 12 = 24 tris)
    for size, rot, loc in [
        ((0.14, 0.10, 2.25), (0.04, -0.04, 0.30), (1.95, -0.65, 5.75)),
        ((0.13, 0.09, 1.95), (-0.03, 0.04, 1.10), (-1.80, -1.05, 5.85)),
    ]:
        strand = common.make_box(size, color="swamp_moss")
        common.transform(strand, rot=rot, loc=loc)
        model.add(strand)

    model.add_collider("trunk", common.make_frustum(0.60, 0.25, -0.05, 6.20, segments=6, color="swamp_bark"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=(0.0, 0.0, 0.0)) else 1


if __name__ == "__main__":
    sys.exit(main())
