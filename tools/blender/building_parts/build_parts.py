"""Builds every player-built base part model and the test assembly preview.

Run: blender -b --python tools/blender/building_parts/build_parts.py [-- --only=base_floor,base_wall]
Exits with 1 if any part is over budget or fails to build.
"""
import importlib
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, os.path.join(HERE, "..", "buildings"))
sys.path.insert(0, HERE)

PARTS = [
    "base_floor",
    "base_wall",
    "base_wall_window",
    "base_wall_door",
    "base_half_wall",
    "base_roof",
    "base_stairs",
]


def _only_names():
    for arg in sys.argv:
        if arg.startswith("--only="):
            return set(arg.split("=", 1)[1].split(","))
    return None


def main():
    only = _only_names()
    failures = []
    built = 0

    target_parts = [p for p in PARTS if only is None or p in only]
    for name in target_parts:
        try:
            module = importlib.import_module(name)
            if module.main() != 0:
                failures.append(name)
            else:
                built += 1
        except Exception as err:
            print(f"FAILED to build {name}: {err}")
            failures.append(name)

    # Render test assembly preview if room is requested or building all
    if only is None or "room" in only or "base_preview_room" in only:
        try:
            import base_preview_room
            base_preview_room.main()
        except Exception as err:
            print(f"FAILED to render preview room: {err}")
            failures.append("base_preview_room")

    if failures:
        print("BASE PARTS FAILED:", ", ".join(failures))
        return 1

    print(f"BASE PARTS OK: {built} parts built within budget")
    return 0


if __name__ == "__main__":
    sys.exit(main())
