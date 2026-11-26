"""Ground-level overgrowth pieces: weeds_crack, grass_tall_patch, moss_patch, leaves_litter."""
import math
import os
import random
import sys

_HERE = os.path.dirname(os.path.abspath(__file__))
_BLENDER_DIR = os.path.abspath(os.path.join(_HERE, ".."))
for _p in (_HERE, _BLENDER_DIR):
    if _p not in sys.path:
        sys.path.insert(0, _p)

import common  # noqa: E402
import og_util  # noqa: E402

# data: budgets.json nature
BUDGETS = {
    "weeds_crack": 800,
    "grass_tall_patch": 1500,
    "moss_patch": 600,
    "leaves_litter": 800,
}


def _build_weeds_crack(model):
    """3.0 m long crack line of 12 weed tufts up to 0.4 m tall with dirt clods."""
    rng = random.Random(501)
    z0 = og_util.GROUND_Z_LIFT  # new: DECISIONS 2026-11-26

    # Thin jagged dark crack line along X (-1.48..+1.48)
    prev_x, prev_y = -1.48, 0.0  # new: DECISIONS 2026-11-26
    for i in range(1, 9):  # new: DECISIONS 2026-11-26
        nx = -1.48 + 2.96 * (i / 8.0)
        ny = math.sin(i * 0.9) * 0.09 + rng.uniform(-0.03, 0.03)
        seg_len = math.hypot(nx - prev_x, ny - prev_y)
        ang = math.atan2(ny - prev_y, nx - prev_x)
        crack = common.make_box((seg_len + 0.03, 0.055, 0.012), color="rock_dark")
        common.transform(crack, rot=(0.0, 0.0, ang),
                         loc=((prev_x + nx) * 0.5, (prev_y + ny) * 0.5, z0 + 0.006))
        model.add(crack)
        prev_x, prev_y = nx, ny

    # 12 weed tufts along the crack + dirt clods
    tufts = 12  # new: DECISIONS 2026-11-26
    for i in range(tufts):
        t = (i + 0.5) / tufts
        tx = -1.42 + 2.84 * t + rng.uniform(-0.07, 0.07)  # new: DECISIONS 2026-11-26
        ty = math.sin(t * 7.2) * 0.09 + rng.uniform(-0.04, 0.04)  # new: DECISIONS 2026-11-26

        # Small dirt clods around tuft root
        for _ in range(2):
            clod = common.make_icosphere(
                subdiv=1, jitter=0.25, seed=rng.randint(1, 999),
                radius=(rng.uniform(0.04, 0.07), rng.uniform(0.03, 0.06), 0.025),
                loc=(tx + rng.uniform(-0.06, 0.06), ty + rng.uniform(-0.05, 0.05), z0 + 0.025),
                color="dirt",
            )
            model.add(clod)

        blades = rng.randint(4, 5)  # new: DECISIONS 2026-11-26
        for b in range(blades):
            h = rng.uniform(0.18, 0.40)  # new: DECISIONS 2026-11-26
            lean = rng.uniform(0.20, 0.65)
            yaw = rng.uniform(0.0, 2.0 * math.pi)
            col = "grass" if (i + b) % 2 == 0 else "leaf_mid"
            model.add(og_util.make_blade(
                tx + rng.uniform(-0.03, 0.03),
                ty + rng.uniform(-0.03, 0.03),
                z0 + 0.005,
                h, lean, yaw, color=col, width=0.032,
            ))


def _build_grass_tall_patch(model):
    """2 x 2 m patch of tall dead-green grass up to 1.2 m with dirt root mounds."""
    rng = random.Random(502)
    z0 = og_util.GROUND_Z_LIFT  # new: DECISIONS 2026-11-26

    # 10 low dirt root mounds on a jittered 3x3 + center grid across the 2 x 2 m patch
    clumps = []
    grid_pts = [
        (gx * 0.68 + rng.uniform(-0.14, 0.14), gy * 0.68 + rng.uniform(-0.14, 0.14))  # new: DECISIONS 2026-11-26
        for gx in (-1, 0, 1)
        for gy in (-1, 0, 1)
    ] + [(rng.uniform(-0.22, 0.22), rng.uniform(-0.22, 0.22))]
    for i, (cx, cy) in enumerate(grid_pts):  # new: DECISIONS 2026-11-26
        clumps.append((cx, cy))
        mound = common.make_icosphere(
            subdiv=1, jitter=0.22, seed=600 + i,
            radius=(rng.uniform(0.20, 0.28), rng.uniform(0.20, 0.28), 0.055),
            loc=(cx, cy, z0 + 0.055), color="dirt",
        )
        model.add(mound)

    # 200 tall leaning grass blades anchored inside the dirt mounds
    for idx in range(200):  # new: DECISIONS 2026-11-26
        cx, cy = clumps[idx % len(clumps)]
        ang = rng.uniform(0.0, 2.0 * math.pi)
        rad = rng.uniform(0.0, 0.18)  # new: DECISIONS 2026-11-26
        bx = max(-0.96, min(0.96, cx + math.cos(ang) * rad))
        by = max(-0.96, min(0.96, cy + math.sin(ang) * rad))
        dist_edge = max(abs(bx), abs(by))
        h = rng.uniform(0.58, 1.20) * (1.0 - 0.22 * dist_edge)  # new: DECISIONS 2026-11-26
        lean = rng.uniform(0.14, 0.52)
        yaw = rng.uniform(0.0, 2.0 * math.pi)
        r = rng.random()
        col = "grass" if r < 0.55 else ("leaf_mid" if r < 0.80 else "bark_light")  # new: DECISIONS 2026-11-26
        model.add(og_util.make_blade(bx, by, z0 + 0.015, h, lean, yaw, color=col, width=0.048))


def _build_moss_patch(model):
    """Organic 1 x 1 m flat moss mat, ~1-4 cm thick, swamp_moss + grime_green."""
    rng = random.Random(503)
    z0 = og_util.GROUND_Z_LIFT  # new: DECISIONS 2026-11-26

    # 5 subdiv=2 + 2 subdiv=1 cushion lobes = 440 tris, leaving plenty of budget under 600
    lobes = [
        (0.00, 0.00, 0.34, 0.30, 0.016, "swamp_moss", 2),    # new: DECISIONS 2026-11-26
        (0.24, 0.15, 0.24, 0.22, 0.013, "grime_green", 2),   # new: DECISIONS 2026-11-26
        (-0.25, 0.17, 0.22, 0.24, 0.013, "swamp_moss", 2),   # new: DECISIONS 2026-11-26
        (0.18, -0.24, 0.25, 0.21, 0.013, "swamp_moss", 2),   # new: DECISIONS 2026-11-26
        (-0.22, -0.21, 0.26, 0.22, 0.014, "grime_green", 2),  # new: DECISIONS 2026-11-26
        (0.36, -0.06, 0.14, 0.16, 0.011, "grime_green", 1),  # new: DECISIONS 2026-11-26
        (-0.36, 0.02, 0.15, 0.14, 0.011, "swamp_moss", 1),   # new: DECISIONS 2026-11-26
    ]
    for idx, (lx, ly, rx, ry, rz, col, sub) in enumerate(lobes):
        lobe = common.make_icosphere(
            subdiv=sub, jitter=0.14, seed=700 + idx,
            radius=(rx, ry, rz),
            loc=(lx, ly, z0 + rz),
            color=col,
        )
        model.add(lobe)

    # Small fuzzy moss tufts on top (14 * 6 = 84 tris -> total ~524 <= 600)
    for _ in range(14):  # new: DECISIONS 2026-11-26
        tx = rng.uniform(-0.34, 0.34)
        ty = rng.uniform(-0.34, 0.34)
        col = "grime_green" if rng.random() < 0.6 else "swamp_moss"
        model.add(og_util.make_blade(tx, ty, z0 + 0.020, rng.uniform(0.035, 0.07),
                                     rng.uniform(0.1, 0.45), rng.uniform(0.0, 6.28), color=col, width=0.028))


def _build_leaves_litter(model):
    """1.5 x 1.5 m litter of 34 small dead leaves and 5 paper scraps, lying 1-2 cm above ground."""
    rng = random.Random(504)
    z0 = og_util.GROUND_Z_LIFT  # new: DECISIONS 2026-11-26

    # 34 dead leaves (bark_light, dirt)
    for i in range(34):  # new: DECISIONS 2026-11-26
        lx = rng.uniform(-0.68, 0.68)  # new: DECISIONS 2026-11-26
        ly = rng.uniform(-0.68, 0.68)  # new: DECISIONS 2026-11-26
        lw = rng.uniform(0.09, 0.15)
        ll = rng.uniform(0.13, 0.21)
        col = "bark_light" if i % 3 != 0 else "dirt"
        leaf = og_util.make_leaf_diamond(lw, ll, 0.012, color=col)
        # Lie almost flat with slight random tilt so light catches facets
        common.transform(
            leaf,
            rot=(math.pi * 0.5 + rng.uniform(-0.18, 0.18), rng.uniform(-0.18, 0.18), rng.uniform(0.0, 6.28)),
            loc=(lx, ly, z0 + 0.014 + rng.uniform(0.0, 0.012)),
        )
        model.add(leaf)

    # 5 crumpled paper scraps spread across quadrants (white, concrete_grey)
    paper_locs = [
        (-0.45, -0.38), (0.42, -0.46), (-0.42, 0.40), (0.38, 0.36), (0.04, -0.06),  # new: DECISIONS 2026-11-26
    ]
    for i, (bx, by) in enumerate(paper_locs):  # new: DECISIONS 2026-11-26
        px = bx + rng.uniform(-0.12, 0.12)
        py = by + rng.uniform(-0.12, 0.12)
        col = "white" if i % 2 == 0 else "concrete_grey"
        scrap = og_util.make_paper_scrap(
            rng.uniform(0.18, 0.25), rng.uniform(0.22, 0.28), 0.006,
            bend_z=rng.uniform(0.012, 0.025), color=col,
        )
        common.transform(
            scrap,
            rot=(rng.uniform(-0.08, 0.08), rng.uniform(-0.08, 0.08), rng.uniform(0.0, 6.28)),
            loc=(px, py, z0 + 0.012 + i * 0.002),
        )
        model.add(scrap)


def build_all():
    """Builds weeds_crack, grass_tall_patch, moss_patch, leaves_litter. Returns [(name, ok)]."""
    builders = [
        ("weeds_crack", _build_weeds_crack),
        ("grass_tall_patch", _build_grass_tall_patch),
        ("moss_patch", _build_moss_patch),
        ("leaves_litter", _build_leaves_litter),
    ]
    return [
        (name, common.run_model(name, "nature", BUDGETS[name], fn, grip=(0.0, 0.0, 0.0)))
        for name, fn in builders
    ]
