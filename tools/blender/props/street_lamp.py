"""street_lamp: bent metal street lamp, 6 m pole, dead light head (<= 800 tris). Has a collider on the pole."""
import math
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402

NAME, CATEGORY, BUDGET = "street_lamp", "props", 800


def _build_arm_segment(p0, p1, r0, r1, segments=6, color="metal"):
    """Frustum oriented along the vector from p0 to p1."""
    dx = p1[0] - p0[0]
    dy = p1[1] - p0[1]
    dz = p1[2] - p0[2]
    length = math.sqrt(dx * dx + dy * dy + dz * dz)
    if length < 1e-4:
        return common.new_bm()
    bm = common.make_frustum(r0, r1, 0.0, length, segments=segments, color=color)
    # Align +Z with (dx, dy, dz)
    yaw = math.atan2(dx, dy)
    pitch = math.atan2(math.sqrt(dx * dx + dy * dy), dz)
    # Rotation: pitch around X then yaw around Z
    common.transform(bm, rot=(-pitch, 0.0, -yaw), loc=p0)
    return bm


def build(model):
    # Base plate on the ground (z=0)
    base_plate = common.make_frustum(0.25, 0.22, 0.0, 0.06, segments=8, color="metal")
    base_collar = common.make_frustum(0.22, 0.13, 0.06, 0.42, segments=8, color="rust")
    bolts = [common.make_box((0.04, 0.04, 0.08), loc=(0.18 * math.cos(a), 0.18 * math.sin(a), 0.04), color="metal")
             for a in (0.0, math.pi / 2, math.pi, 3 * math.pi / 2)]
    model.add(common.combine([base_plate, base_collar] + bolts))

    # Lower mast with inspection hatch
    lower_pole = common.make_frustum(0.12, 0.095, 0.42, 3.40, segments=8, color="metal")
    # Rust band and open maintenance hatch (sleeve sticks out 1.5 cm)
    rust_band = common.make_frustum(0.132, 0.128, 0.70, 1.15, segments=8, color="rust")
    hatch_hole = common.make_box((0.08, 0.04, 0.28), loc=(0.0, -0.10, 0.92), color="gun_black")
    hatch_cover = common.make_box((0.09, 0.015, 0.26), loc=(0.05, -0.14, 0.88), color="rust")
    common.transform(hatch_cover, rot=(0.1, 0.2, -0.3))  # dangling loose hatch cover
    model.add(common.combine([lower_pole, rust_band, hatch_hole, hatch_cover]))

    # Mid pole section with slight collision dent/bend
    mid_pole = common.make_frustum(0.095, 0.08, 0.0, 1.40, segments=8, color="metal")
    common.transform(mid_pole, rot=(0.05, -0.04, 0.0), loc=(0.0, 0.0, 3.40))
    model.add(mid_pole)

    # Upper pole and curved outreach arm towards -Y (the street)
    # Waypoints for the sweeping curved arch:
    pts = [
        (0.0, 0.07, 4.75),
        (0.0, -0.15, 5.35),
        (0.0, -0.55, 5.80),
        (0.0, -1.15, 6.05),
        (0.0, -1.75, 5.95),
        (0.0, -2.15, 5.70),
    ]
    radii = [0.08, 0.075, 0.068, 0.062, 0.055, 0.050]
    arm_parts = []
    for i in range(len(pts) - 1):
        c = "rust" if i in (1, 3) else "metal"
        arm_parts.append(_build_arm_segment(pts[i], pts[i + 1], radii[i], radii[i + 1], segments=6, color=c))

    # Diagonal support strut between mast and arm
    strut = _build_arm_segment((0.0, 0.02, 5.0), (0.0, -0.85, 5.82), 0.022, 0.022, segments=5, color="rust")
    arm_parts.append(strut)
    model.add(common.combine(arm_parts))

    # Dead light head (cobra-head style luminaire at the end of the arm)
    # Head position at pts[-1]
    hx, hy, hz = pts[-1]
    head_mount = common.make_frustum(0.055, 0.07, 0.0, 0.12, segments=6, color="metal")
    common.transform(head_mount, rot=(math.pi / 2 - 0.5, 0.0, 0.0), loc=(hx, hy, hz))

    # Luminaire body: tilted downward towards the street (-Y, -Z)
    lum_top = common.make_box((0.30, 0.65, 0.12), loc=(0.0, -0.25, 0.0), color="metal")
    lum_taper = common.make_box((0.24, 0.20, 0.08), loc=(0.0, -0.55, 0.0), color="rust")
    # Glass frame and dead unlit glass panel underneath (stepped levels with >= 1.5 cm gaps)
    glass_frame = common.make_box((0.26, 0.52, 0.02), loc=(0.0, -0.28, -0.08), color="gun_black")
    glass_intact = common.make_box((0.22, 0.26, 0.02), loc=(0.0, -0.38, -0.11), color="glass_tint")
    # Broken half of glass panel: dark cavity with broken bulb and dangling wire
    broken_cavity = common.make_box((0.22, 0.24, 0.03), loc=(0.0, -0.14, -0.055), color="gun_black")
    broken_socket = common.make_frustum(0.04, 0.03, 0.0, 0.06, segments=6, color="brass")
    common.transform(broken_socket, rot=(math.pi, 0.0, 0.0), loc=(0.0, -0.16, -0.04))
    broken_bulb = common.make_frustum(0.03, 0.01, 0.0, 0.04, segments=5, color="white")
    common.transform(broken_bulb, rot=(math.pi + 0.3, 0.2, 0.0), loc=(0.0, -0.16, -0.07))
    # Dangling loose wire hanging down from broken head
    wire1 = _build_arm_segment((0.02, -0.12, -0.08), (0.04, -0.14, -0.32), 0.008, 0.008, segments=4, color="gun_black")
    wire2 = _build_arm_segment((0.04, -0.14, -0.32), (0.02, -0.18, -0.55), 0.008, 0.008, segments=4, color="gun_black")

    luminaire = common.combine([lum_top, lum_taper, glass_frame, glass_intact, broken_cavity,
                                broken_socket, broken_bulb, wire1, wire2])
    # Tilt luminaire along arm angle (~25 deg)
    common.transform(luminaire, rot=(-0.45, 0.0, 0.0), loc=(hx, hy - 0.05, hz - 0.02))
    model.add(luminaire)

    # Collider: vertical box covering the pole from ground up to 6.0 m
    model.add_collider("pole", common.make_box((0.36, 0.36, 6.0), loc=(0.0, 0.0, 3.0), color="white"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
