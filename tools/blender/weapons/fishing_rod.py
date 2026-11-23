"""fishing_rod: long brown rod with a dark grip, a small metal reel near the handle and two line guides. Held in one hand."""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402
from parts import box, loft_y  # noqa: E402

NAME, CATEGORY, BUDGET = "fishing_rod", "weapons", 800
GRIP = (0.0, 0.0, 0.0)  # middle of the grip


def build(model):
    # Dark grip, about 0.25 m long, centred on the hand
    model.add(loft_y([(-0.125, 0.016), (0.125, 0.016)], "gun_black", segments=10))
    # Thin tapered brown rod from the grip top up to about 1.6 m
    model.add(loft_y([(0.125, 0.011), (0.80, 0.007), (1.60, 0.0025)], "wood_crate", segments=8))
    # Small metal reel box just in front of the grip
    model.add(box((0.040, 0.050, 0.050), (0.030, -0.080, 0.0), "metal"))
    # Two tiny metal line guides along the rod
    for y in (0.45, 0.95):
        model.add(box((0.012, 0.012, 0.012), (0.0, y, 0.0), "metal"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=GRIP) else 1


if __name__ == "__main__":
    sys.exit(main())
