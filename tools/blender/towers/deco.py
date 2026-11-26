"""Art Deco tower kit: 24 m and 32 m base, floor, floor_b, crown (+ _lod1 each)."""
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, os.path.join(HERE, "..", "buildings"))
sys.path.insert(0, HERE)

import bmesh  # noqa: E402
from mathutils import Matrix, Vector  # noqa: E402

import common  # noqa: E402
import damaged_parts  # noqa: E402
import kit  # noqa: E402
import roof_parts  # noqa: E402
import roof_props  # noqa: E402
import shell  # noqa: E402

FLOOR_H = 3.8
BASE_H = 7.6
CROWN_H = 5.5
BAY = 4.0


def _crack_bar(k, a, b, color="soot"):
    va, vb = Vector(a), Vector(b)
    d = vb - va
    z_axis = d.normalized()
    up = Vector((0.0, 0.0, 1.0)) if abs(z_axis.z) < 0.9 else Vector((1.0, 0.0, 0.0))
    x_axis = up.cross(z_axis).normalized()
    y_axis = z_axis.cross(x_axis).normalized()
    bm = common.make_box((0.03, 0.03, d.length), color=color)
    mat = Matrix.Translation((va + vb) / 2) @ Matrix((x_axis, y_axis, z_axis)).transposed().to_4x4()
    bmesh.ops.transform(bm, matrix=mat, verts=list(bm.verts))
    bm.normal_update()
    k.model.add(bm)


def _add_piers(k, size, z0, z1, wall_inset, pier_out, skip_center_front=False):
    """Vertical Art Deco piers every 4 m bay on all 4 faces."""
    hs = size / 2.0
    n_bays = int(round(size / BAY))
    face_in = hs - wall_inset
    face_out = face_in + pier_out
    for i in range(n_bays + 1):
        u = -hs + i * BAY
        hw = 0.34 if i in (0, n_bays) else 0.26
        # Front (+Y) and Back (-Y)
        if not (skip_center_front and abs(u) < 0.5):
            k.box((u - hw, face_in, z0), (u + hw, face_out, z1), "limestone_dark")
        k.box((u - hw, -face_out, z0), (u + hw, -face_in, z1), "limestone_dark")
        # Left (-X) and Right (+X), skip corners already covered
        if 0 < i < n_bays:
            k.box((-face_out, u - hw, z0), (-face_in, u + hw, z1), "limestone_dark")
            k.box((face_in, u - hw, z0), (face_out, u + hw, z1), "limestone_dark")


def _chevron_spandrel(k, axis, sign, u_center, wall_face, z0):
    """Zig-zag Art Deco spandrel of 4 stepped boxes above a window bay."""
    steps = ((1.30, 0.14, "limestone_dark"), (0.92, 0.14, "mortar"),
             (0.56, 0.14, "limestone_dark"), (0.24, 0.12, "mortar"))
    z = z0
    for hw, h, col in steps:
        d0, d1 = sorted((wall_face, wall_face + sign * 0.09))
        if axis == 1:  # +Y / -Y wall
            k.box((u_center - hw, d0, z), (u_center + hw, d1, z + h), col)
        else:  # +X / -X wall
            k.box((d0, u_center - hw, z), (d1, u_center + hw, z + h), col)
        z += h


def _window_bay(k, axis, sign, u_center, wall_face, z0, z1, state, detailed=True):
    """Recessed window decal with stone sill, lintel, and broken/boarded state."""
    hw = 1.15
    glass_col = "soot" if state == 1 else "glass_dark"
    g0, g1 = sorted((wall_face, wall_face + sign * 0.02))
    s0, s1 = sorted((wall_face, wall_face + sign * 0.09))

    if axis == 1:
        k.box((u_center - hw, g0, z0), (u_center + hw, g1, z1), glass_col)
        k.box((u_center - 0.04, s0, z0), (u_center + 0.04, g1, z1), "metal")
        k.box((u_center - hw - 0.12, s0, z0 - 0.12), (u_center + hw + 0.12, s1, z0), "limestone_dark")
        k.box((u_center - hw - 0.10, s0, z1), (u_center + hw + 0.10, s1, z1 + 0.14), "limestone_dark")
    else:
        k.box((g0, u_center - hw, z0), (g1, u_center + hw, z1), glass_col)
        k.box((s0, u_center - 0.04, z0), (g1, u_center + 0.04, z1), "metal")
        k.box((s0, u_center - hw - 0.12, z0 - 0.12), (s1, u_center + hw + 0.12, z0), "limestone_dark")
        k.box((s0, u_center - hw - 0.10, z1), (s1, u_center + hw + 0.10, z1 + 0.14), "limestone_dark")

    if not detailed:
        return

    if state == 2:  # boarded
        for row_f in (0.28, 0.65):
            rz = z0 + (z1 - z0) * row_f
            b0, b1 = sorted((wall_face + sign * 0.03, wall_face + sign * 0.07))
            if axis == 1:
                k.box((u_center - hw - 0.05, b0, rz - 0.07), (u_center + hw + 0.05, b1, rz + 0.07), "bark_light")
            else:
                k.box((b0, u_center - hw - 0.05, rz - 0.07), (b1, u_center + hw + 0.05, rz + 0.07), "bark_light")
    elif state == 3:  # cracked glass diagonal bars
        off = wall_face + sign * 0.035
        if axis == 1:
            _crack_bar(k, (u_center - 0.7, off, z0 + 0.2), (u_center + 0.5, off, z1 - 0.2))
            _crack_bar(k, (u_center - 0.2, off, z1 - 0.3), (u_center + 0.8, off, z0 + 0.3))
        else:
            _crack_bar(k, (off, u_center - 0.7, z0 + 0.2), (off, u_center + 0.5, z1 - 0.2))
            _crack_bar(k, (off, u_center - 0.2, z1 - 0.3), (off, u_center + 0.8, z0 + 0.3))


def _bay_state(side_idx, bay_idx, variant):
    code = (side_idx * 5 + bay_idx * 3 + variant * 2) % 7
    if code in (0, 4):
        return 0  # intact dark glass
    if code in (1, 5):
        return 1  # soot / missing
    if code == 2:
        return 2  # boarded
    return 3  # cracked


def _build_floor(model, size, variant):
    k = kit.Kit(model)
    hs = size / 2.0
    inset = 0.25
    wall_face = hs - inset
    n_bays = int(round(size / BAY))

    # Bottom and top slab bands so stacked modules have zero gap
    k.box((-hs, -hs, 0.0), (hs, hs, 0.22), "limestone_dark")
    k.box((-hs, -hs, FLOOR_H - 0.22), (hs, hs, FLOOR_H), "limestone_dark")
    # Recessed stone core
    k.box((-wall_face, -wall_face, 0.22), (wall_face, wall_face, FLOOR_H - 0.22), "limestone")
    _add_piers(k, size, 0.0, FLOOR_H, inset, inset)

    sides = ((1, 1, True), (0, -1, True), (0, 1, True), (1, -1, False))
    for s_idx, (axis, sign, detailed) in enumerate(sides):
        face = sign * wall_face
        for b in range(n_bays):
            uc = -hs + (b + 0.5) * BAY
            st = _bay_state(s_idx, b, variant)
            _window_bay(k, axis, sign, uc, face, 0.82, 2.68, st, detailed=detailed)
            if detailed:
                _chevron_spandrel(k, axis, sign, uc, face, 2.86)


def _build_base(model, size):
    k = kit.Kit(model)
    hs = size / 2.0
    n_bays = int(round(size / BAY))

    shell.lobby_shell(k, size, "limestone")
    # Plinth band around exterior, leaving front door x -1.4..1.4 open
    k.box((-hs - 0.04, hs - 0.02, 0.0), (-1.45, hs + 0.06, 0.48), "limestone_dark")
    k.box((1.45, hs - 0.02, 0.0), (hs + 0.04, hs + 0.06, 0.48), "limestone_dark")
    k.box((-hs - 0.04, -hs - 0.06, 0.0), (hs + 0.04, -hs + 0.02, 0.48), "limestone_dark")
    k.box((-hs - 0.06, -hs, 0.0), (-hs + 0.02, hs, 0.48), "limestone_dark")
    k.box((hs - 0.02, -hs, 0.0), (hs + 0.06, hs, 0.48), "limestone_dark")

    # Piers on all 4 sides; skip x=0 on lower front so the 2.8 m door stays clear
    _add_piers(k, size, 0.0, BASE_H, 0.03, 0.22, skip_center_front=True)
    k.box((-0.26, hs - 0.03, 4.35), (0.26, hs + 0.19, BASE_H), "limestone_dark")

    # Front door frame + stepped Art Deco arch + brass sconces
    door = shell.door_opening(size)
    k.frame(door, hs, 1, color="limestone_dark", t=0.16, depth=0.14, sill=False)
    for idx, (hw, z0, z1, col) in enumerate((
        (1.80, 3.12, 3.40, "limestone_dark"),
        (1.40, 3.40, 3.68, "mortar"),
        (0.95, 3.68, 3.96, "limestone_dark"),
        (0.50, 3.96, 4.24, "mortar"),
    )):
        k.box((-hw, hs, z0), (hw, hs + 0.18 - idx * 0.02, z1), col)
    for sx in (-1.65, 1.65):
        k.box((sx - 0.08, hs + 0.02, 1.75), (sx + 0.08, hs + 0.16, 2.55), "brass")

    # Top cornice band (roof=True so top preview shows the open lobby hall)
    k.box((-hs - 0.08, -hs - 0.08, BASE_H - 0.28), (hs + 0.08, hs + 0.08, BASE_H), "limestone_dark", roof=True)

    # Ground-floor and mezzanine windows
    sides = ((1, 1, True), (0, -1, True), (0, 1, True), (1, -1, False))
    for s_idx, (axis, sign, detailed) in enumerate(sides):
        face = sign * hs
        for b in range(n_bays):
            uc = -hs + (b + 0.5) * BAY
            # Ground floor: skip front center 2 bays that straddle the door arch
            if not (axis == 1 and sign > 0 and abs(uc) < 3.0):
                _window_bay(k, axis, sign, uc, face, 0.85, 2.85, _bay_state(s_idx, b, 0), detailed=detailed)
            # Mezzanine floor (z 4.45..6.45)
            _window_bay(k, axis, sign, uc, face, 4.45, 6.45, _bay_state(s_idx, b, 1), detailed=detailed)
            if detailed:
                _chevron_spandrel(k, axis, sign, uc, face, 6.62)


def _build_crown(model, size):
    k = kit.Kit(model)
    hs = size / 2.0

    # 3-ledge stepped cornice (z 0..0.80)
    k.box((-hs, -hs, 0.0), (hs, hs, 0.28), "limestone_dark")
    k.box((-hs - 0.14, -hs - 0.14, 0.28), (hs + 0.14, hs + 0.14, 0.54), "mortar")
    k.box((-hs - 0.06, -hs - 0.06, 0.54), (hs + 0.06, hs + 0.06, 0.80), "limestone_dark")

    # Roof deck + parapet
    roof_parts.roof_deck(k, size - 0.2, 0.80, "limestone")

    # Stepped Art Deco corner pinnacles + mid-wall parapet crests
    for sx in (-1.0, 1.0):
        for sy in (-1.0, 1.0):
            cx, cy = sx * (hs - 0.65), sy * (hs - 0.65)
            k.box((cx - 0.55, cy - 0.55, 1.60), (cx + 0.55, cy + 0.55, 2.35), "limestone_dark")
            k.box((cx - 0.35, cy - 0.35, 2.35), (cx + 0.35, cy + 0.35, 3.05), "mortar")
            k.box((cx - 0.18, cy - 0.18, 3.05), (cx + 0.18, cy + 0.18, 3.60), "limestone_dark")

    # Ribbed mechanical penthouse
    pw = size * 0.22
    k.box((-pw, -pw, 1.00), (pw, pw, 3.00), "limestone")
    k.box((-pw - 0.10, -pw - 0.10, 3.00), (pw + 0.10, pw + 0.10, 3.25), "limestone_dark")
    for r in range(-2, 3):
        rx = r * (pw * 0.40)
        k.box((rx - 0.15, pw, 1.00), (rx + 0.15, pw + 0.16, 3.00), "limestone_dark")
        k.box((rx - 0.15, -pw - 0.16, 1.00), (rx + 0.15, -pw, 3.00), "limestone_dark")
    k.box((-pw * 0.6, pw + 0.02, 1.4), (pw * 0.6, pw + 0.08, 2.4), "rock_dark")

    # Roof parts & roof_props (staying within CROWN_H=5.5 m and footprint)
    if size == 24:
        roof_props.roof_water_tank_ny(k, -6.4, -6.0, 1.00, r=1.25, leg_h=1.15, tank_h=1.90)
        roof_props.roof_stair_hut(k, 6.4, -5.6, 1.00, wall_color="limestone")
        roof_props.roof_machinery(k, -6.2, 5.8, 1.00)
        roof_props.roof_antenna(k, 6.6, 6.2, 1.00, h=4.2)
        roof_parts.ac_unit(k, 0.0, 6.8, 1.00)
        roof_parts.vent_stack(k, 0.0, -6.8, 1.00, 1.6)
    else:
        roof_props.roof_water_tank_ny(k, -9.4, -8.8, 1.00, r=1.35, leg_h=1.25, tank_h=2.05)
        roof_props.roof_water_tank_ny(k, -9.4, 8.2, 1.00, r=1.25, leg_h=1.15, tank_h=1.90)
        roof_props.roof_stair_hut(k, 9.2, -8.4, 1.00, wall_color="limestone")
        roof_props.roof_machinery(k, 0.0, -9.0, 1.00)
        roof_props.roof_antenna(k, 9.6, 9.0, 1.00, h=4.3)
        roof_props.roof_antenna(k, -0.8, 9.2, 1.00, h=3.6)
        roof_parts.ac_unit(k, 9.2, 0.2, 1.00)
        roof_parts.ac_unit(k, -9.2, 0.2, 1.00)
        roof_parts.vent_stack(k, 3.8, 8.8, 1.00, 1.6)


def _build_floor_c(model, size):
    k = kit.Kit(model)
    hs = size / 2.0
    inset = 0.25
    wall_face = hs - inset
    n_bays = int(round(size / BAY))

    k.box((-hs, -hs, 0.0), (hs, hs, 0.22), "limestone_dark")
    k.box((-hs, -hs, FLOOR_H - 0.22), (hs, hs, FLOOR_H), "limestone_dark")
    damaged_parts.damaged_wall_core(k, size, 0.22, FLOOR_H - 0.22, "limestone",
                                    trim_color="limestone_dark", inset=inset)

    face_in = hs - inset
    face_out = hs
    for i in range(n_bays + 1):
        u = -hs + i * BAY
        hw = 0.34 if i in (0, n_bays) else 0.26
        top_f = FLOOR_H - 0.38 if i == n_bays else FLOOR_H
        k.box((u - hw, face_in, 0.0), (u + hw, face_out, top_f), "limestone_dark")
        k.box((u - hw, -face_out, 0.0), (u + hw, -face_in, FLOOR_H), "limestone_dark")
        if 0 < i < n_bays:
            k.box((-face_out, u - hw, 0.0), (-face_in, u + hw, FLOOR_H), "limestone_dark")
            k.box((face_in, u - hw, 0.0), (face_out, u + hw, FLOOR_H), "limestone_dark")

    sides = ((1, 1, True), (0, -1, True), (0, 1, True), (1, -1, False))
    for s_idx, (axis, sign, detailed) in enumerate(sides):
        face = sign * wall_face
        for b in range(n_bays):
            uc = -hs + (b + 0.5) * BAY
            idx = s_idx * n_bays + b
            if (s_idx == 0 and b == 1) or (s_idx == 2 and b == 2):
                damaged_parts.missing_frame(k, face, axis, sign, uc, 0.82, 2.68, 1.15)
                damaged_parts.soot_streak(k, face, axis, sign, uc, 2.68, width=1.10)
            elif (s_idx == 0 and b == 4) or (s_idx == 2 and b == 0) or (s_idx == 1 and b == 2):
                _window_bay(k, axis, sign, uc, face, 0.82, 2.68, 1, detailed=detailed)
                damaged_parts.shattered_pane(k, face, axis, sign, uc, 0.82, 2.68, 1.10, d_base=0.025)
            else:
                st = (1, 0, 2, 1, 0, 1)[idx % 6]
                _window_bay(k, axis, sign, uc, face, 0.82, 2.68, st, detailed=detailed)
            if (s_idx == 0 and b == 3) or (s_idx == 1 and b == 1):
                damaged_parts.soot_streak(k, face, axis, sign, uc, 2.68, width=1.10)
            if detailed and idx % 2 == 0:
                damaged_parts.rust_streak(k, face, axis, sign, uc, 0.82, width=0.65)
            if detailed and not (b == n_bays - 1 and ((axis == 1 and sign > 0) or (axis == 0 and sign > 0))):
                _chevron_spandrel(k, axis, sign, uc, face, 2.86)


def _build_lod1(model, size, height, kind):
    k = kit.Kit(model)
    hs = size / 2.0
    if kind == "floor_c":
        damaged_parts.damaged_wall_core(k, size, 0.0, height, "limestone", patch=1.2, short_h=0.40)
        k.box((-hs, -hs, height - 0.25), (hs, hs, height), "limestone_dark")
        k.box((-hs - 0.02, -hs + 0.5, 0.8), (hs + 0.02, hs - 0.5, height - 0.8), "glass_dark")
        k.box((-hs * 0.65, hs - 0.02, 0.8), (-hs * 0.20, hs + 0.05, height - 0.8), "soot")
        k.box((hs * 0.10, hs - 0.02, 0.8), (hs * 0.55, hs + 0.05, height - 0.8), "soot")
        k.box((hs - 0.02, -hs * 0.35, 0.8), (hs + 0.05, hs * 0.15, height - 0.8), "soot")
    elif kind == "crown":
        k.box((-hs, -hs, 0.0), (hs, hs, height), "limestone")
        k.box((-hs - 0.1, -hs - 0.1, 0.25), (hs + 0.1, hs + 0.1, 0.75), "limestone_dark")
        pw = size * 0.22
        k.box((-pw, -pw, 0.75), (pw, pw, 3.2), "limestone_dark")
        if size == 24:
            model.add(common.make_frustum(1.25, 1.25, 1.0, 4.9, segments=6, center=(-6.4, -6.0), color="bark_light"))
            k.box((4.9, -7.3, 1.0), (7.9, -3.9, 3.4), "limestone")
            k.box((-7.8, 4.7, 1.0), (-4.6, 6.9, 2.7), "concrete_grey")
            k.box((6.4, 6.0, 1.0), (6.8, 6.4, 5.2), "metal")
        else:
            model.add(common.make_frustum(1.35, 1.35, 1.0, 5.1, segments=6, center=(-9.4, -8.8), color="bark_light"))
            model.add(common.make_frustum(1.25, 1.25, 1.0, 4.9, segments=6, center=(-9.4, 8.2), color="bark_light"))
            k.box((7.7, -10.1, 1.0), (10.7, -6.7, 3.4), "limestone")
            k.box((-1.6, -10.1, 1.0), (1.6, -7.9, 2.7), "concrete_grey")
            k.box((9.4, 8.8, 1.0), (9.8, 9.2, 5.3), "metal")
            k.box((-1.0, 9.0, 1.0), (-0.6, 9.4, 4.6), "metal")
    else:
        k.box((-hs, -hs, 0.0), (hs, hs, height), "limestone")
        # Simplified dark window bands and corner piers
        k.box((-hs - 0.02, -hs + 0.5, 0.8), (hs + 0.02, hs - 0.5, height - 0.8), "glass_dark")
        k.box((-hs + 0.5, -hs - 0.02, 0.8), (hs - 0.5, hs + 0.02, height - 0.8), "glass_dark")
        k.box((-hs, -hs, height - 0.25), (hs, hs, height), "limestone_dark")


def build_r40_all():
    results = []
    for size in (24, 32):
        floor_b = 4500 if size == 24 else 6000
        specs = [
            (f"tw_deco_{size}_floor_c", floor_b, lambda m, s=size: _build_floor_c(m, s)),
            (f"tw_deco_{size}_floor_c_lod1", floor_b // 10, lambda m, s=size: _build_lod1(m, s, FLOOR_H, "floor_c")),
            (f"tw_deco_{size}_crown", 8000, lambda m, s=size: _build_crown(m, s)),
            (f"tw_deco_{size}_crown_lod1", 800, lambda m, s=size: _build_lod1(m, s, CROWN_H, "crown")),
        ]
        for name, budget, fn in specs:
            ok = common.run_model(name, "buildings", budget, fn,
                                  grip=(0.0, 0.0, 0.0), top_preview=False, view=kit.FRONT_VIEW)
            results.append((name, ok))
    return results


def build_all():
    results = []
    for size in (24, 32):
        floor_b = 4500 if size == 24 else 6000
        specs = [
            (f"tw_deco_{size}_base", 15000, True, lambda m, s=size: _build_base(m, s)),
            (f"tw_deco_{size}_base_lod1", 1500, False, lambda m, s=size: _build_lod1(m, s, BASE_H, "base")),
            (f"tw_deco_{size}_floor", floor_b, False, lambda m, s=size: _build_floor(m, s, 0)),
            (f"tw_deco_{size}_floor_lod1", floor_b // 10, False, lambda m, s=size: _build_lod1(m, s, FLOOR_H, "floor")),
            (f"tw_deco_{size}_floor_b", floor_b, False, lambda m, s=size: _build_floor(m, s, 1)),
            (f"tw_deco_{size}_floor_b_lod1", floor_b // 10, False, lambda m, s=size: _build_lod1(m, s, FLOOR_H, "floor")),
            (f"tw_deco_{size}_crown", 8000, False, lambda m, s=size: _build_crown(m, s)),
            (f"tw_deco_{size}_crown_lod1", 800, False, lambda m, s=size: _build_lod1(m, s, CROWN_H, "crown")),
        ]
        for name, budget, top_prev, fn in specs:
            ok = common.run_model(name, "buildings", budget, fn,
                                  grip=(0.0, 0.0, 0.0), top_preview=top_prev, view=kit.FRONT_VIEW)
            results.append((name, ok))
    return results


if __name__ == "__main__":
    res = build_all()
    failed = [n for n, ok in res if not ok]
    print(f"DECO: {len(res) - len(failed)}/{len(res)} OK")
    sys.exit(1 if failed else 0)
