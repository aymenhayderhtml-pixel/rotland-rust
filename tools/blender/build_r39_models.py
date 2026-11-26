"""Builds all R39 New Halden tower kit models, landmarks, lobby props, and stack previews.

Run: `flock /tmp/rotland-blender.lock blender -b --python tools/blender/build_r39_models.py`
Prints FAILED lines and exits 1 if any model fails or goes over budget.
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, "towers"))

import brick  # noqa: E402
import deco  # noqa: E402
import glass  # noqa: E402
import landmarks  # noqa: E402
import lobby_props  # noqa: E402
import slab  # noqa: E402
import stack_preview  # noqa: E402
import stone  # noqa: E402

GROUPS = [
    ("brick", brick.build_all),
    ("stone", stone.build_all),
    ("glass", glass.build_all),
    ("slab", slab.build_all),
    ("deco", deco.build_all),
    ("landmarks", landmarks.build_all),
    ("lobby_props", lobby_props.build_all),
]


def main():
    failures = []
    for group, build_all in GROUPS:
        for name, ok in build_all():
            if not ok:
                print(f"FAILED: {name} (over budget or not written)")
                failures.append(name)
            print(f"R39 {group}: {name} {'OK' if ok else 'FAILED'}")

    for name, ok in stack_preview.build_all():
        if not ok:
            print(f"FAILED: {name} (stack preview failed)")
            failures.append(name)
        print(f"R39 stack_preview: {name} {'OK' if ok else 'FAILED'}")

    if failures:
        print("R39 BUILD FAILED:", ", ".join(failures))
        return 1
    print("R39 BUILD OK")
    return 0


if __name__ == "__main__":
    sys.exit(main())
