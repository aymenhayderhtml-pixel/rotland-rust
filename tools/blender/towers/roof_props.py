"""Standalone rooftop prop GLBs (`roof_water_tank_ny`, `roof_machinery`, `roof_stair_hut`, `roof_antenna`)."""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, os.path.join(HERE, "..", "buildings"))
sys.path.insert(0, HERE)

import common  # noqa: E402
import kit  # noqa: E402
import roof_parts  # noqa: E402

CATEGORY = "buildings"
BUDGET = 1500


def roof_water_tank_ny(k, x=0.0, y=0.0, z0=0.0, r=1.40, leg_h=1.45, tank_h=2.25):
    roof_parts.water_tank_ny(k, x, y, z0, r=r, leg_h=leg_h, tank_h=tank_h)


def roof_machinery(k, x=0.0, y=0.0, z0=0.0):
    roof_parts.machinery(k, x, y, z0)


def roof_stair_hut(k, x=0.0, y=0.0, z0=0.0, wall_color="concrete_grey"):
    roof_parts.stair_hut(k, x, y, z0, wall_color=wall_color)


def roof_antenna(k, x=0.0, y=0.0, z0=0.0, h=4.2):
    roof_parts.lattice_antenna(k, x, y, z0, h=h)


def build_all():
    specs = [
        ("roof_water_tank_ny", lambda m: roof_water_tank_ny(kit.Kit(m))),
        ("roof_machinery", lambda m: roof_machinery(kit.Kit(m))),
        ("roof_stair_hut", lambda m: roof_stair_hut(kit.Kit(m))),
        ("roof_antenna", lambda m: roof_antenna(kit.Kit(m))),
    ]
    results = []
    for name, fn in specs:
        ok = common.run_model(name, CATEGORY, BUDGET, fn, grip=(0.0, 0.0, 0.0), view=kit.FRONT_VIEW)
        results.append((name, ok))
    return results


if __name__ == "__main__":
    out = build_all()
    failed = [n for n, ok in out if not ok]
    if failed:
        print("ROOF PROPS FAILED:", failed)
        sys.exit(1)
