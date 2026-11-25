"""ice_shards: cluster of sharp crystalline ice spikes jutting from snow base (<= 110 tris). No collider."""
import os
import sys

_DIR = os.path.dirname(os.path.abspath(__file__))
_PARENT = os.path.join(_DIR, "..")
for _p in (_DIR, _PARENT):
    if _p not in sys.path:
        sys.path.insert(0, _p)

import common  # noqa: E402

NAME, CATEGORY, BUDGET = "ice_shards", "props", 110


def build(model):
    # Low snow/ice base mound
    base = common.make_icosphere(subdiv=1, jitter=0.20, seed=42,
                                 radius=(0.32, 0.28, 0.10), loc=(0.0, 0.0, 0.02), color="snow_white")
    model.add(base)

    # Five angled 4-sided crystal spikes
    spikes = [
        (0.11, 0.015, 0.72, (0.00, 0.00, -0.03), (0.08, -0.06, 0.3), "ice_blue"),
        (0.08, 0.012, 0.56, (0.14, -0.08, -0.02), (-0.22, 0.28, 0.9), "ice_blue"),
        (0.075, 0.010, 0.48, (-0.13, 0.09, -0.02), (0.26, -0.22, 1.7), "snow_shadow"),
        (0.065, 0.010, 0.38, (-0.08, -0.13, -0.02), (-0.32, -0.18, 2.4), "ice_blue"),
        (0.055, 0.008, 0.30, (0.08, 0.13, -0.02), (0.30, 0.20, 0.5), "snow_white"),
    ]
    for r0, r1, h, loc, rot, color in spikes:
        shard = common.make_frustum(r0, r1, 0.0, h, segments=4, color=color)
        common.transform(shard, loc=loc, rot=rot)
        model.add(shard)


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=(0.0, 0.0, 0.0)) else 1


if __name__ == "__main__":
    sys.exit(main())
