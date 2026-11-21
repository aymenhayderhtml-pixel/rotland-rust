"""trash_bags: pile of 4-5 black trash bags (lumpy icospheres) (<= 600 tris). Has a collider."""
import math
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402

NAME, CATEGORY, BUDGET = "trash_bags", "props", 600


def _make_bag_knot(loc, rot, color="gun_black"):
    """Tied neck knot and flaring tabs at the bag closure."""
    knot_bm = common.make_frustum(0.04, 0.02, 0.0, 0.06, segments=4, color=color)
    tabs = common.make_box((0.02, 0.09, 0.02), loc=(0.0, 0.0, 0.065), color=color)
    knot = common.combine([knot_bm, tabs])
    common.transform(knot, rot=rot, loc=loc)
    return knot


def build(model):
    parts = []

    # 5 lumpy trash bags arranged in a natural slumped street pile
    # Bag definitions: (loc, scale, jitter, seed, rot, knot_offset, knot_rot, color)
    bags_data = [
        # Bag 1: front-left, resting on the pavement
        ((-0.26, -0.22, 0.20), (0.34, 0.30, 0.21), 0.15, 101, (0.1, -0.15, 0.3),
         (-0.24, -0.20, 0.38), (0.2, 0.3, 0.5), "gun_black"),

        # Bag 2: front-right, slightly larger and squashed
        ((0.26, -0.16, 0.19), (0.33, 0.35, 0.20), 0.16, 202, (-0.1, 0.2, -0.4),
         ((0.30, -0.12, 0.36)), (-0.3, -0.2, 1.2), "tire_black"),

        # Bag 3: rear-center, nestled behind bags 1 and 2
        ((0.02, 0.26, 0.21), (0.36, 0.32, 0.22), 0.14, 303, (0.15, 0.1, -0.2),
         ((0.04, 0.28, 0.40)), (0.1, -0.4, -0.8), "pants_dark"),

        # Bag 4: stacked on top-left over bags 1 and 3
        ((-0.12, 0.04, 0.44), (0.29, 0.27, 0.23), 0.15, 404, (-0.2, -0.1, 0.6),
         ((-0.10, 0.06, 0.64)), (-0.2, 0.1, 0.3), "gun_black"),

        # Bag 5: stacked on top-right, leaning against bag 4
        ((0.16, 0.08, 0.42), (0.28, 0.29, 0.22), 0.16, 505, (0.25, 0.15, -0.5),
         ((0.20, 0.10, 0.61)), (0.3, -0.3, -0.6), "tire_black"),
    ]

    for loc, scale, jitter, seed, rot, knot_loc, knot_rot, color in bags_data:
        bag_bm = common.make_icosphere(subdiv=2, jitter=jitter, seed=seed, radius=scale,
                                       loc=loc, rot=rot, color=color)
        knot_bm = _make_bag_knot(loc=knot_loc, rot=knot_rot, color=color)
        parts.extend([bag_bm, knot_bm])

    # Debris around the pile:
    # A crushed cardboard carton corner peeking from under bag 1
    carton = common.make_box((0.18, 0.14, 0.06), loc=(-0.44, -0.30, 0.03), color="wood_crate")
    common.transform(carton, rot=(0.0, 0.0, 0.4))
    # A discarded empty tin can lying on the pavement near bag 2
    can = common.make_frustum(0.035, 0.035, 0.0, 0.11, segments=6, color="metal")
    common.transform(can, rot=(0.0, math.pi / 2, 0.8), loc=(0.48, -0.28, 0.035))
    parts.extend([carton, can])

    model.add(common.combine(parts))

    # Low collider covering the pile
    model.add_collider("pile", common.make_box((1.18, 1.05, 0.66), loc=(0.0, 0.03, 0.33), color="white"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
