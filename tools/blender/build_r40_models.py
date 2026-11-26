"""Builds all R40 New Halden overgrowth, damage, survivor trace, and tower variety models.

Run: `flock /tmp/rotland-blender.lock blender -b --python tools/blender/build_r40_models.py`
Prints FAILED lines and exits 1 if any model fails or goes over budget.
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, "overgrowth"))
sys.path.insert(0, os.path.join(HERE, "damage"))
sys.path.insert(0, os.path.join(HERE, "traces"))
sys.path.insert(0, os.path.join(HERE, "towers"))

import overgrowth  # noqa: E402
import damage  # noqa: E402
import traces  # noqa: E402
import towers_r40  # noqa: E402

GROUPS = [
    ("overgrowth", overgrowth.build_all),
    ("damage", damage.build_all),
    ("traces", traces.build_all),
    ("towers_r40", towers_r40.build_all),
]


def main():
    failures = []
    for group, build_all in GROUPS:
        for name, ok in build_all():
            if not ok:
                print(f"FAILED: {name} (over budget or not written)")
                failures.append(name)
            print(f"R40 {group}: {name} {'OK' if ok else 'FAILED'}")

    if failures:
        print("R40 BUILD FAILED:", ", ".join(failures))
        return 1
    print("R40 BUILD OK")
    return 0


if __name__ == "__main__":
    sys.exit(main())
