"""Builds every weapon .glb into assets/models/weapons/. Returns 1 if any is over budget.

Run: blender -b --python tools/blender/weapons/build_weapons.py
"""
import importlib
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, ".."))

WEAPONS = ["bat", "machete", "pistol", "knife", "axe", "shotgun", "rifle", "ak47", "smg", "crowbar", "katana"]


def main():
    failures = []
    for name in WEAPONS:
        if importlib.import_module(name).main() != 0:
            failures.append(name)
    if failures:
        print("WEAPON BUILD FAILED:", ", ".join(failures))
        return 1
    print("WEAPON BUILD OK:", len(WEAPONS), "weapons within budget")
    return 0


if __name__ == "__main__":
    sys.exit(main())
