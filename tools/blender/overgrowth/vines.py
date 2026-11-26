"""Wall and cornice vine sheets: vine_wall_s, vine_wall_m, vine_wall_l, vine_cornice."""
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

# new: DECISIONS 2026-11-26 (front +Y view so preview camera faces the ivy sheet)
VINE_VIEW = (0.55, 1.0, 0.30)

# new: DECISIONS 2026-11-26 (width, height, leaf_count, strand_count, stem_count, leaf_scale, seed, budget)
WALL_SPECS = {
    "vine_wall_s": (2.6, 4.0, 125, 3, 5, 1.0, 101, 1500),   # data: budgets.json nature
    "vine_wall_m": (3.0, 8.0, 260, 4, 6, 1.35, 202, 3000),  # data: budgets.json nature
    "vine_wall_l": (3.2, 14.0, 520, 5, 7, 1.75, 303, 6000), # data: budgets.json nature
}

# new: DECISIONS 2026-11-26
CORNICE_SPEC = {
    "width": 3.0,
    "drop": 1.2,
    "top_depth": 0.32,
    "leaves": 115,
    "strands": 4,
    "seed": 404,
    "budget": 1500,  # data: budgets.json nature
}


def _pick_leaf_color(rng):
    r = rng.random()
    if r < 0.05:  # new: DECISIONS 2026-11-26 (5% dark wall gaps inside foliage)
        return "rock_dark"
    if r < 0.45:  # new: DECISIONS 2026-11-26
        return "leaf_dark"
    if r < 0.85:  # new: DECISIONS 2026-11-26
        return "leaf_mid"
    return "leaf_light"


def _add_climbing_stems(model, width, height, stem_count, rng):
    """Branching woody vines from Z=0 up the wall at Y=0.02..0.045; returns stem sample points."""
    segs = max(4, int(height * 0.65))  # new: DECISIONS 2026-11-26
    anchors = []
    for i in range(stem_count):
        t = (i + 0.5) / stem_count - 0.5
        x = t * width * 0.76  # new: DECISIONS 2026-11-26
        y = 0.022 + rng.uniform(0.0, 0.015)  # new: DECISIONS 2026-11-26
        top_z = height * rng.uniform(0.72, 0.98)  # new: DECISIONS 2026-11-26
        r0 = 0.032 if i % 2 == 0 else 0.022  # new: DECISIONS 2026-11-26
        pts = [(x, y, 0.0)]
        for s in range(1, segs + 1):
            frac = s / segs
            x_cur = x + math.sin(frac * 4.5 + i) * 0.22 + rng.uniform(-0.08, 0.08)  # new: DECISIONS 2026-11-26
            x_cur = max(-width * 0.46, min(width * 0.46, x_cur))
            y_cur = 0.022 + 0.018 * (1.0 - frac) + rng.uniform(0.0, 0.012)  # new: DECISIONS 2026-11-26
            pts.append((x_cur, y_cur, frac * top_z))
        for a, b in zip(pts, pts[1:]):
            ra = r0 * (1.0 - 0.60 * (a[2] / max(height, 0.1)))  # new: DECISIONS 2026-11-26
            rb = r0 * (1.0 - 0.65 * (b[2] / max(height, 0.1)))  # new: DECISIONS 2026-11-26
            model.add(og_util.make_twig(a, b, ra, max(0.006, rb), color="bark_dark", segs=4))
            anchors.append(((a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5, (a[2] + b[2]) * 0.5))
    return anchors


def _add_hanging_strands(model, anchors, z_min, z_max, strand_count, rng):
    """3-5 thin hanging dark-leaf strands dangling from upper stem anchors (never floating)."""
    top_anchors = [a for a in anchors if z_min <= a[2] <= z_max] or anchors
    for i in range(strand_count):
        ax, ay, az = top_anchors[(i * max(1, len(top_anchors) // strand_count)) % len(top_anchors)]
        x = ax + rng.uniform(-0.04, 0.04)  # new: DECISIONS 2026-11-26
        z_top = az
        length = rng.uniform(0.6, 1.5)  # new: DECISIONS 2026-11-26
        y_top = max(ay + 0.02, 0.045)  # new: DECISIONS 2026-11-26
        y_bot = min(0.27, y_top + rng.uniform(0.06, 0.14))  # new: DECISIONS 2026-11-26
        mid_z = z_top - length * 0.5  # new: DECISIONS 2026-11-26
        mid_x = x + rng.uniform(-0.06, 0.06)  # new: DECISIONS 2026-11-26
        mid_y = (y_top + y_bot) * 0.5 + 0.02  # new: DECISIONS 2026-11-26

        # Thin hanging box strand + small leaves along it
        for p0, p1 in [((x, y_top, z_top), (mid_x, mid_y, mid_z)),
                       ((mid_x, mid_y, mid_z), (x + rng.uniform(-0.05, 0.05), y_bot, z_top - length))]:
            seg_len = math.dist(p0, p1)
            box = common.make_box((0.032, 0.024, seg_len), color="leaf_dark")  # new: DECISIONS 2026-11-26
            dx, dy, dz = p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]
            pitch = math.atan2(dy, -dz)
            roll = math.atan2(dx, -dz)
            cx = (p0[0] + p1[0]) * 0.5
            cy = (p0[1] + p1[1]) * 0.5
            cz = (p0[2] + p1[2]) * 0.5
            common.transform(box, rot=(pitch, roll, 0.0), loc=(cx, cy, cz))
            model.add(box)

        for k in range(4):  # new: DECISIONS 2026-11-26
            frac = (k + 0.5) / 4.0
            lx = x + rng.uniform(-0.06, 0.06)
            ly = y_top + (y_bot - y_top) * frac + 0.015
            lz = z_top - length * frac
            leaf = og_util.make_leaf_diamond(0.11, 0.15, 0.02, color="leaf_dark" if k % 2 == 0 else "leaf_mid")
            common.transform(leaf, rot=(rng.uniform(0.1, 0.4), rng.uniform(-0.3, 0.3), rng.uniform(-0.5, 0.5)),
                             loc=(lx, ly, lz))
            model.add(leaf)


def _build_wall_vine(spec_name):
    width, height, leaf_count, strand_count, stem_count, leaf_scale, seed, _ = WALL_SPECS[spec_name]

    def build(model):
        rng = random.Random(seed)
        anchors = _add_climbing_stems(model, width, height, stem_count, rng)

        # Group leaves into dense clusters along the climbing stems, thick at bottom and thinning up
        clusters = max(16, leaf_count // 4)  # new: DECISIONS 2026-11-26
        placed = 0
        while placed < leaf_count:
            # Bias cluster height toward the bottom so base is lush and upper half branches out
            t_z = rng.random() ** 1.4  # new: DECISIONS 2026-11-26
            # Pick a stem anchor near this height
            candidates = [a for a in anchors if abs(a[2] / max(height, 0.1) - t_z) < 0.22]  # new: DECISIONS 2026-11-26
            ax, _, az = rng.choice(candidates or anchors)
            cx = ax + rng.uniform(-0.22, 0.22)
            cz = max(0.12, min(height - 0.12, az + rng.uniform(-0.20 * leaf_scale, 0.20 * leaf_scale)))
            cluster_size = min(leaf_count - placed, rng.randint(3, 5))  # new: DECISIONS 2026-11-26
            for _ in range(cluster_size):
                col = _pick_leaf_color(rng)
                spread = 0.22 * leaf_scale  # new: DECISIONS 2026-11-26
                lx = max(-width * 0.5, min(width * 0.5, cx + rng.uniform(-spread, spread)))
                lz = max(0.06, min(height - 0.05, cz + rng.uniform(-spread, spread)))
                if col == "rock_dark":
                    ly = rng.uniform(0.014, 0.020)  # new: DECISIONS 2026-11-26 (tucked behind leaves)
                    lw = rng.uniform(0.16, 0.26) * leaf_scale
                    ll = rng.uniform(0.20, 0.32) * leaf_scale
                else:
                    ly = rng.uniform(0.028, 0.075 + 0.12 * (1.0 - t_z * 0.6))  # new: DECISIONS 2026-11-26
                    lw = rng.uniform(0.20, 0.34) * leaf_scale
                    ll = rng.uniform(0.24, 0.40) * leaf_scale
                leaf = og_util.make_leaf_diamond(lw, ll, 0.024, color=col)
                rot = (
                    rng.uniform(-0.35, 0.45),
                    rng.uniform(-0.45, 0.45),
                    rng.uniform(-0.30, 0.30),
                )
                common.transform(leaf, rot=rot, loc=(lx, ly, lz))
                model.add(leaf)
                placed += 1

        _add_hanging_strands(model, anchors, height * 0.66, height * 0.94, strand_count, rng)

    return build


def _build_cornice(model):
    """3.0 m wide ivy sheet draped over a ledge edge (Z=0 ledge top, hanging 1.2 m down)."""
    s = CORNICE_SPEC
    rng = random.Random(s["seed"])
    w = s["width"]
    drop = s["drop"]
    top_d = s["top_depth"]

    # Horizontal woody stems curling over the ledge edge (Y = -top_d..+0.03, Z = +0.025)
    stem_xs = []
    cornice_anchors = []
    for i in range(6):  # new: DECISIONS 2026-11-26
        x = ((i + 0.5) / 6.0 - 0.5) * w * 0.84
        stem_xs.append(x)
        cornice_anchors.append((x, 0.03, 0.0))
        model.add(og_util.make_twig((x, -top_d, 0.025), (x + rng.uniform(-0.08, 0.08), 0.03, 0.025),
                                    0.022, 0.016, color="bark_dark", segs=4))
        model.add(og_util.make_twig((x, 0.03, 0.025), (x + rng.uniform(-0.12, 0.12), 0.038, -drop * 0.88),
                                    0.016, 0.008, color="bark_dark", segs=4))

    # Top mat resting on ledge (Z = +0.02..+0.06, Y = -top_d..0)
    top_leaves = s["leaves"] // 3
    for _ in range(top_leaves):
        x = rng.uniform(-w * 0.48, w * 0.48)
        y = rng.uniform(-top_d, 0.02)
        z = rng.uniform(0.022, 0.06)  # new: DECISIONS 2026-11-26
        col = _pick_leaf_color(rng)
        if col == "rock_dark":
            z = 0.018  # new: DECISIONS 2026-11-26
        leaf = og_util.make_leaf_diamond(rng.uniform(0.18, 0.28), rng.uniform(0.20, 0.32), 0.022, color=col)
        common.transform(leaf, rot=(math.pi * 0.5 + rng.uniform(-0.2, 0.2), rng.uniform(-0.3, 0.3),
                                    rng.uniform(0.0, 3.14)), loc=(x, y, z))
        model.add(leaf)

    # Front hanging curtain clustered around stems (Z = -drop..0, Y = +0.02..+0.12)
    for idx in range(s["leaves"] - top_leaves):
        sx = stem_xs[idx % len(stem_xs)]
        t_z = rng.random() ** 1.15  # new: DECISIONS 2026-11-26
        z = -t_z * drop
        x = max(-w * 0.48, min(w * 0.48, sx + rng.uniform(-0.26, 0.26)))
        col = _pick_leaf_color(rng)
        y = 0.018 if col == "rock_dark" else rng.uniform(0.028, 0.11)  # new: DECISIONS 2026-11-26
        leaf = og_util.make_leaf_diamond(rng.uniform(0.18, 0.28), rng.uniform(0.20, 0.32), 0.022, color=col)
        common.transform(leaf, rot=(rng.uniform(-0.3, 0.4), rng.uniform(-0.4, 0.4), rng.uniform(-0.25, 0.25)),
                         loc=(x, y, z))
        model.add(leaf)

    _add_hanging_strands(model, cornice_anchors, -0.15, 0.05, s["strands"], rng)


def build_all():
    """Builds vine_wall_s, vine_wall_m, vine_wall_l, vine_cornice. Returns [(name, ok)]."""
    results = []
    for name, (_, _, _, _, _, _, _, budget) in WALL_SPECS.items():
        ok = common.run_model(name, "nature", budget, _build_wall_vine(name),
                              grip=(0.0, 0.0, 0.0), view=VINE_VIEW)
        results.append((name, ok))
    ok = common.run_model("vine_cornice", "nature", CORNICE_SPEC["budget"], _build_cornice,
                          grip=(0.0, 0.0, 0.0), view=VINE_VIEW)
    results.append(("vine_cornice", ok))
    return results
