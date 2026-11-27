"""Shared facade parts for the Brick (brownstone / tenement) tower kit."""
import math

import common
import kit


def add_window(k, face_coord, axis, sign, u_center, z0, z1, variant="normal",
               roof=False, detailed=True):
    """Brick window with brownstone surround, limestone lintel + mortar, sill, and damage.

    axis=1 for front/back (+Y/-Y), axis=0 for left/right (+X/-X).
    """
    w_half = 0.65
    win_lo = [0.0, 0.0, z0]
    win_hi = [0.0, 0.0, z1]
    u = 1 - axis
    win_lo[u] = u_center - w_half
    win_hi[u] = u_center + w_half
    win_lo[axis] = min(face_coord, face_coord - sign * 0.06)
    win_hi[axis] = max(face_coord, face_coord - sign * 0.06)

    def box_on_face(u0, u1, bz0, bz1, d0, d1, color):
        lo = [0.0, 0.0, bz0]
        hi = [0.0, 0.0, bz1]
        lo[u], hi[u] = u0, u1
        a, b = face_coord + sign * d0, face_coord + sign * d1
        lo[axis], hi[axis] = (min(a, b), max(a, b))
        k.box(tuple(lo), tuple(hi), color, roof=roof)

    # Glass pane or empty soot hole
    pane_color = "soot" if variant == "empty" else "glass_dark"
    box_on_face(u_center - w_half + 0.06, u_center + w_half - 0.06,
                z0 + 0.06, z1 - 0.06, 0.0, 0.025, pane_color)

    # Limestone lintel + mortar line + sill
    box_on_face(u_center - w_half - 0.10, u_center + w_half + 0.10,
                z1, z1 + 0.22, 0.0, 0.08, "limestone")
    box_on_face(u_center - w_half + 0.02, u_center + w_half - 0.02,
                z0 - 0.12, z0, 0.0, 0.10, "limestone")

    if detailed:
        # Mortar joint line across lintel
        box_on_face(u_center - w_half - 0.08, u_center + w_half + 0.08,
                    z1 + 0.10, z1 + 0.13, 0.07, 0.09, "mortar")
        # Brownstone side surrounds
        box_on_face(u_center - w_half - 0.08, u_center - w_half + 0.04,
                    z0, z1, 0.0, 0.05, "brownstone")
        box_on_face(u_center + w_half - 0.04, u_center + w_half + 0.08,
                    z0, z1, 0.0, 0.05, "brownstone")

    if variant == "boarded":
        k.boards((tuple(win_lo), tuple(win_hi)), face_coord, sign,
                 color="bark_light", rows=(0.22, 0.5, 0.78))
    elif variant == "ac":
        add_window_ac(k, face_coord, axis, sign, u_center, z0 + 0.04, roof=roof)
    elif variant == "cracked":
        # Diagonal crack strip across pane
        cx = u_center if axis == 1 else face_coord + sign * 0.032
        cy = face_coord + sign * 0.032 if axis == 1 else u_center
        bm = common.make_box((0.04, 0.015, (z1 - z0) * 0.85), color="soot")
        rot = (0.0, 0.55, 0.0) if axis == 1 else (0.55, 0.0, math.pi / 2)
        common.transform(bm, rot=rot, loc=(cx, cy, (z0 + z1) / 2))
        k.model.add(bm, roof=roof)

    if detailed and variant in ("ac", "rust", "empty"):
        # Rust streak running down below the sill
        box_on_face(u_center - 0.22, u_center + 0.18,
                    z0 - 0.65, z0 - 0.12, 0.0, 0.025, "rust_dark")


def add_window_ac(k, face_coord, axis, sign, u_center, z0, roof=False):
    """Window AC unit (0.6 x 0.5 x 0.5 m) with dark grille and brackets."""
    u = 1 - axis

    def box_rel(u0, u1, bz0, bz1, d0, d1, color):
        lo = [0.0, 0.0, bz0]
        hi = [0.0, 0.0, bz1]
        lo[u], hi[u] = u0, u1
        a, b = face_coord + sign * d0, face_coord + sign * d1
        lo[axis], hi[axis] = (min(a, b), max(a, b))
        k.box(tuple(lo), tuple(hi), color, roof=roof)

    box_rel(u_center - 0.30, u_center + 0.30, z0, z0 + 0.50, 0.0, 0.50, "metal")
    box_rel(u_center - 0.24, u_center + 0.24, z0 + 0.06, z0 + 0.44, 0.49, 0.53, "rock_dark")
    box_rel(u_center - 0.22, u_center - 0.16, z0 - 0.18, z0, 0.0, 0.40, "rust_dark")
    box_rel(u_center + 0.16, u_center + 0.22, z0 - 0.18, z0, 0.0, 0.40, "rust_dark")


def add_fire_escape_floor(k, face_coord, axis, sign, u_center, z0, z1,
                          zig_dir=1, has_landing=True, roof=False):
    """Iron fire escape landing (1.2 m deep) with rail and zig-zag stair between floors."""
    if not has_landing:
        return
    u = 1 - axis
    half_len = 2.4
    depth = 1.15  # within 1.2 m overhang limit

    def box_rel(u0, u1, bz0, bz1, d0, d1, color):
        lo = [0.0, 0.0, bz0]
        hi = [0.0, 0.0, bz1]
        lo[u], hi[u] = u0, u1
        a, b = face_coord + sign * d0, face_coord + sign * d1
        lo[axis], hi[axis] = (min(a, b), max(a, b))
        k.box(tuple(lo), tuple(hi), color, roof=roof)

    # Landing deck + support brackets
    box_rel(u_center - half_len, u_center + half_len, z0, z0 + 0.07, 0.0, depth, "metal")
    for bu in (u_center - half_len + 0.2, u_center, u_center + half_len - 0.2):
        box_rel(bu - 0.04, bu + 0.04, z0 - 0.32, z0, 0.0, depth * 0.95, "rust_dark")

    # Outer & end rails
    box_rel(u_center - half_len, u_center + half_len, z0 + 0.85, z0 + 0.92,
            depth - 0.06, depth, "metal")
    box_rel(u_center - half_len, u_center + half_len, z0 + 0.44, z0 + 0.50,
            depth - 0.05, depth, "rust_dark")
    for pu in (u_center - half_len, u_center - half_len * 0.5, u_center,
               u_center + half_len * 0.5, u_center + half_len):
        box_rel(pu - 0.03, pu + 0.03, z0 + 0.07, z0 + 0.92, depth - 0.06, depth, "metal")
    for eu in (u_center - half_len, u_center + half_len):
        box_rel(eu - 0.03, eu + 0.03, z0 + 0.85, z0 + 0.92, 0.0, depth, "metal")

    # Zig-zag diagonal stair from z0 to z1 - 0.10 (kept within module height z0..z1)
    span = 3.2
    sz0, sz1 = z0 + 0.05, z1 - 0.12
    length = math.hypot(span, sz1 - sz0)
    ang = math.atan2(sz1 - sz0, span) * zig_dir
    rz0, rz1 = min(z0 + 0.75, sz1 - 0.2), z1 - 0.06
    r_len = math.hypot(span, rz1 - rz0)
    r_ang = math.atan2(rz1 - rz0, span) * zig_dir
    if axis == 1:
        loc = (u_center, face_coord + sign * (depth * 0.52), (sz0 + sz1) / 2)
        rot = (0.0, -ang, 0.0)
        stair_size = (length, 0.48, 0.08)
        rail_loc = (u_center, face_coord + sign * (depth * 0.78), (rz0 + rz1) / 2)
        rail_rot = (0.0, -r_ang, 0.0)
        rail_size = (r_len, 0.04, 0.05)
    else:
        loc = (face_coord + sign * (depth * 0.52), u_center, (sz0 + sz1) / 2)
        rot = (ang, 0.0, 0.0)
        stair_size = (0.48, length, 0.08)
        rail_loc = (face_coord + sign * (depth * 0.78), u_center, (rz0 + rz1) / 2)
        rail_rot = (r_ang, 0.0, 0.0)
        rail_size = (0.04, r_len, 0.05)

    stair_bm = common.make_box(stair_size, color="rust_dark")
    common.transform(stair_bm, rot=rot, loc=loc)
    k.model.add(stair_bm, roof=roof)

    rail_bm = common.make_box(rail_size, color="metal")
    common.transform(rail_bm, rot=rail_rot, loc=rail_loc)
    k.model.add(rail_bm, roof=roof)

    # Walkable fire-escape colliders (landing slab, outer/end guard walls, <= 45 deg ramp)
    add_fire_escape_colliders(k, face_coord, axis, sign, u_center, z0, z1, zig_dir=zig_dir)


def add_fire_escape_colliders(k, face_coord, axis, sign, u_center, z0, z1=None, zig_dir=1):
    """Colliders for one fire-escape landing (+ optional zig-zag ramp <= 43 deg) and railings."""
    u = 1 - axis
    half_len = 2.4
    depth = 1.15

    def col_rel(u0, u1, bz0, bz1, d0, d1, base="fe"):
        lo = [0.0, 0.0, bz0]
        hi = [0.0, 0.0, bz1]
        lo[u], hi[u] = u0, u1
        a, b = face_coord + sign * d0, face_coord + sign * d1
        lo[axis], hi[axis] = (min(a, b), max(a, b))
        k.collider(tuple(lo), tuple(hi), base)

    # Landing slab + outer/end railing walls
    col_rel(u_center - half_len, u_center + half_len, z0, z0 + 0.07, 0.0, depth, "fe_land")
    col_rel(u_center - half_len, u_center + half_len, z0 + 0.07, z0 + 0.95, depth - 0.08, depth, "fe_rail")
    for eu in (u_center - half_len, u_center + half_len):
        col_rel(eu - 0.04, eu + 0.04, z0 + 0.07, z0 + 0.95, 0.0, depth, "fe_rail")

    if z1 is None:
        return
    # Ramp from current landing top (z0 + 0.07) to next module's landing top (z1 + 0.12)
    # Run 4.10 m, rise 3.80 m -> ~42.82 deg (under 46 deg player slope limit).
    half_run = 2.05 * zig_dir
    d_mid = face_coord + sign * (depth * 0.52)
    if axis == 1:
        a = (u_center - half_run, d_mid, z0 + 0.07)
        b = (u_center + half_run, d_mid, z1 + 0.12)
        span_vec = (0.0, 1.0, 0.0)
    else:
        a = (d_mid, u_center - half_run, z0 + 0.07)
        b = (d_mid, u_center + half_run, z1 + 0.12)
        span_vec = (1.0, 0.0, 0.0)
    k.model.add_collider(k._label("fe_ramp"), kit._plane_bm(a, b, span_vec, 0.52, 0.10, "white"))
