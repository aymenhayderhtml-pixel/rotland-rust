"""ruin: collapsed 2-floor brick building, 13 x 9 m footprint on a 16 x 12 m lot.

Walls broken at different heights with stepped, jagged tops; no roof.
A section of the 2nd-floor concrete slab hangs suspended at an angle in the back-left.
Rubble piles inside and outside, with open gaps at the front and side for player entry.
"""
import math
import os
import random
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, HERE)

import bmesh  # noqa: E402
from mathutils import Matrix, Vector  # noqa: E402

import common  # noqa: E402
import kit  # noqa: E402

NAME, CATEGORY, BUDGET = "ruin", "buildings", 4000
F = kit.FLOOR_TOP

# Lot: 16 x 12 m (X: [-8, 8], Y: [-6, 6])
# All building geometry must stay >= 1 m inside lot on every side:
# X must stay in [-7.0, 7.0], Y in [-5.0, 5.0]


def _brick_wall_section(k, lo, hi, base="ruin_wall"):
    """Adds a brick wall segment with mortar lines."""
    x0, y0, z0 = lo
    x1, y1, z1 = hi
    k.solid(lo, hi, "red_paint", base=base)
    h = z1 - z0
    if h > 1.0:
        step = 0.55
        z = z0 + step
        while z < z1 - 0.15:
            k.box((x0 - 0.01, y0 - 0.01, z), (x1 + 0.01, y1 + 0.01, z + 0.04), "dirt")
            z += step


def _rubble_pile_natural(k, cx, cy, rx, ry, height, seed=1, blocks=5, base="rubble"):
    """Creates a natural low-poly rubble mound using an icosphere base and scattered debris."""
    cx = max(-6.8 + rx, min(6.8 - rx, cx))
    cy = max(-4.8 + ry, min(4.8 - ry, cy))

    mound = common.make_icosphere(subdiv=1, jitter=0.20, seed=seed,
                                  radius=(rx, ry, height * 0.7),
                                  loc=(cx, cy, F + height * 0.2), color="rock_dark")
    k.model.add(mound)
    k.collider((cx - rx * 0.8, cy - ry * 0.8, F), (cx + rx * 0.8, cy + ry * 0.8, F + height * 0.55), base=base)

    rng = random.Random(seed)
    colors = ["red_paint", "rock_mid", "dirt", "red_paint", "rock_dark"]
    for i in range(blocks):
        angle = rng.uniform(0, math.tau)
        dist = rng.uniform(0.1, rx * 0.65)
        bx = cx + math.cos(angle) * dist
        by = cy + math.sin(angle) * dist
        bz = F + height * rng.uniform(0.3, 0.85)
        col = colors[i % len(colors)]
        sz = rng.uniform(0.22, 0.38)
        k.box((bx - sz / 2, by - sz / 2, bz), (bx + sz / 2, by + sz / 2, bz + sz * 0.6), col)


def build(model):
    k = kit.Kit(model)

    # 1. Ground floor: cracked slab with dirt and brick dust patches
    k.solid((-6.5, -4.5, 0.0), (6.5, 4.5, F), "dirt", base="floor")
    k.box((-6.2, -4.2, F + 0.008), (1.95, 1.5, F + 0.02), "rock_dark")
    k.box((2.0, -4.2, F + 0.008), (6.2, 0.0, F + 0.02), "rock_mid")

    # 2. Back Wall (Y = -4.5 to -4.22) - Solid, stepped jagged heights
    # Corner tall section (X: -6.5 to -5.2, height 5.2m reaching 2nd floor)
    _brick_wall_section(k, (-6.5, -4.5, F), (-5.2, -4.22, 5.2), base="bw_1")
    _brick_wall_section(k, (-5.2, -4.5, F), (-3.8, -4.22, 4.5), base="bw_2")
    _brick_wall_section(k, (-3.8, -4.5, F), (-2.6, -4.22, 3.8), base="bw_3")
    _brick_wall_section(k, (-2.6, -4.5, F), (-1.2, -4.22, 3.0), base="bw_4")
    _brick_wall_section(k, (-1.2, -4.5, F), (0.2, -4.22, 2.3), base="bw_5")
    _brick_wall_section(k, (0.2, -4.5, F), (1.6, -4.22, 1.6), base="bw_6")
    _brick_wall_section(k, (1.6, -4.5, F), (3.0, -4.22, 0.9), base="bw_7")
    _brick_wall_section(k, (3.0, -4.5, F), (4.6, -4.22, 1.8), base="bw_8")
    _brick_wall_section(k, (4.6, -4.5, F), (6.5, -4.22, 3.0), base="bw_9")

    # Stepped upper teeth that connect smoothly to the wall below
    k.box((-6.5, -4.48, 5.2), (-5.8, -4.24, 5.45), "red_paint")
    k.box((-5.2, -4.48, 4.5), (-4.5, -4.24, 4.75), "red_paint")
    k.box((-3.8, -4.48, 3.8), (-3.2, -4.24, 4.05), "red_paint")
    k.box((-2.6, -4.48, 3.0), (-2.0, -4.24, 3.25), "red_paint")
    k.box((0.2, -4.48, 2.3), (0.8, -4.24, 2.55), "red_paint")
    k.box((5.2, -4.48, 3.0), (6.0, -4.24, 3.25), "red_paint")

    # 3. Left Wall (X = -6.5 to -6.22) - Continuous stepped down from 2nd floor
    _brick_wall_section(k, (-6.5, -4.22, F), (-6.22, -2.8, 4.8), base="lw_1")
    _brick_wall_section(k, (-6.5, -2.8, F), (-6.22, -1.4, 3.9), base="lw_2")
    _brick_wall_section(k, (-6.5, -1.4, F), (-6.22, 0.2, 2.8), base="lw_3")
    _brick_wall_section(k, (-6.5, 0.2, F), (-6.22, 1.8, 2.0), base="lw_4")
    _brick_wall_section(k, (-6.5, 1.8, F), (-6.22, 3.2, 2.6), base="lw_5")
    _brick_wall_section(k, (-6.5, 3.2, F), (-6.22, 4.5, 3.3), base="lw_6")

    # Stepped upper teeth on left wall
    k.box((-6.48, -4.22, 4.8), (-6.24, -3.5, 5.05), "red_paint")
    k.box((-6.48, -2.8, 3.9), (-6.24, -2.1, 4.15), "red_paint")
    k.box((-6.48, -1.4, 2.8), (-6.24, -0.6, 3.05), "red_paint")
    k.box((-6.48, 3.2, 3.3), (-6.24, 4.0, 3.55), "red_paint")

    # 4. Right Wall (X = 6.22 to 6.5) - Features the SIDE BREACH
    _brick_wall_section(k, (6.22, -4.22, F), (6.5, -1.5, 2.8), base="rw_1")
    k.box((6.20, -3.2, 1.0), (6.52, -2.0, 1.08), "rock_dark")  # broken window sill
    k.box((6.22, -4.22, 2.8), (6.5, -3.4, 3.05), "red_paint")  # stepped tooth

    # SIDE BREACH: Y = -1.5 to 1.2 is OPEN at floor level (2.7 m wide doorway)
    _brick_wall_section(k, (6.22, -1.5, F), (6.5, -1.1, 0.32), base="rw_stub1")
    _brick_wall_section(k, (6.22, 0.8, F), (6.5, 1.2, 0.36), base="rw_stub2")

    _brick_wall_section(k, (6.22, 1.2, F), (6.5, 2.8, 1.8), base="rw_2")
    _brick_wall_section(k, (6.22, 2.8, F), (6.5, 4.5, 2.3), base="rw_3")
    k.box((6.22, 3.5, 2.3), (6.5, 4.5, 2.55), "red_paint")

    # 5. Front Wall (Y = 4.22 to 4.5) - Features the MAIN FRONT BREACH
    _brick_wall_section(k, (-6.22, 4.22, F), (-4.5, 4.5, 3.3), base="fw_1")
    k.box((-6.22, 4.22, 3.3), (-5.4, 4.5, 3.55), "red_paint")
    _brick_wall_section(k, (-4.5, 4.22, F), (-3.0, 4.5, 1.8), base="fw_2")
    _brick_wall_section(k, (-3.0, 4.22, F), (-1.2, 4.5, 0.85), base="fw_3")

    # MAIN FRONT BREACH: X = -1.2 to 4.2 is completely OPEN at floor level (5.4 m wide entry)
    k.box((-1.2, 4.20, F), (4.2, 4.52, F + 0.04), "rock_dark")

    _brick_wall_section(k, (4.2, 4.22, F), (5.2, 4.5, 1.4), base="fw_4")
    _brick_wall_section(k, (5.2, 4.22, F), (6.22, 4.5, 2.3), base="fw_5")

    # 6. Upper Floor Slab Hanging at an Angle
    # 6a. Intact horizontal section in the back-left corner (at Z = 3.0 m)
    slab_z = 3.0
    slab_th = 0.18
    k.solid((-6.22, -4.22, slab_z - slab_th), (-2.8, -1.8, slab_z), "rock_mid", base="slab_horiz")
    k.box((-2.82, -4.22, slab_z - slab_th - 0.02), (-2.78, -1.8, slab_z + 0.02), "rock_dark")

    # 6b. Hanging tilted slab section
    start_pt = (-2.8, -3.6, slab_z)
    end_pt = (0.5, -1.6, 1.1)
    span_w = 2.0

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
    tilted_bm = common.make_box((length, span_w, slab_th), color="rock_mid")
    rot_mat = Matrix((v_dir, v_span, v_up)).transposed().to_4x4()
    mat = Matrix.Translation(mid_pt) @ rot_mat
    bmesh.ops.transform(tilted_bm, matrix=mat, verts=list(tilted_bm.verts))
    tilted_bm.normal_update()
    k.model.add(tilted_bm)

    # Colliders for tilted slab (two stepped boxes)
    k.collider((-2.8, -3.8, 1.9), (-1.1, -1.6, 2.9), "tilted_slab_hi")
    k.collider((-1.1, -3.2, 0.9), (0.6, -1.0, 1.9), "tilted_slab_lo")

    # 6c. Exposed twisted rebar rods protruding from the lower broken edge
    for ry, rz in [(-2.2, 1.08), (-1.8, 1.12), (-1.4, 1.16)]:
        k.box((0.45, ry, rz), (1.05, ry + 0.04, rz + 0.04), "gun_black")
        k.box((1.01, ry, rz - 0.12), (1.05, ry + 0.04, rz + 0.04), "gun_black")

    # Broken concrete pillar (placed clear of the slab so it does not penetrate)
    k.solid((0.8, -3.2, F), (1.2, -2.8, 1.8), "rock_mid", base="broken_pillar")
    k.box((0.85, -3.15, 1.8), (1.15, -2.85, 2.05), "rock_dark")

    # 7. Natural Rubble Piles inside and outside (all within X: [-7, 7], Y: [-5, 5])
    # Rubble 1: Inside beneath the hanging slab (X = 0.5, Y = -1.2)
    _rubble_pile_natural(k, 0.5, -1.2, rx=1.3, ry=1.1, height=0.75, seed=12, blocks=6, base="rubble_inside1")

    # Rubble 2: Outside front breach (X = 1.8, Y = 4.75 - stays inside Y=5.0)
    _rubble_pile_natural(k, 1.8, 4.75, rx=1.1, ry=0.22, height=0.38, seed=24, blocks=4, base="rubble_street")

    # Rubble 3: Inside back-right corner (X = 4.2, Y = -3.2)
    _rubble_pile_natural(k, 4.2, -3.2, rx=1.2, ry=1.0, height=0.60, seed=36, blocks=5, base="rubble_inside2")
    # Fallen timber beam leaning against back wall
    k.box((3.6, -4.22, 0.3), (4.8, -3.2, 0.45), "bark_dark")
    k.box((4.2, -4.22, 0.45), (4.5, -4.18, 1.6), "bark_dark")

    # Rubble 4: Outside right side breach (X = 6.7, Y = -0.2 - stays inside X=7.0)
    _rubble_pile_natural(k, 6.7, -0.2, rx=0.25, ry=0.9, height=0.32, seed=48, blocks=3, base="rubble_side")

    # Rubble 5: Inside front-left corner (X = -4.2, Y = 2.8)
    _rubble_pile_natural(k, -4.2, 2.8, rx=1.0, ry=0.9, height=0.48, seed=60, blocks=4, base="rubble_frontleft")

    # Scattered loose bricks on the floor
    for bx, by, bz in [
        (-2.5, 1.2, F),
        (-0.5, 2.5, F),
        (2.8, -0.5, F),
        (3.5, 1.8, F),
        (-1.5, -3.0, F),
        (0.2, 3.5, F),
    ]:
        k.box((bx - 0.14, by - 0.08, bz), (bx + 0.14, by + 0.08, bz + 0.07), "red_paint")

    # 8. Interior partition wall remnant
    _brick_wall_section(k, (-1.8, -4.22, F), (-1.55, -2.4, 1.8), base="part_wall")
    k.box((-1.8, -2.4, F), (-1.55, -2.0, 0.85), "red_paint")
    # Rusted metal pipes on back wall
    k.box((-3.5, -4.24, 0.5), (-3.42, -4.18, 3.2), "metal")
    k.box((-3.5, -4.24, 1.5), (-2.5, -4.18, 1.58), "metal")

    # 9. Empties / Markers
    # Lot size
    k.empty("lot_size", (0.0, 0.0, 0.0), scale=(16.0, 12.0, 1.0))
    # Entrances (breaches)
    k.empty("door_1", (1.5, 4.4, F))   # main front breach
    k.empty("door_2", (6.4, -0.2, F))  # right side breach
    # 3 Loot markers
    k.empty("loot_1", (-4.5, -3.2, F))          # sheltered under 2nd floor slab
    k.empty("loot_2", (0.5, -1.2, F + 0.75))    # on top of main inside rubble pile
    k.empty("loot_3", (4.5, -3.2, F))          # back-right corner behind rubble
    # 3 Zombie spawn markers
    k.empty("spawn_1", (-3.5, -1.8, F))  # under hanging slab
    k.empty("spawn_2", (3.0, 1.5, F))    # front right inside
    k.empty("spawn_3", (-4.5, 2.0, F))   # front left inside


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, top_preview=True, view=kit.FRONT_VIEW) else 1


if __name__ == "__main__":
    sys.exit(main())
