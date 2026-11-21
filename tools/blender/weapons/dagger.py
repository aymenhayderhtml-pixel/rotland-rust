"""dagger: double-edged combat dagger with a steel crossguard, pommel and grooved grip. One hand."""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402
from parts import box, convex, cyl_y, loft_y, prism_yz, rbox  # noqa: E402,F401

NAME, CATEGORY, BUDGET = "dagger", "weapons", 600
GRIP = (0.0, -0.040, 0.0)  # center of the handle


def build(model):
    # Steel pommel
    model.add(loft_y([(-0.105, 0.008), (-0.098, 0.016), (-0.088, 0.014)], "metal", segments=10))
    model.add(box((0.006, 0.008, 0.006), (0.0, -0.106, 0.0), "rock_light"))

    # Ribbed ergonomic grip
    model.add(loft_y([(-0.088, 0.013), (-0.065, 0.015), (-0.040, 0.0135), (-0.015, 0.0145), (0.000, 0.013)],
                     "gun_black", segments=10))
    for y in (-0.075, -0.055, -0.035, -0.015):
        model.add(loft_y([(y - 0.004, 0.0155), (y + 0.004, 0.0155)], "leather", segments=10))

    # Symmetrical steel crossguard
    model.add(rbox((0.012, 0.010, 0.076), (0.0, 0.005, 0.0), "metal", 0.25))
    for z in (-0.036, 0.036):
        model.add(cyl_y(0.007, 0.007, 0.000, 0.010, "metal", z=z, segments=8))

    # Double-edged blade: thick diamond spine, bright cutting edges on both sides
    blade_spine = [
        (0.010, 0.012),
        (0.110, 0.011),
        (0.155, 0.000),   # sharp tip
        (0.110, -0.011),
        (0.010, -0.012),
    ]
    model.add(prism_yz(blade_spine, 0.0065, "metal"))

    # Top cutting edge bevel
    edge_top = [
        (0.010, 0.004),
        (0.110, 0.004),
        (0.155, 0.000),
        (0.120, 0.014),
        (0.010, 0.014),
    ]
    model.add(prism_yz(edge_top, 0.0035, "rock_light"))

    # Bottom cutting edge bevel
    edge_bot = [
        (0.010, -0.004),
        (0.110, -0.004),
        (0.155, 0.000),
        (0.120, -0.014),
        (0.010, -0.014),
    ]
    model.add(prism_yz(edge_bot, 0.0035, "rock_light"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=GRIP) else 1


if __name__ == "__main__":
    sys.exit(main())
