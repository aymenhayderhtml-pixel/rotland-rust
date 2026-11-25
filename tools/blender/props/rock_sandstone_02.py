"""rock_sandstone_02: taller layered sandstone butte/hoodoo (~2.0 m wide, ~2.6 m tall)."""
import math
import os
import random
import sys

import bmesh

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402

NAME, CATEGORY, BUDGET = "rock_sandstone_02", "props", 200


def _layered_slab(rx, ry, z0, z1, center, seed, segments=8, lower_dark=True, mid_scale=0.96, top_scale=0.94):
    """Flat-topped irregular sedimentary slab with distinct sandstone and sandstone_dark strata."""
    rng = random.Random(seed)
    bm = common.new_bm()
    cx, cy = center
    rot = rng.uniform(0.0, 2.0 * math.pi)
    z_mid = z0 + (z1 - z0) * 0.48

    radii_noise = [rng.uniform(0.84, 1.15) for _ in range(segments)]
    mid_jitter = [rng.uniform(0.93, 1.04) for _ in range(segments)]

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
    r_mid = make_ring(z_mid, mid_scale, mid_jitter)
    r_top = make_ring(z1, top_scale)

    bot_color = "sandstone_dark" if lower_dark else "sandstone"
    upper_color = "sandstone" if lower_dark else "sandstone_dark"

    for i in range(segments):
        j = (i + 1) % segments
        f_low = bm.faces.new([r_bot[i], r_bot[j], r_mid[j], r_mid[i]])
        common.paint_faces(bm, [f_low], bot_color)
        f_hi = bm.faces.new([r_mid[i], r_mid[j], r_top[j], r_top[i]])
        common.paint_faces(bm, [f_hi], upper_color)

    common.paint_faces(bm, [bm.faces.new(list(reversed(r_bot)))], "sandstone_dark")
    common.paint_faces(bm, [bm.faces.new(r_top)], "sandstone")

    bmesh.ops.recalc_face_normals(bm, faces=list(bm.faces))
    return bm


def build(model):
    # Tall weathered hoodoo stack: wide base, slightly pinched middle layer, overhanging flat cap
    model.add(_layered_slab(1.02, 0.86, -0.08, 0.84, (0.00, 0.00), seed=201,
                            segments=8, lower_dark=True, mid_scale=0.95, top_scale=0.88))
    model.add(_layered_slab(0.76, 0.66, 0.78, 1.68, (0.14, -0.08), seed=202,
                            segments=8, lower_dark=False, mid_scale=0.94, top_scale=0.98))
    model.add(_layered_slab(0.92, 0.78, 1.62, 2.56, (0.18, -0.10), seed=203,
                            segments=8, lower_dark=True, mid_scale=1.02, top_scale=0.96))

    # Low stepped side shelf at the base
    model.add(_layered_slab(0.56, 0.44, -0.08, 0.54, (-0.64, 0.34), seed=204,
                            segments=6, lower_dark=True, mid_scale=0.95, top_scale=0.90))

    model.add_collider("hull", common.make_frustum(0.98, 0.88, -0.08, 2.56, segments=6, color="sandstone"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=(0.0, 0.0, 0.0)) else 1


if __name__ == "__main__":
    sys.exit(main())
