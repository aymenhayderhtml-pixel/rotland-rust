"""Builds all R40-07 tower variety models: 20 `_floor_c`, 10 rebuilt `_crown`, and 4 `roof_*` GLBs.

Run: `flock /tmp/rotland-blender.lock blender -b --python tools/blender/towers/towers_r40.py`
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, HERE)

import brick  # noqa: E402
import deco  # noqa: E402
import glass  # noqa: E402
import roof_props  # noqa: E402
import slab  # noqa: E402
import stone  # noqa: E402

GROUPS = [
    ("roof_props", roof_props.build_all),
    ("brick", brick.build_r40_all),
    ("stone", stone.build_r40_all),
    ("glass", glass.build_r40_all),
    ("slab", slab.build_r40_all),
    ("deco", deco.build_r40_all),
]


def build_all():
    results = []
    for _, fn in GROUPS:
        results.extend(fn())
    return results


def main():
    failures = []
    for group, fn in GROUPS:
        for name, ok in fn():
            if not ok:
                print(f"FAILED: {name} (over budget or not written)")
                failures.append(name)
            print(f"R40-07 {group}: {name} {'OK' if ok else 'FAILED'}")
    if failures:
        print("TOWERS R40 FAILED:", ", ".join(failures))
        return 1
    print("TOWERS R40 OK")
    return 0


if __name__ == "__main__":
    sys.exit(main())
