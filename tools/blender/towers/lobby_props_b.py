"""Lobby interior props (part 2): bench, rubble pile, dead planter."""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, os.path.join(HERE, "..", "metro"))

import common  # noqa: E402
import jam_util  # noqa: E402

BUDGET = 6000


def build_bench(model):
    """2.0 x 0.5 x 0.45 m lobby bench with one broken wooden slat."""
    for x in (-0.82, 0.0, 0.82):
        # Leg frame: two vertical posts + seat bracket + foot pads
        model.add(jam_util.pbox((x - 0.03, -0.22, 0.0), (x + 0.03, -0.16, 0.39), "metal", bevel=0.008))
        model.add(jam_util.pbox((x - 0.03, 0.16, 0.0), (x + 0.03, 0.22, 0.39), "metal", bevel=0.008))
        model.add(jam_util.pbox((x - 0.035, -0.24, 0.36), (x + 0.035, 0.24, 0.41), "metal", bevel=0.008))
        model.add(jam_util.pbox((x - 0.045, -0.24, 0.0), (x + 0.045, -0.14, 0.025), "gun_black", bevel=0.004))
        model.add(jam_util.pbox((x - 0.045, 0.14, 0.0), (x + 0.045, 0.24, 0.025), "gun_black", bevel=0.004))

    # Horizontal stretcher bar
    model.add(jam_util.pbox((-0.85, -0.02, 0.14), (0.85, 0.02, 0.18), "metal", bevel=0.006))

    # 4 wooden slats along the seat; the front-most slat is broken in the middle
    for idx, y in enumerate((-0.18, -0.06, 0.06)):
        color = "bark_light" if idx % 2 == 0 else "wood_crate"
        model.add(jam_util.pbox((-1.0, y - 0.045, 0.41), (1.0, y + 0.045, 0.45), color, bevel=0.008))

    # Broken front slat: left piece intact, right piece cracked and drooping
    model.add(jam_util.pbox((-1.0, 0.135, 0.41), (0.05, 0.225, 0.45), "bark_light", bevel=0.008))
    broken = jam_util.pbox((-0.42, -0.045, -0.02), (0.42, 0.045, 0.02), "bark_dark", bevel=0.008)
    common.transform(broken, rot=(0.12, 0.32, -0.05), loc=(0.56, 0.18, 0.29))
    model.add(broken)

    model.add_collider("bench", common.make_box((2.0, 0.5, 0.45), loc=(0.0, 0.0, 0.225), color="white"))


def build_rubble(model):
    """2.5 x 2.0 m rubble pile up to 0.35 m: concrete, plaster, red bricks, fallen ceiling tile."""
    # Low stepped rubble mound base (organic silhouette, not a flat plate)
    model.add(jam_util.pbox((-0.95, -0.75, 0.0), (0.85, 0.70, 0.08), "concrete_stain", bevel=0.03))
    model.add(jam_util.pbox((-0.70, -0.55, 0.06), (0.65, 0.55, 0.16), "rock_dark", bevel=0.03))

    # Broken concrete chunks
    chunks = [
        ((-0.55, -0.35, 0.12), (0.15, 0.25, 0.26), (0.08, -0.14, 0.3), "rock_mid"),
        ((0.20, -0.50, 0.10), (0.75, -0.05, 0.24), (-0.12, 0.18, -0.4), "concrete_grey"),
        ((-0.90, 0.15, 0.08), (-0.35, 0.60, 0.21), (0.15, 0.10, 0.7), "rock_mid"),
        ((0.10, 0.20, 0.14), (0.55, 0.65, 0.28), (-0.18, -0.08, 0.2), "concrete_grey"),
    ]
    for lo, hi, rot, col in chunks:
        cx, cy, cz = (lo[0] + hi[0]) / 2, (lo[1] + hi[1]) / 2, (lo[2] + hi[2]) / 2
        bm = jam_util.pbox((lo[0] - cx, lo[1] - cy, lo[2] - cz),
                           (hi[0] - cx, hi[1] - cy, hi[2] - cz), col, bevel=0.02)
        common.transform(bm, rot=rot, loc=(cx, cy, cz))
        model.add(bm)

    # Plaster slabs
    for loc, rot in [((-0.25, 0.35, 0.22), (0.12, -0.22, 0.5)), ((0.45, -0.20, 0.20), (-0.14, 0.15, -0.6))]:
        bm = jam_util.pbox((-0.22, -0.16, -0.025), (0.22, 0.16, 0.025), "white", bevel=0.008)
        common.transform(bm, rot=rot, loc=loc)
        model.add(bm)

    # Scattered red brick bits
    bricks = [
        ((-0.65, -0.60, 0.13), (0.05, 0.12, 0.4), "brick_red"),
        ((-0.10, -0.70, 0.12), (-0.10, 0.08, 1.1), "brick_dark"),
        ((0.75, 0.25, 0.13), (0.14, -0.12, -0.3), "brick_red"),
        ((-0.45, 0.05, 0.27), (0.20, 0.15, 0.8), "brick_red"),
        ((0.30, 0.48, 0.14), (-0.08, -0.18, 2.0), "brick_dark"),
    ]
    for loc, rot, col in bricks:
        bm = jam_util.pbox((-0.11, -0.055, -0.035), (0.11, 0.055, 0.035), col, bevel=0.008)
        common.transform(bm, rot=rot, loc=loc)
        model.add(bm)

    # Fallen ceiling tile + bent T-bar rail
    tile = jam_util.pbox((-0.30, -0.30, -0.015), (0.30, 0.30, 0.015), "mortar", bevel=0.006)
    common.transform(tile, rot=(0.22, -0.18, 0.45), loc=(-0.15, -0.10, 0.29))
    model.add(tile)
    tbar = jam_util.pbox((-0.45, -0.02, -0.015), (0.45, 0.02, 0.015), "metal", bevel=0.004)
    common.transform(tbar, rot=(-0.15, 0.24, -0.35), loc=(0.25, 0.15, 0.27))
    model.add(tbar)

    model.add_collider("rubble", common.make_box((2.5, 2.0, 0.35), loc=(0.0, 0.0, 0.175), color="white"))


def build_plant_dead(model):
    """0.9 x 0.9 x 0.7 m concrete planter with dead shrub stems and dry leaves."""
    # Planter plinth, body, rim, and corner pilasters
    model.add(jam_util.pbox((-0.41, -0.41, 0.0), (0.41, 0.41, 0.06), "rock_dark", bevel=0.01))
    model.add(jam_util.pbox((-0.43, -0.43, 0.06), (0.43, 0.43, 0.62), "rock_mid", bevel=0.015))
    model.add(jam_util.pbox((-0.45, -0.45, 0.62), (0.45, 0.45, 0.70), "limestone", bevel=0.015))
    for sx in (-0.42, 0.42):
        for sy in (-0.42, 0.42):
            model.add(jam_util.pbox((sx - 0.03, sy - 0.03, 0.06), (sx + 0.03, sy + 0.03, 0.62),
                                    "concrete_stain", bevel=0.008))

    # Soil bed
    model.add(jam_util.pbox((-0.36, -0.36, 0.65), (0.36, 0.36, 0.71), "soil_dark", bevel=0.01))

    # Dead woody stems & twigs (all anchored into the soil or main stem)
    stems = [
        ((0.0, 0.0, 0.68), (0.04, 0.04, 1.32), (0.08, -0.06, 0.0), 0.045, 0.022),
        ((-0.08, 0.06, 0.68), (0.03, 0.03, 1.18), (-0.18, -0.14, 0.4), 0.035, 0.016),
        ((0.09, -0.06, 0.68), (0.03, 0.03, 1.12), (0.14, 0.18, -0.3), 0.032, 0.015),
        ((0.01, -0.02, 0.96), (0.02, 0.02, 1.36), (-0.25, 0.22, 0.8), 0.022, 0.008),
        ((-0.02, 0.02, 0.92), (0.02, 0.02, 1.25), (0.28, 0.12, -0.6), 0.020, 0.008),
    ]
    for base, top, rot, r0, r1 in stems:
        h = top[2] - base[2]
        bm = common.make_frustum(r0, r1, 0.0, h, segments=6, color="bark_dark")
        common.transform(bm, rot=rot, loc=base)
        model.add(bm)

    # Dry leaves clinging to twigs and fallen on soil
    leaves = [
        ((0.08, -0.06, 1.25), (0.4, 0.2, 0.5)),
        ((-0.16, 0.14, 1.10), (-0.3, 0.5, 1.2)),
        ((0.18, -0.15, 1.02), (0.5, -0.3, -0.4)),
        ((-0.20, -0.12, 0.72), (0.05, 0.08, 0.9)),
        ((0.16, 0.22, 0.72), (-0.06, 0.04, 2.1)),
    ]
    for loc, rot in leaves:
        leaf = jam_util.pbox((-0.06, -0.03, -0.006), (0.06, 0.03, 0.006), "dry_brush", bevel=0.003)
        common.transform(leaf, rot=rot, loc=loc)
        model.add(leaf)

    model.add_collider("planter", common.make_box((0.9, 0.9, 0.7), loc=(0.0, 0.0, 0.35), color="white"))


def build_all():
    return [
        ("lobby_bench", common.run_model("lobby_bench", "buildings", BUDGET, build_bench)),
        ("lobby_rubble", common.run_model("lobby_rubble", "buildings", BUDGET, build_rubble)),
        ("lobby_plant_dead", common.run_model("lobby_plant_dead", "buildings", BUDGET, build_plant_dead)),
    ]
