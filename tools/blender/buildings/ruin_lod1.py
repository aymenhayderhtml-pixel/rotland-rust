"""ruin_lod1: low-detail exterior shell of ruin (budget 500 tris).

Same footprint (13 x 9 m on 16 x 12 m lot, with >=1m margin), same stepped jagged wall heights,
collapsed upper floor slab, main rubble mounds, and open breaches.
No small debris blocks, mortar lines, or rebar details.
"""
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, HERE)

import bmesh  # noqa: E402
from mathutils import Matrix, Vector  # noqa: E402

import common  # noqa: E402
import kit  # noqa: E402

NAME, CATEGORY, BUDGET = "ruin_lod1", "buildings", 500
F = kit.FLOOR_TOP


def build(model):
    k = kit.Kit(model)

    # 1. Floor
    k.box((-6.5, -4.5, 0.0), (6.5, 4.5, F), "dirt")

    # 2. Back Wall (Y = -4.5 to -4.25) - simplified stepped blocks
    k.box((-6.5, -4.5, F), (-5.0, -4.25, 5.0), "red_paint")
    k.box((-5.0, -4.5, F), (-3.0, -4.25, 4.0), "red_paint")
    k.box((-3.0, -4.5, F), (-1.0, -4.25, 2.6), "red_paint")
    k.box((-1.0, -4.5, F), (1.5, -4.25, 1.6), "red_paint")
    k.box((1.5, -4.5, F), (3.5, -4.25, 1.0), "red_paint")
    k.box((3.5, -4.5, F), (6.5, -4.25, 2.8), "red_paint")

    # 3. Left Wall (X = -6.5 to -6.25) - stepped blocks
    k.box((-6.5, -4.25, F), (-6.25, -2.5, 4.4), "red_paint")
    k.box((-6.5, -2.5, F), (-6.25, 0.5, 2.2), "red_paint")
    k.box((-6.5, 0.5, F), (-6.25, 4.5, 3.0), "red_paint")

    # 4. Right Wall (X = 6.25 to 6.5) - with side breach open
    k.box((6.25, -4.25, F), (6.5, -1.5, 2.8), "red_paint")
    # Breach Y = -1.5 to 1.2 is OPEN
    k.box((6.25, 1.2, F), (6.5, 4.5, 2.0), "red_paint")

    # 5. Front Wall (Y = 4.25 to 4.5) - with main breach open
    k.box((-6.25, 4.25, F), (-4.5, 4.5, 3.0), "red_paint")
    k.box((-4.5, 4.25, F), (-1.2, 4.5, 1.2), "red_paint")
    # Main breach X = -1.2 to 4.2 is OPEN
    k.box((4.2, 4.25, F), (6.25, 4.5, 1.8), "red_paint")

    # 6. Upper Floor Slab
    # Horizontal section in back-left
    slab_z = 3.0
    k.box((-6.25, -4.25, slab_z - 0.18), (-2.8, -1.8, slab_z), "rock_mid")

    # Hanging tilted slab
    start_pt = (-2.8, -3.6, slab_z)
    end_pt = (0.5, -1.6, 1.1)
    dx = end_pt[0] - start_pt[0]
    dy = end_pt[1] - start_pt[1]
    dz = end_pt[2] - start_pt[2]
    length = math.sqrt(dx * dx + dy * dy + dz * dz)
    v_dir = Vector((dx, dy, dz)).normalized()
    v_span = Vector((0.0, 1.0, 0.0))
    v_up = v_dir.cross(v_span).normalized()
    if v_up.z < 0:
        v_up = -v_up

    mid_pt = Vector(((start_pt[0] + end_pt[0]) / 2, (start_pt[1] + end_pt[1]) / 2, (start_pt[2] + end_pt[2]) / 2))
    tilted_bm = common.make_box((length, 2.0, 0.18), color="rock_mid")
    rot_mat = Matrix((v_dir, v_span, v_up)).transposed().to_4x4()
    mat = Matrix.Translation(mid_pt) @ rot_mat
    bmesh.ops.transform(tilted_bm, matrix=mat, verts=list(tilted_bm.verts))
    tilted_bm.normal_update()
    k.model.add(tilted_bm)

    # Broken pillar (matches LOD0 at (0.8, -3.2))
    k.box((0.8, -3.2, F), (1.2, -2.8, 1.8), "rock_mid")

    # 7. Simplified Rubble Piles (within X: [-7, 7], Y: [-5, 5])
    # Inside main rubble
    k.box((-0.7, -2.4, F), (1.7, 0.0, 0.7), "rock_dark")
    k.box((-0.2, -1.9, 0.7), (1.2, -0.5, 0.95), "red_paint")
    # Street rubble (stays inside Y = 5.0)
    k.box((0.8, 4.4, F), (2.8, 4.9, 0.35), "dirt")
    # Inside back-right rubble
    k.box((3.2, -4.25, F), (5.2, -2.2, 0.55), "rock_dark")

    # 8. Marker
    k.empty("lot_size", (0.0, 0.0, 0.0), scale=(16.0, 12.0, 1.0))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, view=kit.FRONT_VIEW) else 1


if __name__ == "__main__":
    sys.exit(main())
