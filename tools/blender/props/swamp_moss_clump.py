"""swamp_moss_clump: spongy mossy mound with tiny pale swamp mushrooms (<= 140 tris). No collider."""
import os
import sys

_DIR = os.path.dirname(os.path.abspath(__file__))
_PARENT = os.path.join(_DIR, "..")
for _p in (_DIR, _PARENT):
    if _p not in sys.path:
        sys.path.insert(0, _p)

import common  # noqa: E402

NAME, CATEGORY, BUDGET = "swamp_moss_clump", "props", 140


def build(model):
    # Three overlapping mossy cushions
    model.add(common.make_icosphere(subdiv=1, jitter=0.22, seed=91,
                                    radius=(0.28, 0.24, 0.14), loc=(0.00, 0.00, 0.06), color="swamp_moss"))
    model.add(common.make_icosphere(subdiv=1, jitter=0.20, seed=92,
                                    radius=(0.18, 0.16, 0.10), loc=(0.18, -0.10, 0.04), color="leaf_dark"))
    model.add(common.make_icosphere(subdiv=1, jitter=0.20, seed=93,
                                    radius=(0.16, 0.15, 0.09), loc=(-0.16, 0.12, 0.04), color="swamp_mud"))

    # Two tiny pale swamp mushrooms growing out of the moss
    for x, y, z, h, cap in [(0.05, 0.06, 0.16, 0.09, 0.055), (-0.08, -0.04, 0.14, 0.07, 0.042)]:
        stem = common.make_frustum(0.012, 0.014, z, z + h, segments=3, color="bone_white")
        head = common.make_frustum(cap, 0.0, z + h - 0.01, z + h + 0.04, segments=4, color="reed_tan")
        model.add(stem)
        model.add(head)


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=(0.0, 0.0, 0.0)) else 1


if __name__ == "__main__":
    sys.exit(main())
