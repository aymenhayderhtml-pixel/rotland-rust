"""Wall decals for survivor traces: 6 graffiti warnings and the missing-person poster wall.

All decals lie in the XZ plane, 2-3 cm off the wall at +Y, with origin at bottom centre on the wall (y=0).
"""
import os
import random
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, ".."))

import common  # noqa: E402
from graffiti_font import quad_bar, text_bms  # noqa: E402

GRAFFITI_BUDGET = 300  # new: DECISIONS 2026-11-26
POSTER_BUDGET = 1500  # new: DECISIONS 2026-11-26
DECAL_VIEW = (0.25, 1.0, 0.15)  # new: DECISIONS 2026-11-26 (+Y front view)

# new: DECISIONS 2026-11-26 (text decals: name, text, letter_w, letter_h, drip_xs)
TEXT_SIGNS = [
    ("graffiti_dead_inside", "DEAD INSIDE", 0.36, 0.62, (-1.8, -0.3, 1.5)),
    ("graffiti_turn_back", "TURN BACK", 0.42, 0.65, (-1.5, 0.2, 1.6)),
    ("graffiti_no_food", "NO FOOD", 0.46, 0.70, (-1.2, 0.4, 1.3)),
    ("graffiti_went_north", "WE WENT NORTH", 0.32, 0.56, (-1.9, 0.0, 1.7)),
]


def _drip_quad(x, z_top, z_bot, width, y):
    """Thin vertical soot paint drip quad facing +Y."""
    return quad_bar(x, z_bot, x, z_top, y, width, "soot")


def _build_text_graffiti(text, letter_w, letter_h, drip_xs):
    base_z = 0.22  # new: DECISIONS 2026-11-26 (leaves room for drips down to z=0)

    def build(model):
        for bm in text_bms(text, 0.0, base_z, letter_w, letter_h, 0.023, "white",
                          flat=True, thick_scale=1.45):
            model.add(bm)
        for bm in text_bms(text, 0.0, base_z, letter_w, letter_h, 0.028, "red_paint",
                          flat=True, thick_scale=1.0):
            model.add(bm)
        for idx, dx in enumerate(drip_xs):
            z_bot = 0.0 + (idx % 2) * 0.05  # new: DECISIONS 2026-11-26
            model.add(_drip_quad(dx, base_z + 0.06, z_bot, 0.038, 0.025))

    return build


def _build_graffiti_x(model):
    """Large painted X warning (two crossed bars with white outline and soot drips)."""
    hw, z0, z1 = 1.05, 0.25, 2.25  # new: DECISIONS 2026-11-26
    strokes = [(-hw, z0, hw, z1), (-hw, z1, hw, z0)]
    for x0, za, x1, zb in strokes:
        model.add(quad_bar(x0, za, x1, zb, 0.023, 0.24, "white"))
        model.add(quad_bar(x0, za, x1, zb, 0.028, 0.16, "red_paint"))
    for dx, zb in ((-0.75, 0.02), (0.0, 0.08), (0.85, 0.0)):
        model.add(_drip_quad(dx, 0.36 if dx else 1.25, zb, 0.045, 0.025))


def _build_graffiti_arrow(model):
    """Wide survivor direction arrow pointing right (-X from +Y view) with head and drips."""
    # From +Y view, -X is right and +X is left; tail at +1.8, tip at -1.8 (4.0 m span)
    z_mid = 0.85  # new: DECISIONS 2026-11-26
    strokes = [
        (1.85, z_mid, -1.85, z_mid),
        (-1.85, z_mid, -1.05, z_mid + 0.55),
        (-1.85, z_mid, -1.05, z_mid - 0.55),
    ]
    for x0, za, x1, zb in strokes:
        model.add(quad_bar(x0, za, x1, zb, 0.023, 0.22, "white"))
        model.add(quad_bar(x0, za, x1, zb, 0.028, 0.14, "red_paint"))
    for dx, zt, zb in ((1.1, z_mid - 0.05, 0.12), (-0.4, z_mid - 0.05, 0.0), (-1.15, z_mid - 0.45, 0.05)):
        model.add(_drip_quad(dx, zt, zb, 0.045, 0.025))


def _make_poster(paper_col, w, h):
    """Single missing-person poster in local XZ centered at (0, 0), facing +Y."""
    parts = []
    # Paper backing
    bm = common.new_bm()
    common.add_quad(bm, [(-w / 2, 0.0, -h / 2), (w / 2, 0.0, -h / 2),
                         (w / 2, 0.0, h / 2), (-w / 2, 0.0, h / 2)],
                    outward=(0.0, 1.0, 0.0), color=paper_col)
    parts.append(bm)
    # Two small tape tabs at top corners
    for sx in (-1, 1):
        parts.append(quad_bar(sx * (w * 0.34), h * 0.46, sx * (w * 0.44), h * 0.52, 0.002, 0.025, "canvas_khaki"))
    # "MISSING" red header
    parts.extend(text_bms("MISSING", 0.0, h * 0.31, w * 0.088, h * 0.12, 0.002, "red_paint",
                          flat=True, thick_scale=1.15))
    # Grey photo frame
    pw, ph = w * 0.56, h * 0.42  # new: DECISIONS 2026-11-26
    pz = h * 0.02
    photo_col = "rock_dark" if paper_col == "concrete_grey" else "rock_mid"
    pbm = common.new_bm()
    common.add_quad(pbm, [(-pw / 2, 0.002, pz - ph / 2), (pw / 2, 0.002, pz - ph / 2),
                          (pw / 2, 0.002, pz + ph / 2), (-pw / 2, 0.002, pz + ph / 2)],
                    outward=(0.0, 1.0, 0.0), color=photo_col)
    parts.append(pbm)
    # Generic head-and-shoulders silhouette (head octagon + shoulders trapezoid)
    sbm = common.new_bm()
    head_z, hr = pz + ph * 0.14, pw * 0.22  # new: DECISIONS 2026-11-26
    head_pts = [(-hr * 0.7, 0.004, head_z - hr * 0.8), (hr * 0.7, 0.004, head_z - hr * 0.8),
                (hr, 0.004, head_z), (hr * 0.7, 0.004, head_z + hr * 0.9),
                (-hr * 0.7, 0.004, head_z + hr * 0.9), (-hr, 0.004, head_z)]
    common.add_quad(sbm, head_pts, outward=(0.0, 1.0, 0.0), color="soot")
    sh_bot, sh_top = pz - ph * 0.46, pz - ph * 0.06
    common.add_quad(sbm, [(-pw * 0.36, 0.004, sh_bot), (pw * 0.36, 0.004, sh_bot),
                          (pw * 0.22, 0.004, sh_top), (-pw * 0.22, 0.004, sh_top)],
                    outward=(0.0, 1.0, 0.0), color="soot")
    parts.append(sbm)
    # Blurred unreadable info lines underneath
    for k, line_w in enumerate((0.68, 0.56, 0.44)):
        lz = -h * 0.26 - k * (h * 0.075)
        parts.append(quad_bar(-w * line_w * 0.5, lz, w * line_w * 0.5, lz, 0.002, h * 0.025, "rock_dark"))
    return common.combine(parts)


def _build_poster_wall(model):
    """2.0 x 1.5 m board with 5 tilted missing-person posters 2 cm off the board."""
    rng = random.Random(405)
    # Board sits 1 cm off the wall (y = 0.01..0.025), posters 2 cm off board (y = 0.045)
    bw, bh = 2.0, 1.5  # new: DECISIONS 2026-11-26
    board = common.make_box((bw, 0.015, bh), loc=(0.0, 0.0175, bh / 2), color="bark_dark")
    model.add(board)
    # Trim border around the board
    for sx in (-1, 1):
        model.add(common.make_box((0.05, 0.022, bh), loc=(sx * (bw / 2 - 0.025), 0.021, bh / 2), color="wood_crate"))
    for z in (0.025, bh - 0.025):
        model.add(common.make_box((bw, 0.022, 0.05), loc=(0.0, 0.021, z), color="wood_crate"))

    # 5 posters clustered on the board, 2 cm proud of board front (y = 0.045)
    specs = [
        (-0.56, 1.04, 0.06, "white", 0.38, 0.52),
        (0.02, 1.08, -0.04, "bone_white", 0.40, 0.54),
        (0.58, 1.00, 0.08, "concrete_grey", 0.36, 0.50),
        (-0.32, 0.46, -0.07, "concrete_grey", 0.38, 0.52),
        (0.34, 0.44, 0.05, "white", 0.40, 0.52),
    ]
    for idx, (px, pz, tilt, col, pw, ph) in enumerate(specs):
        poster = _make_poster(col, pw, ph)
        common.transform(poster, rot=(0.0, tilt + rng.uniform(-0.02, 0.02), 0.0),
                         loc=(px, 0.045 + idx * 0.002, pz))
        model.add(poster)


def build_all():
    """Builds the 6 graffiti decals and poster_wall. Returns [(name, ok)]."""
    results = []
    for name, text, lw, lh, drips in TEXT_SIGNS:
        ok = common.run_model(name, "props", GRAFFITI_BUDGET,
                              _build_text_graffiti(text, lw, lh, drips),
                              grip=(0.0, 0.0, 0.0), view=DECAL_VIEW)
        results.append((name, ok))
    results.append(("graffiti_x", common.run_model("graffiti_x", "props", GRAFFITI_BUDGET,
                                                   _build_graffiti_x, grip=(0.0, 0.0, 0.0), view=DECAL_VIEW)))
    results.append(("graffiti_arrow", common.run_model("graffiti_arrow", "props", GRAFFITI_BUDGET,
                                                       _build_graffiti_arrow, grip=(0.0, 0.0, 0.0), view=DECAL_VIEW)))
    results.append(("poster_wall", common.run_model("poster_wall", "props", POSTER_BUDGET,
                                                    _build_poster_wall, grip=(0.0, 0.0, 0.0), view=DECAL_VIEW)))
    return results
