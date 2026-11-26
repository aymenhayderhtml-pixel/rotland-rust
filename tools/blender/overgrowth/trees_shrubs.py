"""Urban trees and wild shrubs: tree_asphalt, sapling_city, shrub_wild, roof_bush, roof_tree."""
import math
import os
import random
import sys

_HERE = os.path.dirname(os.path.abspath(__file__))
_BLENDER_DIR = os.path.abspath(os.path.join(_HERE, ".."))
_PROPS_DIR = os.path.join(_BLENDER_DIR, "props")
for _p in (_HERE, _BLENDER_DIR, _PROPS_DIR):
    if _p not in sys.path:
        sys.path.insert(0, _p)

import common  # noqa: E402
import og_util  # noqa: E402
import veg  # noqa: E402

# data: budgets.json nature
BUDGETS = {
    "tree_asphalt": 6000,
    "sapling_city": 1500,
    "shrub_wild": 2000,
    "roof_bush": 2000,
    "roof_tree": 6000,
}


def _build_tree_asphalt(model):
    """4.5 m young tree breaking through a cracked, heaved 1.6 x 1.6 m asphalt slab. No collider."""
    rng = random.Random(601)

    # Four heaved-up broken asphalt plates radiating from the central root hole (1.6 x 1.6 m total)
    plates = [
        ((-0.42, -0.40, 0.07), (0.14, -0.15, 0.05), (0.76, 0.74, 0.14)),   # new: DECISIONS 2026-11-26
        ((0.42, -0.40, 0.08), (0.16, 0.14, -0.04), (0.76, 0.74, 0.14)),    # new: DECISIONS 2026-11-26
        ((-0.40, 0.42, 0.08), (-0.15, -0.14, -0.06), (0.74, 0.76, 0.14)),  # new: DECISIONS 2026-11-26
        ((0.42, 0.42, 0.07), (-0.13, 0.16, 0.05), (0.74, 0.76, 0.14)),     # new: DECISIONS 2026-11-26
    ]
    for loc, rot, size in plates:
        slab = common.make_box(size, color="rock_dark")
        common.transform(slab, rot=rot, loc=loc)
        model.add(slab)

    # Two dark crack strips along the broken slab seams
    for rot_z in (0.04, 1.58):  # new: DECISIONS 2026-11-26
        crack = common.make_box((1.56, 0.06, 0.03), color="bark_dark")
        common.transform(crack, rot=(0.0, 0.0, rot_z), loc=(0.0, 0.0, 0.15))
        model.add(crack)

    # Dirt mound emerging in the central asphalt break
    model.add(common.make_icosphere(
        subdiv=2, jitter=0.22, seed=602, radius=(0.36, 0.34, 0.14),
        loc=(0.0, 0.0, 0.14), color="dirt",
    ))

    # Gnarled young trunk + root flare emerging from the slab hole
    veg.trunk(model, 0.15, 0.09, 2.7, "bark_dark", seed=603, roots=5, segs=7)  # new: DECISIONS 2026-11-26
    veg.leafy_crown(
        model, seed=603, center=(0.12, -0.05, 3.6), size=(1.35, 1.25, 1.15),  # new: DECISIONS 2026-11-26
        colors=("leaf_dark", "leaf_mid", "leaf_light"), count=8, blob_r=0.58,
        branch_from=2.2, bark="bark_dark", limbs=4, limb_r=0.07,
    )
    # Small weed blades pushing up from the cracks
    for _ in range(8):  # new: DECISIONS 2026-11-26
        ang = rng.uniform(0.0, 2.0 * math.pi)
        dist = rng.uniform(0.24, 0.55)
        model.add(og_util.make_blade(
            math.cos(ang) * dist, math.sin(ang) * dist, 0.14,
            rng.uniform(0.18, 0.32), rng.uniform(0.2, 0.5), ang, color="grass", width=0.03,
        ))


def _build_sapling_city(model):
    """1.8 m city sapling emerging from a dirt patch in a broken pavement hole. No collider."""
    # Dirt mound + broken pavement rim chunks
    model.add(common.make_icosphere(
        subdiv=2, jitter=0.22, seed=610, radius=(0.38, 0.35, 0.07),
        loc=(0.0, 0.0, 0.06), color="dirt",
    ))
    og_util.add_pavement_rim(model, radius=0.42, pieces=6, seed=611)  # new: DECISIONS 2026-11-26

    # Slender forked sapling trunk
    model.add(og_util.make_twig((0.0, 0.0, 0.04), (0.04, -0.02, 0.95), 0.055, 0.035, color="bark_light", segs=6))
    branches = [
        ((0.04, -0.02, 0.90), (0.28, 0.18, 1.55), 0.028, 0.012),    # new: DECISIONS 2026-11-26
        ((0.04, -0.02, 0.95), (-0.26, -0.16, 1.62), 0.026, 0.012),  # new: DECISIONS 2026-11-26
        ((0.03, -0.01, 0.78), (-0.12, 0.28, 1.38), 0.022, 0.010),   # new: DECISIONS 2026-11-26
        ((0.04, -0.02, 0.95), (0.05, -0.04, 1.72), 0.030, 0.014),   # new: DECISIONS 2026-11-26
    ]
    for s, e, r0, r1 in branches:
        model.add(og_util.make_twig(s, e, r0, r1, color="bark_light", segs=5))

    # Airy sapling leaf clusters
    clusters = [
        ((0.04, -0.03, 1.74), 0.34, "leaf_light"),  # new: DECISIONS 2026-11-26
        ((0.28, 0.18, 1.52), 0.28, "leaf_mid"),     # new: DECISIONS 2026-11-26
        ((-0.26, -0.16, 1.58), 0.28, "leaf_mid"),   # new: DECISIONS 2026-11-26
        ((-0.12, 0.28, 1.36), 0.24, "leaf_dark"),   # new: DECISIONS 2026-11-26
        ((0.16, -0.22, 1.24), 0.24, "leaf_dark"),   # new: DECISIONS 2026-11-26
        ((-0.02, 0.02, 1.44), 0.26, "leaf_mid"),    # new: DECISIONS 2026-11-26
    ]
    for idx, (loc, rad, col) in enumerate(clusters):
        model.add(common.make_icosphere(
            subdiv=2, jitter=0.22, seed=620 + idx,
            radius=(rad, rad, rad * 0.85), loc=loc, color=col,
        ))


def _build_shrub_wild(model):
    """1.6 m wild overgrown shrub: leaf_dark + pine_mid blobs with dead bark_light twigs poking out."""
    rng = random.Random(630)
    # Woody base stems
    for i in range(4):  # new: DECISIONS 2026-11-26
        ang = 2.0 * math.pi * i / 4.0 + 0.3
        model.add(og_util.make_twig(
            (0.0, 0.0, 0.0), (math.cos(ang) * 0.35, math.sin(ang) * 0.35, 0.70),
            0.045, 0.020, color="bark_dark", segs=5,
        ))

    blobs = [
        (0.00, 0.00, 0.65, 0.56, "leaf_dark"),   # new: DECISIONS 2026-11-26
        (0.42, 0.16, 0.52, 0.42, "pine_mid"),    # new: DECISIONS 2026-11-26
        (-0.40, 0.22, 0.48, 0.44, "leaf_dark"),  # new: DECISIONS 2026-11-26
        (0.16, -0.44, 0.50, 0.42, "pine_mid"),   # new: DECISIONS 2026-11-26
        (-0.30, -0.34, 0.64, 0.38, "leaf_dark"),  # new: DECISIONS 2026-11-26
        (0.28, 0.42, 0.78, 0.36, "leaf_mid"),    # new: DECISIONS 2026-11-26
        (-0.08, 0.06, 1.15, 0.44, "pine_mid"),   # new: DECISIONS 2026-11-26
        (0.26, -0.14, 1.02, 0.34, "leaf_dark"),  # new: DECISIONS 2026-11-26
        (-0.26, 0.14, 1.08, 0.32, "leaf_mid"),   # new: DECISIONS 2026-11-26
    ]
    for idx, (x, y, z, r, col) in enumerate(blobs):
        model.add(common.make_icosphere(
            subdiv=2, jitter=0.22, seed=640 + idx,
            radius=(r, r, r * 0.85), loc=(x, y, z), color=col,
        ))

    # Dead twigs poking out through the shrub crown
    for i in range(8):  # new: DECISIONS 2026-11-26
        ang = rng.uniform(0.0, 2.0 * math.pi)
        z_start = rng.uniform(0.35, 0.88)
        r_out = rng.uniform(0.68, 0.96)  # new: DECISIONS 2026-11-26
        z_end = z_start + rng.uniform(0.35, 0.65)
        col = "bark_light" if i % 3 != 0 else "bark_dark"
        end_pt = (math.cos(ang) * r_out, math.sin(ang) * r_out, z_end)
        model.add(og_util.make_twig(
            (math.cos(ang) * 0.25, math.sin(ang) * 0.25, z_start),
            end_pt, 0.028, 0.010, color=col, segs=4,
        ))
        if i % 2 == 0:
            # Small fork near twig tip
            fork_ang = ang + rng.uniform(0.35, 0.65)
            model.add(og_util.make_twig(
                (end_pt[0] * 0.78, end_pt[1] * 0.78, (z_start + z_end) * 0.55),
                (math.cos(fork_ang) * (r_out + 0.08), math.sin(fork_ang) * (r_out + 0.08), z_end + 0.08),
                0.015, 0.006, color=col, segs=4,
            ))


def _build_roof_bush(model):
    """1.5 m roof bush with a bark_dark stub and root collar. No collider."""
    # Gnarled stub + spreading branches
    veg.trunk(model, 0.11, 0.06, 0.55, "bark_dark", seed=650, roots=4, segs=6)  # new: DECISIONS 2026-11-26
    for ang, dz in ((0.4, 0.95), (2.5, 0.88), (4.6, 0.92)):  # new: DECISIONS 2026-11-26
        model.add(og_util.make_twig(
            (0.0, 0.0, 0.40), (math.cos(ang) * 0.38, math.sin(ang) * 0.38, dz),
            0.042, 0.018, color="bark_dark", segs=5,
        ))

    blobs = [
        (0.00, 0.00, 0.78, 0.46, "leaf_dark"),   # new: DECISIONS 2026-11-26
        (0.34, 0.18, 0.68, 0.34, "leaf_mid"),    # new: DECISIONS 2026-11-26
        (-0.32, 0.20, 0.70, 0.34, "pine_mid"),   # new: DECISIONS 2026-11-26
        (0.12, -0.34, 0.66, 0.34, "leaf_dark"),  # new: DECISIONS 2026-11-26
        (-0.24, -0.26, 0.84, 0.30, "leaf_mid"),  # new: DECISIONS 2026-11-26
        (0.06, 0.04, 1.12, 0.36, "leaf_mid"),    # new: DECISIONS 2026-11-26
        (-0.18, 0.08, 1.06, 0.28, "leaf_dark"),  # new: DECISIONS 2026-11-26
    ]
    for idx, (x, y, z, r, col) in enumerate(blobs):
        model.add(common.make_icosphere(
            subdiv=2, jitter=0.22, seed=660 + idx,
            radius=(r, r, r * 0.82), loc=(x, y, z), color=col,
        ))


def _build_roof_tree(model):
    """3.5 m small roof tree: short 0.6 m trunk before fork, round crown. No collider."""
    # Low dirt/moss root pad + short 0.6 m trunk + spreading limbs
    model.add(common.make_icosphere(
        subdiv=2, jitter=0.18, seed=670, radius=(0.45, 0.42, 0.08),
        loc=(0.0, 0.0, 0.06), color="dirt",
    ))
    veg.trunk(model, 0.15, 0.11, 0.65, "bark_dark", seed=671, roots=5, segs=7)  # new: DECISIONS 2026-11-26
    veg.leafy_crown(
        model, seed=671, center=(0.05, 0.0, 2.25), size=(1.35, 1.30, 1.15),  # new: DECISIONS 2026-11-26
        colors=("leaf_dark", "leaf_mid", "leaf_mid"), count=10, blob_r=0.56,
        branch_from=0.60, bark="bark_dark", limbs=4, limb_r=0.075,
    )


def build_all():
    """Builds tree_asphalt, sapling_city, shrub_wild, roof_bush, roof_tree. Returns [(name, ok)]."""
    builders = [
        ("tree_asphalt", _build_tree_asphalt),
        ("sapling_city", _build_sapling_city),
        ("shrub_wild", _build_shrub_wild),
        ("roof_bush", _build_roof_bush),
        ("roof_tree", _build_roof_tree),
    ]
    return [
        (name, common.run_model(name, "nature", BUDGETS[name], fn))
        for name, fn in builders
    ]
