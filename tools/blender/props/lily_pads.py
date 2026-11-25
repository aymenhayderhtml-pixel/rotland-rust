"""lily_pads: flat cluster of notched lily pads with two small blossoms (<= 100 tris). No collider."""
import math
import os
import sys

_DIR = os.path.dirname(os.path.abspath(__file__))
_PARENT = os.path.join(_DIR, "..")
for _p in (_DIR, _PARENT):
    if _p not in sys.path:
        sys.path.insert(0, _p)

import common  # noqa: E402

NAME, CATEGORY, BUDGET = "lily_pads", "props", 100


def _pad(radius, loc, notch_angle, color="leaf_mid"):
    """7-wedge notched circular pad facing +Z."""
    bm = common.new_bm()
    center = bm.verts.new((0.0, 0.0, 0.0))
    span = math.tau * 0.86
    ring = []
    for i in range(8):
        a = notch_angle + span * (i / 7.0)
        ring.append(bm.verts.new((math.cos(a) * radius, math.sin(a) * radius, 0.0)))
    for i in range(7):
        bm.faces.new([center, ring[i], ring[i + 1]])
    common.paint(bm, color)
    common.transform(bm, loc=loc)
    return bm


def _blossom(loc):
    cup = common.make_frustum(0.020, 0.052, 0.0, 0.035, segments=5, color="bone_white")
    core = common.make_frustum(0.018, 0.010, 0.015, 0.042, segments=4, color="wheat_gold")
    flower = common.combine([cup, core])
    common.transform(flower, loc=loc)
    return flower


def build(model):
    pads = [
        (0.20, (0.00, 0.00, 0.020), 0.4, "leaf_mid"),
        (0.16, (0.28, 0.14, 0.022), 2.1, "swamp_moss"),
        (0.18, (-0.24, 0.16, 0.018), 4.2, "leaf_dark"),
        (0.14, (-0.14, -0.24, 0.021), 1.3, "leaf_mid"),
        (0.12, (0.22, -0.20, 0.019), 3.6, "swamp_moss"),
    ]
    for rad, loc, angle, color in pads:
        model.add(_pad(rad, loc, angle, color))

    model.add(_blossom((0.03, -0.02, 0.022)))
    model.add(_blossom((-0.22, 0.18, 0.020)))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=(0.0, 0.0, 0.0)) else 1


if __name__ == "__main__":
    sys.exit(main())
