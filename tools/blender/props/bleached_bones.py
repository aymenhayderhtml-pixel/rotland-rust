"""bleached_bones: sun-bleached desert animal bone scatter (~0.65 m across, <= 140 tris). No collider."""
import math
import os
import sys

_DIR = os.path.dirname(os.path.abspath(__file__))
_PARENT = os.path.join(_DIR, "..")
for _p in (_DIR, _PARENT):
    if _p not in sys.path:
        sys.path.insert(0, _p)

import common  # noqa: E402

NAME, CATEGORY, BUDGET = "bleached_bones", "props", 140


def _long_bone(length, radius, loc, rot_z, tilt_x=0.05):
    shaft = common.make_frustum(radius, radius * 0.85, -length * 0.5, length * 0.5, segments=4, color="bone_white")
    common.transform(shaft, rot=(math.pi * 0.5, 0.0, 0.0))
    k1 = common.make_box((radius * 2.4, radius * 1.6, radius * 1.6), loc=(0.0, -length * 0.48, 0.0), color="bone_white")
    k2 = common.make_box((radius * 2.2, radius * 1.5, radius * 1.5), loc=(0.0, length * 0.48, 0.0), color="sand_light")
    bone = common.combine([shaft, k1, k2])
    common.transform(bone, loc=loc, rot=(tilt_x, 0.0, rot_z))
    return bone


def _rib(length, loc, rot):
    # Two-segment angled rib arc
    s1 = common.make_frustum(0.016, 0.013, 0.0, length * 0.55, segments=3, color="bone_white")
    s2 = common.make_frustum(0.013, 0.008, 0.0, length * 0.50, segments=3, color="sand_light")
    common.transform(s2, loc=(0.0, 0.04, length * 0.52), rot=(-0.45, 0.0, 0.0))
    rib = common.combine([s1, s2])
    common.transform(rib, loc=loc, rot=rot)
    return rib


def build(model):
    # Main femur-like bone lying flat
    model.add(_long_bone(0.42, 0.022, loc=(-0.05, 0.04, 0.025), rot_z=0.55, tilt_x=0.04))
    # Smaller broken bone angled across
    model.add(_long_bone(0.28, 0.016, loc=(0.14, -0.08, 0.020), rot_z=-0.85, tilt_x=-0.06))
    # Three curved ribs half-buried in sand
    model.add(_rib(0.28, loc=(-0.16, -0.10, 0.00), rot=(1.25, 0.25, 0.40)))
    model.add(_rib(0.25, loc=(-0.10, -0.14, 0.00), rot=(1.30, 0.15, 0.65)))
    model.add(_rib(0.22, loc=(0.02, 0.16, 0.00), rot=(1.28, -0.20, -1.10)))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=(0.0, 0.0, 0.0)) else 1


if __name__ == "__main__":
    sys.exit(main())
