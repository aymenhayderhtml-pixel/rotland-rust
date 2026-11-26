"""Shared facade damage helpers for R40 `_floor_c` tower modules."""
import math

import common


def box_on_face(k, face_coord, axis, sign, u0, u1, z0, z1, d0, d1, color, roof=False):
    """Places an axis-aligned box relative to a wall face (`axis=1` for +/-Y, `0` for +/-X)."""
    u = 1 - axis
    lo = [0.0, 0.0, min(z0, z1)]
    hi = [0.0, 0.0, max(z0, z1)]
    lo[u], hi[u] = min(u0, u1), max(u0, u1)
    a, b = face_coord + sign * d0, face_coord + sign * d1
    lo[axis], hi[axis] = min(a, b), max(a, b)
    k.box(tuple(lo), tuple(hi), color, roof=roof)


def shattered_pane(k, face_coord, axis, sign, u_center, z0, z1, w_half, d_base=0.0, roof=False):
    """Dark soot hole with a thin `glass_dark` diagonal cross and jagged corner shards."""
    uw = max(0.20, w_half - 0.07)
    zh = max(0.30, (z1 - z0) - 0.14)
    box_on_face(k, face_coord, axis, sign,
                u_center - uw, u_center + uw, z0 + 0.07, z1 - 0.07,
                d_base, d_base + 0.022, "soot", roof=roof)

    diag = math.hypot(uw * 1.90, zh)
    ang = math.atan2(uw * 1.90, zh)
    d_mid = face_coord + sign * (d_base + 0.032)
    z_mid = (z0 + z1) * 0.5
    for sgn in (-1.0, 1.0):
        if axis == 1:
            bm = common.make_box((0.065, 0.020, diag), color="glass_dark")
            common.transform(bm, rot=(0.0, sgn * ang, 0.0), loc=(u_center, d_mid, z_mid))
        else:
            bm = common.make_box((0.020, 0.065, diag), color="glass_dark")
            common.transform(bm, rot=(sgn * ang, 0.0, 0.0), loc=(d_mid, u_center, z_mid))
        k.model.add(bm, roof=roof)

    # Jagged remaining corner shards
    box_on_face(k, face_coord, axis, sign,
                u_center - uw, u_center - uw * 0.35, z0 + 0.07, z0 + 0.07 + zh * 0.24,
                d_base + 0.012, d_base + 0.030, "glass_dark", roof=roof)
    box_on_face(k, face_coord, axis, sign,
                u_center + uw * 0.40, u_center + uw, z1 - 0.07 - zh * 0.22, z1 - 0.07,
                d_base + 0.012, d_base + 0.030, "glass_dark", roof=roof)


def missing_frame(k, face_coord, axis, sign, u_center, z0, z1, w_half, d_base=0.0, roof=False):
    """Bare soot opening with torn/bent `metal` and `rust_dark` frame remnants."""
    box_on_face(k, face_coord, axis, sign,
                u_center - w_half, u_center + w_half, z0, z1,
                d_base, d_base + 0.026, "soot", roof=roof)
    # Partial torn top & right metal frame strips
    box_on_face(k, face_coord, axis, sign,
                u_center - w_half - 0.03, u_center + w_half * 0.25, z1 - 0.02, z1 + 0.06,
                d_base, d_base + 0.060, "metal", roof=roof)
    box_on_face(k, face_coord, axis, sign,
                u_center + w_half - 0.04, u_center + w_half + 0.04, z0 + 0.28, z1 - 0.14,
                d_base, d_base + 0.055, "metal", roof=roof)
    box_on_face(k, face_coord, axis, sign,
                u_center - w_half - 0.03, u_center - w_half * 0.15, z0 - 0.04, z0 + 0.04,
                d_base, d_base + 0.050, "rust_dark", roof=roof)

    # One dangling bent left-jamb metal piece
    stub_h = max(0.35, (z1 - z0) * 0.45)
    d_mid = face_coord + sign * (d_base + 0.045)
    u_pos = u_center - w_half + 0.04
    if axis == 1:
        bm = common.make_box((0.055, 0.05, stub_h), color="metal")
        common.transform(bm, rot=(0.0, -0.22 * sign, 0.0), loc=(u_pos, d_mid, z0 + stub_h * 0.55))
    else:
        bm = common.make_box((0.05, 0.055, stub_h), color="metal")
        common.transform(bm, rot=(0.22 * sign, 0.0, 0.0), loc=(d_mid, u_pos, z0 + stub_h * 0.55))
    k.model.add(bm, roof=roof)


def soot_streak(k, face_coord, axis, sign, u_center, z_top, width=0.95, drop=0.80,
                d_base=0.0, roof=False):
    """Soot plume streaking above and down from the top of a window opening."""
    hw = width * 0.5
    box_on_face(k, face_coord, axis, sign,
                u_center - hw, u_center + hw, z_top - 0.14, z_top + drop * 0.60,
                d_base + 0.012, d_base + 0.036, "soot", roof=roof)
    box_on_face(k, face_coord, axis, sign,
                u_center - hw * 0.45 + 0.10, u_center + hw * 0.45 + 0.10,
                z_top - drop * 0.55, z_top + drop * 0.15,
                d_base + 0.015, d_base + 0.040, "soot", roof=roof)
    box_on_face(k, face_coord, axis, sign,
                u_center - hw * 0.85, u_center - hw * 0.35, z_top - 0.22, z_top + drop * 0.42,
                d_base + 0.012, d_base + 0.034, "soot", roof=roof)


def rust_streak(k, face_coord, axis, sign, u_center, z_bot, width=0.52, drop=0.72,
                d_base=0.0, roof=False):
    """Rust stain running down below a window sill."""
    hw = width * 0.5
    box_on_face(k, face_coord, axis, sign,
                u_center - hw, u_center + hw, z_bot - 0.08 - drop * 0.72, z_bot - 0.08,
                d_base + 0.012, d_base + 0.030, "rust_dark", roof=roof)
    box_on_face(k, face_coord, axis, sign,
                u_center - hw * 0.35 + 0.06, u_center + hw * 0.35 + 0.06,
                z_bot - 0.08 - drop, z_bot - 0.08 - drop * 0.30,
                d_base + 0.012, d_base + 0.028, "rust_dark", roof=roof)


def damaged_wall_core(k, size, z_bot, z_top, wall_color, trim_color=None,
                      patch=1.15, short_h=0.38, inset=0.0, roof=False):
    """Wall core with a torn-out (+X, +Y) corner 0.38 m shorter, exposing `soot` inside."""
    h = size / 2.0 - inset
    # Two L-shaped wall blocks leaving [h-patch..h] x [h-patch..h] open at top
    k.box((-h, -h, z_bot), (h - patch, h, z_top), wall_color, roof=roof)
    k.box((h - patch, -h, z_bot), (h, h - patch, z_top), wall_color, roof=roof)
    # Recessed soot core inside the corner so stacks have no see-through hole
    k.box((h - patch - 0.05, h - patch - 0.05, z_bot),
          (h - 0.28, h - 0.28, z_top), "soot", roof=roof)
    # Shortened outer corner wall chunk (0.38 m shorter than z_top)
    cut_z = max(z_bot + 0.20, z_top - short_h)
    k.box((h - patch, h - patch, z_bot), (h, h, cut_z), wall_color, roof=roof)
    # Jagged broken edge chunks and exposed rebar stubs
    edge_col = trim_color or wall_color
    k.box((h - patch, h - 0.35, cut_z), (h - patch * 0.45, h, cut_z + 0.14), edge_col, roof=roof)
    k.box((h - 0.35, h - patch, cut_z), (h, h - patch * 0.50, cut_z + 0.10), edge_col, roof=roof)
    k.box((h - 0.44, h - 0.22, cut_z - 0.02), (h - 0.38, h - 0.16, z_top - 0.03), "rust_dark", roof=roof)
    k.box((h - 0.22, h - 0.48, cut_z - 0.02), (h - 0.16, h - 0.42, z_top - 0.06), "rust_dark", roof=roof)
