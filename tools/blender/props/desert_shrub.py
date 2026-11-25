"""desert_shrub: low dry tumbleweed-like desert shrub (~0.8 m wide), dry_brush + sandstone_dark."""
import math
import os
import random
import sys

from mathutils import Euler, Vector

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402
import veg  # noqa: E402

NAME, CATEGORY, BUDGET = "desert_shrub", "props", 150


def build(model):
    rng = random.Random(3106)

    # Low woody root crown sunk 5 cm into the sand
    model.add(common.make_frustum(0.09, 0.05, -0.05, 0.12, segments=5, color="sandstone_dark"))

    # 6 main crooked branches radiating outward + 6 forked outer twigs
    branch_specs = [
        (0.15, 0.95, 0.42),
        (1.18, 0.85, 0.46),
        (2.22, 1.02, 0.40),
        (3.30, 0.90, 0.45),
        (4.35, 1.00, 0.43),
        (5.40, 0.88, 0.44),
    ]
    for i, (yaw, tilt, length) in enumerate(branch_specs):
        start = Vector((math.cos(yaw) * 0.03, math.sin(yaw) * 0.03, 0.04))
        end = start + Euler((0.0, tilt, yaw), "XYZ").to_matrix() @ Vector((0.0, 0.0, length))
        color = "sandstone_dark" if i % 2 == 0 else "dry_brush"
        model.add(veg.limb(start, end, 0.028, 0.011, color, segs=3))

        fork_base = start.lerp(end, 0.55)
        fork_yaw = yaw + (0.55 if i % 2 == 0 else -0.55) + rng.uniform(-0.15, 0.15)
        fork_tilt = max(0.35, tilt - 0.35)
        fork_end = fork_base + Euler((0.0, fork_tilt, fork_yaw), "XYZ").to_matrix() @ Vector((0.0, 0.0, length * 0.55))
        model.add(veg.limb(fork_base, fork_end, 0.015, 0.0, "dry_brush", segs=3))

    # Two angular dry brush clusters nestled in the branches for a tumbleweed silhouette
    for idx, (loc, rad, rot) in enumerate([
        ((0.05, -0.03, 0.24), (0.31, 0.27, 0.17), (0.1, -0.15, 0.4)),
        ((-0.07, 0.05, 0.28), (0.25, 0.29, 0.16), (-0.12, 0.18, 1.9)),
    ]):
        cluster = common.make_icosphere(
            subdiv=1, jitter=0.30, seed=40 + idx, radius=rad, loc=loc, rot=rot, color="dry_brush"
        )
        common.paint_faces(cluster, [f for f in cluster.faces if f.normal.z < -0.35], "sandstone_dark")
        model.add(cluster)


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=(0.0, 0.0, 0.0)) else 1


if __name__ == "__main__":
    sys.exit(main())
