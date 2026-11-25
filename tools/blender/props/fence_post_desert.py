"""fence_post_desert: weathered leaning desert fence post with broken rail and wire (<= 120 tris). No collider."""
import os
import sys

_DIR = os.path.dirname(os.path.abspath(__file__))
_PARENT = os.path.join(_DIR, "..")
for _p in (_DIR, _PARENT):
    if _p not in sys.path:
        sys.path.insert(0, _p)

import common  # noqa: E402

NAME, CATEGORY, BUDGET = "fence_post_desert", "props", 120


def build(model):
    # Main weathered wooden post, sunk slightly at base and leaning
    post = common.make_frustum(0.075, 0.060, -0.04, 1.05, segments=5, color="dry_brush")
    common.transform(post, rot=(0.08, -0.11, 0.2))
    model.add(post)

    # Broken horizontal cross-rail stub
    rail = common.make_box((0.52, 0.055, 0.085), loc=(0.16, 0.02, 0.68), color="wood_crate")
    common.transform(rail, rot=(0.05, 0.14, -0.12))
    model.add(rail)

    # Two rusted wire strands jutting from the post
    w1 = common.make_frustum(0.006, 0.004, 0.0, 0.34, segments=3, color="metal")
    common.transform(w1, loc=(0.02, 0.04, 0.88), rot=(1.42, 0.1, 0.3))
    model.add(w1)

    w2 = common.make_frustum(0.006, 0.004, 0.0, 0.28, segments=3, color="metal")
    common.transform(w2, loc=(-0.03, -0.03, 0.52), rot=(-1.35, -0.15, -0.5))
    model.add(w2)

    # Two small sandstone rocks wedged at the base
    model.add(common.make_icosphere(subdiv=1, jitter=0.22, seed=17, radius=(0.12, 0.09, 0.07),
                                    loc=(0.11, -0.06, 0.03), color="sandstone"))
    model.add(common.make_icosphere(subdiv=1, jitter=0.20, seed=23, radius=(0.09, 0.08, 0.06),
                                    loc=(-0.10, 0.08, 0.02), color="sand_dark"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=(0.0, 0.0, 0.0)) else 1


if __name__ == "__main__":
    sys.exit(main())
