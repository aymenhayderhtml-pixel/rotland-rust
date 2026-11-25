"""rock_sandstone_01: wide flat-topped layered sandstone outcrop (~2.7 m wide, ~1.8 m tall)."""
import math
import os
import random
import sys

import bmesh

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402

NAME, CATEGORY, BUDGET = "rock_sandstone_01", "props", 200


def _layered_slab(rx, ry, z0, z1, center, seed, segments=8, lower_dark=True, top_inset=0.92):
    """Flat-topped irregular sedimentary slab with a dark sandstone horizontal band."""
    rng = random.Random(seed)
    bm = common.new_bm()
    cx, cy = center
    rot = rng.uniform(0.0, 2.0 * math.pi)
    z_mid = z0 + (z1 - z0) * 0.46

    radii_noise = [rng.uniform(0.84, 1.14) for _ in range(segments)]
    mid_pinch = [rng.uniform(0.92, 1.03) for _ in range(segments)]

    def make_ring(z_val, scale_xy, extra=None):
        verts = []
        for i in range(segments):
            angle = rot + 2.0 * math.pi * i / segments
            n = radii_noise[i] * (extra[i] if extra else 1.0)
            verts.append(bm.verts.new((
                cx + math.cos(angle) * rx * scale_xy * n,
                cy + math.sin(angle) * ry * scale_xy * n,
                z_val,
            )))
        return verts

    r_bot = make_ring(z0, 1.0)
    r_mid = make_ring(z_mid, 0.97, mid_pinch)
    r_top = make_ring(z1, top_inset)

    bot_color = "sandstone_dark" if lower_dark else "sandstone"
    upper_color = "sandstone" if lower_dark else "sandstone_dark"

    for i in range(segments):
        j = (i + 1) % segments
        f_low = bm.faces.new([r_bot[i], r_bot[j], r_mid[j], r_mid[i]])
        common.paint_faces(bm, [f_low], bot_color)
        f_hi = bm.faces.new([r_mid[i], r_mid[j], r_top[j], r_top[i]])
        common.paint_faces(bm, [f_hi], upper_color)

    f_bot = bm.faces.new(list(reversed(r_bot)))
    common.paint_faces(bm, [f_bot], "sandstone_dark")
    f_top = bm.faces.new(r_top)
    common.paint_faces(bm, [f_top], "sandstone")

    bmesh.ops.recalc_face_normals(bm, faces=list(bm.faces))
    return bm


def _talus_chunk(rx, ry, z0, z1, center, seed):
    """Small 6-sided angular sandstone chunk at the outcrop base."""
    rng = random.Random(seed)
    bm = common.new_bm()
    cx, cy = center
    rot = rng.uniform(0.0, 2.0 * math.pi)
    noise = [rng.uniform(0.82, 1.16) for _ in range(6)]
    r_bot = [bm.verts.new((cx + math.cos(rot + math.pi * i / 3.0) * rx * noise[i],
                           cy + math.sin(rot + math.pi * i / 3.0) * ry * noise[i], z0))
             for i in range(6)]
    r_top = [bm.verts.new((cx + math.cos(rot + math.pi * i / 3.0) * rx * 0.82 * noise[i],
                           cy + math.sin(rot + math.pi * i / 3.0) * ry * 0.82 * noise[i], z1))
             for i in range(6)]
    for i in range(6):
        j = (i + 1) % 6
        f = bm.faces.new([r_bot[i], r_bot[j], r_top[j], r_top[i]])
        common.paint_faces(bm, [f], "sandstone_dark" if i % 2 == 0 else "sandstone")
    common.paint_faces(bm, [bm.faces.new(list(reversed(r_bot)))], "sandstone_dark")
    common.paint_faces(bm, [bm.faces.new(r_top)], "sandstone")
    bmesh.ops.recalc_face_normals(bm, faces=list(bm.faces))
    return bm


def build(model):
    # Three stacked, slightly offset flat-topped slabs (sunk 8 cm into terrain)
    model.add(_layered_slab(1.32, 1.04, -0.08, 0.68, (0.04, -0.02), seed=101, lower_dark=True, top_inset=0.94))
    model.add(_layered_slab(1.04, 0.84, 0.62, 1.32, (-0.16, 0.08), seed=102, lower_dark=False, top_inset=0.93))
    model.add(_layered_slab(0.86, 0.70, 1.26, 1.82, (-0.22, 0.12), seed=103, lower_dark=True, top_inset=0.95))

    # Two small fallen sandstone chunks near the base
    model.add(_talus_chunk(0.36, 0.28, -0.06, 0.28, (1.05, 0.42), seed=104))
    model.add(_talus_chunk(0.30, 0.24, -0.06, 0.22, (-0.98, -0.58), seed=105))

    model.add_collider("hull", common.make_frustum(1.26, 0.85, -0.08, 1.82, segments=6, color="sandstone"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=(0.0, 0.0, 0.0)) else 1


if __name__ == "__main__":
    sys.exit(main())
