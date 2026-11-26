"""Walkable rubble mounds (`rubble_field_l` and `rubble_field_m`) with stepped concrete slabs,
rebar, pipes, bricks, and soot chunks.

Each mound has 3 convex colliders (`ramp_a`, `ramp_b`, `pile`):
- `ramp_a` (+Y side) and `ramp_b` (-Y side) slope <= 28.5 degrees (<= 30 deg limit).
- Visual stepped slabs use <= 0.24 m step height (under mover max_step_height 0.35 m).
"""
import math
import os
import random
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))

import bmesh  # noqa: E402
from mathutils import Matrix, Vector  # noqa: E402

import common  # noqa: E402

BUDGET_L = 8000  # data: budgets.json props (rubble_field_l <= 8000)
BUDGET_M = 4000  # new: DECISIONS 2026-11-26 (rubble_field_m <= 4000)
STEP_HEIGHT_M = 0.24  # new: DECISIONS 2026-11-26 (walkable step <= 0.30 m, mover max 0.35 m)


def _angled_part(bm_list, a, b, r_or_w, color, cyl=True):
    va, vb = Vector(a), Vector(b)
    d = vb - va
    if d.length < 1e-4:  # new: DECISIONS 2026-11-26
        return
    z_axis = d.normalized()
    up = Vector((0.0, 0.0, 1.0)) if abs(z_axis.z) < 0.9 else Vector((1.0, 0.0, 0.0))  # new: DECISIONS 2026-11-26
    x_axis = up.cross(z_axis).normalized()
    y_axis = z_axis.cross(x_axis).normalized()
    if cyl:
        bm = common.make_frustum(r_or_w, r_or_w, -d.length / 2, d.length / 2, segments=6, color=color)  # new: DECISIONS 2026-11-26
    else:
        bm = common.make_box((r_or_w, r_or_w, d.length), color=color)
    mat = Matrix.Translation((va + vb) / 2) @ Matrix((x_axis, y_axis, z_axis)).transposed().to_4x4()
    bmesh.ops.transform(bm, matrix=mat, verts=list(bm.verts))
    bm.normal_update()
    bm_list.append(bm)


def _convex_wedge(pts):
    bm = common.new_bm()
    for p in pts:
        bm.verts.new(p)
    bm.verts.ensure_lookup_table()
    bmesh.ops.convex_hull(bm, input=list(bm.verts))
    bmesh.ops.recalc_face_normals(bm, faces=list(bm.faces))
    return bm


def _add_rubble_colliders(model, half_x, half_y, top_y, height):
    """3 convex hulls: `ramp_a` (+Y), `ramp_b` (-Y), `pile` (core). Slope = atan(H / (half_y - top_y))."""
    pile_bm = common.make_box((half_x * 1.65, top_y * 2.0, height),
                              loc=(0.0, 0.0, height / 2.0), color="white")  # new: DECISIONS 2026-11-26
    ramp_a = _convex_wedge([
        (-half_x * 0.92, 0.0, 0.0), (half_x * 0.92, 0.0, 0.0),
        (-half_x * 0.85, half_y, 0.0), (half_x * 0.85, half_y, 0.0),
        (-half_x * 0.78, 0.0, height), (half_x * 0.78, 0.0, height),
        (-half_x * 0.78, top_y, height), (half_x * 0.78, top_y, height),
    ])  # new: DECISIONS 2026-11-26
    ramp_b = _convex_wedge([
        (-half_x * 0.92, 0.0, 0.0), (half_x * 0.92, 0.0, 0.0),
        (-half_x * 0.85, -half_y, 0.0), (half_x * 0.85, -half_y, 0.0),
        (-half_x * 0.78, 0.0, height), (half_x * 0.78, 0.0, height),
        (-half_x * 0.78, -top_y, height), (half_x * 0.78, -top_y, height),
    ])  # new: DECISIONS 2026-11-26
    model.add_collider("ramp_a", ramp_a)
    model.add_collider("ramp_b", ramp_b)
    model.add_collider("pile", pile_bm)


def _organic_core_mound(parts, half_x, half_y, top_y, height, rng):
    """Irregular faceted earth/concrete core mound (no square box terrace corners)."""
    # Use a 10-sided tapered frustum per tier with jittered XY perimeter so outline is broken
    n_tiers = max(4, int(math.ceil(height / STEP_HEIGHT_M)))
    segs = 12  # new: DECISIONS 2026-11-26
    for t in range(n_tiers):
        z0 = max(0.0, t * STEP_HEIGHT_M - 0.08)  # new: DECISIONS 2026-11-26
        z1 = min(height - 0.12, (t + 1) * STEP_HEIGHT_M)  # new: DECISIONS 2026-11-26
        if z1 <= z0 + 0.02:
            continue
        frac0 = t / float(n_tiers)
        frac1 = (t + 1) / float(n_tiers)
        rx0 = half_x * (0.98 - 0.28 * frac0)  # new: DECISIONS 2026-11-26
        ry0 = half_y * (1.0 - frac0) + top_y * frac0
        rx1 = half_x * (0.98 - 0.28 * frac1)
        ry1 = half_y * (1.0 - frac1) + top_y * frac1
        col = "dirt" if t == 0 else ("concrete_stain" if t % 2 else "concrete_grey")  # new: DECISIONS 2026-11-26
        bm = common.new_bm()
        bot = []
        top = []
        for i in range(segs):
            a = 2.0 * math.pi * i / segs + (0.14 if t % 2 else 0.0)
            jit = rng.uniform(0.86, 1.06)  # new: DECISIONS 2026-11-26
            bot.append(bm.verts.new((math.cos(a) * rx0 * jit, math.sin(a) * ry0 * jit, z0)))
            top.append(bm.verts.new((math.cos(a) * rx1 * jit, math.sin(a) * ry1 * jit, z1)))
        for i in range(segs):
            j = (i + 1) % segs
            bm.faces.new([bot[i], bot[j], top[j], top[i]])
        bm.faces.new(list(reversed(bot)))
        bm.faces.new(top)
        bmesh.ops.recalc_face_normals(bm, faces=list(bm.faces))
        common.paint(bm, col)
        parts.append(bm)


def _build_mound(model, length, width, height, seed, detail_scale):
    rng = random.Random(seed)  # new: DECISIONS 2026-11-26
    half_x = length / 2.0
    half_y = width / 2.0
    top_y = 0.65 if detail_scale > 0.8 else 0.50  # new: DECISIONS 2026-11-26 (plateau half-width)

    parts = []
    _organic_core_mound(parts, half_x, half_y, top_y, height, rng)

    # Dense overlapping stepped concrete slabs across all tiers (step <= 0.24 m)
    n_tiers = max(4, int(math.ceil(height / STEP_HEIGHT_M)))
    slab_colors = ("concrete_grey", "concrete_stain", "limestone_dark", "limestone")
    for t in range(n_tiers):
        z_hi = min(height, (t + 1) * STEP_HEIGHT_M)
        frac = t / float(n_tiers)
        tx = half_x * (0.94 - 0.25 * frac)  # new: DECISIONS 2026-11-26
        ty = half_y * (1.0 - frac) + top_y * frac
        n_slabs = int((10 + (n_tiers - t) * 2) * detail_scale)  # new: DECISIONS 2026-11-26
        for s in range(n_slabs):
            ang = 2.0 * math.pi * (s + rng.uniform(-0.3, 0.3)) / max(1, n_slabs)
            r_scale = rng.uniform(0.72, 0.98)  # new: DECISIONS 2026-11-26
            sx = math.cos(ang) * tx * r_scale
            sy = math.sin(ang) * ty * r_scale
            sw = rng.uniform(1.0, 2.3) * detail_scale  # new: DECISIONS 2026-11-26
            sd = rng.uniform(0.85, 1.55) * detail_scale  # new: DECISIONS 2026-11-26
            sh = rng.uniform(0.16, STEP_HEIGHT_M)  # new: DECISIONS 2026-11-26 (step <= 0.24 m)
            slab = common.make_box((sw, sd, sh + 0.16), color=rng.choice(slab_colors))
            # Slope downward outward away from center
            tilt_x = -math.sin(ang) * rng.uniform(0.10, 0.28)  # new: DECISIONS 2026-11-26
            tilt_y = math.cos(ang) * rng.uniform(0.06, 0.22)  # new: DECISIONS 2026-11-26
            common.transform(slab, rot=(tilt_x, tilt_y, ang + rng.uniform(-0.4, 0.4)),
                             loc=(sx, sy, z_hi - sh * 0.40))
            parts.append(slab)

    # Top ridge slabs
    n_top = max(4, int(7 * detail_scale))
    for i in range(n_top):
        cx = -half_x * 0.60 + i * (half_x * 1.2 / max(1, n_top - 1))
        slab = common.make_box((rng.uniform(1.3, 2.1), rng.uniform(1.1, 1.6), 0.24),
                               color=rng.choice(slab_colors))  # new: DECISIONS 2026-11-26
        common.transform(slab, rot=(rng.uniform(-0.08, 0.08), rng.uniform(-0.08, 0.08), rng.uniform(-0.4, 0.4)),
                         loc=(cx, rng.uniform(-0.30, 0.30), height - 0.11))
        parts.append(slab)

    # Broken brick clusters & dark soot chunks scattered on the slopes
    n_bricks = int(42 * detail_scale)  # new: DECISIONS 2026-11-26
    for i in range(n_bricks):
        ang = rng.uniform(0.0, 2.0 * math.pi)
        r_f = rng.uniform(0.32, 0.96)
        bx = math.cos(ang) * half_x * r_f * 0.88
        by = math.sin(ang) * half_y * r_f
        bz = height * max(0.06, (1.0 - r_f)) * 0.92 + 0.10  # new: DECISIONS 2026-11-26
        col = "soot" if i % 4 == 0 else ("brick_red" if i % 2 == 0 else "brick_dark")  # new: DECISIONS 2026-11-26
        sz = (0.42, 0.28, 0.20) if col == "soot" else (0.34, 0.18, 0.13)  # new: DECISIONS 2026-11-26
        b_bm = common.make_box(sz, color=col)
        common.transform(b_bm, rot=(rng.uniform(-0.4, 0.4), rng.uniform(-0.4, 0.4), rng.uniform(0, 3.14)),
                         loc=(bx, by, bz))
        parts.append(b_bm)

    # Exposed rebar (some bent) & metal pipes emerging near sides/ends
    n_rebar = int(18 * detail_scale)  # new: DECISIONS 2026-11-26
    for i in range(n_rebar):
        ang = rng.uniform(0.0, 2.0 * math.pi)
        r_f = rng.uniform(0.45, 0.90)
        rx = math.cos(ang) * half_x * r_f * 0.82
        ry = math.sin(ang) * half_y * r_f
        rz = height * max(0.12, (1.0 - r_f)) * 0.88
        p0 = (rx, ry, rz - 0.12)
        p1 = (rx + math.cos(ang) * rng.uniform(0.35, 0.85),
              ry + math.sin(ang) * rng.uniform(0.35, 0.85),
              rz + rng.uniform(0.18, 0.55))
        _angled_part(parts, p0, p1, 0.028, "rust_dark", cyl=True)  # new: DECISIONS 2026-11-26
        if i % 2 == 0:
            p2 = (p1[0] + rng.uniform(-0.3, 0.3), p1[1] + math.sin(ang) * 0.30, p1[2] - 0.18)  # new: DECISIONS 2026-11-26
            _angled_part(parts, p1, p2, 0.024, "rust_dark", cyl=True)

    n_pipes = int(6 * detail_scale)  # new: DECISIONS 2026-11-26
    for i in range(n_pipes):
        px = -half_x * 0.65 + i * (half_x * 1.3 / max(1, n_pipes - 1))
        side = 1.0 if i % 2 == 0 else -1.0
        p0 = (px, side * (top_y + 0.3), height * 0.58)
        p1 = (px + rng.uniform(-0.6, 0.6), side * (half_y * 0.78), 0.24)
        _angled_part(parts, p0, p1, rng.uniform(0.09, 0.14), "metal", cyl=True)  # new: DECISIONS 2026-11-26

    model.add(common.combine(parts))
    _add_rubble_colliders(model, half_x, half_y, top_y, height)


def build_l(model):
    _build_mound(model, length=12.0, width=10.8, height=2.5, seed=41, detail_scale=1.0)  # new: DECISIONS 2026-11-26


def build_m(model):
    _build_mound(model, length=8.0, width=7.8, height=1.8, seed=42, detail_scale=0.72)  # new: DECISIONS 2026-11-26


def build_all():
    ok_l = common.run_model("rubble_field_l", "props", BUDGET_L, build_l,
                            grip=(0.0, 0.0, 0.0), view=(1.0, -1.0, 0.7))
    ok_m = common.run_model("rubble_field_m", "props", BUDGET_M, build_m,
                            grip=(0.0, 0.0, 0.0), view=(1.0, -1.0, 0.7))
    return [("rubble_field_l", ok_l), ("rubble_field_m", ok_m)]
