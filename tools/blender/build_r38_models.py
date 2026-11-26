"""Builds the R38 jam models (cars and burnt cars, buses, army truck, luggage and body props, highway gantry and barrier).

Run: blender -b --python tools/blender/build_r38_models.py
Writes assets/models/vehicles/ and assets/models/props/ plus previews. Does not touch the palette sheets.
Prints FAILED lines and exits 1 if any model fails or goes over budget.
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, "metro"))

import jam_bus  # noqa: E402
import jam_cars  # noqa: E402
import jam_props_bags  # noqa: E402
import jam_props_bodies  # noqa: E402
import jam_props_road  # noqa: E402
import jam_truck  # noqa: E402

GROUPS = [
    ("cars", jam_cars.build_all),
    ("buses", jam_bus.build_all),
    ("truck", jam_truck.build_all),
    ("props", jam_props_bags.build_all),
    ("bodies", jam_props_bodies.build_all),
    ("road", jam_props_road.build_all),
]


def main():
    failures = []
    for group, build_all in GROUPS:
        for name, ok in build_all():
            if not ok:
                print(f"FAILED: {name} (over budget or not written)")
                failures.append(name)
            print(f"R38 {group}: {name} {'OK' if ok else 'FAILED'}")
    if failures:
        print("R38 BUILD FAILED:", ", ".join(failures))
        return 1
    print("R38 BUILD OK")
    return 0


if __name__ == "__main__":
    sys.exit(main())
