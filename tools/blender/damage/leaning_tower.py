"""Leaning 18-floor Art Deco tower (`tw_leaning_tower` and `_lod1`), leaning 8 deg toward +X.

Reuses deco._build_base, deco._build_floor, deco._build_crown (unmodified).
Origin: bottom centre of the un-rotated 24 m footprint, Z=0 at ground.
Lean-side (+X) corner is sunk 0.2 m into ground; lifted -X base edge is grounded by a sheared
foundation/rubble plinth and fallen column stub so no corner floats.
Rubble wedge at +X (2.5 m tall) serves as neighbour-roof contact and `ramp_a` collider.
"""
import math
import os
import random
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, os.path.join(HERE, "..", "buildings"))
sys.path.insert(0, os.path.join(HERE, "..", "towers"))

import bmesh  # noqa: E402
from mathutils import Matrix, Vector  # noqa: E402

import common  # noqa: E402
import deco  # noqa: E402
import kit  # noqa: E402

BUDGET_LOD0 = 80000  # data: budgets.json buildings (tw_leaning_tower <= 80000)
BUDGET_LOD1 = 8000  # new: DECISIONS 2026-11-26 (10% LOD1 budget for leaning tower)
TOWER_SIZE = 24.0  # new: DECISIONS 2026-11-26 (24 m Deco tower)
NUM_FLOORS = 18  # new: DECISIONS 2026-11-26 (18 floors -> ~80 m tall)
LEAN_DEG = 8.0  # new: DECISIONS 2026-11-26 (leans 8 degrees toward +X)
LEAN_RAD = math.radians(LEAN_DEG)  # new: DECISIONS 2026-11-26
SINK_Z = 0.2  # new: DECISIONS 2026-11-26 (lean-side corner sunk 0.2 m into ground)
WEDGE_H = 2.5  # new: DECISIONS 2026-11-26 (neighbour roof height rubble wedge at +X)


def _lean_matrix():
    """Rotate +8 deg about Y around pivot (+12, 0, 0), then sink Z by 0.2 m."""
    hs = TOWER_SIZE / 2.0
    return (Matrix.Translation((hs, 0.0, -SINK_Z))
            @ Matrix.Rotation(LEAN_RAD, 4, "Y")
            @ Matrix.Translation((-hs, 0.0, 0.0)))


def _collect_deco_part(fn, z_offset):
    """Calls a deco builder into a scratch Model, translates by z_offset, drops its own colliders."""
    tmp = common.Model("tmp", "buildings", 100000)
    fn(tmp)
    out = []
    for bm, roof in tmp._visual:
        common.transform(bm, loc=(0.0, 0.0, z_offset))
        out.append((bm, roof))
    for _, bm in tmp._parts:
        common.transform(bm, loc=(0.0, 0.0, z_offset))
        out.append((bm, False))
    for _, bm in tmp._colliders:
        bm.free()
    return out


def _add_rubble_wedge(k):
    """2.5 m tall rubble wedge at +X burying the lower lean side, plus fallen column stub & foundation skirt."""
    hs = TOWER_SIZE / 2.0
    rng = random.Random(4008)  # new: DECISIONS 2026-11-26

    # Grounded foundation & sheared rubble plinth under the lifted -X side so no corner floats
    lift_z = TOWER_SIZE * math.sin(LEAN_RAD) - SINK_Z  # ~3.14 m at x=-12  # new: DECISIONS 2026-11-26
    for i in range(6):
        x0 = -hs + i * 3.8  # new: DECISIONS 2026-11-26
        x1 = min(hs - 0.5, x0 + 4.0)
        z_top = max(0.25, lift_z * (1.0 - (x0 + hs) / TOWER_SIZE) + 0.15)  # new: DECISIONS 2026-11-26
        col = "limestone_dark" if i % 2 == 0 else "concrete_stain"  # new: DECISIONS 2026-11-26
        k.box((x0, -hs + 0.1, -SINK_Z), (x1, hs - 0.1, z_top), col)
    # Crushed masonry skirt along -X edge
    for i in range(8):
        sy = -hs + 1.5 + i * 3.0  # new: DECISIONS 2026-11-26
        k.box((-hs - 0.9, sy - 1.1, -SINK_Z), (-hs + 1.6, sy + 1.1, rng.uniform(0.7, 1.4)),
              "concrete_grey" if i % 2 == 0 else "limestone_dark")  # new: DECISIONS 2026-11-26

    # Neighbour-roof rubble wedge on +X side (2.5 m tall at x=hs..hs+6.5) with irregular sloped hull
    wedge_bm = common.new_bm()
    wedge_pts = [
        (hs - 1.2, -hs - 0.3, -SINK_Z), (hs + 6.8, -hs - 0.1, 0.0),
        (hs + 6.8, hs + 0.1, 0.0), (hs - 1.2, hs + 0.3, -SINK_Z),
        (hs - 1.2, -hs - 0.2, WEDGE_H), (hs + 1.6, -hs, WEDGE_H),
        (hs + 1.6, hs, WEDGE_H), (hs - 1.2, hs + 0.2, WEDGE_H),
    ]  # new: DECISIONS 2026-11-26
    for p in wedge_pts:
        wedge_bm.verts.new(p)
    wedge_bm.verts.ensure_lookup_table()
    bmesh.ops.convex_hull(wedge_bm, input=list(wedge_bm.verts))
    bmesh.ops.recalc_face_normals(wedge_bm, faces=list(wedge_bm.faces))
    common.paint(wedge_bm, "concrete_stain")
    k.model.add(wedge_bm)
    # Sloped rubble slabs, bricks, and dark chunks over +X wedge
    for i in range(28):
        ry = rng.uniform(-hs + 0.5, hs - 0.5)
        rx = hs + rng.uniform(0.3, 5.6)
        rz = WEDGE_H * max(0.12, 1.0 - (rx - hs) / 6.8)
        col = rng.choice(("concrete_grey", "limestone_dark", "concrete_stain", "soot", "brick_dark"))  # new: DECISIONS 2026-11-26
        slab = common.make_box((rng.uniform(1.2, 2.3), rng.uniform(1.1, 2.0), 0.36), color=col)  # new: DECISIONS 2026-11-26
        common.transform(slab, rot=(rng.uniform(-0.18, 0.18), rng.uniform(0.16, 0.44), rng.uniform(-0.45, 0.45)),
                         loc=(rx, ry, rz * 0.88))
        k.model.add(slab)

    # Fallen column stub at the foot (-X / -Y visible corner and -X / +Y front)
    stub = common.make_frustum(0.40, 0.35, 0.0, 2.8, segments=8, color="limestone")  # new: DECISIONS 2026-11-26
    common.transform(stub, rot=(0.25, 1.38, 0.3), loc=(-hs - 1.2, -hs + 2.2, 0.34))  # new: DECISIONS 2026-11-26
    k.model.add(stub)
    k.box((-hs - 1.6, -hs + 1.7, 0.0), (-hs - 0.8, -hs + 2.7, 0.55), "limestone_dark")  # column plinth


def _add_base_cracks(k, lean_mat):
    """5 dark soot crack lines across the tilted base floor faces (+Y, -X, +X)."""
    hs = TOWER_SIZE / 2.0
    crack_segs = [
        ((-8.5, hs + 0.05, 0.6), (-5.2, hs + 0.05, 3.8)),
        ((-5.2, hs + 0.05, 3.8), (-6.1, hs + 0.05, 6.2)),
        ((4.0, hs + 0.05, 0.4), (7.8, hs + 0.05, 4.4)),
        ((hs + 0.05, -6.0, 0.8), (hs + 0.05, -2.5, 5.2)),
        ((-hs - 0.05, 2.5, 0.5), (-hs - 0.05, 6.8, 4.9)),
    ]  # new: DECISIONS 2026-11-26 (5 crack lines on base floor)
    for a, b in crack_segs:
        va, vb = lean_mat @ Vector(a), lean_mat @ Vector(b)
        d = vb - va
        z_axis = d.normalized()
        up = Vector((0.0, 0.0, 1.0)) if abs(z_axis.z) < 0.9 else Vector((1.0, 0.0, 0.0))  # new: DECISIONS 2026-11-26
        x_axis = up.cross(z_axis).normalized()
        y_axis = z_axis.cross(x_axis).normalized()
        bm = common.make_box((0.12, 0.12, d.length), color="soot")  # new: DECISIONS 2026-11-26
        mat = Matrix.Translation((va + vb) / 2) @ Matrix((x_axis, y_axis, z_axis)).transposed().to_4x4()
        bmesh.ops.transform(bm, matrix=mat, verts=list(bm.verts))
        bm.normal_update()
        k.model.add(bm)


def build_lod0(model):
    lean_mat = _lean_matrix()
    stack_bms = []
    stack_bms.extend(_collect_deco_part(lambda m: deco._build_base(m, 24), 0.0))
    cur_z = deco.BASE_H
    for f in range(NUM_FLOORS):
        variant = f % 2
        stack_bms.extend(_collect_deco_part(lambda m, v=variant: deco._build_floor(m, 24, v), cur_z))
        cur_z += deco.FLOOR_H
    stack_bms.extend(_collect_deco_part(lambda m: deco._build_crown(m, 24), cur_z))
    total_h = cur_z + deco.CROWN_H  # 7.6 + 68.4 + 5.5 = 81.5 m

    for bm, roof in stack_bms:
        bmesh.ops.transform(bm, matrix=lean_mat, verts=list(bm.verts))
        bm.normal_update()
        model.add(bm, roof=roof)

    k = kit.Kit(model)
    _add_rubble_wedge(k)
    _add_base_cracks(k, lean_mat)

    # Colliders: tilted `body` box + +X rubble wedge `ramp_a`
    body_col = common.make_box((TOWER_SIZE, TOWER_SIZE, total_h), loc=(0.0, 0.0, total_h / 2.0), color="white")
    bmesh.ops.transform(body_col, matrix=lean_mat, verts=list(body_col.verts))
    model.add_collider("body", body_col)

    hs = TOWER_SIZE / 2.0
    ramp_bm = common.new_bm()
    for p in (
        (hs - 0.5, -hs, -SINK_Z), (hs + 6.6, -hs, 0.0), (hs + 6.6, hs, 0.0), (hs - 0.5, hs, -SINK_Z),
        (hs - 0.5, -hs, WEDGE_H), (hs + 1.5, -hs, WEDGE_H), (hs + 1.5, hs, WEDGE_H), (hs - 0.5, hs, WEDGE_H),
    ):  # new: DECISIONS 2026-11-26 (walkable rubble wedge collider at +X)
        ramp_bm.verts.new(p)
    ramp_bm.verts.ensure_lookup_table()
    bmesh.ops.convex_hull(ramp_bm, input=list(ramp_bm.verts))
    bmesh.ops.recalc_face_normals(ramp_bm, faces=list(ramp_bm.faces))
    model.add_collider("ramp_a", ramp_bm)


def build_lod1(model):
    lean_mat = _lean_matrix()
    hs = TOWER_SIZE / 2.0
    total_h = deco.BASE_H + NUM_FLOORS * deco.FLOOR_H + deco.CROWN_H  # 81.5 m

    parts = []
    # Main tower shaft
    parts.append(common.make_box((TOWER_SIZE, TOWER_SIZE, total_h - deco.CROWN_H),
                                 loc=(0.0, 0.0, (total_h - deco.CROWN_H) / 2.0), color="limestone"))
    # Window stripes per floor on all 4 sides
    for f in range(NUM_FLOORS + 2):
        z0 = 1.0 + f * deco.FLOOR_H
        if z0 + 2.0 > total_h - deco.CROWN_H:
            break
        parts.append(common.make_box((TOWER_SIZE + 0.06, TOWER_SIZE - 1.2, 2.0),
                                     loc=(0.0, 0.0, z0 + 1.0), color="glass_dark"))
        parts.append(common.make_box((TOWER_SIZE - 1.2, TOWER_SIZE + 0.06, 2.0),
                                     loc=(0.0, 0.0, z0 + 1.0), color="glass_dark"))
    # Crown silhouette
    cz = total_h - deco.CROWN_H
    parts.append(common.make_box((TOWER_SIZE + 0.2, TOWER_SIZE + 0.2, 0.8),
                                 loc=(0.0, 0.0, cz + 0.4), color="limestone_dark"))
    pw = TOWER_SIZE * 0.22  # new: DECISIONS 2026-11-26 (matches deco.py crown penthouse)
    parts.append(common.make_box((pw * 2.0, pw * 2.0, 2.4),
                                 loc=(0.0, 0.0, cz + 2.0), color="limestone_dark"))
    parts.append(common.make_box((2.0, 2.0, 3.6),
                                 loc=(-hs * 0.52, -hs * 0.45, cz + 2.6), color="bark_light"))

    for bm in parts:
        bmesh.ops.transform(bm, matrix=lean_mat, verts=list(bm.verts))
        bm.normal_update()
        model.add(bm)

    # Simplified foundation plinth and +X rubble wedge (no colliders)
    k = kit.Kit(model)
    k.box((-hs, -hs, -SINK_Z), (hs, hs, 3.2), "limestone_dark")
    k.box((hs - 0.5, -hs, -SINK_Z), (hs + 6.5, hs, WEDGE_H * 0.6), "concrete_stain")


def build_all():
    ok0 = common.run_model("tw_leaning_tower", "buildings", BUDGET_LOD0, build_lod0,
                           grip=(0.0, 0.0, 0.0), view=(0.6, -1.0, 0.35))  # new: DECISIONS 2026-11-26
    ok1 = common.run_model("tw_leaning_tower_lod1", "buildings", BUDGET_LOD1, build_lod1,
                           grip=(0.0, 0.0, 0.0), view=(0.6, -1.0, 0.35))  # new: DECISIONS 2026-11-26
    return [("tw_leaning_tower", ok0), ("tw_leaning_tower_lod1", ok1)]
