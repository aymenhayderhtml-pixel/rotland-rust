"""Builds all 13 R40 overgrowth nature models.

Run standalone: `flock /tmp/rotland-blender.lock blender -b --python tools/blender/overgrowth/overgrowth.py`
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
BLENDER_DIR = os.path.abspath(os.path.join(HERE, ".."))
for _p in (HERE, BLENDER_DIR):
    if _p not in sys.path:
        sys.path.insert(0, _p)

import ground_weeds  # noqa: E402
import trees_shrubs  # noqa: E402
import vines  # noqa: E402


def build_all():
    """Builds all 13 R40 overgrowth models. Returns [(name, ok)]."""
    results = []
    results.extend(vines.build_all())
    results.extend(ground_weeds.build_all())
    results.extend(trees_shrubs.build_all())
    return results


if __name__ == "__main__":
    failed = [name for name, ok in build_all() if not ok]
    if failed:
        print("OVERGROWTH FAILED:", ", ".join(failed))
        sys.exit(1)
    print("OVERGROWTH OK")
