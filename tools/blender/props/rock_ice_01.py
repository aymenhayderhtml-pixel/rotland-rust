"""rock_ice_01: angular icy rock (1.8 x 1.5 x 1.35 m) in ice_blue with snow_white top faces (<= 200 tris)."""
import os
import sys

_DIR = os.path.dirname(os.path.abspath(__file__))
_PARENT = os.path.join(_DIR, "..")
for _p in (_DIR, _PARENT):
    if _p not in sys.path:
        sys.path.insert(0, _p)

import common  # noqa: E402

NAME, CATEGORY, BUDGET = "rock_ice_01", "props", 200
SINK_Z = -0.05


def _seat_and_paint_ice(bm, snow_thresh=0.42, shadow_thresh=-0.18):
    for v in bm.verts:
        if v.co.z < SINK_Z:
            v.co.z = SINK_Z
    bm.normal_update()
    common.paint_faces(bm, [f for f in bm.faces if f.normal.z < shadow_thresh], "snow_shadow")
    common.paint_faces(bm, [f for f in bm.faces if f.normal.z > snow_thresh], "snow_white")
    return bm


def build(model):
    # Main angular glacial rock (80 tris)
    body = common.make_icosphere(
        subdiv=2, jitter=0.28, seed=21,
        radius=(0.88, 0.72, 0.72), loc=(-0.10, 0.04, 0.60), rot=(0.08, -0.06, 0.35),
        color="ice_blue",
    )
    model.add(_seat_and_paint_ice(body, snow_thresh=0.42))

    # Secondary crystalline ice shard jutting off the side (20 tris)
    shard = common.make_icosphere(
        subdiv=1, jitter=0.28, seed=29,
        radius=(0.48, 0.38, 0.54), loc=(0.56, -0.24, 0.38), rot=(0.18, 0.32, 0.85),
        color="ice_blue",
    )
    model.add(_seat_and_paint_ice(shard, snow_thresh=0.45))

    # Two small angular ice chunks around the base (40 tris)
    for k, (loc, rad, rot) in enumerate([
        ((-0.72, -0.44, 0.16), (0.28, 0.24, 0.22), (0.1, 0.2, 0.5)),
        ((0.28, 0.58, 0.14), (0.24, 0.22, 0.20), (-0.1, 0.15, 1.4)),
    ]):
        chunk = common.make_icosphere(
            subdiv=1, jitter=0.24, seed=41 + 7 * k,
            radius=rad, loc=loc, rot=rot, color="ice_blue",
        )
        model.add(_seat_and_paint_ice(chunk, snow_thresh=0.50))

    # Convex hull collider matching the main ice mass
    hull = common.make_icosphere(
        subdiv=1, jitter=0.24, seed=21,
        radius=(0.95, 0.76, 0.72), loc=(0.06, -0.02, 0.58), rot=(0.08, -0.06, 0.35),
    )
    for v in hull.verts:
        if v.co.z < SINK_Z:
            v.co.z = SINK_Z
    model.add_collider("hull", hull)


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=(0.0, 0.0, 0.0)) else 1


if __name__ == "__main__":
    sys.exit(main())
