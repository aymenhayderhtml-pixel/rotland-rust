"""bus_stop: bus shelter with roof on posts, back panel, a bench; one panel broken (<= 1500 tris)."""
import math
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402

NAME, CATEGORY, BUDGET = "bus_stop", "props", 1500


def build(model):
    parts = []

    # 1. Main posts: 4 corner posts and 2 rear bay divider posts
    # Coordinates: X in [-1.75, 1.75], Rear Y = +0.70, Front Y = -0.70
    post_locs = [
        (-1.75, -0.70, 2.25),  # front left
        (1.75, -0.70, 2.25),   # front right
        (-1.75, 0.70, 2.40),   # rear left
        (-0.58, 0.70, 2.40),   # rear mid-left
        (0.58, 0.70, 2.40),    # rear mid-right
        (1.75, 0.70, 2.40),    # rear right
    ]
    for x, y, h in post_locs:
        # Post shaft
        parts.append(common.make_box((0.08, 0.08, h), loc=(x, y, h / 2), color="metal"))
        # Rusted footing at base
        parts.append(common.make_box((0.14, 0.14, 0.08), loc=(x, y, 0.04), color="rust"))

    # 2. Horizontal perimeter framing
    # Rear top beam (spanning full width)
    parts.append(common.make_box((3.60, 0.08, 0.08), loc=(0.0, 0.70, 2.36), color="metal"))
    # Front top beam
    parts.append(common.make_box((3.60, 0.08, 0.08), loc=(0.0, -0.70, 2.21), color="metal"))
    # Left and right top cross beams
    parts.append(common.make_box((0.08, 1.40, 0.08), loc=(-1.75, 0.0, 2.30), color="metal"))
    parts.append(common.make_box((0.08, 1.40, 0.08), loc=(1.75, 0.0, 2.30), color="metal"))
    # Intermediate roof rafters
    parts.append(common.make_box((0.06, 1.40, 0.06), loc=(-0.58, 0.0, 2.30), color="metal"))
    parts.append(common.make_box((0.06, 1.40, 0.06), loc=(0.58, 0.0, 2.30), color="metal"))

    # Rear lower sill beam (holding bottom of glass panels)
    parts.append(common.make_box((3.56, 0.06, 0.06), loc=(0.0, 0.70, 0.25), color="rust"))
    # Rear mid waist rail
    parts.append(common.make_box((3.56, 0.06, 0.06), loc=(0.0, 0.70, 1.15), color="metal"))

    # Left side panel rails
    parts.append(common.make_box((0.06, 1.36, 0.06), loc=(-1.75, 0.0, 0.25), color="rust"))
    parts.append(common.make_box((0.06, 1.36, 0.06), loc=(-1.75, 0.0, 1.15), color="metal"))

    # 3. Roof canopy: curved/sloped overhang
    # Main roof plane: 3.9 m wide x 1.85 m deep, gently sloped forward
    roof_main = common.make_box((3.90, 1.85, 0.05), loc=(0.0, 0.0, 2.38), color="metal")
    common.transform(roof_main, rot=(0.07, 0.0, 0.0))  # sloped down toward front
    # Roof edge fascia trim
    roof_fascia_front = common.make_box((3.92, 0.04, 0.12), loc=(0.0, -0.92, 2.25), color="rust")
    roof_fascia_rear = common.make_box((3.92, 0.04, 0.10), loc=(0.0, 0.92, 2.38), color="metal")
    parts.extend([roof_main, roof_fascia_front, roof_fascia_rear])

    # 4. Glass panels:
    # Bay 1 (Left: X in [-1.71, -0.62], width ~1.09) - INTACT
    panel_left_bottom = common.make_box((1.09, 0.015, 0.84), loc=(-1.165, 0.70, 0.70), color="glass_tint")
    panel_left_top = common.make_box((1.09, 0.015, 1.15), loc=(-1.165, 0.70, 1.75), color="glass_tint")
    parts.extend([panel_left_bottom, panel_left_top])

    # Bay 2 (Center: X in [-0.54, 0.54], width ~1.08) - INTACT with timetable poster
    panel_mid_bottom = common.make_box((1.08, 0.015, 0.84), loc=(0.0, 0.70, 0.70), color="glass_tint")
    panel_mid_top = common.make_box((1.08, 0.015, 1.15), loc=(0.0, 0.70, 1.75), color="glass_tint")
    # Weathered timetable / schedule frame and paper poster inside
    poster_frame = common.make_box((0.55, 0.025, 0.80), loc=(0.0, 0.685, 1.35), color="gun_black")
    poster_paper = common.make_box((0.50, 0.03, 0.74), loc=(0.0, 0.68, 1.35), color="white")
    poster_header = common.make_box((0.46, 0.032, 0.12), loc=(0.0, 0.678, 1.66), color="red_paint")
    poster_lines = [common.make_box((0.44, 0.032, 0.03), loc=(0.0, 0.678, 1.50 - 0.08 * i), color="gun_black")
                    for i in range(5)]
    parts.extend([panel_mid_bottom, panel_mid_top, poster_frame, poster_paper, poster_header] + poster_lines)

    # Bay 3 (Right: X in [0.62, 1.71]) - BROKEN!
    # Glass shattered: only jagged triangular shards remain along edges
    # Shard 1: bottom-left corner shard
    shard1 = common.new_bm()
    common.add_quad(shard1, [(0.62, 0.70, 0.28), (0.95, 0.70, 0.28),
                             (0.80, 0.70, 0.65), (0.62, 0.70, 0.75)],
                    outward=(0.0, -1.0, 0.0), color="glass_tint")
    # Shard 2: bottom-right corner shard
    shard2 = common.new_bm()
    common.add_quad(shard2, [(1.35, 0.70, 0.28), (1.71, 0.70, 0.28),
                             (1.71, 0.70, 0.58), (1.50, 0.70, 0.45)],
                    outward=(0.0, -1.0, 0.0), color="glass_tint")
    # Shard 3: top edge shard hanging down
    shard3 = common.new_bm()
    common.add_quad(shard3, [(0.90, 0.70, 2.32), (1.45, 0.70, 2.32),
                             (1.30, 0.70, 2.05), (1.05, 0.70, 2.12)],
                    outward=(0.0, -1.0, 0.0), color="glass_tint")
    # Shard 4: mid-rail tooth
    shard4 = common.new_bm()
    common.add_quad(shard4, [(0.62, 0.70, 1.18), (0.85, 0.70, 1.18),
                             (0.75, 0.70, 1.45), (0.62, 0.70, 1.50)],
                    outward=(0.0, -1.0, 0.0), color="glass_tint")
    parts.extend([shard1, shard2, shard3, shard4])

    # Broken glass debris pile on the ground below the right bay
    debris = [
        common.make_box((0.18, 0.12, 0.02), loc=(1.05, 0.65, 0.01), color="glass_tint"),
        common.make_box((0.14, 0.16, 0.025), loc=(1.25, 0.68, 0.015), color="glass_tint"),
        common.make_box((0.10, 0.08, 0.015), loc=(0.85, 0.60, 0.01), color="glass_tint"),
        common.make_box((0.12, 0.14, 0.02), loc=(1.45, 0.62, 0.01), color="glass_tint"),
        common.make_box((0.08, 0.06, 0.03), loc=(1.15, 0.72, 0.02), color="rock_mid"),
    ]
    parts.extend(debris)

    # Left end glass panel
    side_panel_bottom = common.make_box((0.015, 1.34, 0.84), loc=(-1.75, 0.0, 0.70), color="glass_tint")
    side_panel_top = common.make_box((0.015, 1.34, 1.10), loc=(-1.75, 0.0, 1.72), color="glass_tint")
    parts.extend([side_panel_bottom, side_panel_top])

    # 5. Bench: wooden slats with steel legs
    # Wooden slats spanning from X=-1.55 to +0.45 (length 2.0 m)
    for y_slat in (0.35, 0.48, 0.61):
        parts.append(common.make_box((2.00, 0.10, 0.035), loc=(-0.55, y_slat, 0.48), color="wood_crate"))
    # Bench steel leg frames (at X = -1.40 and X = 0.30)
    for x_leg in (-1.40, 0.30):
        parts.append(common.make_box((0.05, 0.36, 0.04), loc=(x_leg, 0.48, 0.45), color="metal"))
        parts.append(common.make_box((0.05, 0.05, 0.45), loc=(x_leg, 0.35, 0.225), color="rust"))
        parts.append(common.make_box((0.05, 0.05, 0.45), loc=(x_leg, 0.61, 0.225), color="rust"))
    # Bench backrest slat
    parts.append(common.make_box((2.00, 0.035, 0.12), loc=(-0.55, 0.67, 0.78), color="wood_crate"))

    # 6. Bus Stop Sign on the front-left post
    sign_bracket = common.make_box((0.04, 0.25, 0.04), loc=(-1.75, -0.80, 2.10), color="metal")
    sign_face = common.make_box((0.02, 0.38, 0.48), loc=(-1.75, -0.92, 1.95), color="car_blue")
    sign_symbol = common.make_box((0.025, 0.24, 0.24), loc=(-1.75, -0.92, 1.95), color="white")
    parts.extend([sign_bracket, sign_face, sign_symbol])

    model.add(common.combine(parts))

    # Colliders
    # Back wall collider
    model.add_collider("back", common.make_box((3.60, 0.16, 2.40), loc=(0.0, 0.70, 1.20), color="white"))
    # Left end collider
    model.add_collider("left", common.make_box((0.16, 1.48, 2.40), loc=(-1.75, 0.0, 1.20), color="white"))
    # Roof collider
    model.add_collider("roof", common.make_box((3.90, 1.85, 0.12), loc=(0.0, 0.0, 2.35), color="white"))
    # Bench collider
    model.add_collider("bench", common.make_box((2.10, 0.45, 0.50), loc=(-0.55, 0.48, 0.25), color="white"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
