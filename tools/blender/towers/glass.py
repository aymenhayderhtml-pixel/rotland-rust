"""Glass curtain-wall office tower kit (24 m and 32 m): base, floor, floor_b, crown + LOD1."""
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
BLENDER_DIR = os.path.dirname(HERE)
for p in (HERE, BLENDER_DIR, os.path.join(BLENDER_DIR, "buildings"), os.path.join(BLENDER_DIR, "metro")):
    if p not in sys.path:
        sys.path.insert(0, p)

import common  # noqa: E402
import damaged_parts  # noqa: E402
import kit  # noqa: E402
import roof_parts  # noqa: E402
import roof_props  # noqa: E402
import shell  # noqa: E402

CATEGORY = "buildings"
FLOOR_H = 3.8  # new: DECISIONS 2026-11-26
BASE_H = 7.6   # new: DECISIONS 2026-11-26
CROWN_H = 4.5  # new: DECISIONS 2026-11-26
SPANDREL_H = 0.45  # half of 0.9 m spandrel band at top and bottom of each module
BAY_W = 4.0
SUB_PANES = 3  # 4 m / 3 = 1.33 m (~1.5 m mullion spacing)

BUDGET_FLOOR = {24: 4500, 32: 6000}
BUDGET_BASE = 15000
BUDGET_CROWN = 8000


def _pane_state(face_idx, idx, variant):
    """Deterministic pattern: ~20% dead (glass_dark), ~10% missing (soot), some cracked."""
    h = (face_idx * 31 + idx * 17 + (13 if variant == "b" else 0)) % 20
    if variant == "b":
        if h in (2, 11):
            return "soot", False
        if h in (0, 5, 9, 15):
            return "glass_dark", h in (5, 15)
        if h in (4, 8, 12, 16):
            return "glass_tint", True  # row of cracks
        return "glass_tint", False
    if h in (3, 14):
        return "soot", False
    if h in (1, 6, 12, 18):
        return "glass_dark", h == 6
    return "glass_tint", h in (7, 17)


def _crack_bar(k, cx, cy, cz, along_x, length=0.9, angle=0.55):
    bm = common.make_box((length, 0.018, 0.024) if along_x else (0.018, length, 0.024), color="soot")
    rot = (0.0, angle, 0.0) if along_x else (angle, 0.0, 0.0)
    common.transform(bm, rot=rot, loc=(cx, cy, cz))
    k.model.add(bm)


def _curtain_face(k, size, face, z0, z1, variant="a", skip_door=False, simple=False):
    """Builds one curtain-wall band from z0 to z1 on face ('+Y', '-Y', '-X', '+X')."""
    half = size / 2.0
    bays = size // 4
    panes = bays * SUB_PANES if not simple else bays
    pw = size / panes
    along_x = face in ("+Y", "-Y")
    sign = 1.0 if face in ("+Y", "+X") else -1.0
    fi = {"+Y": 0, "+X": 1, "-X": 2, "-Y": 3}[face]

    def place_box(u0, u1, d0, d1, zb, zt, color):
        if along_x:
            lo = (u0, sign * half + min(d0, d1) * sign, zb)
            hi = (u1, sign * half + max(d0, d1) * sign, zt)
        else:
            lo = (sign * half + min(d0, d1) * sign, u0, zb)
            hi = (sign * half + max(d0, d1) * sign, u1, zt)
        k.box((min(lo[0], hi[0]), min(lo[1], hi[1]), lo[2]),
              (max(lo[0], hi[0]), max(lo[1], hi[1]), hi[2]), color)

    # Horizontal transoms and chrome cap rails
    mid_z = (z0 + z1) / 2.0
    place_box(-half, half, -0.08, 0.04, z0, z0 + 0.06, "metal")
    place_box(-half, half, -0.08, 0.04, z1 - 0.06, z1, "metal")
    if not skip_door:
        place_box(-half, half, -0.06, 0.03, mid_z - 0.04, mid_z + 0.04, "metal")
    place_box(-half, half, 0.02, 0.07, z0 + 0.02, z0 + 0.08, "chrome")
    place_box(-half, half, 0.02, 0.07, z1 - 0.08, z1 - 0.02, "chrome")

    # Vertical mullions & bay chrome caps
    for i in range(panes + 1):
        u = -half + i * pw
        if skip_door and -1.55 < u < 1.55:
            continue
        is_bay = (i % (1 if simple else SUB_PANES)) == 0
        mw = 0.06
        place_box(u - mw, u + mw, -0.10, 0.04, z0, z1, "metal")
        if is_bay and not simple:
            place_box(u - 0.035, u + 0.035, 0.03, 0.07, z0, z1, "chrome")

    # Individual recessed panes (0.1 m behind outer mullion face)
    for i in range(panes):
        u0 = -half + i * pw + 0.06
        u1 = -half + (i + 1) * pw - 0.06
        uc = (u0 + u1) / 2.0
        if skip_door and -1.5 < uc < 1.5:
            continue
        color, cracked = _pane_state(fi, i, variant)
        place_box(u0, u1, -0.12, -0.08, z0 + 0.06, z1 - 0.06, color)
        if cracked and not simple and color != "soot":
            off = sign * (half - 0.06)
            cx, cy = (uc, off) if along_x else (off, uc)
            _crack_bar(k, cx, cy, mid_z, along_x, length=min(pw * 0.8, 1.0))


def _spandrel_band(k, size, z0, z1, rust_seed=0, simple_back=True, roof=False):
    half = size / 2.0
    k.box((-half, -half, z0), (half, half, z1), "concrete_grey", roof=roof)
    bays = size // 4
    for i in range(bays):
        u = -half + (i + 0.5) * BAY_W
        if (i + rust_seed) % 3 == 0:
            k.box((u - 0.14, half - 0.01, z0 - 0.25), (u + 0.14, half + 0.03, z0 + 0.05), "rust", roof=roof)
        if (i + rust_seed) % 4 == 1:
            k.box((-half - 0.03, u - 0.12, z0 - 0.22), (-half + 0.01, u + 0.12, z0 + 0.05), "rust", roof=roof)
        if (i + rust_seed) % 4 == 2:
            k.box((half - 0.01, u - 0.12, z0 - 0.22), (half + 0.03, u + 0.12, z0 + 0.05), "rust", roof=roof)
        if not simple_back and (i + rust_seed) % 5 == 0:
            k.box((u - 0.14, -half - 0.03, z0 - 0.22), (u + 0.14, -half + 0.01, z0 + 0.05), "rust", roof=roof)


def _build_floor(size, variant):
    name = f"tw_glass_{size}_floor" + ("_b" if variant == "b" else "")

    def fn(model):
        k = kit.Kit(model)
        half = size / 2.0
        # Recessed dark inner core behind soot/glass slots
        k.box((-half + 0.14, -half + 0.14, SPANDREL_H), (half - 0.14, half - 0.14, FLOOR_H - SPANDREL_H), "soot")
        _spandrel_band(k, size, 0.0, SPANDREL_H, rust_seed=1 if variant == "b" else 0)
        _spandrel_band(k, size, FLOOR_H - SPANDREL_H, FLOOR_H, rust_seed=2 if variant == "b" else 1)
        for face in ("+Y", "+X", "-X"):
            _curtain_face(k, size, face, SPANDREL_H, FLOOR_H - SPANDREL_H, variant=variant)
        _curtain_face(k, size, "-Y", SPANDREL_H, FLOOR_H - SPANDREL_H, variant=variant, simple=True)

    return common.run_model(name, CATEGORY, BUDGET_FLOOR[size], fn, grip=(0.0, 0.0, 0.0), view=kit.FRONT_VIEW)


def _build_floor_lod1(size, variant):
    name = f"tw_glass_{size}_floor" + ("_b" if variant == "b" else "") + "_lod1"

    def fn(model):
        k = kit.Kit(model)
        half = size / 2.0
        k.box((-half, -half, 0.0), (half, half, SPANDREL_H), "concrete_grey")
        k.box((-half, -half, FLOOR_H - SPANDREL_H), (half, half, FLOOR_H), "concrete_grey")
        k.box((-half + 0.04, -half + 0.04, SPANDREL_H), (half - 0.04, half - 0.04, FLOOR_H - SPANDREL_H), "glass_tint")
        k.box((-half * 0.5, half - 0.02, SPANDREL_H + 0.2), (-half * 0.1, half + 0.04, FLOOR_H - SPANDREL_H - 0.2), "glass_dark")
        k.box((half * 0.2, half - 0.02, SPANDREL_H + 0.2), (half * 0.6, half + 0.04, FLOOR_H - SPANDREL_H - 0.2), "glass_dark")

    return common.run_model(name, CATEGORY, BUDGET_FLOOR[size] // 10, fn, grip=(0.0, 0.0, 0.0), view=kit.FRONT_VIEW)


def _build_base(size):
    name = f"tw_glass_{size}_base"

    def fn(model):
        k = kit.Kit(model)
        half = size / 2.0
        fy = shell.front_face_y(size)
        shell.lobby_shell(k, size, "concrete_grey")

        # Ground-floor curtain wall bands outside the 2.8 m door gap
        for face in ("+Y", "+X", "-X"):
            _curtain_face(k, size, face, 0.15, 3.6, variant="a", skip_door=(face == "+Y"))
            _curtain_face(k, size, face, 4.05, BASE_H - SPANDREL_H, variant="b")
        _curtain_face(k, size, "-Y", 4.05, BASE_H - SPANDREL_H, variant="b", simple=True)

        # Mid spandrel (z 3.6..4.05) and top spandrel (z 7.15..7.6)
        _spandrel_band(k, size, 3.6, 4.05, rust_seed=0, roof=True)
        _spandrel_band(k, size, BASE_H - SPANDREL_H, BASE_H - 0.01, rust_seed=1, roof=True)

        # Revolving door frame in the door gap (visual only, no collider so player walks through)
        k.box((-1.42, fy - 0.18, 0.1), (-1.34, fy + 0.15, 3.1), "metal")
        k.box((1.34, fy - 0.18, 0.1), (1.42, fy + 0.15, 3.1), "metal")
        k.box((-1.42, fy - 0.18, 3.02), (1.42, fy + 0.15, 3.12), "metal")
        # Curved drum header & broken central wing pushed to side
        k.box((-1.36, fy - 0.06, 2.95), (1.36, fy + 0.06, 3.04), "chrome")
        k.box((1.05, fy - 0.12, 0.12), (1.12, fy + 0.08, 2.95), "metal")
        k.box((1.12, fy - 0.02, 0.12), (1.34, fy + 0.04, 2.95), "metal")

        # Sign band (chrome, no words) above the lobby entrance
        k.box((-3.4, fy - 0.02, 3.18), (3.4, fy + 0.22, 3.62), "chrome")
        k.box((-3.2, fy + 0.18, 3.24), (3.2, fy + 0.24, 3.56), "glass_dark")

        # Corner concrete piers
        for sx in (-1.0, 1.0):
            for sy in (-1.0, 1.0):
                k.box((sx * (half - 0.22), sy * (half - 0.22), 0.0),
                      (sx * (half + 0.04), sy * (half + 0.04), BASE_H), "concrete_grey")

    return common.run_model(name, CATEGORY, BUDGET_BASE, fn, grip=(0.0, 0.0, 0.0),
                            top_preview=True, view=kit.FRONT_VIEW)


def _build_base_lod1(size):
    name = f"tw_glass_{size}_base_lod1"

    def fn(model):
        k = kit.Kit(model)
        half = size / 2.0
        k.box((-half, -half, 0.0), (half, half, BASE_H), "glass_tint")
        k.box((-half - 0.02, -half - 0.02, 3.6), (half + 0.02, half + 0.02, 4.05), "concrete_grey")
        k.box((-half - 0.02, -half - 0.02, BASE_H - SPANDREL_H), (half + 0.02, half + 0.02, BASE_H), "concrete_grey")
        k.box((-1.4, half - 0.05, 0.1), (1.4, half + 0.06, 3.1), "soot")

    return common.run_model(name, CATEGORY, BUDGET_BASE // 10, fn, grip=(0.0, 0.0, 0.0), view=kit.FRONT_VIEW)


def _build_crown(size):
    name = f"tw_glass_{size}_crown"

    def fn(model):
        k = kit.Kit(model)
        half = size / 2.0
        _spandrel_band(k, size, 0.0, SPANDREL_H, rust_seed=2)
        k.box((-half, -half, SPANDREL_H), (half, half, 0.65), "concrete_grey")
        k.box((-half - 0.04, -half - 0.04, 0.65), (half + 0.04, half + 0.04, 0.80), "chrome")

        # Glass penthouse (inset 3 m from perimeter, z 0.65..3.25)
        ph = half - 3.0
        pz0, pz1 = 0.65, 3.25
        k.box((-ph + 0.15, -ph + 0.15, pz0), (ph - 0.15, ph - 0.15, pz1 - 0.1), "soot")
        k.box((-ph - 0.1, -ph - 0.1, pz1 - 0.15), (ph + 0.1, ph + 0.1, pz1), "concrete_grey")
        k.box((-ph - 0.12, -ph - 0.12, pz1), (ph + 0.12, ph + 0.12, pz1 + 0.08), "chrome")

        # Penthouse mullions & panes (two panes gone on +Y, two bent mullions)
        cols = 6 if size == 24 else 8
        step = (2.0 * ph) / cols
        for i in range(cols):
            u0 = -ph + i * step
            u1 = u0 + step
            if i not in (2, 5):
                c = "glass_dark" if i % 3 == 0 else "glass_tint"
                k.box((u0 + 0.06, ph - 0.08, pz0), (u1 - 0.06, ph - 0.02, pz1 - 0.15), c)
            k.box((u0 + 0.06, -ph + 0.02, pz0), (u1 - 0.06, -ph + 0.08, pz1 - 0.15), "glass_dark")
            k.box((-ph + 0.02, u0 + 0.06, pz0), (-ph + 0.08, u1 - 0.06, pz1 - 0.15), "glass_tint")
            k.box((ph - 0.08, u0 + 0.06, pz0), (ph - 0.02, u1 - 0.06, pz1 - 0.15), "glass_dark")

        for i in range(cols + 1):
            u = -ph + i * step
            if i in (2, 5):
                # Bent mullions
                bm = common.make_box((0.08, 0.10, pz1 - pz0), color="metal")
                common.transform(bm, rot=(0.14, -0.18 if i == 2 else 0.15, 0.0),
                                 loc=(u, ph - 0.02, (pz0 + pz1) / 2.0))
                k.model.add(bm)
            else:
                k.box((u - 0.04, ph - 0.06, pz0), (u + 0.04, ph + 0.04, pz1), "metal")
            k.box((u - 0.04, -ph - 0.04, pz0), (u + 0.04, -ph + 0.06, pz1), "metal")
            k.box((-ph - 0.04, u - 0.04, pz0), (-ph + 0.06, u + 0.04, pz1), "metal")
            k.box((ph - 0.06, u - 0.04, pz0), (ph + 0.04, u + 0.04, pz1), "metal")

        # Roof plant box (metal) + shared roof_props / roof_parts
        k.box((-ph + 0.6, -ph + 0.6, pz1 + 0.08), (-ph + 3.2, -ph + 2.4, pz1 + 0.95), "metal")
        k.box((-ph + 0.7, -ph + 0.7, pz1 + 0.95), (-ph + 3.1, -ph + 2.3, pz1 + 1.05), "rust")
        roof_parts.vent_stack(k, -half + 1.6, half - 1.6, 0.65, 1.8)
        if size == 24:
            roof_props.roof_water_tank_ny(k, -4.8, 4.6, pz1 + 0.08, r=1.25, leg_h=1.20, tank_h=1.95)
            roof_props.roof_stair_hut(k, 4.8, -4.4, pz1 + 0.08)
            roof_props.roof_machinery(k, 0.2, 4.8, pz1 + 0.08)
            roof_props.roof_antenna(k, 5.2, 5.2, pz1 + 0.08, h=3.6)
            roof_parts.ac_unit(k, ph - 2.0, 0.2, pz1 + 0.08)
        else:
            roof_props.roof_water_tank_ny(k, -8.2, 7.6, pz1 + 0.08, r=1.35, leg_h=1.35, tank_h=2.15)
            roof_props.roof_water_tank_ny(k, 7.8, -7.8, pz1 + 0.08, r=1.25, leg_h=1.20, tank_h=1.95)
            roof_props.roof_stair_hut(k, 0.2, -7.8, pz1 + 0.08)
            roof_props.roof_machinery(k, -0.4, 7.8, pz1 + 0.08)
            roof_props.roof_antenna(k, 8.2, 8.0, pz1 + 0.08, h=4.2)
            roof_props.roof_antenna(k, -8.0, -3.5, pz1 + 0.08, h=3.4)
            roof_parts.ac_unit(k, ph - 2.2, 0.4, pz1 + 0.08)
            roof_parts.ac_unit(k, -ph + 2.4, 1.5, pz1 + 0.08)

    return common.run_model(name, CATEGORY, BUDGET_CROWN, fn, grip=(0.0, 0.0, 0.0), view=kit.FRONT_VIEW)


def _build_crown_lod1(size):
    name = f"tw_glass_{size}_crown_lod1"

    def fn(model):
        k = kit.Kit(model)
        half = size / 2.0
        ph = half - 3.0
        k.box((-half, -half, 0.0), (half, half, 0.8), "concrete_grey")
        k.box((-ph, -ph, 0.8), (ph, ph, 3.3), "glass_tint")
        if size == 24:
            model.add(common.make_frustum(1.25, 1.25, 3.3, 6.8, segments=6, center=(-4.8, 4.6), color="bark_light"))
            k.box((3.3, -6.1, 3.3), (6.3, -2.7, 5.8), "concrete_grey")
            k.box((-1.4, 3.7, 3.3), (1.8, 5.9, 5.0), "concrete_grey")
            k.box((5.0, 5.0, 3.3), (5.4, 5.4, 7.0), "metal")
        else:
            model.add(common.make_frustum(1.35, 1.35, 3.3, 7.2, segments=6, center=(-8.2, 7.6), color="bark_light"))
            model.add(common.make_frustum(1.25, 1.25, 3.3, 6.8, segments=6, center=(7.8, -7.8), color="bark_light"))
            k.box((-1.3, -9.5, 3.3), (1.7, -6.1, 5.8), "concrete_grey")
            k.box((-2.0, 6.7, 3.3), (1.2, 8.9, 5.0), "concrete_grey")
            k.box((8.0, 7.8, 3.3), (8.4, 8.2, 7.6), "metal")
            k.box((-8.2, -3.7, 3.3), (-7.8, -3.3, 6.8), "metal")

    return common.run_model(name, CATEGORY, BUDGET_CROWN // 10, fn, grip=(0.0, 0.0, 0.0), view=kit.FRONT_VIEW)


def _curtain_face_c(k, size, face, z0, z1, simple=False):
    """Damaged curtain-wall band for `_floor_c` with shattered crosses, missing frames, soot & rust."""
    half = size / 2.0
    bays = size // 4
    panes = bays * SUB_PANES if not simple else bays
    pw = size / panes
    along_x = face in ("+Y", "-Y")
    sign = 1.0 if face in ("+Y", "+X") else -1.0
    fi = {"+Y": 0, "+X": 1, "-X": 2, "-Y": 3}[face]
    face_coord = sign * half

    def place_box(u0, u1, d0, d1, zb, zt, color):
        damaged_parts.box_on_face(k, face_coord, 1 if along_x else 0, sign, u0, u1, zb, zt, d0, d1, color)

    mid_z = (z0 + z1) / 2.0
    # Transoms stop short at the (+X, +Y) torn corner bay
    top_u_hi = half - 1.2 if face in ("+Y", "+X") else half
    place_box(-half, half, -0.08, 0.04, z0, z0 + 0.06, "metal")
    place_box(-half, top_u_hi, -0.08, 0.04, z1 - 0.06, z1, "metal")
    if not simple:
        place_box(-half, top_u_hi, -0.06, 0.03, mid_z - 0.04, mid_z + 0.04, "metal")
    place_box(-half, half, 0.02, 0.07, z0 + 0.02, z0 + 0.08, "chrome")
    place_box(-half, top_u_hi, 0.02, 0.07, z1 - 0.08, z1 - 0.02, "chrome")

    miss_panes = {(0, 3), (0, 4), (1, 10), (1, 11)} if not simple else set()
    for i in range(panes + 1):
        u = -half + i * pw
        if (fi, i) in miss_panes or (fi, i - 1) in miss_panes:
            continue
        if face in ("+Y", "+X") and u > half - 1.15:
            continue
        is_bay = (i % (1 if simple else SUB_PANES)) == 0
        place_box(u - 0.06, u + 0.06, -0.10, 0.04, z0, z1, "metal")
        if is_bay and not simple:
            place_box(u - 0.035, u + 0.035, 0.03, 0.07, z0, z1, "chrome")

    for i in range(panes):
        u0 = -half + i * pw + 0.06
        u1 = -half + (i + 1) * pw - 0.06
        uc = (u0 + u1) / 2.0
        axis = 1 if along_x else 0
        if face in ("+Y", "+X") and uc > half - 1.15:
            continue
        if (fi, i) in ((0, 3), (1, 10)):
            damaged_parts.missing_frame(k, face_coord, axis, sign, uc, z0, z1, pw * 0.5, d_base=-0.09)
            damaged_parts.soot_streak(k, face_coord, axis, sign, uc, z1, width=pw * 0.9, d_base=0.01)
            continue
        if (fi, i) in ((0, 9), (1, 2), (2, 4), (0, 14)):
            damaged_parts.shattered_pane(k, face_coord, axis, sign, uc, z0, z1, pw * 0.46, d_base=-0.08)
            continue
        code = (fi * 17 + i * 11) % 10
        color = "soot" if code in (1, 5) else ("glass_dark" if code in (0, 3, 7) else "glass_tint")
        place_box(u0, u1, -0.12, -0.08, z0 + 0.06, z1 - 0.06, color)
        if not simple and code in (1, 7):
            damaged_parts.soot_streak(k, face_coord, axis, sign, uc, z1, width=pw * 0.85, d_base=0.01)
        if not simple and i % 3 == 0:
            damaged_parts.rust_streak(k, face_coord, axis, sign, uc, z0 + 0.06, width=0.38, d_base=0.01)


def _build_floor_c(size):
    name = f"tw_glass_{size}_floor_c"

    def fn(model):
        k = kit.Kit(model)
        half = size / 2.0
        k.box((-half + 0.14, -half + 0.14, 0.05), (half - 0.14, half - 0.14, FLOOR_H - 0.05), "soot")
        _spandrel_band(k, size, 0.0, SPANDREL_H, rust_seed=3)
        # Top spandrel with (+X, +Y) torn corner 0.35 m shorter revealing soot core
        k.box((-half, -half, FLOOR_H - SPANDREL_H), (half - 1.15, half, FLOOR_H), "concrete_grey")
        k.box((half - 1.15, -half, FLOOR_H - SPANDREL_H), (half, half - 1.15, FLOOR_H), "concrete_grey")
        k.box((half - 1.15, half - 1.15, FLOOR_H - 0.12), (half, half, FLOOR_H), "concrete_grey")
        k.box((half - 0.42, half - 0.20, FLOOR_H - 0.48), (half - 0.36, half - 0.14, FLOOR_H - 0.08), "rust_dark")
        for face in ("+Y", "+X", "-X"):
            _curtain_face_c(k, size, face, SPANDREL_H, FLOOR_H - SPANDREL_H)
        _curtain_face_c(k, size, "-Y", SPANDREL_H, FLOOR_H - SPANDREL_H, simple=True)

    return common.run_model(name, CATEGORY, BUDGET_FLOOR[size], fn, grip=(0.0, 0.0, 0.0), view=kit.FRONT_VIEW)


def _build_floor_c_lod1(size):
    name = f"tw_glass_{size}_floor_c_lod1"

    def fn(model):
        k = kit.Kit(model)
        half = size / 2.0
        k.box((-half, -half, 0.0), (half, half, SPANDREL_H), "concrete_grey")
        k.box((-half, -half, FLOOR_H - SPANDREL_H), (half, half, FLOOR_H), "concrete_grey")
        damaged_parts.damaged_wall_core(k, size, SPANDREL_H, FLOOR_H - SPANDREL_H, "glass_tint",
                                        patch=1.2, short_h=0.40, inset=0.04)
        for ox in (-half * 0.6, -half * 0.1, half * 0.4):
            k.box((ox - 1.1, half - 0.02, SPANDREL_H + 0.15), (ox + 1.1, half + 0.04, FLOOR_H - SPANDREL_H - 0.15), "soot")
        k.box((half - 0.02, -half * 0.3, SPANDREL_H + 0.15), (half + 0.04, half * 0.2, FLOOR_H - SPANDREL_H - 0.15), "glass_dark")

    return common.run_model(name, CATEGORY, BUDGET_FLOOR[size] // 10, fn, grip=(0.0, 0.0, 0.0), view=kit.FRONT_VIEW)


def build_r40_all():
    results = []
    for size in (24, 32):
        results.append((f"tw_glass_{size}_floor_c", _build_floor_c(size)))
        results.append((f"tw_glass_{size}_floor_c_lod1", _build_floor_c_lod1(size)))
        results.append((f"tw_glass_{size}_crown", _build_crown(size)))
        results.append((f"tw_glass_{size}_crown_lod1", _build_crown_lod1(size)))
    return results


def build_all():
    results = []
    for size in (24, 32):
        results.append((f"tw_glass_{size}_base", _build_base(size)))
        results.append((f"tw_glass_{size}_base_lod1", _build_base_lod1(size)))
        results.append((f"tw_glass_{size}_floor", _build_floor(size, "a")))
        results.append((f"tw_glass_{size}_floor_lod1", _build_floor_lod1(size, "a")))
        results.append((f"tw_glass_{size}_floor_b", _build_floor(size, "b")))
        results.append((f"tw_glass_{size}_floor_b_lod1", _build_floor_lod1(size, "b")))
        results.append((f"tw_glass_{size}_crown", _build_crown(size)))
        results.append((f"tw_glass_{size}_crown_lod1", _build_crown_lod1(size)))
    return results


if __name__ == "__main__":
    res = build_all()
    failed = [n for n, ok in res if not ok]
    for n, ok in res:
        print(f"glass: {n} {'OK' if ok else 'FAILED'}")
    if failed:
        sys.exit(1)
