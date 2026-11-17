"""katana: curved blade with a bright edge, brass collar, round guard and a wrapped handle. Two-handed."""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402
from parts import box, convex, cyl_y, hull_side, loft_y, prism_yz, rbox  # noqa: E402,F401

NAME, CATEGORY, BUDGET = "katana", "weapons", 1500
GRIP = (0.0, -0.09, 0.0)  # middle of the wrapped handle, where the hands meet


def _curve(y):
    return 0.03 * (y / 0.8) ** 2  # gentle upward curve toward the tip


def build(model):
    # Handle: leather core, raised black wraps and a metal pommel cap
    model.add(loft_y([(-0.205, 0.0145), (0.0, 0.0145)], "leather", segments=8))
    for k in range(8):
        y = -0.195 + k * 0.025
        model.add(loft_y([(y, 0.0158), (y + 0.009, 0.0158)], "gun_black", segments=8))
    model.add(loft_y([(-0.224, 0.006), (-0.221, 0.016), (-0.205, 0.017)], "metal", segments=10))
    # Round guard and the brass collar
    model.add(cyl_y(0.036, 0.036, 0.002, 0.009, "gun_black", segments=14))
    model.add(rbox((0.012, 0.030, 0.030), (0.0, 0.025, 0.0), "brass", 0.3))
    # Curved blade: steel back, bright thin edge
    ys = [0.02 + i * 0.06 for i in range(13)]  # 0.02 .. 0.74
    back = [(y, _curve(y) + 0.013 - 0.006 * y / 0.74) for y in ys]
    mid = [(y, _curve(y) - 0.002) for y in ys]
    edge = [(y, _curve(y) - 0.013 + 0.006 * y / 0.74) for y in ys]
    tip = (0.80, _curve(0.80) + 0.002)
    model.add(prism_yz(back + [tip] + list(reversed(mid)), 0.008, "metal"))
    model.add(prism_yz(mid + [tip] + list(reversed(edge)), 0.005, "rock_light"))
    model.add_empty("grip_L", (0.0, -0.14, 0.0))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=GRIP) else 1


if __name__ == "__main__":
    sys.exit(main())
