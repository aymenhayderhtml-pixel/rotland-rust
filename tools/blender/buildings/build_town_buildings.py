"""Builds the 10 new R33 outer-biome town buildings (desert, snow, mountains, swamp).

Run: blender -b --python tools/blender/buildings/build_town_buildings.py
Exits with 1 if any building is over budget or fails to build.
Never rebuilds old core buildings.
"""
import importlib
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, HERE)

BUILDINGS = [
    "adobe_house",
    "motel",
    "water_tower",
    "desert_store",
    "log_cabin_snow",
    "research_station",
    "radio_tower",
    "ranger_station",
    "stilt_shack",
    "bait_shop",
]


def main():
    failures = []
    for name in BUILDINGS:
        module = importlib.import_module(name)
        if module.main() != 0:
            failures.append(name)
    if failures:
        print("TOWN BUILDINGS FAILED:", ", ".join(failures))
        return 1
    print("TOWN BUILDINGS OK:", len(BUILDINGS), "buildings within budget")
    return 0


if __name__ == "__main__":
    sys.exit(main())
