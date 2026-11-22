"""mailbox: blue street mailbox with curved top, hopper slot and four legs (<= 400 tris)."""
import math
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402

NAME, CATEGORY, BUDGET = "mailbox", "props", 400


def build(model):
    parts = []

    # 1. Four corner legs (from ground z=0 to z=0.34)
    # USPS collection box stands on 4 angled steel legs
    leg_coords = [
        (-0.23, -0.23),
        (0.23, -0.23),
        (-0.23, 0.23),
        (0.23, 0.23),
    ]
    for lx, ly in leg_coords:
        # Leg post resting on top of foot pad (z from 0.025 to 0.34)
        parts.append(common.make_box((0.045, 0.045, 0.315), loc=(lx, ly, 0.1825), color="pants_dark"))
        # Foot mounting pad on ground
        parts.append(common.make_box((0.08, 0.08, 0.025), loc=(lx, ly, 0.0125), color="rust"))
    # Diagonal brace bars between front legs and rear legs
    parts.append(common.make_box((0.42, 0.03, 0.03), loc=(0.0, -0.23, 0.12), color="pants_dark"))
    parts.append(common.make_box((0.42, 0.03, 0.03), loc=(0.0, 0.23, 0.12), color="pants_dark"))

    # 2. Main rectangular body (from z=0.34 to z=0.96)
    # Width X = 0.52 (-0.26 to 0.26), Depth Y = 0.52 (-0.26 to 0.26), Height = 0.62
    body = common.make_box((0.52, 0.52, 0.62), loc=(0.0, 0.0, 0.65), color="car_blue")
    # Base rim collar (z from 0.31 to 0.37, collar wraps around bottom)
    base_rim = common.make_box((0.54, 0.54, 0.06), loc=(0.0, 0.0, 0.34), color="rust")
    parts.extend([body, base_rim])

    # 3. Front mail carrier door (facing -Y)
    door_panel = common.make_box((0.42, 0.02, 0.48), loc=(0.0, -0.27, 0.62), color="car_blue")
    door_lock = common.make_box((0.04, 0.02, 0.04), loc=(0.14, -0.285, 0.78), color="brass")
    door_hinges = [
        common.make_box((0.025, 0.025, 0.05), loc=(-0.21, -0.285, 0.42), color="metal"),
        common.make_box((0.025, 0.025, 0.05), loc=(-0.21, -0.285, 0.82), color="metal"),
    ]
    # White collection schedule decal on front door (stepping out cleanly)
    decal = common.make_box((0.18, 0.01, 0.20), loc=(0.0, -0.285, 0.62), color="white")
    decal_stripe = common.make_box((0.16, 0.01, 0.04), loc=(0.0, -0.292, 0.69), color="red_paint")
    parts.extend([door_panel, door_lock, decal, decal_stripe] + door_hinges)

    # 4. Arched curved hood top (from z=0.96 to z=1.24)
    # The top arches over Y from front to rear. We construct 5-segment arch along X
    arch_bm = common.new_bm()
    arch_angles = [0.0, math.pi * 0.2, math.pi * 0.4, math.pi * 0.6, math.pi * 0.8, math.pi]
    r_arch = 0.26
    z_center = 0.96
    # Generates quads bridging X = -0.26 to +0.26 across each angle interval
    for i in range(len(arch_angles) - 1):
        a0 = arch_angles[i]
        a1 = arch_angles[i + 1]
        y0 = -r_arch * math.cos(a0)
        z0 = z_center + r_arch * math.sin(a0) * 1.05
        y1 = -r_arch * math.cos(a1)
        z1 = z_center + r_arch * math.sin(a1) * 1.05
        # Quad along X from -0.26 to +0.26
        common.add_quad(arch_bm, [(-0.26, y0, z0), (0.26, y0, z0), (0.26, y1, z1), (-0.26, y1, z1)],
                        outward=(0.0, y0 + y1, z0 + z1 - 2 * z_center), color="car_blue")
    # End caps for arch (left and right sides at X = -0.26 and +0.26)
    left_cap_verts = [arch_bm.verts.new((-0.26, -r_arch * math.cos(a), z_center + r_arch * math.sin(a) * 1.05))
                      for a in arch_angles]
    right_cap_verts = [arch_bm.verts.new((0.26, -r_arch * math.cos(a), z_center + r_arch * math.sin(a) * 1.05))
                       for a in arch_angles]
    # Add face for end caps
    face_l = arch_bm.faces.new(list(reversed(left_cap_verts)))
    face_r = arch_bm.faces.new(right_cap_verts)
    common.paint_faces(arch_bm, [face_l, face_r], "car_blue")
    parts.append(arch_bm)

    # 5. Mail drop slot / hopper door on front (-Y side)
    # Drop visor hood: angled downward over slot
    visor = common.make_box((0.36, 0.08, 0.02), color="car_blue")
    common.transform(visor, rot=(0.35, 0.0, 0.0), loc=(0.0, -0.28, 1.12))
    # Pull handle
    handle_bar = common.make_box((0.28, 0.03, 0.025), loc=(0.0, -0.29, 1.05), color="chrome")
    handle_mount_l = common.make_box((0.02, 0.03, 0.02), loc=(-0.13, -0.27, 1.05), color="metal")
    handle_mount_r = common.make_box((0.02, 0.03, 0.02), loc=(0.13, -0.27, 1.05), color="metal")
    # Drop chute cavity
    slot_cavity = common.make_box((0.32, 0.02, 0.07), loc=(0.0, -0.262, 1.06), color="gun_black")
    parts.extend([visor, handle_bar, handle_mount_l, handle_mount_r, slot_cavity])

    # 6. Ruined details: rust patch on bottom corner and top edge (sticks out 2 cm)
    rust_corner = common.make_box((0.08, 0.08, 0.12), loc=(0.24, -0.24, 0.40), color="rust")
    rust_top = common.make_box((0.14, 0.04, 0.03), loc=(-0.12, -0.26, 0.96), color="rust")
    parts.extend([rust_corner, rust_top])

    model.add(common.combine(parts))

    # Collider: covering the entire mailbox
    model.add_collider("box", common.make_box((0.56, 0.56, 1.25), loc=(0.0, 0.0, 0.625), color="white"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
