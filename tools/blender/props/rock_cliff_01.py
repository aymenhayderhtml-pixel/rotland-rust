"""rock_cliff_01: tall angular cliff slab (8 m high x 6 m wide x 3 m deep) with strata ledges and snow caps (<= 350 tris)."""
import math
import os
import random
import sys

import bmesh

_DIR = os.path.dirname(os.path.abspath(__file__))
_PARENT = os.path.join(_DIR, "..")
for _p in (_DIR, _PARENT):
    if _p not in sys.path:
        sys.path.insert(0, _p)

import common  # noqa: E402

NAME, CATEGORY, BUDGET = "rock_cliff_01", "props", 350
SINK_Z = -0.05


def _strata_slab(rings, segs, seed, center=(0.0, 0.0), rot_z=0.0, jitter=0.11,
                 band_colors=None, top_color="snow_white", snow_ledges=True):
    """Stacked angular rock rings forming a faceted cliff slab with horizontal ledge stripes."""
    rng = random.Random(seed)
    bm = common.new_bm()
    cx, cy = center
    ring_verts = []

    for r_idx, (z, rx, ry, sx, sy) in enumerate(rings):
        row = []
        for i in range(segs):
            a = rot_z + 2.0 * math.pi * i / segs
            radial = 1.0 + jitter * rng.uniform(-1.0, 1.0)
            x = cx + sx + math.cos(a) * rx * radial
            y = cy + sy + math.sin(a) * ry * radial
            vz = z if r_idx == 0 else z + 0.07 * rng.uniform(-1.0, 1.0)
            row.append(bm.verts.new((x, y, vz)))
        ring_verts.append(row)

    band_faces = []
    for b in range(len(rings) - 1):
        r0, r1 = ring_verts[b], ring_verts[b + 1]
        faces = []
        for i in range(segs):
            j = (i + 1) % segs
            faces.append(bm.faces.new([r0[i], r0[j], r1[j], r1[i]]))
        band_faces.append(faces)

    bot_face = bm.faces.new(list(reversed(ring_verts[0])))
    top_face = bm.faces.new(ring_verts[-1])

    bmesh.ops.recalc_face_normals(bm, faces=list(bm.faces))
    bm.normal_update()

    common.paint_faces(bm, [bot_face], "rock_dark")
    common.paint_faces(bm, [top_face], top_color)
    for b, faces in enumerate(band_faces):
        color = band_colors[b] if band_colors else "rock_mid"
        common.paint_faces(bm, faces, color)
        if snow_ledges:
            for f in faces:
                cz = f.calc_center_median().z
                if f.normal.z > 0.48 and cz > 2.2:
                    common.paint_faces(bm, [f], "snow_white")
                elif f.normal.z > 0.28 and cz > 3.5:
                    common.paint_faces(bm, [f], "snow_shadow")
    return bm


def _snow_crust(rx_bot, ry_bot, rx_top, ry_top, z0, z1, segs, seed, center=(0.0, 0.0), rot_z=0.0):
    """Crisp faceted snow slab capping a cliff summit or upper ledge."""
    rings = [
        (z0, rx_bot, ry_bot, 0.0, 0.0),
        (z1, rx_top, ry_top, 0.0, 0.0),
    ]
    bm = _strata_slab(
        rings, segs=segs, seed=seed, center=center, rot_z=rot_z, jitter=0.08,
        band_colors=["snow_shadow"], top_color="snow_white", snow_ledges=False,
    )
    common.paint_faces(bm, [f for f in bm.faces if f.normal.z > 0.35], "snow_white")
    return bm


def build(model):
    # Main 8 m tall cliff slab (10-sided, 7 rings -> 136 tris)
    main_rings = [
        (SINK_Z, 2.95, 1.48, 0.00, 0.00),
        (1.65,   2.80, 1.40, -0.05, 0.04),
        (2.15,   2.48, 1.24, -0.08, 0.06),
        (4.15,   2.40, 1.20, -0.10, 0.08),
        (4.65,   2.02, 1.02, -0.14, 0.06),
        (6.75,   1.92, 0.96, -0.18, 0.04),
        (7.80,   1.42, 0.72, -0.22, 0.02),
    ]
    main_bands = ["rock_dark", "rock_light", "rock_mid", "rock_light", "rock_dark", "rock_mid"]
    model.add(_strata_slab(main_rings, segs=10, seed=601, rot_z=0.18, jitter=0.10, band_colors=main_bands))

    # Summit snow cap reaching 8.05 m (8-sided -> 28 tris)
    model.add(_snow_crust(1.50, 0.78, 1.15, 0.56, 7.66, 8.05, segs=8, seed=607, center=(-0.22, 0.02), rot_z=0.18))

    # Right stepped shoulder buttress creating a mid-cliff snow ledge (8-sided, 4 rings -> 60 tris)
    shoulder_rings = [
        (SINK_Z, 1.38, 0.96, 0.00, 0.00),
        (1.85,   1.30, 0.90, 0.04, -0.04),
        (3.55,   1.15, 0.80, 0.02, -0.02),
        (4.65,   0.92, 0.64, -0.04, 0.02),
    ]
    shoulder_bands = ["rock_dark", "rock_mid", "rock_light"]
    model.add(_strata_slab(shoulder_rings, segs=8, seed=613, center=(1.55, -0.42),
                           rot_z=-0.25, jitter=0.10, band_colors=shoulder_bands))
    # Shoulder ledge snow crust (6-sided -> 20 tris)
    model.add(_snow_crust(0.98, 0.68, 0.74, 0.50, 4.54, 4.82, segs=6, seed=619, center=(1.51, -0.40), rot_z=-0.25))

    # Left lower rocky outcrop buttress (7-sided, 3 rings -> 38 tris)
    outcrop_rings = [
        (SINK_Z, 1.18, 0.88, 0.00, 0.00),
        (1.55,   1.08, 0.80, -0.04, 0.04),
        (2.95,   0.82, 0.62, 0.02, 0.02),
    ]
    model.add(_strata_slab(outcrop_rings, segs=7, seed=631, center=(-1.75, 0.42),
                           rot_z=0.40, jitter=0.11, band_colors=["rock_dark", "rock_mid"]))

    # Colliders for main cliff slab and right shoulder
    col_main = common.make_frustum(2.75, 1.45, SINK_Z, 7.90, segments=6, color="rock_dark")
    common.transform(col_main, loc=(-0.10, 0.04, 0.0), scale=(1.0, 0.50, 1.0))
    model.add_collider("main", col_main)

    col_shoulder = common.make_frustum(1.30, 0.90, SINK_Z, 4.70, segments=6, center=(1.55, -0.42), color="rock_dark")
    common.transform(col_shoulder, scale=(1.0, 0.70, 1.0))
    model.add_collider("shoulder", col_shoulder)


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=(0.0, 0.0, 0.0)) else 1


if __name__ == "__main__":
    sys.exit(main())
