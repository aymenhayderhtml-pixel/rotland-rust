"""Builds every POI building (cabin, house, gas station, shed, barn, watchtower) and the settlement.

Run: blender -b --python tools/blender/buildings/build_buildings.py
Exits with 1 if any building is over budget or fails to build.
"""
import importlib
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, HERE)

BUILDINGS = ["cabin", "house_small", "gas_station", "shed", "barn", "watchtower", "settlement"]


def main():
    failures = []
    for name in BUILDINGS:
        module = importlib.import_module(name)
        if module.main() != 0:
            failures.append(name)
    if failures:
        print("BUILDINGS FAILED:", ", ".join(failures))
        return 1
    print("BUILDINGS OK:", len(BUILDINGS), "buildings within budget")
    return 0


if __name__ == "__main__":
    sys.exit(main())
