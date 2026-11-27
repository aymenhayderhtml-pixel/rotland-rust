"""Rooftop prop kit for R43: scaffold plank bridges (4/8/12 m), step crate, radio console."""
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))

import common  # noqa: E402


def _plank_bridge(model, length):
    """Rough scaffold plank bridge: 3 boards (1.2 m wide), cross battens, 1-side rope rail.

    Long along Blender Y (game Z), origin at middle, top surface at z = 0.0 (game y = 0.0).
    """
    half_y = length / 2.0
    segs = max(3, int(length // 2.0))
    max_sag = 0.025 + 0.0025 * length
    board_colors = ("wood_crate", "bark_light", "wood_crate")
    y_jitters = (-0.04, 0.02, -0.02)
    z_jitters = (0.0, -0.005, 0.003)

    # 3 scaffold boards side by side (total width ~1.18 m)
    for bi, bx in enumerate((-0.40, 0.0, 0.40)):
        col = board_colors[bi]
        yj = y_jitters[bi]
        zj = z_jitters[bi]
        for si in range(segs):
            t0 = si / segs
            t1 = (si + 1) / segs
            y0 = -half_y + t0 * length + yj
            y1 = -half_y + t1 * length + yj
            ym = (y0 + y1) / 2.0
            sag = max_sag * (1.0 - (ym / half_y) ** 2)
            # Slope slightly to follow sag curve
            sag0 = max_sag * (1.0 - ((y0 - yj) / half_y) ** 2)
            sag1 = max_sag * (1.0 - ((y1 - yj) / half_y) ** 2)
            tilt = math.atan2(sag0 - sag1, y1 - y0)
            bm = common.make_box((0.36, (y1 - y0) + 0.01, 0.055), color=col)
            common.transform(bm, rot=(tilt, 0.0, 0.0), loc=(bx, ym, -0.0275 - sag + zj))
            model.add(bm)

    # Nailed cross battens underneath + nail heads on top
    batten_count = max(3, int(length // 1.8) + 1)
    for i in range(batten_count):
        t = i / (batten_count - 1)
        by = -half_y + 0.25 + t * (length - 0.50)
        sag = max_sag * (1.0 - (by / half_y) ** 2)
        model.add(common.make_box((1.14, 0.11, 0.045), loc=(0.0, by, -0.078 - sag), color="bark_dark"))
        for bx in (-0.40, 0.0, 0.40):
            model.add(common.make_box((0.03, 0.03, 0.012), loc=(bx, by, 0.003 - sag), color="metal"))

    # Rope rail on -X side (posts + sagging rope + frayed end at +Y)
    rx = -0.56
    post_count = max(3, int(length // 2.0) + 1)
    post_ys = [-half_y + 0.22 + (length - 0.44) * (i / (post_count - 1)) for i in range(post_count)]
    for py in post_ys:
        sag = max_sag * (1.0 - (py / half_y) ** 2)
        model.add(common.make_box((0.07, 0.07, 0.94), loc=(rx, py, 0.39 - sag), color="bark_dark"))
        model.add(common.make_box((0.09, 0.09, 0.06), loc=(rx, py, 0.78 - sag), color="canvas_khaki"))

    for i in range(len(post_ys) - 1):
        py0, py1 = post_ys[i], post_ys[i + 1]
        span = py1 - py0
        # 2-segment catenary rope per bay
        for sub in range(2):
            sy0 = py0 + span * (sub * 0.5)
            sy1 = py0 + span * ((sub + 1) * 0.5)
            sz0 = 0.78 if sub == 0 else 0.72
            sz1 = 0.72 if sub == 0 else 0.78
            r_len = math.hypot(sy1 - sy0, sz1 - sz0)
            tilt = math.atan2(sz1 - sz0, sy1 - sy0)
            bm = common.make_box((0.035, r_len + 0.02, 0.035), color="canvas_khaki")
            common.transform(bm, rot=(tilt, 0.0, 0.0), loc=(rx, (sy0 + sy1) / 2.0, (sz0 + sz1) / 2.0))
            model.add(bm)

    # Frayed rope end dangling past the +Y post
    end_y = post_ys[-1]
    for fx, fz_rot, flen in ((-0.02, 0.25, 0.26), (0.01, -0.15, 0.22), (0.03, 0.05, 0.18)):
        bm = common.make_box((0.022, 0.022, flen), color="canvas_khaki")
        common.transform(bm, rot=(0.35, fz_rot, 0.0), loc=(rx + fx, end_y + 0.08, 0.78 - flen * 0.45))
        model.add(bm)

    # Colliders: thin walk slab (top at z = 0.0) + rail side wall
    model.add_collider("deck", common.make_box((1.20, length, 0.12), loc=(0.0, 0.0, -0.06), color="white"))
    model.add_collider("rail", common.make_box((0.10, length, 0.88), loc=(rx, 0.0, 0.44), color="white"))


def build_step_crate(model):
    """Stack of a wooden pallet + sturdy crate as a 0.46 m step, with walkable collider."""
    # Bottom pallet (1.04 x 0.94 x 0.14 m)
    for rx in (-0.42, 0.0, 0.42):
        model.add(common.make_box((0.10, 0.94, 0.09), loc=(rx, 0.0, 0.045), color="bark_dark"))
    for i in range(5):
        sy = -0.38 + i * 0.19
        col = "wood_crate" if i % 2 == 0 else "bark_light"
        model.add(common.make_box((1.04, 0.14, 0.05), loc=(0.0, sy, 0.115), color=col))

    # Top crate (0.84 x 0.74 x 0.32 m, z 0.14..0.46)
    model.add(common.make_box((0.80, 0.70, 0.30), loc=(0.0, 0.0, 0.29), color="wood_crate"))
    for sx in (-0.39, 0.39):
        model.add(common.make_box((0.06, 0.74, 0.32), loc=(sx, 0.0, 0.30), color="bark_dark"))
    for sy in (-0.34, 0.34):
        model.add(common.make_box((0.84, 0.06, 0.32), loc=(0.0, sy, 0.30), color="bark_dark"))
    for sz in (0.16, 0.44):
        model.add(common.make_box((0.85, 0.75, 0.04), loc=(0.0, 0.0, sz), color="bark_dark"))
    for cx in (-0.40, 0.40):
        for cy in (-0.35, 0.35):
            model.add(common.make_box((0.08, 0.08, 0.08), loc=(cx, cy, 0.43), color="metal"))

    model.add_collider("pallet", common.make_box((1.04, 0.94, 0.14), loc=(0.0, 0.0, 0.07), color="white"))
    model.add_collider("crate", common.make_box((0.84, 0.74, 0.32), loc=(0.0, 0.0, 0.30), color="white"))


def build_radio_console(model):
    """Field radio set on a crate table: handset, battery pack, cable to hut, notepad."""
    # Crate table (1.10 x 0.70 x 0.84 m)
    for sx in (-0.38, 0.38):
        model.add(common.make_box((0.30, 0.62, 0.78), loc=(sx, 0.0, 0.39), color="wood_crate"))
        model.add(common.make_box((0.32, 0.64, 0.06), loc=(sx, 0.0, 0.03), color="bark_dark"))
        model.add(common.make_box((0.32, 0.64, 0.06), loc=(sx, 0.0, 0.75), color="bark_dark"))
    model.add(common.make_box((0.50, 0.08, 0.12), loc=(0.0, 0.0, 0.36), color="bark_dark"))
    # Tabletop planks
    for i, ty in enumerate((-0.22, 0.0, 0.22)):
        col = "wood_crate" if i != 1 else "bark_light"
        model.add(common.make_box((1.10, 0.21, 0.06), loc=(0.0, ty, 0.81), color=col))
    for cx in (-0.52, 0.52):
        for cy in (-0.32, 0.32):
            model.add(common.make_box((0.07, 0.07, 0.07), loc=(cx, cy, 0.81), color="metal"))

    # Field radio chassis on table (-0.16, 0.06, z=0.84..1.14)
    rx, ry, rz = -0.16, 0.06, 0.84
    model.add(common.make_box((0.50, 0.30, 0.30), loc=(rx, ry, rz + 0.15), color="cloth_olive"))
    model.add(common.make_box((0.46, 0.03, 0.24), loc=(rx, ry - 0.155, rz + 0.15), color="gun_black"))
    # Frequency dial + glowing window + speaker grille + knobs
    model.add(common.make_box((0.18, 0.03, 0.09), loc=(rx - 0.10, ry - 0.17, rz + 0.20), color="water_blue"))
    model.add(common.make_box((0.02, 0.035, 0.08), loc=(rx - 0.08, ry - 0.175, rz + 0.20), color="red_paint"))
    model.add(common.make_box((0.15, 0.03, 0.14), loc=(rx + 0.12, ry - 0.17, rz + 0.17), color="rock_dark"))
    for gz in (0.13, 0.17, 0.21):
        model.add(common.make_box((0.13, 0.035, 0.015), loc=(rx + 0.12, ry - 0.175, rz + gz), color="metal"))
    for kx in (-0.16, -0.09, -0.02):
        model.add(common.make_frustum(0.022, 0.022, 0.0, 0.03, segments=6, color="brass"))
        bm = common.make_box((0.035, 0.03, 0.035), color="brass")
        common.transform(bm, loc=(rx + kx, ry - 0.18, rz + 0.09))
        model.add(bm)
    # Whip antenna
    model.add(common.make_frustum(0.018, 0.008, rz + 0.30, rz + 0.92, segments=6,
                                  center=(rx + 0.20, ry + 0.08), color="metal"))
    model.add(common.make_frustum(0.014, 0.006, rz + 0.92, rz + 0.98, segments=6,
                                  center=(rx + 0.20, ry + 0.08), color="red_paint"))

    # Handset on cradle/table + cord
    hx, hy = rx - 0.32, ry - 0.16
    model.add(common.make_box((0.05, 0.18, 0.04), loc=(hx, hy, rz + 0.02), color="gun_black"))
    model.add(common.make_box((0.06, 0.06, 0.05), loc=(hx, hy - 0.07, rz + 0.04), color="gun_black"))
    model.add(common.make_box((0.06, 0.06, 0.05), loc=(hx, hy + 0.07, rz + 0.04), color="gun_black"))
    model.add(common.make_box((0.14, 0.025, 0.025), loc=(hx + 0.09, hy - 0.03, rz + 0.03), color="rock_dark"))

    # Battery pack + terminals + power lead
    bx, by = 0.24, 0.12
    model.add(common.make_box((0.24, 0.18, 0.22), loc=(bx, by, rz + 0.11), color="tire_black"))
    model.add(common.make_box((0.04, 0.04, 0.03), loc=(bx - 0.06, by, rz + 0.235), color="red_paint"))
    model.add(common.make_box((0.04, 0.04, 0.03), loc=(bx + 0.06, by, rz + 0.235), color="brass"))
    model.add(common.make_box((0.16, 0.025, 0.025), loc=(0.09, by, rz + 0.10), color="rock_dark"))

    # Notepad with frequencies + pencil
    nx, ny = 0.28, -0.16
    model.add(common.make_box((0.20, 0.25, 0.015), loc=(nx, ny, rz + 0.008), color="white"))
    for lz in (-0.06, -0.01, 0.04):
        model.add(common.make_box((0.14, 0.015, 0.006), loc=(nx, ny + lz, rz + 0.017), color="soot"))
    pencil = common.make_box((0.016, 0.16, 0.016), color="brass")
    common.transform(pencil, rot=(0.0, 0.0, 0.35), loc=(nx + 0.06, ny, rz + 0.024))
    model.add(pencil)

    # Cable running off the back toward the radio hut
    model.add(common.make_box((0.03, 0.26, 0.03), loc=(rx, 0.28, rz + 0.03), color="gun_black"))
    model.add(common.make_box((0.03, 0.03, 0.84), loc=(rx, 0.36, 0.42), color="gun_black"))
    model.add(common.make_box((0.03, 0.55, 0.03), loc=(rx, 0.62, 0.015), color="gun_black"))

    model.add_collider("table", common.make_box((1.10, 0.70, 0.86), loc=(0.0, 0.0, 0.43), color="white"))


def build_all():
    specs = [
        ("roof_plank_4", 500, lambda m: _plank_bridge(m, 4.0), (0.0, 0.0, 0.0)),
        ("roof_plank_8", 700, lambda m: _plank_bridge(m, 8.0), (0.0, 0.0, 0.0)),
        ("roof_plank_12", 900, lambda m: _plank_bridge(m, 12.0), (0.0, 0.0, 0.0)),
        ("roof_step_crate", 400, build_step_crate, None),
        ("radio_console", 600, build_radio_console, None),
    ]
    results = []
    for name, budget, fn, grip in specs:
        ok = common.run_model(name, "props", budget, fn, grip=grip)
        results.append((name, ok))
    return results


if __name__ == "__main__":
    out = build_all()
    failed = [n for n, ok in out if not ok]
    if failed:
        print("ROOFTOP KIT FAILED:", failed)
        sys.exit(1)
