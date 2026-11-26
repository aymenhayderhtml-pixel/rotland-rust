"""Survivor trace camp & barrier props: rooftop_camp, boarded_door, sandbags_line, body_sheet_b."""
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, os.path.join(HERE, "..", "metro"))
sys.path.insert(0, os.path.join(HERE, "..", "vehicles"))

import car_parts  # noqa: E402
import common  # noqa: E402
from jam_util import extrude_sections, pbox  # noqa: E402

CAMP_BUDGET = 6000  # new: DECISIONS 2026-11-26
DOOR_BUDGET = 1500  # new: DECISIONS 2026-11-26
SANDBAG_BUDGET = 1500  # new: DECISIONS 2026-11-26
BODY_BUDGET = 8000  # new: DECISIONS 2026-11-26
FRONT_VIEW = (0.55, 1.0, 0.45)  # new: DECISIONS 2026-11-26
CAMP_VIEW = (0.80, -1.0, 0.65)  # new: DECISIONS 2026-11-26 (faces tent entrance & lean-to)


def _add_tent(model, cx, cy):
    """A-frame tent (2.1 x 2.5 x 1.6 m) at (cx, cy) with ridge pole and open front flap."""
    hw, hl, h = 1.05, 1.25, 1.60  # new: DECISIONS 2026-11-26
    # Ground sheet
    model.add(common.make_box((hw * 2.0, hl * 2.0, 0.04), loc=(cx, cy, 0.02), color="cloth_olive"))
    # Ridge pole and two end uprights
    model.add(car_parts._beam((cx, cy - hl - 0.08, h), (cx, cy + hl + 0.08, h), 0.035, color="bark_light", segments=6))
    for sy in (-1, 1):
        model.add(car_parts._beam((cx, cy + sy * hl, 0.0), (cx, cy + sy * hl, h), 0.03, color="bark_dark", segments=6))
    # Tent roof slopes (two tilted panels) + blue tarp patch on one slope
    slope_len = math.hypot(hw + 0.08, h)
    slope_ang = math.atan2(h, hw)
    for sx in (-1, 1):
        panel = common.make_box((slope_len, hl * 2.05, 0.035), color="canvas_khaki")
        common.transform(panel, rot=(0.0, sx * slope_ang, 0.0),
                         loc=(cx + sx * hw * 0.5, cy, h * 0.5))
        model.add(panel)
    patch = common.make_box((slope_len * 0.65, hl * 1.15, 0.02), color="tarp_blue")
    common.transform(patch, rot=(0.0, -slope_ang, 0.0), loc=(cx - hw * 0.56, cy + 0.15, h * 0.45))
    model.add(patch)
    # Back triangle closed, front left flap tied open
    back = common.new_bm()
    pts = [(cx - hw, cy + hl, 0.0), (cx + hw, cy + hl, 0.0), (cx, cy + hl, h)]
    common.add_quad(back, [pts[0], pts[1], pts[2], pts[2]], outward=(0.0, 1.0, 0.0), color="canvas_khaki")
    model.add(back)
    flap = common.make_box((hw * 0.55, 0.03, h * 0.72), color="canvas_khaki")
    common.transform(flap, rot=(0.0, 0.0, 0.45), loc=(cx - hw * 0.72, cy - hl - 0.05, h * 0.36))
    model.add(flap)
    # Sleeping roll inside
    roll = common.make_frustum(0.18, 0.18, 0.0, 1.6, segments=8, color="shirt_blue")
    common.transform(roll, rot=(math.pi / 2, 0.0, 0.08), loc=(cx + 0.25, cy + 0.75, 0.22))
    model.add(roll)
    # Tent-only convex collider
    model.add_collider("tent", common.make_box((hw * 2.0, hl * 2.0, h), loc=(cx, cy, h / 2), color="white"))


def _add_leanto_mattress(model, cx, cy):
    """Tarp lean-to (tarp_blue) over a pallet-frame mattress (white + bark_light)."""
    # 4 corner posts (front -Y taller than back +Y)
    for sx, py, ph in ((-0.95, cy - 0.90, 1.65), (0.95, cy - 0.90, 1.65),
                       (-0.95, cy + 0.95, 0.85), (0.95, cy + 0.95, 0.85)):
        model.add(common.make_box((0.08, 0.08, ph), loc=(cx + sx, py, ph / 2), color="bark_dark"))
    # Sloped blue tarp resting cleanly on top of the 4 posts (rot_x < 0: high at -Y, low at +Y)
    tarp = common.make_box((2.15, 2.15, 0.035), color="tarp_blue")
    common.transform(tarp, rot=(-0.41, 0.0, 0.0), loc=(cx, cy + 0.02, 1.26))
    model.add(tarp)
    # Wooden pallet frame + white mattress + pillow & blanket
    model.add(common.make_box((1.15, 1.95, 0.12), loc=(cx, cy + 0.05, 0.06), color="bark_light"))
    model.add(pbox((cx - 0.52, cy - 0.85, 0.12), (cx + 0.52, cy + 0.92, 0.30), "white", bevel=0.03))
    model.add(pbox((cx - 0.36, cy + 0.52, 0.30), (cx + 0.36, cy + 0.86, 0.40), "bone_white", bevel=0.02))
    model.add(pbox((cx - 0.50, cy - 0.82, 0.30), (cx + 0.50, cy + 0.08, 0.35), "shirt_red", bevel=0.015))


def _add_barrel(model, x, y, z, color, rot=(0.0, 0.0, 0.0)):
    barrel = common.combine([
        common.make_frustum(0.29, 0.29, 0.0, 0.88, segments=10, color=color),
        common.make_frustum(0.305, 0.305, 0.24, 0.28, segments=10, color="rust_dark"),
        common.make_frustum(0.305, 0.305, 0.60, 0.64, segments=10, color="rust_dark"),
    ])
    common.transform(barrel, rot=rot, loc=(x, y, z))
    model.add(barrel)


def _add_fire_barrel(model, cx, cy):
    """Metal burn barrel with soot inside and charred wood."""
    model.add(common.make_frustum(0.30, 0.30, 0.0, 0.86, segments=12, color="metal"))
    model.add(common.make_frustum(0.31, 0.31, 0.66, 0.86, segments=12, color="soot"))
    model.add(common.make_frustum(0.26, 0.26, 0.72, 0.82, segments=8, color="soot"))
    for rz, tx in ((0.4, 0.6), (2.2, 0.55), (-1.2, 0.68)):
        log = common.make_box((0.065, 0.065, 0.36), color="bark_dark")
        common.transform(log, rot=(tx, 0.0, rz), loc=(cx, cy, 0.84))
        model.add(log)


def _add_washing_line(model, x0, y0, x1, y1):
    """Two posts, two thin leather strands, and 3 hanging cloth pieces."""
    for x, y in ((x0, y0), (x1, y1)):
        model.add(common.make_frustum(0.05, 0.04, 0.0, 1.80, segments=6, center=(x, y), color="bark_dark"))
    mx, my = (x0 + x1) / 2, (y0 + y1) / 2
    for z in (1.68, 1.46):
        model.add(car_parts._beam((x0, y0, z), (mx, my, z - 0.07), 0.012, color="leather", segments=4))
        model.add(car_parts._beam((mx, my, z - 0.07), (x1, y1, z), 0.012, color="leather", segments=4))
    dx, dy = x1 - x0, y1 - y0
    yaw = math.atan2(dy, dx)
    for t, col, w, h in ((0.24, "shirt_red", 0.38, 0.44), (0.50, "shirt_blue", 0.34, 0.48),
                         (0.76, "white", 0.32, 0.42)):
        cx = x0 + dx * t
        cy = y0 + dy * t
        cloth = common.make_box((w, 0.03, h), color=col)
        common.transform(cloth, rot=(0.06, 0.0, yaw), loc=(cx, cy, 1.40))
        model.add(cloth)


def _add_crate_stack(model, cx, cy):
    """Stack of 3 wooden crates."""
    crates = [
        ((0.0, 0.0, 0.0), (0.76, 0.62, 0.54), 0.08),
        ((0.68, -0.35, 0.0), (0.66, 0.56, 0.48), -0.25),
        ((0.12, -0.08, 0.54), (0.58, 0.50, 0.44), 0.32),
    ]
    for (ox, oy, oz), (sx, sy, sz), rz in crates:
        bm = pbox((-sx / 2, -sy / 2, 0.0), (sx / 2, sy / 2, sz), "wood_crate", bevel=0.015)
        common.transform(bm, rot=(0.0, 0.0, rz), loc=(cx + ox, cy + oy, oz))
        model.add(bm)


def _build_rooftop_camp(model):
    """6 x 6 m survivor rooftop camp."""
    _add_tent(model, -1.45, 1.15)
    _add_leanto_mattress(model, 1.35, 1.25)
    _add_fire_barrel(model, -0.05, -0.35)
    # 2 crate/bucket seats by the fire barrel
    model.add(common.make_box((0.40, 0.40, 0.38), loc=(-0.75, -0.65, 0.19), color="wood_crate"))
    model.add(common.make_frustum(0.18, 0.20, 0.0, 0.36, segments=8, center=(0.62, -0.55), color="tarp_blue"))
    # 4 barrels
    _add_barrel(model, -2.35, -1.35, 0.0, "red_paint")
    _add_barrel(model, -1.70, -1.55, 0.0, "metal")
    _add_barrel(model, 2.35, -0.15, 0.0, "red_paint")
    _add_barrel(model, -2.30, -0.65, 0.29, "metal", rot=(math.pi / 2, 0.0, 0.45))
    # Crate stack & washing line
    _add_crate_stack(model, 1.55, -1.65)
    _add_washing_line(model, -1.60, -2.50, 0.85, -2.55)


def _build_boarded_door(model):
    """1.2 x 2.2 m door frame 0.1 m proud of a wall (+Y), 3 crossed planks and nails."""
    w, h, depth = 1.20, 2.20, 0.10  # new: DECISIONS 2026-11-26
    jt = 0.09  # jamb thickness
    # Frame (y = 0.0..0.10)
    model.add(common.make_box((jt, depth, h), loc=(-w / 2 + jt / 2, depth / 2, h / 2), color="bark_dark"))
    model.add(common.make_box((jt, depth, h), loc=(w / 2 - jt / 2, depth / 2, h / 2), color="bark_dark"))
    model.add(common.make_box((w, depth, jt), loc=(0.0, depth / 2, h - jt / 2), color="bark_dark"))
    # Recessed dark door leaf
    model.add(common.make_box((w - 2 * jt, 0.04, h - jt), loc=(0.0, 0.03, (h - jt) / 2), color="brownstone"))
    # 3 crossed planks across the front (y = 0.10..0.14)
    planks = [
        (0.0, 1.58, 0.14, "wood_crate", 0.0),
        (-0.02, 1.15, -0.32, "bark_light", 0.02),
        (0.02, 0.72, 0.24, "wood_crate", 0.04),
    ]
    for px, pz, tilt, col, dy in planks:
        plank = common.make_box((1.30, 0.035, 0.16), color=col)
        common.transform(plank, rot=(0.0, tilt, 0.0), loc=(px, depth + 0.018 + dy, pz))
        model.add(plank)
        for sx in (-0.48, 0.48):
            nail_z = pz - sx * math.sin(tilt)
            nail = common.make_frustum(0.028, 0.028, 0.0, 0.022, segments=6, color="metal")
            common.transform(nail, rot=(math.pi / 2, 0.0, 0.0),
                             loc=(px + sx * math.cos(tilt), depth + 0.048 + dy, nail_z))
            model.add(nail)
    model.add_collider("board", common.make_box((w, depth + 0.08, h), loc=(0.0, (depth + 0.08) / 2, h / 2), color="white"))


def _sandbag_pillow(length, width, height, loc, rot_z, color):
    """Low-poly plump sandbag (extruded rounded octagonal profile with pinched ends, ~44 tris)."""
    hl, hw, hh = length / 2, width / 2, height / 2  # new: DECISIONS 2026-11-26
    ring_pts = [(math.cos(2 * math.pi * k / 6), math.sin(2 * math.pi * k / 6)) for k in range(6)]
    sections = []
    for fx, sy, sz in ((-1.0, 0.55, 0.32), (-0.72, 0.96, 0.94), (0.0, 1.0, 1.0),
                       (0.72, 0.96, 0.94), (1.0, 0.55, 0.32)):
        sections.append((fx * hl, [(y * hw * sy, z * hh * sz) for y, z in ring_pts]))
    bag = extrude_sections(sections, color)
    common.transform(bag, rot=(0.0, 0.0, rot_z), loc=loc)
    return bag


def _build_sandbags_line(model):
    """Two-row-high 4 m sandbag barrier (9 bottom + 8 top bags) with 0.8 m tall collider."""
    # 9 bags on bottom row (4.0 m span), 8 bags staggered on top row
    for i in range(9):
        x = -1.78 + i * 0.445  # new: DECISIONS 2026-11-26
        col = "cloth_olive" if i in (2, 6) else "canvas_khaki"
        model.add(_sandbag_pillow(0.47, 0.38, 0.25, (x, 0.0, 0.115), 0.05 if i % 2 == 0 else -0.05, col))
    for i in range(8):
        x = -1.56 + i * 0.445  # new: DECISIONS 2026-11-26
        col = "cloth_olive" if i == 4 else "canvas_khaki"
        model.add(_sandbag_pillow(0.46, 0.36, 0.24, (x, 0.0, 0.31), -0.06 if i % 2 == 0 else 0.06, col))
    model.add_collider("row", common.make_box((4.0, 0.50, 0.80), loc=(0.0, 0.0, 0.40), color="white"))


def _build_body_sheet_b(model):
    """Second pose of R38 body under a sheet: curled/bent knee, arms out, one shoe out, blood stain under head."""
    # Ground-level sheet skirt & diagonal fold
    model.add(common.make_icosphere(subdiv=4, jitter=0.02, seed=21, radius=(0.56, 0.96, 0.09),
                                    loc=(0.0, 0.0, 0.09), color="white"))
    fold = common.make_icosphere(subdiv=3, jitter=0.015, seed=22, radius=(0.36, 0.14, 0.08), color="bone_white")
    common.transform(fold, rot=(0.0, 0.0, 0.35), loc=(0.02, 0.08, 0.18))
    model.add(fold)
    # Head, shoulders, twisted torso, bent hip & legs under the sheet
    lumps = [
        ((0.06, 0.80, 0.19), (0.14, 0.15, 0.14), (0.0, 0.0, -0.2), 4, "white"),  # head
        ((0.0, 0.48, 0.19), (0.28, 0.20, 0.13), (0.0, 0.0, 0.1), 3, "white"),  # shoulders
        ((-0.04, 0.10, 0.21), (0.22, 0.34, 0.14), (0.0, 0.0, 0.12), 4, "white"),  # torso
        ((-0.08, -0.28, 0.20), (0.24, 0.20, 0.14), (0.0, 0.0, -0.15), 3, "white"),  # hips
        ((-0.24, -0.56, 0.18), (0.12, 0.28, 0.12), (0.0, 0.0, -0.42), 3, "white"),  # bent left thigh
        ((-0.14, -0.84, 0.15), (0.10, 0.24, 0.09), (0.0, 0.0, 0.28), 3, "white"),  # left shin (covered)
        ((0.12, -0.70, 0.15), (0.10, 0.36, 0.10), (0.0, 0.0, -0.04), 3, "white"),  # right leg
        # Arms spread out under sheet folds
        ((0.36, 0.42, 0.12), (0.24, 0.10, 0.08), (0.0, 0.0, -0.25), 3, "white"),  # right arm out
        ((-0.36, 0.34, 0.12), (0.22, 0.10, 0.08), (0.0, 0.0, 0.38), 3, "white"),  # left arm out
    ]
    for k, (loc, rad, rot, sub, col) in enumerate(lumps):
        bm = common.make_icosphere(subdiv=sub, jitter=0.02, seed=30 + k, radius=rad, color=col)
        common.transform(bm, rot=rot, loc=loc)
        model.add(bm)
    # Hand connected directly to the right sleeve cuff, and one shoe connected by ankle cuff
    model.add(common.make_icosphere(subdiv=2, radius=(0.06, 0.05, 0.035), loc=(0.56, 0.37, 0.09), color="skin_pale"))
    model.add(pbox((0.08, -1.04, 0.03), (0.16, -0.92, 0.11), "pants_dark", bevel=0.015))
    model.add(pbox((0.06, -1.20, 0.0), (0.18, -0.98, 0.10), "shoe_dark", bevel=0.02))
    # Dark blood stain pooling under the head
    model.add(common.make_frustum(0.28, 0.26, 0.002, 0.012, segments=10, center=(0.10, 0.84), color="blood_dark"))
    model.add(common.make_frustum(0.18, 0.16, 0.002, 0.011, segments=8, center=(-0.05, 0.90), color="blood_dark"))


def build_all():
    """Builds rooftop_camp, boarded_door, sandbags_line, and body_sheet_b."""
    return [
        ("rooftop_camp", common.run_model("rooftop_camp", "props", CAMP_BUDGET,
                                          _build_rooftop_camp, top_preview=True, view=CAMP_VIEW)),
        ("boarded_door", common.run_model("boarded_door", "props", DOOR_BUDGET,
                                          _build_boarded_door, grip=(0.0, 0.0, 0.0), view=FRONT_VIEW)),
        ("sandbags_line", common.run_model("sandbags_line", "props", SANDBAG_BUDGET,
                                           _build_sandbags_line, view=FRONT_VIEW)),
        ("body_sheet_b", common.run_model("body_sheet_b", "props", BODY_BUDGET, _build_body_sheet_b)),
    ]
