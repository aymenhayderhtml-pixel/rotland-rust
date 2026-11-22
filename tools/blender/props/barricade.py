"""barricade: police/army road barricade with wooden sawhorses and sandbag row (<= 1500 tris). Has a collider."""
import math
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402

NAME, CATEGORY, BUDGET = "barricade", "props", 1500


def _make_sandbag(length, width, height, loc, rot_z=0.0, color="canvas_khaki"):
    """Oblong sandbag with rounded bevelled profile and gathered end seam."""
    # Main body
    body = common.make_box((length - 0.06, width, height), color=color)
    # Tapered / pinched end ears where the sack is tied
    end_seam = common.make_box((0.06, width * 0.75, height * 0.5), loc=(-length / 2 + 0.03, 0.0, 0.0), color=color)
    bag = common.combine([body, end_seam])
    common.transform(bag, rot=(0.0, 0.0, rot_z), loc=loc)
    return bag


def _make_sawhorse(x_center, rot_z=0.0, tilt_x=0.0):
    """Heavy wooden A-frame sawhorse with top crossbeam, 4 splayed legs and cross braces."""
    parts = []
    beam_len = 0.70
    # Top heavy crossbeam (runs along Y)
    beam = common.make_box((0.09, beam_len, 0.12), loc=(0.0, 0.0, 0.94), color="wood_crate")
    parts.append(beam)

    # 4 splayed legs: A-frame
    # Legs angle from top (Y = +/-0.22) out to (Y = +/-0.34) at ground
    for sy in (-1, 1):
        for sx in (-1, 1):
            p_top = (sx * 0.04, sy * 0.22, 0.90)
            p_bot = (sx * 0.16, sy * 0.34, 0.0)
            dx = p_bot[0] - p_top[0]
            dy = p_bot[1] - p_top[1]
            dz = p_bot[2] - p_top[2]
            leg_len = math.sqrt(dx * dx + dy * dy + dz * dz)
            leg = common.make_box((0.06, 0.06, leg_len), color="wood_crate")
            # Orientation
            pitch = math.atan2(math.sqrt(dx * dx + dy * dy), -dz)
            yaw = math.atan2(dx, dy)
            mid = ((p_top[0] + p_bot[0]) / 2, (p_top[1] + p_bot[1]) / 2, (p_top[2] + p_bot[2]) / 2)
            common.transform(leg, rot=(-pitch, 0.0, -yaw), loc=mid)
            parts.append(leg)

        # Crossbar tie brace between left and right legs
        brace = common.make_box((0.04, 0.04, 0.28), loc=(0.0, sy * 0.28, 0.30), color="wood_crate")
        parts.append(brace)

    sawhorse = common.combine(parts)
    common.transform(sawhorse, rot=(tilt_x, 0.0, rot_z), loc=(x_center, 0.12, 0.0))
    return sawhorse


def build(model):
    parts = []

    # 1. Two wooden sawhorses: left (X = -1.25) and right (X = +1.25), set back at Y = 0.12
    # Left sawhorse is upright; right sawhorse is slightly skewed/shoved
    parts.append(_make_sawhorse(-1.25, rot_z=0.0, tilt_x=0.0))
    parts.append(_make_sawhorse(1.25, rot_z=0.08, tilt_x=0.05))

    # 2. Barricade horizontal barrier planks (spanning 4 m from X = -2.0 to +2.0)
    # Mounted on the front of the sawhorse legs at Y = -0.16
    # Upper rail at Z = 0.94, lower rail at Z = 0.68
    # Striped warning pattern: alternating white and red segments (0.40 m each, 10 stripes)
    stripe_w = 0.40
    stripe_colors = ["red_paint", "white"]
    rail_y = -0.16

    for i in range(10):
        x_stripe = -1.80 + i * stripe_w
        c = stripe_colors[i % 2]
        # Upper rail stripe
        parts.append(common.make_box((stripe_w, 0.04, 0.16), loc=(x_stripe, rail_y, 0.94), color=c))
        # Lower rail stripe (stop before the broken right end at i=9)
        if i < 9:
            parts.append(common.make_box((stripe_w, 0.04, 0.16), loc=(x_stripe, rail_y, 0.68), color=c))

    # Broken/splintered splinter hanging off right end of lower rail
    splinter = common.make_box((0.25, 0.03, 0.06), color="wood_crate")
    common.transform(splinter, rot=(0.0, 0.3, 0.0), loc=(1.65, rail_y - 0.01, 0.64))
    parts.append(splinter)

    # Central metal warning sign mounted between rails: "STOP" / "ROAD CLOSED"
    # Offset sign_back proud of rails (rail front is at -0.18) and sign elements proud of sign_back
    sign_back = common.make_box((0.70, 0.012, 0.36), loc=(0.0, rail_y - 0.035, 0.81), color="white")
    sign_border = common.make_box((0.66, 0.008, 0.04), loc=(0.0, rail_y - 0.048, 0.95), color="red_paint")
    sign_border_b = common.make_box((0.66, 0.008, 0.04), loc=(0.0, rail_y - 0.048, 0.67), color="red_paint")
    sign_text_bar = common.make_box((0.54, 0.008, 0.14), loc=(0.0, rail_y - 0.048, 0.81), color="red_paint")
    parts.extend([sign_back, sign_border, sign_border_b, sign_text_bar])

    # 3. Sandbag row stacked in front of the barricade (at Y = -0.32, spanning ~3.6 m across)
    # Course 1 (Ground level, Z = 0.07): 8 sandbags end-to-end
    c1_xs = [-1.55, -1.10, -0.66, -0.22, 0.22, 0.66, 1.10, 1.55]
    for idx, x in enumerate(c1_xs):
        color = "cloth_olive" if idx in (2, 6) else "canvas_khaki"
        rot = 0.04 if idx % 2 == 0 else -0.04
        parts.append(_make_sandbag(0.46, 0.26, 0.14, loc=(x, -0.32, 0.07), rot_z=rot, color=color))

    # Course 2 (Second layer, staggered, Z = 0.21): 7 sandbags
    c2_xs = [-1.32, -0.88, -0.44, 0.0, 0.44, 0.88, 1.32]
    for idx, x in enumerate(c2_xs):
        color = "shirt_zombie" if idx == 3 else ("cloth_olive" if idx == 1 else "canvas_khaki")
        rot = -0.08 if idx % 2 == 0 else 0.07
        parts.append(_make_sandbag(0.45, 0.25, 0.13, loc=(x, -0.32, 0.21), rot_z=rot, color=color))

    # Course 3 (Third top layer, Z = 0.34): 5 sandbags in center
    c3_xs = [-0.88, -0.44, 0.0, 0.44, 0.88]
    for idx, x in enumerate(c3_xs):
        color = "canvas_khaki"
        rot = 0.12 if idx % 2 == 0 else -0.11
        parts.append(_make_sandbag(0.44, 0.24, 0.12, loc=(x, -0.32, 0.34), rot_z=rot, color=color))

    # One burst sandbag on far right with spilled dirt on ground
    # Lift burst bag bottom slightly (z=0.01) and sink dirt bottom slightly (z=-0.01) so they don't coplanar with ground z=0
    burst_bag = _make_sandbag(0.42, 0.26, 0.10, loc=(1.80, -0.28, 0.06), rot_z=0.35, color="canvas_khaki")
    spilled_dirt = common.make_frustum(0.22, 0.08, -0.01, 0.06, segments=6, center=(1.85, -0.38), color="dirt")
    parts.extend([burst_bag, spilled_dirt])

    model.add(common.combine(parts))

    # Collider: covering the barricade across 4 m
    model.add_collider("barrier", common.make_box((4.05, 0.88, 1.05), loc=(0.0, -0.10, 0.525), color="white"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
