"""boulder_big: massive rounded alpine boulder (~5.4 x 4.4 x 3.3 m) with snow-dusted top (<= 300 tris)."""
import os
import sys

_DIR = os.path.dirname(os.path.abspath(__file__))
_PARENT = os.path.join(_DIR, "..")
for _p in (_DIR, _PARENT):
    if _p not in sys.path:
        sys.path.insert(0, _p)

import common  # noqa: E402

NAME, CATEGORY, BUDGET = "boulder_big", "props", 300
SINK_Z = -0.05


def _seat_and_paint_boulder(bm, snow_z=0.68, shadow_z=0.55):
    for v in bm.verts:
        if v.co.z < SINK_Z:
            v.co.z = SINK_Z
    bm.normal_update()
    common.paint_faces(bm, [f for f in bm.faces if f.normal.z < 0.08], "rock_dark")
    common.paint_faces(bm, [f for f in bm.faces if f.normal.z > 0.35], "rock_light")
    common.paint_faces(bm, [f for f in bm.faces if shadow_z < f.normal.z <= snow_z], "snow_shadow")
    common.paint_faces(bm, [f for f in bm.faces if f.normal.z > snow_z], "snow_white")
    return bm


def build(model):
    # Main rounded alpine boulder mass (80 tris)
    main_rock = common.make_icosphere(
        subdiv=2, jitter=0.18, seed=71,
        radius=(2.35, 2.05, 1.78), loc=(-0.20, 0.10, 1.50), rot=(0.04, -0.05, 0.32),
        color="rock_mid",
    )
    model.add(_seat_and_paint_boulder(main_rock))

    # Fused shoulder lobe giving a weighty natural silhouette (80 tris)
    shoulder = common.make_icosphere(
        subdiv=2, jitter=0.20, seed=79,
        radius=(1.45, 1.35, 1.15), loc=(1.15, -0.65, 0.92), rot=(0.08, 0.14, 1.05),
        color="rock_mid",
    )
    model.add(_seat_and_paint_boulder(shoulder, snow_z=0.72, shadow_z=0.58))

    # Opposite lower buttress stone (20 tris)
    buttress = common.make_icosphere(
        subdiv=1, jitter=0.20, seed=83,
        radius=(1.05, 0.95, 0.78), loc=(-1.45, 0.75, 0.60), rot=(-0.08, 0.10, -0.55),
        color="rock_mid",
    )
    model.add(_seat_and_paint_boulder(buttress, snow_z=0.76, shadow_z=0.64))

    # Three smaller scree rocks nestled around the base (60 tris)
    scree = [
        ((1.85, 0.85, 0.32), (0.55, 0.48, 0.40), (0.1, 0.1, 0.6)),
        ((-1.95, -0.78, 0.28), (0.48, 0.42, 0.36), (-0.1, 0.2, 1.8)),
        ((0.25, -1.85, 0.24), (0.44, 0.38, 0.30), (0.2, -0.1, 2.4)),
    ]
    for k, (loc, rad, rot) in enumerate(scree):
        stone = common.make_icosphere(
            subdiv=1, jitter=0.22, seed=97 + 11 * k,
            radius=rad, loc=loc, rot=rot, color="rock_dark",
        )
        for v in stone.verts:
            if v.co.z < SINK_Z:
                v.co.z = SINK_Z
        stone.normal_update()
        common.paint_faces(stone, [f for f in stone.faces if f.normal.z > 0.45], "rock_mid")
        common.paint_faces(stone, [f for f in stone.faces if f.normal.z > 0.80], "snow_white")
        model.add(stone)

    # Convex hull colliders covering the main boulder and shoulder lobe
    hull_main = common.make_icosphere(
        subdiv=1, jitter=0.16, seed=71,
        radius=(2.32, 2.02, 1.74), loc=(-0.20, 0.10, 1.48), rot=(0.04, -0.05, 0.32),
    )
    hull_side = common.make_icosphere(
        subdiv=1, jitter=0.16, seed=79,
        radius=(1.42, 1.32, 1.12), loc=(1.15, -0.65, 0.90), rot=(0.08, 0.14, 1.05),
    )
    for hull in (hull_main, hull_side):
        for v in hull.verts:
            if v.co.z < SINK_Z:
                v.co.z = SINK_Z
    model.add_collider("main", hull_main)
    model.add_collider("shoulder", hull_side)


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=(0.0, 0.0, 0.0)) else 1


if __name__ == "__main__":
    sys.exit(main())
