"""Shared facade and crown parts for the Stone (pre-war office) tower kit."""
import math

import common
import kit


def add_pilasters(k, size, z0, z1, broken_tops=False, roof=False):
    """Full-height limestone pilasters (0.4 m proud) between every 4 m bay."""
    h = size / 2
    bays = size // 4
    positions = [-h + 4.0 * i for i in range(bays + 1)]
    for idx, p in enumerate(positions):
        top = z1 - 0.55 if (broken_tops and idx in (1, bays - 1)) else z1
        # Front (+Y) and Back (-Y)
        k.box((p - 0.25, h - 0.05, z0), (p + 0.25, h + 0.40, top), "limestone", roof=roof)
        k.box((p - 0.25, -h - 0.40, z0), (p + 0.25, -h + 0.05, z1), "limestone", roof=roof)
    for idx, p in enumerate(positions[1:-1], start=1):
        top = z1 - 0.55 if (broken_tops and idx == 2) else z1
        # Sides (+X, -X)
        k.box((h - 0.05, p - 0.25, z0), (h + 0.40, p + 0.25, top), "limestone", roof=roof)
        k.box((-h - 0.40, p - 0.25, z0), (-h + 0.05, p + 0.25, top), "limestone", roof=roof)


def add_stone_window(k, face_coord, axis, sign, u_center, z0, z1, variant="tint",
                     has_balcony=False, roof=False, detailed=True):
    """Pre-war office window with limestone_dark frame, glass pane, grey mullion, optional balcony."""
    w_half = 1.05
    u = 1 - axis

    def box_rel(u0, u1, bz0, bz1, d0, d1, color):
        lo = [0.0, 0.0, bz0]
        hi = [0.0, 0.0, bz1]
        lo[u], hi[u] = u0, u1
        a, b = face_coord + sign * d0, face_coord + sign * d1
        lo[axis], hi[axis] = (min(a, b), max(a, b))
        k.box(tuple(lo), tuple(hi), color, roof=roof)

    pane_col = "glass_dark" if variant in ("dead", "cracked") else "glass_tint"
    box_rel(u_center - w_half + 0.08, u_center + w_half - 0.08,
            z0 + 0.08, z1 - 0.08, 0.0, 0.025, pane_col)

    # Frame in limestone_dark
    box_rel(u_center - w_half - 0.08, u_center + w_half + 0.08, z1, z1 + 0.14, 0.0, 0.09, "limestone_dark")
    box_rel(u_center - w_half - 0.10, u_center + w_half + 0.10, z0 - 0.12, z0, 0.0, 0.12, "limestone_dark")
    if detailed:
        box_rel(u_center - w_half - 0.06, u_center - w_half + 0.06, z0, z1, 0.0, 0.07, "limestone_dark")
        box_rel(u_center + w_half - 0.06, u_center + w_half + 0.06, z0, z1, 0.0, 0.07, "limestone_dark")
        # Grey cross mullions
        box_rel(u_center - 0.04, u_center + 0.04, z0 + 0.06, z1 - 0.06, 0.01, 0.05, "rock_mid")
        box_rel(u_center - w_half + 0.06, u_center + w_half - 0.06,
                (z0 + z1) / 2 - 0.04, (z0 + z1) / 2 + 0.04, 0.01, 0.05, "rock_mid")

    if variant == "boarded":
        win_lo = [0.0, 0.0, z0]
        win_hi = [0.0, 0.0, z1]
        win_lo[u], win_hi[u] = u_center - w_half, u_center + w_half
        win_lo[axis] = min(face_coord, face_coord - sign * 0.05)
        win_hi[axis] = max(face_coord, face_coord - sign * 0.05)
        k.boards((tuple(win_lo), tuple(win_hi)), face_coord, sign,
                 color="bark_light", rows=(0.25, 0.55, 0.82))
    elif variant == "cracked":
        cx = u_center if axis == 1 else face_coord + sign * 0.035
        cy = face_coord + sign * 0.035 if axis == 1 else u_center
        bm = common.make_box((0.04, 0.015, (z1 - z0) * 0.9), color="soot")
        rot = (0.0, -0.5, 0.0) if axis == 1 else (-0.5, 0.0, math.pi / 2)
        common.transform(bm, rot=rot, loc=(cx, cy, (z0 + z1) / 2))
        k.model.add(bm, roof=roof)

    if has_balcony:
        # Metal balcony rail projecting 0.5 m proud
        box_rel(u_center - w_half - 0.1, u_center + w_half + 0.1, z0 - 0.12, z0, 0.0, 0.52, "limestone_dark")
        box_rel(u_center - w_half - 0.1, u_center + w_half + 0.1, z0 + 0.82, z0 + 0.88, 0.46, 0.52, "metal")
        for bu in (u_center - w_half, u_center - 0.35, u_center + 0.35, u_center + w_half):
            box_rel(bu - 0.03, bu + 0.03, z0, z0 + 0.85, 0.46, 0.52, "metal")


def add_dentil_cornice(k, size, z_bot):
    """Heavy projecting cornice (z_bot..z_bot+1.0) with a row of dentil blocks underneath."""
    h = size / 2
    k.box((-h - 0.25, -h - 0.25, z_bot), (h + 0.25, h + 0.25, z_bot + 0.28), "limestone_dark")
    k.box((-h - 0.72, -h - 0.72, z_bot + 0.52), (h + 0.72, h + 0.72, z_bot + 1.0), "limestone")
    # Dentil row under the overhang on all 4 sides
    step = 0.9
    pos = -h - 0.4
    while pos <= h + 0.4:
        k.box((pos - 0.16, h + 0.25, z_bot + 0.28), (pos + 0.16, h + 0.62, z_bot + 0.52), "limestone_dark")
        k.box((pos - 0.16, -h - 0.62, z_bot + 0.28), (pos + 0.16, -h - 0.25, z_bot + 0.52), "limestone_dark")
        k.box((h + 0.25, pos - 0.16, z_bot + 0.28), (h + 0.62, pos + 0.16, z_bot + 0.52), "limestone_dark")
        k.box((-h - 0.62, pos - 0.16, z_bot + 0.28), (-h - 0.25, pos + 0.16, z_bot + 0.52), "limestone_dark")
        pos += step


def add_balustrade(k, size, z_bot, z_top):
    """Limestone parapet balustrade around roof perimeter."""
    h = size / 2
    rail_t = 0.28
    # Top and bottom rails
    for (lo, hi) in (
        ((-h, h - rail_t, z_bot), (h, h, z_bot + 0.22)),
        ((-h, -h, z_bot), (h, -h + rail_t, z_bot + 0.22)),
        ((-h, -h + rail_t, z_bot), (-h + rail_t, h - rail_t, z_bot + 0.22)),
        ((h - rail_t, -h + rail_t, z_bot), (h, h - rail_t, z_bot + 0.22)),
        ((-h, h - rail_t, z_top - 0.22), (h, h, z_top)),
        ((-h, -h, z_top - 0.22), (h, -h + rail_t, z_top)),
        ((-h, -h + rail_t, z_top - 0.22), (-h + rail_t, h - rail_t, z_top)),
        ((h - rail_t, -h + rail_t, z_top - 0.22), (h, h - rail_t, z_top)),
    ):
        k.box(lo, hi, "limestone_dark")

    # Corner piers + intermediate balusters every 1.2 m
    for cx in (-h + 0.2, h - 0.2):
        for cy in (-h + 0.2, h - 0.2):
            k.box((cx - 0.3, cy - 0.3, z_bot), (cx + 0.3, cy + 0.3, z_top + 0.2), "limestone")
    pos = -h + 1.2
    while pos <= h - 1.2:
        k.box((pos - 0.09, h - rail_t + 0.04, z_bot + 0.22),
              (pos + 0.09, h - 0.04, z_top - 0.22), "limestone")
        k.box((pos - 0.09, -h + 0.04, z_bot + 0.22),
              (pos + 0.09, -h + rail_t - 0.04, z_top - 0.22), "limestone")
        k.box((h - rail_t + 0.04, pos - 0.09, z_bot + 0.22),
              (h - 0.04, pos + 0.09, z_top - 0.22), "limestone")
        k.box((-h + 0.04, pos - 0.09, z_bot + 0.22),
              (-h + rail_t - 0.04, pos + 0.09, z_top - 0.22), "limestone")
        pos += 1.2


def add_flagpole(k, x, y, z0, h=4.2):
    """Thin metal flagpole with a torn red_paint flag."""
    k.model.add(common.make_frustum(0.06, 0.04, z0, z0 + h, segments=6, center=(x, y), color="metal"))
    k.model.add(common.make_frustum(0.11, 0.11, z0 + h, z0 + h + 0.14, segments=6, center=(x, y), color="brass"))
    # Torn flag (3 ragged strips)
    k.box((x, y - 0.02, z0 + h - 0.75), (x + 1.15, y + 0.02, z0 + h - 0.15), "red_paint")
    k.box((x + 1.15, y - 0.02, z0 + h - 0.55), (x + 1.55, y + 0.02, z0 + h - 0.18), "red_paint")
    k.box((x + 1.15, y - 0.02, z0 + h - 0.85), (x + 1.40, y + 0.02, z0 + h - 0.62), "red_paint")
