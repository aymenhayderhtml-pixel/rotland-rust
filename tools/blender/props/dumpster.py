"""dumpster: green metal dumpster with lid half open (<= 800 tris). Has a collider."""
import math
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402

NAME, CATEGORY, BUDGET = "dumpster", "props", 800


def build(model):
    parts = []

    # 1. Base skids and feet (Z = 0.0 to 0.08)
    # Two longitudinal steel channel runners at bottom along Y
    skid_l = common.make_box((0.10, 1.28, 0.08), loc=(-0.72, 0.0, 0.04), color="metal")
    skid_r = common.make_box((0.10, 1.28, 0.08), loc=(0.72, 0.0, 0.04), color="metal")
    # Corner caster plates / rust pads
    pads = [
        common.make_box((0.14, 0.14, 0.03), loc=(x, y, 0.015), color="rust")
        for x in (-0.72, 0.72) for y in (-0.56, 0.56)
    ]
    parts.extend([skid_l, skid_r] + pads)

    # 2. Main green container body
    # Dimensions: X in [-0.88, 0.88], Y in [-0.62, 0.62]
    # Z_bottom = 0.08, Z_rear = 1.18, Z_front = 0.94
    # Floor
    floor = common.make_box((1.76, 1.24, 0.04), loc=(0.0, 0.0, 0.10), color="rust")
    # Rear wall: vertical, height 1.08 (Z from 0.08 to 1.18)
    wall_rear = common.make_box((1.76, 0.04, 1.08), loc=(0.0, 0.62, 0.63), color="pine_dark")
    # Front wall: vertical, height 0.84 (Z from 0.08 to 0.94)
    wall_front = common.make_box((1.76, 0.04, 0.84), loc=(0.0, -0.62, 0.51), color="pine_dark")
    parts.extend([floor, wall_rear, wall_front])

    # Side walls (trapezoidal: high in back, lower in front)
    # We construct left side (at X = -0.88) and right side (at X = +0.88)
    for x_side in (-0.88, 0.88):
        side_bm = common.new_bm()
        c0 = (x_side, -0.62, 0.08)
        c1 = (x_side, 0.62, 0.08)
        c2 = (x_side, 0.62, 1.18)
        c3 = (x_side, -0.62, 0.94)
        outward = (-1.0 if x_side < 0 else 1.0, 0.0, 0.0)
        common.add_quad(side_bm, [c0, c1, c2, c3], outward=outward, color="pine_dark")
        parts.append(side_bm)

    # Welded top rim collar
    rim_front = common.make_box((1.80, 0.06, 0.04), loc=(0.0, -0.63, 0.94), color="metal")
    rim_rear = common.make_box((1.80, 0.06, 0.04), loc=(0.0, 0.63, 1.18), color="rust")
    parts.extend([rim_front, rim_rear])

    # 3. Exterior details: fork pockets on left and right sides
    # Horizontal sleeve channels where garbage truck arms lift the dumpster
    pocket_l = common.make_box((0.08, 1.10, 0.16), loc=(-0.92, 0.0, 0.58), color="metal")
    pocket_r = common.make_box((0.08, 1.10, 0.16), loc=(0.92, 0.0, 0.58), color="metal")
    # Front vertical stiffener ribs
    rib_l = common.make_box((0.04, 0.03, 0.76), loc=(-0.44, -0.64, 0.50), color="rust")
    rib_r = common.make_box((0.04, 0.03, 0.76), loc=(0.44, -0.64, 0.50), color="rust")
    # Rust stain patches along bottom front edge
    rust_patch = common.make_box((0.40, 0.02, 0.15), loc=(0.20, -0.63, 0.18), color="rust")
    parts.extend([pocket_l, pocket_r, rib_l, rib_r, rust_patch])

    # 4. Interior trash visible
    trash_pile = common.make_box((1.60, 1.05, 0.40), loc=(0.0, 0.0, 0.35), color="soil_dark")
    trash_bag_peek = common.make_box((0.42, 0.38, 0.32), loc=(0.35, 0.05, 0.65), color="gun_black")
    trash_box = common.make_box((0.28, 0.24, 0.20), loc=(0.40, -0.25, 0.68), color="wood_crate")
    common.transform(trash_box, rot=(0.15, -0.2, 0.3))
    parts.extend([trash_pile, trash_bag_peek, trash_box])

    # 5. Hinge rod across rear top rim
    hinge_rod = common.make_frustum(0.02, 0.02, -0.88, 0.88, segments=6, color="metal")
    common.transform(hinge_rod, rot=(0.0, math.pi / 2, 0.0), loc=(0.0, 0.61, 1.20))
    parts.append(hinge_rod)

    # 6. Split lids:
    # Top opening slopes from (Y=0.62, Z=1.18) down to (Y=-0.62, Z=0.94)
    # Slope angle = atan2(1.18 - 0.94, 1.24) = atan2(0.24, 1.24) = ~0.191 rad (~11 deg)
    lid_length = math.sqrt(1.24 * 1.24 + 0.24 * 0.24) + 0.04  # ~1.30 m
    lid_width = 0.86  # each lid covers half width

    # Left lid: CLOSED (X from -0.87 to -0.01)
    lid_left = common.make_box((lid_width, lid_length, 0.03), color="tire_black")
    # Slope downward from hinge
    common.transform(lid_left, rot=(0.191, 0.0, 0.0), loc=(-0.44, 0.0, 1.06))
    lid_handle_l = common.make_box((0.24, 0.03, 0.03), color="metal")
    common.transform(lid_handle_l, rot=(0.191, 0.0, 0.0), loc=(-0.44, -0.48, 0.98))
    parts.extend([lid_left, lid_handle_l])

    # Right lid: HALF OPEN (X from 0.01 to 0.87)
    # Hinged at (X = 0.44, Y = 0.61, Z = 1.20)
    # Tilted up backwards/open around the hinge: angle ~ -0.95 rad (~ -55 deg)
    lid_right = common.make_box((lid_width, lid_length, 0.03), color="tire_black")
    # Put lid origin at its rear hinge edge, rotate, then translate to hinge position
    common.transform(lid_right, loc=(0.0, -lid_length / 2, 0.0))
    common.transform(lid_right, rot=(-0.95, 0.0, 0.0), loc=(0.44, 0.61, 1.20))
    parts.append(lid_right)

    model.add(common.combine(parts))

    # Collider: covering the entire dumpster box
    model.add_collider("body", common.make_box((1.86, 1.34, 1.24), loc=(0.0, 0.0, 0.62), color="white"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
