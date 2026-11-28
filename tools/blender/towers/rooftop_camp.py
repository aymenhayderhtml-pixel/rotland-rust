"""R44 rooftop camp props: survivor tent, green smoke barrel, green smoke plume, trader stall.

Blender frame: X = width, Y = depth, Z = up. The front is +Y (Godot -Z).
"""
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, os.path.join(HERE, "..", "traces"))

import bmesh  # noqa: E402
import graffiti_font  # noqa: E402

import common  # noqa: E402
from safehouse import stick  # noqa: E402

PROPS = "props"


def _prism(profile, y0, y1, color):
    """Convex profile (list of (x, z)) pushed from y0 to y1: a tent wall, dome or gable."""
    bm = common.new_bm()
    front = [bm.verts.new((x, y0, z)) for x, z in profile]
    back = [bm.verts.new((x, y1, z)) for x, z in profile]
    bm.faces.new(front)
    bm.faces.new(list(reversed(back)))
    n = len(profile)
    for i in range(n):
        j = (i + 1) % n
        bm.faces.new([front[i], front[j], back[j], back[i]])
    bmesh.ops.recalc_face_normals(bm, faces=list(bm.faces))
    common.paint(bm, color)
    return bm


TENT_PROFILE = [(-1.2, 0.0), (-1.2, 0.8), (-0.9, 1.4), (-0.4, 1.8),
                (0.4, 1.8), (0.9, 1.4), (1.2, 0.8), (1.2, 0.0)]


def build_survivor_tent(model):
    """Dirty olive tunnel tent 2.4 x 1.4 x 2.2 m: zip door on +Y, flaps, ridge pole, guy lines, a bag."""
    model.add(_prism(TENT_PROFILE, -0.7, 0.7, "cloth_olive"))
    model.add(common.make_box((0.6, 0.02, 1.0), loc=(0.0, 0.71, 0.5), color="gun_black"))
    model.add(common.make_box((0.015, 0.02, 1.0), loc=(0.0, 0.725, 0.5), color="metal"))
    for sx, rz in ((-1, 0.25), (1, -0.25)):
        flap = common.make_box((0.34, 0.03, 0.9), color="canvas_khaki")
        common.transform(flap, rot=(0.0, 0.0, rz), loc=(sx * 0.36, 0.73, 0.48))
        model.add(flap)
    model.add(common.make_box((0.3, 0.01, 0.3), loc=(-0.9, 0.705, 0.45), color="soil_dark"))
    model.add(common.make_box((0.25, 0.01, 0.2), loc=(0.85, 0.705, 1.0), color="grime_green"))
    model.add(stick((0.0, -0.7, 1.82), (0.0, 0.7, 1.82), 0.04, "bark_dark"))
    for sx in (-1, 1):
        for sy in (-1, 1):
            model.add(stick((sx * 1.2, sy * 0.6, 0.7), (sx * 2.1, sy * 0.9, 0.0), 0.02, "canvas_khaki"))
            model.add(common.make_box((0.03, 0.03, 0.2), loc=(sx * 2.1, sy * 0.9, 0.1), color="metal"))
    model.add(common.make_box((0.35, 0.25, 0.4), loc=(0.9, 1.0, 0.2), color="canvas_khaki"))
    model.add(common.make_box((0.3, 0.2, 0.05), loc=(0.9, 1.0, 0.42), color="soot"))


def build_green_smoke_barrel(model):
    """Rusty oil barrel 0.6 x 0.9 x 0.6 m with a bright green stain, a lid with a hole. Origin at base centre."""
    model.add(common.make_frustum(0.30, 0.30, 0.0, 0.88, segments=12, color="rust"))
    model.add(common.make_frustum(0.31, 0.31, 0.0, 0.04, segments=12, color="rock_dark"))
    model.add(common.make_frustum(0.28, 0.28, 0.88, 0.90, segments=12, color="gun_black"))
    for z in (0.10, 0.40, 0.70):
        model.add(common.make_frustum(0.305, 0.305, z, z + 0.03, segments=12, color="metal"))
    model.add(common.make_frustum(0.07, 0.07, 0.90, 0.92, segments=8, center=(0.05, 0.0), color="gun_black"))
    # Green stain: a splash down the front, a drip run, and a spill on the lid
    model.add(common.make_box((0.22, 0.02, 0.5), loc=(0.05, 0.30, 0.5), color="crop_green"))
    model.add(common.make_box((0.06, 0.02, 0.16), loc=(0.14, 0.30, 0.2), color="crop_green"))
    model.add(common.make_box((0.10, 0.02, 0.08), loc=(-0.08, 0.30, 0.06), color="crop_green"))
    model.add(common.make_box((0.20, 0.20, 0.02), loc=(-0.12, 0.0, 0.91), color="crop_green"))
    model.add(common.make_box((0.10, 0.02, 0.22), loc=(-0.29, 0.0, 0.5), color="sprout_green"))
    # Dents and rust patches
    model.add(common.make_box((0.08, 0.02, 0.12), loc=(-0.2, -0.28, 0.6), color="rust_dark"))
    model.add(common.make_box((0.10, 0.02, 0.10), loc=(0.2, -0.29, 0.3), color="rust_dark"))
    model.add(common.make_box((0.06, 0.02, 0.06), loc=(0.26, -0.1, 0.78), color="rust_dark"))


def build_green_smoke_plume(model):
    """Column of six lumpy green-grey puffs rising from the barrel, 1.2 x 3.0 x 1.2 m. Origin at base centre."""
    # (x, y, centre z, radius, colour). The first puff touches the ground at z = 0.
    puffs = (
        (0.00, 0.00, 0.55, 0.55, "pine_mid"),
        (0.12, -0.08, 1.20, 0.50, "pine_dark"),
        (-0.10, 0.10, 1.80, 0.44, "pine_mid"),
        (0.08, -0.05, 2.30, 0.38, "pine_dark"),
        (-0.06, 0.06, 2.70, 0.31, "pine_mid"),
        (0.03, 0.00, 2.78, 0.22, "pine_dark"),
    )
    for i, (x, y, z, r, col) in enumerate(puffs):
        model.add(common.make_icosphere(subdiv=2, jitter=0.12, seed=7 + i,
                                        radius=(r, r, r * 0.9), loc=(x, y, z), color=col))


def build_trader_stall(model):
    """Trestle table 2.2 x 1.6 x 1.4 m under a tarp awning, a price board, two crates, tins. Origin at table centre."""
    for x in (-0.85, 0.85):
        for y in (-0.26, 0.26):
            model.add(common.make_box((0.07, 0.07, 0.74), loc=(x, y, 0.37), color="bark_dark"))
        model.add(common.make_box((0.06, 0.6, 0.06), loc=(x, 0.0, 0.25), color="bark_dark"))
    for y, col in ((-0.2, "wood_crate"), (0.0, "bark_light"), (0.2, "wood_crate")):
        model.add(common.make_box((2.0, 0.19, 0.04), loc=(0.0, y, 0.76), color=col))
    # Awning poles and a sagging tarp with a scalloped front edge
    for x in (-1.0, 1.0):
        for y in (-0.75, 0.75):
            model.add(stick((x, y, 0.0), (x, y, 1.36), 0.05, "bark_dark"))
    tarp = common.make_box((2.2, 1.6, 0.03), color="canvas_khaki")
    common.transform(tarp, rot=(0.04, 0.0, 0.0), loc=(0.0, 0.0, 1.37))
    model.add(tarp)
    model.add(common.make_box((0.6, 0.5, 0.005), loc=(-0.5, 0.2, 1.39), color="tarp_blue"))
    for x in (-1.0, -0.6, -0.2, 0.2, 0.6, 1.0):
        model.add(common.make_box((0.3, 0.03, 0.12), loc=(x, 0.80, 1.31), color="canvas_khaki"))
    # Price board standing on the table (no letters): hand-drawn price lines in red
    model.add(common.make_box((0.9, 0.04, 0.5), loc=(0.0, 0.1, 1.03), color="wood_crate"))
    model.add(common.make_box((0.03, 0.2, 0.03), loc=(-0.35, 0.02, 0.88), color="bark_dark"))
    model.add(common.make_box((0.03, 0.2, 0.03), loc=(0.35, 0.02, 0.88), color="bark_dark"))
    for z0, x0, x1 in ((1.2, -0.33, 0.27), (1.05, -0.33, 0.05), (0.9, -0.33, 0.17)):
        model.add(graffiti_font.quad_bar(x0, z0, x1, z0, 0.125, 0.035, "red_paint"))
    model.add(graffiti_font.quad_bar(0.2, 1.2, 0.3, 1.2, 0.125, 0.035, "red_paint"))
    # Tins and jars on the table
    for x, y in ((-0.7, -0.1), (-0.45, 0.12), (0.4, -0.12), (0.65, 0.14)):
        model.add(common.make_frustum(0.04, 0.04, 0.78, 0.86, segments=6, center=(x, y), color="metal"))
    model.add(common.make_frustum(0.05, 0.05, 0.78, 0.84, segments=6, center=(0.1, 0.0), color="brass"))
    # Two stacked crates beside the table, on the floor
    model.add(common.make_box((0.4, 0.4, 0.4), loc=(0.5, -0.55, 0.2), color="wood_crate"))
    model.add(common.make_box((0.36, 0.36, 0.34), loc=(0.5, -0.55, 0.57), color="bark_light"))
    model.add(common.make_box((0.44, 0.02, 0.03), loc=(0.5, -0.55, 0.4), color="bark_dark"))
    # Sack and a dirty rag under the table, near the front
    model.add(common.make_box((0.3, 0.25, 0.22), loc=(-0.55, 0.55, 0.11), color="canvas_khaki"))
    model.add(common.make_box((0.06, 0.26, 0.04), loc=(-0.55, 0.55, 0.23), color="bark_dark"))
    model.add(common.make_box((0.18, 0.12, 0.02), loc=(0.0, -0.6, 0.01), color="rust_dark"))


# name, budget, build function, grip (origin), preview view (None = default)
SPECS = (
    ("survivor_tent", 800, build_survivor_tent, (0.0, 0.0, 0.0), (1.0, 1.0, 0.5)),
    ("green_smoke_barrel", 500, build_green_smoke_barrel, (0.0, 0.0, 0.0), None),
    ("green_smoke_plume", 800, build_green_smoke_plume, None, None),
    ("rooftop_trader_stall", 1200, build_trader_stall, (0.0, 0.0, 0.0), (0.6, 1.0, 0.6)),
)


def build_all():
    """Returns [(name, ok)] for every rooftop camp model."""
    results = []
    for name, budget, fn, grip, view in SPECS:
        ok = common.run_model(name, PROPS, budget, fn, grip=grip, view=view)
        results.append((name, ok))
    return results
