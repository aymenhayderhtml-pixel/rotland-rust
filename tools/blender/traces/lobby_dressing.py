"""Lobby interior dressing (R39 leftover 1): column, hanging ceiling panel, broken lights, rug, chair, debris."""
import math
import os
import random
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, os.path.join(HERE, "..", "metro"))
sys.path.insert(0, os.path.join(HERE, "..", "vehicles"))

import car_parts  # noqa: E402
import common  # noqa: E402
from jam_util import pbox  # noqa: E402

COL_BUDGET = 1500  # new: DECISIONS 2026-11-26
CEIL_BUDGET = 1500  # new: DECISIONS 2026-11-26
LIGHT_BUDGET = 800  # new: DECISIONS 2026-11-26
RUG_BUDGET = 400  # new: DECISIONS 2026-11-26
CHAIR_BUDGET = 1000  # new: DECISIONS 2026-11-26
DEBRIS_BUDGET = 2000  # new: DECISIONS 2026-11-26


def _build_column(model):
    """0.6 x 0.6 m square lobby column, 6.8 m tall, limestone/concrete_grey with chipped corner and cap."""
    w, h = 0.60, 6.80  # new: DECISIONS 2026-11-26
    # Base plinth and top capital
    model.add(pbox((-0.36, -0.36, 0.0), (0.36, 0.36, 0.35), "concrete_grey", bevel=0.015))
    model.add(pbox((-0.38, -0.38, h - 0.38), (0.38, 0.38, h), "concrete_grey", bevel=0.015))
    model.add(pbox((-0.33, -0.33, h - 0.52), (0.33, 0.33, h - 0.38), "limestone_dark", bevel=0.01))
    # Lower and upper intact shaft
    model.add(pbox((-0.30, -0.30, 0.35), (0.30, 0.30, 1.35), "limestone", bevel=0.012))
    model.add(pbox((-0.30, -0.30, 2.35), (0.30, 0.30, h - 0.52), "limestone", bevel=0.012))
    # Chipped mid section (z = 1.35..2.35): missing (+X, +Y) corner with concrete_stain core and rebar
    model.add(common.make_box((0.34, 0.60, 1.00), loc=(-0.13, 0.0, 1.85), color="limestone"))
    model.add(common.make_box((0.26, 0.34, 1.00), loc=(0.17, -0.13, 1.85), color="limestone"))
    model.add(common.make_box((0.20, 0.20, 0.96), loc=(0.14, 0.14, 1.85), color="concrete_stain"))
    for rx, ry in ((0.22, 0.14), (0.14, 0.22)):
        model.add(common.make_frustum(0.012, 0.012, 1.38, 2.32, segments=6, center=(rx, ry), color="rust_dark"))
    model.add_collider("col", common.make_box((w, w, h), loc=(0.0, 0.0, h / 2), color="white"))


def _build_ceiling_hang(model):
    """Sagging 2.5 x 1.2 m ceiling panel hanging by 2 wires, concrete_grey with rust_dark stains."""
    panel = [
        common.make_box((2.50, 1.20, 0.05), color="concrete_grey"),
        common.make_box((2.54, 0.06, 0.07), loc=(0.0, -0.58, 0.0), color="metal"),
        common.make_box((2.54, 0.06, 0.07), loc=(0.0, 0.58, 0.0), color="metal"),
        common.make_box((0.06, 1.20, 0.07), loc=(-1.23, 0.0, 0.0), color="metal"),
        common.make_box((0.06, 1.20, 0.07), loc=(1.23, 0.0, 0.0), color="metal"),
        # Grid T-bars dividing the panel into acoustic ceiling tiles
        common.make_box((0.04, 1.16, 0.065), loc=(-0.42, 0.0, 0.0), color="metal"),
        common.make_box((0.04, 1.16, 0.065), loc=(0.42, 0.0, 0.0), color="metal"),
        common.make_box((2.46, 0.04, 0.065), loc=(0.0, 0.0, 0.0), color="metal"),
        common.make_box((0.85, 0.45, 0.06), loc=(-0.45, 0.15, 0.0), color="rust_dark"),
        common.make_box((0.60, 0.55, 0.06), loc=(0.55, -0.20, 0.0), color="rust_dark"),
    ]
    bm = common.combine(panel)
    # Tilt panel so one end hangs lower (around z = 0.0..0.95)
    common.transform(bm, rot=(0.24, 0.34, 0.0), loc=(0.0, 0.0, 0.55))
    model.add(bm)
    # Two taut hanging wires from the upper left edge up to ceiling anchor z = 1.95
    for wy in (-0.42, 0.42):
        model.add(car_parts._beam((-0.95, wy, 0.85), (-0.95, wy, 1.95), 0.012, color="metal", segments=4))
    # One snapped loose wire dangling from the low corner
    model.add(car_parts._beam((1.05, 0.25, 0.25), (1.18, 0.35, 0.80), 0.010, color="gun_black", segments=4))


def _build_light_broken(model):
    """Cluster of 2 broken light fixtures hanging on wires with glass_dark shards."""
    for idx, (ox, oy, oz, rot) in enumerate([(-0.45, 0.0, 0.72, (0.55, 0.35, 0.2)),
                                             (0.42, 0.22, 0.48, (-0.75, -0.45, -0.3))]):
        parts = [
            common.make_box((1.15, 0.26, 0.07), color="metal"),
            common.make_frustum(0.022, 0.022, -0.50, 0.50, segments=6, color="white"),
            common.make_box((0.32, 0.22, 0.015), loc=(0.24, 0.0, -0.05), color="glass_dark"),
            common.make_box((0.26, 0.18, 0.015), loc=(-0.28, 0.02, -0.06), color="glass_dark"),
        ]
        # Turn tube along X inside housing and tilt one end broken loose
        common.transform(parts[1], rot=(0.25, math.pi / 2 + 0.18, 0.0), loc=(0.0, 0.02, -0.08))
        fix = common.combine(parts)
        common.transform(fix, rot=rot, loc=(ox, oy, oz))
        model.add(fix)
        # Suspension wire up to z = 1.65
        anchor_x = ox - 0.35 if idx == 0 else ox + 0.32
        model.add(car_parts._beam((anchor_x, oy, oz + 0.12), (anchor_x, oy, 1.65),
                                  0.010, color="gun_black", segments=4))


def _build_rug(model):
    """Flat 2.0 x 1.4 m lobby rug, red_paint / brick_dark pattern, 1 cm thick."""
    model.add(common.make_box((2.00, 1.40, 0.008), loc=(0.0, 0.0, 0.004), color="brick_dark"))
    model.add(common.make_box((1.72, 1.12, 0.009), loc=(0.0, 0.0, 0.0045), color="red_paint"))
    model.add(common.make_box((1.20, 0.64, 0.010), loc=(0.0, 0.0, 0.005), color="brick_dark"))
    model.add(common.make_box((0.55, 0.26, 0.011), loc=(0.0, 0.0, 0.0055), color="brass"))
    for sx in (-1.02, 1.02):
        model.add(common.make_box((0.05, 1.32, 0.006), loc=(sx, 0.0, 0.003), color="bone_white"))


def _build_chair_tipped(model):
    """Upholstered wooden lobby chair tipped onto its side (wood_crate frame, leather seat/back)."""
    parts = [
        pbox((-0.24, -0.23, 0.42), (0.24, 0.23, 0.49), "leather", bevel=0.02),
        pbox((-0.23, -0.22, 0.49), (0.23, -0.15, 0.88), "leather", bevel=0.02),
    ]
    for sx in (-0.21, 0.21):
        for sy in (-0.20, 0.20):
            parts.append(common.make_box((0.045, 0.045, 0.42), loc=(sx, sy, 0.21), color="wood_crate"))
        parts.append(common.make_box((0.045, 0.045, 0.46), loc=(sx, -0.20, 0.66), color="wood_crate"))
        parts.append(common.make_box((0.05, 0.38, 0.04), loc=(sx, 0.0, 0.62), color="wood_crate"))
    chair = common.combine(parts)
    # Tip onto its right (+X) side
    common.transform(chair, rot=(0.0, math.pi / 2, 0.35), loc=(0.0, 0.0, 0.25))
    model.add(chair)


def _build_debris(model):
    """Pile of ~20 debris pieces: plaster slabs/chunks, glass_dark shards, a book, a shoe."""
    rng = random.Random(4059)
    # 9 fractured flat plaster slabs and rock chunks (crisp edges, not puffy blocks)
    for i in range(9):
        ang = 2 * math.pi * i / 9 + rng.uniform(-0.2, 0.2)
        dist = rng.uniform(0.12, 0.72)
        sx, sy, sz = rng.uniform(0.18, 0.38), rng.uniform(0.14, 0.30), rng.uniform(0.025, 0.065)
        col = ("concrete_grey", "concrete_stain", "limestone", "brick_red")[i % 4]
        chunk = common.make_box((sx, sy, sz), color=col)
        common.transform(chunk, rot=(rng.uniform(-0.25, 0.25), rng.uniform(-0.25, 0.25), rng.uniform(0, 3.14)),
                         loc=(math.cos(ang) * dist, math.sin(ang) * dist, sz / 2 + 0.01))
        model.add(chunk)
    # 6 jagged triangular glass_dark shards
    for i in range(6):
        w = rng.uniform(0.12, 0.24)
        shard = common.make_frustum(w * 0.55, 0.0, 0.0, 0.012, segments=3, color="glass_dark")
        common.transform(shard, rot=(rng.uniform(-0.15, 0.15), rng.uniform(-0.15, 0.15), rng.uniform(0, 3.14)),
                         loc=(rng.uniform(-0.65, 0.65), rng.uniform(-0.55, 0.55), 0.015))
        model.add(shard)
    # Open book (cover + angled open white page wings)
    book = common.combine([
        common.make_box((0.28, 0.20, 0.012), loc=(0.0, 0.0, 0.006), color="shirt_red"),
        common.make_box((0.12, 0.18, 0.014), loc=(-0.065, 0.0, 0.022), color="white"),
        common.make_box((0.12, 0.18, 0.014), loc=(0.065, 0.0, 0.022), color="bone_white"),
    ])
    common.transform(book, rot=(0.06, -0.08, 0.65), loc=(-0.35, -0.45, 0.02))
    model.add(book)
    # 1 discarded shoe placed cleanly clear of chunks
    shoe = common.combine([
        pbox((-0.055, -0.14, 0.0), (0.055, 0.14, 0.03), "rock_dark", bevel=0.006),
        pbox((-0.05, -0.12, 0.03), (0.05, 0.12, 0.09), "shoe_dark", bevel=0.012),
    ])
    common.transform(shoe, rot=(0.15, 0.35, -0.8), loc=(0.68, 0.48, 0.02))
    model.add(shoe)
    # 2 bent metal ceiling strips
    for mx, my, rz in ((-0.15, 0.38, 0.4), (0.32, -0.35, -0.7)):
        strip = common.make_box((0.55, 0.04, 0.015), color="metal")
        common.transform(strip, rot=(0.1, 0.18, rz), loc=(mx, my, 0.05))
        model.add(strip)


def build_all():
    """Builds the 6 lobby dressing models."""
    return [
        ("lobby_column", common.run_model("lobby_column", "props", COL_BUDGET, _build_column)),
        ("lobby_ceiling_hang", common.run_model("lobby_ceiling_hang", "props", CEIL_BUDGET, _build_ceiling_hang)),
        ("lobby_light_broken", common.run_model("lobby_light_broken", "props", LIGHT_BUDGET, _build_light_broken)),
        ("lobby_rug", common.run_model("lobby_rug", "props", RUG_BUDGET, _build_rug)),
        ("lobby_chair_tipped", common.run_model("lobby_chair_tipped", "props", CHAIR_BUDGET, _build_chair_tipped)),
        ("lobby_debris", common.run_model("lobby_debris", "props", DEBRIS_BUDGET, _build_debris)),
    ]
