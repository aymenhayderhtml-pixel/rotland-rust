"""R41-04 billboard frames (roof_a, roof_b, street) and ad faces (a..d) + _lod1s."""
import math
import os
import sys

import bmesh

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, os.path.join(HERE, "..", "traces"))

import common  # noqa: E402
import graffiti_font  # noqa: E402

FRAME_BUDGET = 4000
FRAME_LOD1_BUDGET = 400
FACE_BUDGET = 400
FACE_LOD1_BUDGET = 40

# Roof frame face plane sits at y=0.20; face sits 4 cm in front at y=0.24.
# Street billboard uses the same face scaled by 0.66 around origin (0, 0, 0).
ROOF_W = 12.0
ROOF_H = 4.0
ROOF_Z0 = 1.60
ROOF_Y_BACK = 0.20
FACE_Y = 0.24
STREET_FACE_SCALE = 0.66

# Extend graffiti_font.FONT with 'Y' for "FLY AWAY"
AD_FONT = dict(graffiti_font.FONT)
AD_FONT["Y"] = [(0, 1, 0.5, 0.48), (1, 1, 0.5, 0.48), (0.5, 0.48, 0.5, 0)]


def _bar(p0, p1, thick, color):
    """Box beam from p0 to p1."""
    from mathutils import Vector
    s = Vector(p0)
    e = Vector(p1)
    d = e - s
    length = d.length
    if length < 1e-4:
        return common.new_bm()
    bm = common.make_box((thick, thick, length), color=color)
    rot = Vector((0.0, 0.0, 1.0)).rotation_difference(d.normalized()).to_matrix()
    bmesh.ops.rotate(bm, verts=list(bm.verts), cent=(0.0, 0.0, 0.0), matrix=rot)
    bmesh.ops.translate(bm, verts=list(bm.verts), vec=(s + e) * 0.5)
    return bm


def _ad_text(text, centre_x, base_z, letter_w, letter_h, y, color, thick_scale=1.25):
    """Stroke text using AD_FONT, facing +Y at plane y."""
    gap = letter_w * 0.32
    space = letter_w * 0.55
    total = sum(space if ch == " " else letter_w for ch in text) + gap * (len(text) - 1)
    thick = letter_h * 0.13 * thick_scale
    x = centre_x - total / 2.0
    out = []
    for ch in text:
        if ch == " ":
            x += space + gap
            continue
        for u0, v0, u1, v1 in AD_FONT[ch]:
            x0, z0 = -(x + u0 * letter_w), base_z + v0 * letter_h
            x1, z1 = -(x + u1 * letter_w), base_z + v1 * letter_h
            out.append(graffiti_font.quad_bar(x0, z0, x1, z1, y, thick, color))
        x += letter_w + gap
    return out


def _catwalk_and_lamps(w, z_cat, z_top, y_back, broken_lamps, lod1=False):
    parts = []
    # Catwalk deck in front of backing
    parts.append(common.make_box((w + 0.4, 0.65, 0.06), loc=(0.0, y_back + 0.36, z_cat), color="rust"))
    if not lod1:
        # Catwalk brackets
        for x in (-w * 0.42, -w * 0.14, w * 0.14, w * 0.42):
            parts.append(_bar((x, y_back, z_cat - 0.35), (x, y_back + 0.62, z_cat - 0.03), 0.05, "rust_dark"))
        # Front safety rail
        parts.append(common.make_box((w + 0.4, 0.04, 0.04), loc=(0.0, y_back + 0.67, z_cat + 0.85), color="rust_dark"))
        for x in (-w * 0.48, -w * 0.24, 0.0, w * 0.24, w * 0.48):
            parts.append(common.make_box((0.04, 0.04, 0.85), loc=(x, y_back + 0.67, z_cat + 0.425), color="rust_dark"))
        # Broken / bent lamp arms along top
        for lx, bend in broken_lamps:
            parts.append(_bar((lx, y_back, z_top - 0.05), (lx, y_back + 0.95, z_top + 0.25 + bend), 0.05, "rust_dark"))
            parts.append(common.make_box((0.28, 0.18, 0.10), loc=(lx, y_back + 0.98, z_top + 0.23 + bend), color="metal"))
    return parts


def _roof_frame_parts(variant_b=False, lod1=False):
    parts = []
    hw = ROOF_W * 0.5
    z0, z1 = ROOF_Z0, ROOF_Z0 + ROOF_H
    # Backing panel (behind face at y=0.20)
    parts.append(common.make_box((ROOF_W + 0.16, 0.08, ROOF_H + 0.16),
                                 loc=(0.0, ROOF_Y_BACK - 0.04, z0 + ROOF_H * 0.5), color="rust_dark"))
    # Vertical steel legs
    leg_xs = (-4.8, -1.6, 1.6, 4.8) if not variant_b else (-4.5, 0.0, 4.5)
    for lx in leg_xs:
        parts.append(common.make_box((0.18, 0.18, z1 + 0.1), loc=(lx, 0.0, (z1 + 0.1) * 0.5), color="rust"))
        if not lod1:
            parts.append(common.make_box((0.42, 0.42, 0.10), loc=(lx, 0.0, 0.05), color="concrete_grey"))

    if not lod1:
        # Back diagonal bracing
        back_y = -1.6 if variant_b else -1.35
        for lx in leg_xs:
            parts.append(_bar((lx, back_y, 0.05), (lx, 0.0, z1 - 0.3), 0.12, "rust_dark"))
            parts.append(_bar((lx, back_y * 0.5, 0.8), (lx, 0.0, 1.6), 0.08, "rust"))
        # X-braces between legs
        for i in range(len(leg_xs) - 1):
            x_a, x_b = leg_xs[i], leg_xs[i + 1]
            parts.append(_bar((x_a, 0.0, 0.2), (x_b, 0.0, z0), 0.07, "rust_dark"))
            parts.append(_bar((x_b, 0.0, 0.2), (x_a, 0.0, z0), 0.07, "rust_dark"))
        # Horizontal frame rails
        for rz in (z0, z0 + ROOF_H * 0.5, z1):
            parts.append(common.make_box((ROOF_W + 0.2, 0.10, 0.12), loc=(0.0, 0.08, rz), color="rust"))

    lamps = [(-3.8, 0.0), (0.2, -0.22), (4.0, 0.08)] if not variant_b else [(-2.8, -0.18), (3.1, 0.05)]
    parts.extend(_catwalk_and_lamps(ROOF_W, z0 - 0.22, z1, ROOF_Y_BACK, lamps, lod1=lod1))
    if variant_b and not lod1:
        # Side electrical junction box
        parts.append(common.make_box((0.40, 0.28, 0.55), loc=(hw + 0.22, 0.0, z0 + 0.8), color="metal"))
    return parts


def _street_frame_parts(lod1=False):
    parts = []
    w = ROOF_W * STREET_FACE_SCALE
    h = ROOF_H * STREET_FACE_SCALE
    z0 = ROOF_Z0 * STREET_FACE_SCALE
    z1 = z0 + h
    y_back = ROOF_Y_BACK * STREET_FACE_SCALE - 0.01  # face sits at 0.24 * 0.66 = 0.1584 (~4 cm in front)

    # One thick central steel pole
    pole_segs = 6 if lod1 else 10
    parts.append(common.make_frustum(0.32, 0.26, 0.0, z0 + h * 0.55, segments=pole_segs, color="rust"))
    if not lod1:
        parts.append(common.make_frustum(0.55, 0.52, 0.0, 0.18, segments=8, color="concrete_grey"))
        # Support T-crossbar behind backing
        parts.append(common.make_box((w * 0.85, 0.22, 0.22), loc=(0.0, y_back - 0.14, z0 + h * 0.5), color="rust_dark"))

    # Backing panel
    parts.append(common.make_box((w + 0.14, 0.07, h + 0.14),
                                 loc=(0.0, y_back - 0.035, z0 + h * 0.5), color="rust_dark"))
    lamps = [(-2.2, -0.10), (2.1, 0.04)]
    parts.extend(_catwalk_and_lamps(w, z0 - 0.18, z1, y_back, lamps, lod1=lod1))
    return parts


def _face_base(bg_color):
    """Main 12x4 m panel at y=FACE_Y facing +Y."""
    bm = common.new_bm()
    hw = ROOF_W * 0.5
    z0, z1 = ROOF_Z0, ROOF_Z0 + ROOF_H
    common.add_quad(bm, [(-hw, FACE_Y, z0), (hw, FACE_Y, z0), (hw, FACE_Y, z1), (-hw, FACE_Y, z1)],
                    outward=(0.0, 1.0, 0.0), color=bg_color)
    return bm


def _peeling_strips(strips):
    """Curled peeling paper strips jutting slightly forward (+Y) from FACE_Y."""
    out = []
    for x0, x1, z_top, z_bot, curl_y, col in strips:
        bm = common.new_bm()
        z_mid = (z_top + z_bot) * 0.5
        y0 = FACE_Y + 0.006
        y1 = FACE_Y + curl_y
        common.add_quad(bm, [(x0, y0, z_top), (x1, y0, z_top), (x1, y0 + curl_y * 0.35, z_mid),
                             (x0, y0 + curl_y * 0.35, z_mid)], outward=(0.0, 1.0, 0.0), color=col)
        common.add_quad(bm, [(x0, y0 + curl_y * 0.35, z_mid), (x1, y0 + curl_y * 0.35, z_mid),
                             (x1, y1, z_bot), (x0, y1, z_bot)], outward=(0.0, 1.0, 0.0), color=col)
        out.append(bm)
    return out


def _build_face_a(model):
    """SUMMER SALE — faded warm ad with sun."""
    model.add(_face_base("sand_light"))
    # Sun circle + rays on +X (viewer's left)
    sun_bm = common.new_bm()
    cx, cz, r = 3.8, 3.6, 1.05
    ring = [(cx + math.cos(2 * math.pi * i / 8) * r, FACE_Y + 0.006, cz + math.sin(2 * math.pi * i / 8) * r)
            for i in range(8)]
    sun_bm.verts.ensure_lookup_table()
    verts = [sun_bm.verts.new(p) for p in ring]
    f = sun_bm.faces.new(list(reversed(verts)))
    common.paint_faces(sun_bm, [f], "carrot_orange")
    model.add(sun_bm)
    for bm in _ad_text("SUMMER", 1.4, 3.85, 0.72, 1.15, FACE_Y + 0.010, "red_paint"):
        model.add(bm)
    for bm in _ad_text("SALE", 1.4, 2.25, 0.85, 1.25, FACE_Y + 0.010, "rust_dark"):
        model.add(bm)
    for bm in _peeling_strips([(-5.2, -3.9, 2.4, 1.45, 0.14, "mortar"), (4.1, 5.4, 5.5, 4.3, 0.12, "wheat_gold")]):
        model.add(bm)


def _build_face_b(model):
    """CALL NOW — faded blue ad with retro phone shape."""
    model.add(_face_base("glass_tint"))
    # Retro phone silhouette on +X (viewer's left)
    model.add(graffiti_font.quad_bar(2.8, 2.5, 4.7, 2.5, FACE_Y + 0.006, 0.85, "gun_black"))
    model.add(graffiti_font.quad_bar(2.5, 3.7, 5.0, 3.7, FACE_Y + 0.008, 0.42, "white"))
    model.add(graffiti_font.quad_bar(2.5, 3.5, 2.5, 3.8, FACE_Y + 0.008, 0.45, "white"))
    model.add(graffiti_font.quad_bar(5.0, 3.5, 5.0, 3.8, FACE_Y + 0.008, 0.45, "white"))
    for bm in _ad_text("CALL", 1.5, 3.85, 0.85, 1.20, FACE_Y + 0.010, "white"):
        model.add(bm)
    for bm in _ad_text("NOW", 1.5, 2.20, 0.95, 1.25, FACE_Y + 0.010, "brass"):
        model.add(bm)
    for bm in _peeling_strips([(1.0, 2.5, 2.2, 1.40, 0.15, "concrete_grey")]):
        model.add(bm)


def _build_face_c(model):
    """FLY AWAY — faded sky/sea ad with airplane."""
    model.add(_face_base("ice_blue"))
    # Sea band at bottom
    sea = common.new_bm()
    common.add_quad(sea, [(-6.0, FACE_Y + 0.004, 1.6), (6.0, FACE_Y + 0.004, 1.6),
                          (6.0, FACE_Y + 0.004, 2.65), (-6.0, FACE_Y + 0.004, 2.65)],
                    outward=(0.0, 1.0, 0.0), color="water_lake")
    model.add(sea)
    # Plane silhouette on +X (viewer's left)
    model.add(graffiti_font.quad_bar(2.5, 4.0, 5.0, 4.3, FACE_Y + 0.008, 0.35, "white"))
    model.add(graffiti_font.quad_bar(3.4, 3.3, 4.2, 4.9, FACE_Y + 0.008, 0.30, "concrete_grey"))
    for bm in _ad_text("FLY AWAY", 1.3, 3.15, 0.72, 1.25, FACE_Y + 0.010, "rust_dark"):
        model.add(bm)
    for bm in _peeling_strips([(-5.6, -4.4, 5.4, 4.0, 0.14, "white")]):
        model.add(bm)


def _build_face_d(model):
    """SAFE HOME — faded olive/khaki ad with house + shield."""
    model.add(_face_base("canvas_khaki"))
    # Shield + house icon on +X (viewer's left)
    shield = common.new_bm()
    common.add_quad(shield, [(2.8, FACE_Y + 0.006, 2.6), (4.8, FACE_Y + 0.006, 2.6),
                             (4.8, FACE_Y + 0.006, 4.5), (2.8, FACE_Y + 0.006, 4.5)],
                    outward=(0.0, 1.0, 0.0), color="leaf_mid")
    model.add(shield)
    model.add(graffiti_font.quad_bar(3.2, 3.1, 4.4, 3.1, FACE_Y + 0.010, 0.75, "white"))
    model.add(graffiti_font.quad_bar(3.0, 3.6, 3.8, 4.2, FACE_Y + 0.010, 0.25, "red_paint"))
    model.add(graffiti_font.quad_bar(3.8, 4.2, 4.6, 3.6, FACE_Y + 0.010, 0.25, "red_paint"))
    for bm in _ad_text("SAFE", 1.5, 3.85, 0.82, 1.15, FACE_Y + 0.010, "white"):
        model.add(bm)
    for bm in _ad_text("HOME", 1.5, 2.25, 0.82, 1.15, FACE_Y + 0.010, "gun_black"):
        model.add(bm)
    for bm in _peeling_strips([(0.2, 1.8, 2.2, 1.45, 0.13, "mortar")]):
        model.add(bm)


def _build_face_lod1(bg_color, band_color):
    def build(model):
        model.add(_face_base(bg_color))
        band = common.new_bm()
        common.add_quad(band, [(-5.4, FACE_Y + 0.006, 2.5), (5.4, FACE_Y + 0.006, 2.5),
                               (5.4, FACE_Y + 0.006, 4.4), (-5.4, FACE_Y + 0.006, 4.4)],
                        outward=(0.0, 1.0, 0.0), color=band_color)
        model.add(band)
    return build


def _build_roof_a(model):
    for bm in _roof_frame_parts(variant_b=False, lod1=False):
        model.add(bm)


def _build_roof_a_lod1(model):
    for bm in _roof_frame_parts(variant_b=False, lod1=True):
        model.add(bm)


def _build_roof_b(model):
    for bm in _roof_frame_parts(variant_b=True, lod1=False):
        model.add(bm)


def _build_roof_b_lod1(model):
    for bm in _roof_frame_parts(variant_b=True, lod1=True):
        model.add(bm)


def _build_street(model):
    for bm in _street_frame_parts(lod1=False):
        model.add(bm)


def _build_street_lod1(model):
    for bm in _street_frame_parts(lod1=True):
        model.add(bm)


def build_all():
    """Exports all billboard frames and faces + LOD1s."""
    specs = [
        ("billboard_roof_a", FRAME_BUDGET, _build_roof_a),
        ("billboard_roof_a_lod1", FRAME_LOD1_BUDGET, _build_roof_a_lod1),
        ("billboard_roof_b", FRAME_BUDGET, _build_roof_b),
        ("billboard_roof_b_lod1", FRAME_LOD1_BUDGET, _build_roof_b_lod1),
        ("billboard_street", FRAME_BUDGET, _build_street),
        ("billboard_street_lod1", FRAME_LOD1_BUDGET, _build_street_lod1),
        ("billboard_face_a", FACE_BUDGET, _build_face_a),
        ("billboard_face_a_lod1", FACE_LOD1_BUDGET, _build_face_lod1("sand_light", "red_paint")),
        ("billboard_face_b", FACE_BUDGET, _build_face_b),
        ("billboard_face_b_lod1", FACE_LOD1_BUDGET, _build_face_lod1("glass_tint", "white")),
        ("billboard_face_c", FACE_BUDGET, _build_face_c),
        ("billboard_face_c_lod1", FACE_LOD1_BUDGET, _build_face_lod1("ice_blue", "water_lake")),
        ("billboard_face_d", FACE_BUDGET, _build_face_d),
        ("billboard_face_d_lod1", FACE_LOD1_BUDGET, _build_face_lod1("canvas_khaki", "leaf_mid")),
    ]
    return [
        (name, common.run_model(name, "props", budget, fn, grip=(0.0, 0.0, 0.0), view=(0.25, 1.0, 0.25)))
        for name, budget, fn in specs
    ]
