"""camp_wall_metal: 4 m section of 3 m corrugated metal sheets on steel posts.

Features:
- Corrugated metal panels on steel I-beam posts.
- Weathered rusty patches and scrap metal repair plates.
- Angled outrigger arms with 3 strands of barbed top wire.
- Tiling along X (-2.0 to +2.0), centered on X, thickness along Y.
- Colliders: solid metal wall.
"""
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, HERE)

import bmesh  # noqa: E402
import common  # noqa: E402
import kit  # noqa: E402

NAME, CATEGORY, BUDGET = "camp_wall_metal", "buildings", 1200
LENGTH = 4.0
HALF_L = LENGTH / 2.0  # 2.0 m
HEIGHT = 3.0


def _build_corrugated_sheet(lo_x, hi_x, z0, z1, waves=20):
    """Build a corrugated metal sheet with sine/zigzag accordion profile."""
    bm = common.new_bm()
    n_pts = waves * 2 + 1
    dx = (hi_x - lo_x) / (n_pts - 1)

    # 4 vertical rows of vertices for 3 horizontal panels
    z_rows = [z0, z0 + (z1 - z0) * 0.35, z0 + (z1 - z0) * 0.70, z1]
    verts_grid = []

    for r, z in enumerate(z_rows):
        row = []
        for i in range(n_pts):
            x = lo_x + i * dx
            # alternating amplitude between -0.015 and +0.02
            y = 0.02 if (i % 2 == 1) else -0.015
            row.append(bm.verts.new((x, y, z)))
        verts_grid.append(row)

    bm.verts.ensure_lookup_table()

    # Create front faces
    for r in range(len(z_rows) - 1):
        for i in range(n_pts - 1):
            v0 = verts_grid[r][i]
            v1 = verts_grid[r][i + 1]
            v2 = verts_grid[r + 1][i + 1]
            v3 = verts_grid[r + 1][i]
            face = bm.faces.new((v0, v1, v2, v3))
            # Slightly varied metal tones
            col = "metal" if (r + i) % 5 != 0 else "rock_mid"
            common.paint_faces(bm, [face], col)

    bmesh.ops.recalc_face_normals(bm, faces=list(bm.faces))
    return bm


def build(model):
    k = kit.Kit(model)

    # 1. Main corrugated sheet spanning -HALF_L to +HALF_L
    sheet_bm = _build_corrugated_sheet(-HALF_L, HALF_L, 0.05, HEIGHT, waves=20)
    model.add(sheet_bm)

    # 2. Structural steel posts at x = -2.0, 0.0, +2.0
    post_h = 3.12
    # Center post (x = 0.0)
    k.box((-0.07, -0.08, 0.0), (0.07, 0.08, post_h), "gun_black")
    k.box((-0.11, -0.11, 0.0), (0.11, 0.11, 0.03), "rock_dark")  # base plate

    # Left post (x = -2.0, half on this section)
    k.box((-HALF_L, -0.08, 0.0), (-HALF_L + 0.07, 0.08, post_h), "gun_black")
    k.box((-HALF_L, -0.11, 0.0), (-HALF_L + 0.11, 0.11, 0.03), "rock_dark")

    # Right post (x = +2.0, half on this section)
    k.box((HALF_L - 0.07, -0.08, 0.0), (HALF_L, 0.08, post_h), "gun_black")
    k.box((HALF_L - 0.11, -0.11, 0.0), (HALF_L, 0.11, 0.03), "rock_dark")

    # 3. Horizontal steel cross-girts on the inside face (-Y)
    for gz in (0.45, 1.50, 2.65):
        k.box((-HALF_L, -0.07, gz), (HALF_L, -0.03, gz + 0.08), "gun_black")

    # 4. Rusty patches and scrap metal repair plates
    # Patch 1: lower left (rust)
    k.box((-1.45, 0.022, 0.65), (-0.75, 0.038, 1.55), "rust")
    # Rivet trim
    k.box((-1.47, 0.020, 0.63), (-1.43, 0.040, 1.57), "gun_black")
    k.box((-0.77, 0.020, 0.63), (-0.73, 0.040, 1.57), "gun_black")

    # Patch 2: upper right (rust)
    k.box((0.45, 0.022, 1.65), (1.15, 0.038, 2.45), "rust")
    k.box((0.43, 0.020, 1.63), (0.47, 0.040, 2.47), "gun_black")

    # Patch 3: upper center (weathered red paint scrap)
    k.box((-0.45, 0.024, 2.05), (0.35, 0.040, 2.85), "red_paint")
    k.box((-0.47, 0.022, 2.03), (-0.43, 0.042, 2.87), "gun_black")

    # Patch 4: lower right (dirt/corrosion patch)
    k.box((1.20, 0.022, 0.35), (1.80, 0.036, 1.15), "dirt")

    # 5. Barbed top wire and angled outriggers
    # Angled arms at each post extending upward and outward (+Y)
    for px in (-HALF_L + 0.035, 0.0, HALF_L - 0.035):
        # Angled steel outrigger bracket (from z=3.05, y=0.0 to z=3.42, y=0.22)
        k.box((px - 0.02, 0.0, 3.05), (px + 0.02, 0.06, 3.18), "gun_black")
        k.box((px - 0.02, 0.05, 3.16), (px + 0.02, 0.14, 3.30), "gun_black")
        k.box((px - 0.02, 0.12, 3.28), (px + 0.02, 0.22, 3.42), "gun_black")

    # 3 strands of taut steel wire across the 4m span
    wire_points = [
        (0.06, 3.18),
        (0.14, 3.30),
        (0.22, 3.42),
    ]
    for wy, wz in wire_points:
        # Long wire strand
        k.box((-HALF_L, wy - 0.007, wz - 0.007), (HALF_L, wy + 0.007, wz + 0.007), "metal")
        # Barbs every 0.6 m
        for b_i in range(7):
            bx = -HALF_L + 0.25 + b_i * 0.60
            # Small barb point
            k.box((bx - 0.015, wy - 0.025, wz - 0.02), (bx + 0.015, wy + 0.025, wz + 0.02), "gun_black")

    # 6. Inside diagonal kicker braces on -Y side
    for px in (-1.95, 0.0, 1.95):
        k.box((px - 0.03, -0.40, 0.0), (px + 0.03, -0.05, 1.40), "gun_black")

    # 7. Colliders
    k.collider((-HALF_L, -0.15, 0.0), (HALF_L, 0.15, HEIGHT), base="metal_wall")


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, view=kit.FRONT_VIEW) else 1


if __name__ == "__main__":
    sys.exit(main())
