"""snow_drift: low, long wind-sculpted snow mound (4 x 2 x 0.6 m), no collider (<= 120 tris)."""
import math
import os
import sys

import bmesh

_DIR = os.path.dirname(os.path.abspath(__file__))
_PARENT = os.path.join(_DIR, "..")
for _p in (_DIR, _PARENT):
    if _p not in sys.path:
        sys.path.insert(0, _p)

import common  # noqa: E402

NAME, CATEGORY, BUDGET = "snow_drift", "props", 120
SINK_Z = -0.05
HALF_LEN = 2.0   # 4.0 m along X
HALF_WID = 1.0   # 2.0 m along Y
PEAK_H = 0.665   # peaks at ~0.60 m above ground (from SINK_Z = -0.05)


def build(model):
    cols, rows = 8, 5  # 7 x 4 = 28 top quads (56 tris) + 22-vert closed base (20 tris) = 76 tris
    bm = common.new_bm()
    grid = []

    for iy in range(rows):
        v = -1.0 + 2.0 * iy / (rows - 1)
        row_verts = []
        for ix in range(cols):
            u = -1.0 + 2.0 * ix / (cols - 1)
            on_edge = ix in (0, cols - 1) or iy in (0, rows - 1)
            # Elliptical disc mapping keeps the perimeter on an exact 4 x 2 m oval
            x = HALF_LEN * u * math.sqrt(max(0.0, 1.0 - 0.5 * v * v))
            y = HALF_WID * v * math.sqrt(max(0.0, 1.0 - 0.5 * u * u))
            if on_edge:
                z = SINK_Z
            else:
                r2 = (x / HALF_LEN) ** 2 + (y / HALF_WID) ** 2
                falloff = max(0.0, 1.0 - r2)
                # Shift the crest slightly toward +Y with a gentle wind-blown S-curve
                y += (0.14 + 0.10 * math.sin(u * math.pi)) * falloff
                z = SINK_Z + PEAK_H * (falloff ** 1.15)
            row_verts.append(bm.verts.new((x, y, z)))
        grid.append(row_verts)

    for iy in range(rows - 1):
        for ix in range(cols - 1):
            bm.faces.new([
                grid[iy][ix],
                grid[iy][ix + 1],
                grid[iy + 1][ix + 1],
                grid[iy + 1][ix],
            ])

    # Close the flat underground base so normals are guaranteed outward
    perimeter = (
        [grid[0][ix] for ix in range(cols)]
        + [grid[iy][cols - 1] for iy in range(1, rows)]
        + [grid[rows - 1][ix] for ix in range(cols - 2, -1, -1)]
        + [grid[iy][0] for iy in range(rows - 2, 0, -1)]
    )
    bm.faces.new(list(reversed(perimeter)))

    bmesh.ops.recalc_face_normals(bm, faces=list(bm.faces))
    bm.normal_update()

    common.paint(bm, "snow_white")
    # Paint the steeper leeward (+Y) side and base in snow_shadow for clear volume readability
    shaded = [f for f in bm.faces if f.normal.z < -0.5 or (f.normal.z > 0.0 and f.normal.y > 0.18)]
    common.paint_faces(bm, shaded, "snow_shadow")
    model.add(bm)


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=(0.0, 0.0, 0.0)) else 1


if __name__ == "__main__":
    sys.exit(main())
