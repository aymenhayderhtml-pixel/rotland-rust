"""fence_broken: a 3 m chain-link fence section, bent and torn with posts and wire mesh (<= 800 tris). Has colliders."""
import math
import os
import sys

from mathutils import Vector

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402

NAME, CATEGORY, BUDGET = "fence_broken", "props", 800


def _posts():
    """Two end posts: left upright post, right post knocked askew and leaning outward."""
    bms = []

    # Left post: vertical 6-sided pipe at x = -1.45, height 2.15 m
    post_l = common.make_frustum(0.04, 0.04, 0.0, 2.15, segments=6, center=(-1.45, 0.0), color="metal")
    cap_l = common.make_frustum(0.045, 0.0, 2.15, 2.22, segments=6, center=(-1.45, 0.0), color="metal")
    footing_l = common.make_box((0.20, 0.20, 0.10), loc=(-1.45, 0.0, 0.05), color="rock_mid")
    bms.extend([post_l, cap_l, footing_l])

    # Right post: knocked askew, tilted ~16 degrees towards +X and +Y
    post_r = common.make_frustum(0.04, 0.04, 0.0, 2.10, segments=6, color="metal")
    cap_r = common.make_frustum(0.045, 0.0, 2.10, 2.17, segments=6, color="metal")
    combined_r = common.combine([post_r, cap_r])
    common.transform(combined_r, rot=(0.14, 0.20, 0.08), loc=(1.45, 0.08, 0.0))
    footing_r = common.make_box((0.22, 0.22, 0.08), loc=(1.45, 0.04, 0.04), color="rock_dark")
    bms.extend([combined_r, footing_r])

    return common.combine(bms)


def _top_rail():
    """Top rail pipe: left section horizontal, buckled and snapped in the middle, sagging to right post."""
    bms = []

    # Left top rail: from x = -1.45 to x = -0.22 at z = 2.05
    rail_l = common.make_box((1.23, 0.045, 0.045), loc=(-0.835, 0.0, 2.05), color="metal")
    bms.append(rail_l)

    # Buckled downward kink at x = -0.22 to 0.02
    kink = common.make_box((0.28, 0.045, 0.045), loc=(0.0, 0.0, 0.0), color="rust")
    common.transform(kink, rot=(0.08, 0.42, 0.0), loc=(-0.10, 0.02, 1.95))
    bms.append(kink)

    # Right top rail: sagging from x = 0.15 to meet right post at x = 1.45
    rail_r = common.make_box((1.28, 0.045, 0.045), loc=(0.0, 0.0, 0.0), color="metal")
    common.transform(rail_r, rot=(0.14, -0.14, 0.06), loc=(0.78, 0.15, 1.96))
    bms.append(rail_r)

    # Bottom tension wire: thin metal wire near ground
    wire_bot_l = common.make_box((1.25, 0.015, 0.015), loc=(-0.82, 0.0, 0.08), color="metal")
    wire_bot_r = common.make_box((1.25, 0.015, 0.015), loc=(0.82, 0.08, 0.08), color="rust")
    bms.extend([wire_bot_l, wire_bot_r])

    return common.combine(bms)


def _chainlink_mesh():
    """Diamond-grid look drawn as thin crossing slats, bounded by posts, with connected curling torn wire ends."""
    bms = []
    bar_t = 0.016

    # Vertical side tension bars clamped to posts
    clamp_l = common.make_box((0.02, 0.02, 1.95), loc=(-1.41, 0.0, 1.05), color="metal")
    bms.append(clamp_l)
    clamp_r = common.make_box((0.02, 0.02, 1.85), loc=(0.0, 0.0, 0.0), color="rust")
    common.transform(clamp_r, rot=(0.14, 0.20, 0.08), loc=(1.40, 0.08, 1.0))
    bms.append(clamp_r)

    # Left panel (x from -1.40 to -0.30, z from 0.10 to 2.02)
    # Slanted bars contained neatly inside frame
    for lx, lz, length in [
        (-1.10, 0.80, 0.80),
        (-0.95, 1.05, 1.25),
        (-0.75, 1.15, 1.35),
        (-0.55, 1.25, 1.25),
    ]:
        bar = common.make_box((length, bar_t, 0.018), loc=(0.0, 0.0, 0.0), color="metal")
        common.transform(bar, rot=(0.0, math.radians(45), 0.0), loc=(lx, 0.0, lz))
        bms.append(bar)

    for lx, lz, length in [
        (-1.10, 1.30, 0.80),
        (-0.95, 1.05, 1.25),
        (-0.75, 0.95, 1.35),
        (-0.55, 0.85, 1.25),
    ]:
        bar = common.make_box((length, bar_t, 0.018), loc=(0.0, 0.0, 0.0), color="metal")
        common.transform(bar, rot=(0.0, math.radians(-45), 0.0), loc=(lx, 0.005, lz))
        bms.append(bar)

    # Horizontal guide wires on left panel
    trim1 = common.make_box((1.12, bar_t, 0.018), loc=(-0.85, 0.0, 1.55), color="metal")
    trim2 = common.make_box((1.12, bar_t, 0.018), loc=(-0.85, 0.0, 0.55), color="metal")
    bms.extend([trim1, trim2])

    # Center tear / breach (x from -0.30 to +0.45):
    # Torn wire ends branching directly from the left mesh edge (-0.30) and right mesh edge (+0.45)
    left_torn_branches = [
        ((0.35, 0.45, 0.1), (-0.22, 0.05, 1.62), 0.28),
        ((-0.40, -0.35, 0.2), (-0.20, -0.06, 1.30), 0.32),
        ((0.50, 0.30, -0.15), (-0.18, 0.08, 0.95), 0.35),
        ((-0.30, 0.55, 0.1), (-0.16, 0.04, 0.60), 0.30),
        ((0.15, 0.70, 0.0), (-0.12, 0.02, 0.28), 0.36),
    ]
    for rot, loc, length in left_torn_branches:
        wire = common.make_box((length, bar_t, 0.016), loc=(0.0, 0.0, 0.0), color="rust")
        common.transform(wire, rot=rot, loc=loc)
        bms.append(wire)

    right_torn_branches = [
        ((0.40, -0.40, -0.2), (0.34, 0.12, 1.55), 0.30),
        ((-0.45, 0.35, 0.15), (0.32, -0.08, 1.20), 0.34),
        ((0.55, -0.25, 0.25), (0.35, 0.14, 0.85), 0.32),
        ((-0.20, -0.60, 0.1), (0.36, -0.04, 0.48), 0.36),
        ((0.25, -0.75, 0.0), (0.28, 0.06, 0.20), 0.38),
    ]
    for rot, loc, length in right_torn_branches:
        wire = common.make_box((length, bar_t, 0.016), loc=(0.0, 0.0, 0.0), color="rust")
        common.transform(wire, rot=rot, loc=loc)
        bms.append(wire)

    # Right panel: distorted mesh leading towards the leaning right post (x from 0.45 to 1.38)
    for lx, lz, length in [
        (0.68, 1.20, 1.15),
        (0.88, 1.10, 1.20),
        (1.08, 0.95, 1.10),
    ]:
        bar = common.make_box((length, bar_t, 0.018), loc=(0.0, 0.0, 0.0), color="rust")
        common.transform(bar, rot=(0.12, math.radians(42), 0.05), loc=(lx, 0.12, lz))
        bms.append(bar)

    for lx, lz, length in [
        (0.68, 0.80, 1.15),
        (0.88, 0.95, 1.20),
        (1.08, 1.10, 1.10),
    ]:
        bar = common.make_box((length, bar_t, 0.018), loc=(0.0, 0.0, 0.0), color="metal")
        common.transform(bar, rot=(0.10, math.radians(-42), 0.08), loc=(lx, 0.12, lz))
        bms.append(bar)

    trim3 = common.make_box((0.85, bar_t, 0.018), loc=(0.0, 0.0, 0.0), color="metal")
    common.transform(trim3, rot=(0.12, -0.10, 0.06), loc=(0.88, 0.14, 1.45))
    trim4 = common.make_box((0.85, bar_t, 0.018), loc=(0.0, 0.0, 0.0), color="rust")
    common.transform(trim4, rot=(0.10, -0.08, 0.06), loc=(0.88, 0.12, 0.55))
    bms.extend([trim3, trim4])

    return common.combine(bms)


def _barbed_wire_top():
    """Barbed wire extensions on top of the posts, with snapped drooping wires in the middle."""
    bms = []

    # Left 45-degree angled bracket at post top
    bracket_l = common.make_box((0.02, 0.02, 0.35), loc=(0.0, 0.0, 0.0), color="metal")
    common.transform(bracket_l, rot=(0.0, math.radians(-38), 0.0), loc=(-1.35, 0.0, 2.22))
    bms.append(bracket_l)

    # Right angled bracket on leaning post
    bracket_r = common.make_box((0.02, 0.02, 0.35), loc=(0.0, 0.0, 0.0), color="rust")
    common.transform(bracket_r, rot=(0.14, math.radians(-25), 0.1), loc=(1.55, 0.18, 2.05))
    bms.append(bracket_r)

    # Strands of barbed wire:
    # Left strand running to break point
    wire_top_l1 = common.make_box((1.15, 0.012, 0.012), loc=(-0.80, 0.0, 2.32), color="metal")
    wire_top_l2 = common.make_box((1.15, 0.012, 0.012), loc=(-0.80, 0.0, 2.22), color="metal")
    bms.extend([wire_top_l1, wire_top_l2])

    # Drooping broken strands dangling down into the center gap
    droop1 = common.make_box((0.012, 0.012, 0.55), loc=(0.0, 0.0, 0.0), color="rust")
    common.transform(droop1, rot=(0.3, 0.7, 0.1), loc=(-0.15, 0.06, 2.05))
    bms.append(droop1)

    droop2 = common.make_box((0.012, 0.012, 0.65), loc=(0.0, 0.0, 0.0), color="rust")
    common.transform(droop2, rot=(-0.25, -0.6, 0.1), loc=(0.40, 0.10, 1.85))
    bms.append(droop2)

    return common.combine(bms)


def build(model):
    model.add(_posts())
    model.add(_top_rail())
    model.add(_chainlink_mesh())
    model.add(_barbed_wire_top())

    # Two box colliders: one on the intact left panel, one on the leaning right post
    model.add_collider("left_panel", common.make_box((1.3, 0.25, 2.1), loc=(-0.85, 0.0, 1.05), color="white"))
    model.add_collider("right_post", common.make_box((0.6, 0.5, 2.1), loc=(1.45, 0.15, 1.05), color="white"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
