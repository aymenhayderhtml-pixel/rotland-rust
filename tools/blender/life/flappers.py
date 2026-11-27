"""R41-04 flag_pole_wall (+_lod1), flag_torn_f0..f3, and plastic_flap_f0..f3."""
import math
import os
import sys

import bmesh

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))

import common  # noqa: E402

POLE_BUDGET = 300
POLE_LOD1_BUDGET = 30
FLAP_BUDGET = 300


def _build_flag_pole_wall(lod1=False):
    def build(model):
        # Wall bracket at origin (0, 0, 0) in XZ plane
        model.add(common.make_box((0.18, 0.04, 0.24), loc=(0.0, 0.02, 0.0), color="rust_dark"))
        # 2.5 m pole angled 45 deg up toward +Y
        segs = 4 if lod1 else 8
        pole = common.make_frustum(0.028, 0.022, 0.0, 2.50, segments=segs, color="metal")
        common.transform(pole, rot=(-math.radians(45.0), 0.0, 0.0), loc=(0.0, 0.02, 0.0))
        model.add(pole)
        if not lod1:
            # Finial ball at pole tip
            tip = 2.50 * math.sqrt(0.5)
            model.add(common.make_icosphere(subdiv=1, radius=(0.048, 0.048, 0.048),
                                            loc=(0.0, tip + 0.02, tip), color="brass"))
    return build


def _add_double_sided_quad(bm, pts, color):
    """Adds front (+Y) and back (-Y) quads separated by 4 mm so both sides render without z-fight."""
    eps = 0.002
    front = [(x, y + eps, z) for (x, y, z) in pts]
    back = [(x, y - eps, z) for (x, y, z) in pts]
    common.add_quad(bm, front, outward=(0.0, 1.0, 0.0), color=color)
    common.add_quad(bm, list(reversed(back)), outward=(0.0, -1.0, 0.0), color=color)


def _build_flag_frame(frame_idx):
    """4-frame waving torn flag (1.5 x 1.0 m), pivot at pole tip (0,0,0), hangs along +X."""
    phase = frame_idx * (math.pi * 0.5)
    cols, rows = 6, 4
    w, h = 1.50, 1.00

    def pt(ix, iz):
        u = ix / cols
        v = iz / rows  # 0 at top (z=0), 1 at bottom (z=-h)
        x = u * w
        # Ragged trailing edge at u=1
        if ix == cols:
            ragged = (0.0, -0.18, -0.05, -0.24, -0.10)[iz]
            x += ragged
        z = -v * h
        # Wave amplitude grows from 0 at pole (u=0) to 0.24 m at trailing edge
        amp = 0.24 * (u ** 1.2)
        y = amp * math.sin(u * math.pi * 2.2 - phase + v * 0.6)
        z += 0.03 * u * math.cos(u * math.pi * 2.0 - phase)
        return (x, y, z)

    def build(model):
        bm = common.new_bm()
        for ix in range(cols):
            for iz in range(rows):
                # Skip one bottom-right torn notch
                if ix == cols - 1 and iz == rows - 1:
                    continue
                p00 = pt(ix, iz)
                p10 = pt(ix + 1, iz)
                p11 = pt(ix + 1, iz + 1)
                p01 = pt(ix, iz + 1)
                # Made-up faded stripes + blue canton
                if ix < 2 and iz < 2:
                    col = "tarp_blue" if (ix + iz) % 2 == 0 else "white"
                else:
                    col = "rust" if iz % 2 == 0 else "mortar"
                _add_double_sided_quad(bm, [p00, p10, p11, p01], col)
        model.add(bm)

    return build


def _build_plastic_frame(frame_idx):
    """4-frame fluttering torn tarp sheet (1.2 x 1.6 m), pivot at top centre (0,0,0)."""
    phase = frame_idx * (math.pi * 0.5)
    cols, rows = 4, 5
    w, h = 1.20, 1.60

    def pt(ix, iz):
        u = ix / cols - 0.5  # -0.5 .. +0.5
        v = iz / rows        # 0 at top, 1 at bottom
        x = u * w
        z = -v * h
        if iz == rows:
            z += (0.14, 0.0, 0.22, -0.04, 0.16)[ix]
        amp = 0.28 * (v ** 1.3)
        y = amp * math.sin(v * math.pi * 2.0 - phase + u * 1.4)
        return (x, y, z)

    def build(model):
        bm = common.new_bm()
        for ix in range(cols):
            for iz in range(rows):
                # Torn missing corner
                if ix == 0 and iz == rows - 1:
                    continue
                p00 = pt(ix, iz)
                p10 = pt(ix + 1, iz)
                p11 = pt(ix + 1, iz + 1)
                p01 = pt(ix, iz + 1)
                col = "tarp_blue" if (ix + iz) % 3 != 0 else "glass_tint"
                _add_double_sided_quad(bm, [p00, p10, p11, p01], col)
        model.add(bm)

    return build


def build_all():
    """Exports flag pole (+lod1), 4 flag frames, and 4 plastic frames."""
    results = []
    results.append((
        "flag_pole_wall",
        common.run_model("flag_pole_wall", "props", POLE_BUDGET, _build_flag_pole_wall(False), grip=(0.0, 0.0, 0.0)),
    ))
    results.append((
        "flag_pole_wall_lod1",
        common.run_model("flag_pole_wall_lod1", "props", POLE_LOD1_BUDGET, _build_flag_pole_wall(True), grip=(0.0, 0.0, 0.0)),
    ))
    for f in range(4):
        name = f"flag_torn_f{f}"
        results.append((
            name,
            common.run_model(name, "props", FLAP_BUDGET, _build_flag_frame(f), grip=(0.0, 0.0, 0.0), view=(0.3, 1.0, 0.2)),
        ))
    for f in range(4):
        name = f"plastic_flap_f{f}"
        results.append((
            name,
            common.run_model(name, "props", FLAP_BUDGET, _build_plastic_frame(f), grip=(0.0, 0.0, 0.0), view=(0.3, 1.0, 0.2)),
        ))
    return results
