"""Builds all R41 New Halden silent life and item/crate models.

Run: `flock /tmp/rotland-blender.lock blender -b --python tools/blender/build_r41_models.py`
Prints FAILED lines and exits 1 if any model fails or goes over budget.
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, "life"))
sys.path.insert(0, os.path.join(HERE, "items_r41"))

import life  # noqa: E402

GROUPS = [
    ("life", life.build_all),
]

try:
    import items_r41  # noqa: E402
    GROUPS.append(("items_r41", items_r41.build_all))
except ImportError:
    print("R41 items_r41: skipped (not present yet)")


def main():
    failures = []
    for group, build_all in GROUPS:
        for name, ok in build_all():
            if not ok:
                print(f"FAILED: {name} (over budget or not written)")
                failures.append(name)
            print(f"R41 {group}: {name} {'OK' if ok else 'FAILED'}")

    if failures:
        print("R41 BUILD FAILED:", ", ".join(failures))
        return 1
    print("R41 BUILD OK")
    return 0


if __name__ == "__main__":
    sys.exit(main())
