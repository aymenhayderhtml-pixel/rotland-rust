"""Builds all R40-04 damage set-piece models.

Run: `flock /tmp/rotland-blender.lock blender -b --python tools/blender/damage/damage.py`
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
if HERE not in sys.path:
    sys.path.insert(0, HERE)

import burnt_overlay  # noqa: E402
import car_burnt_pile  # noqa: E402
import fallen_section  # noqa: E402
import leaning_tower  # noqa: E402
import rubble  # noqa: E402


def build_all():
    results = []
    for mod in (fallen_section, rubble, leaning_tower, burnt_overlay, car_burnt_pile):
        results.extend(mod.build_all())
    return results


if __name__ == "__main__":
    res = build_all()
    failed = [n for n, ok in res if not ok]
    for n, ok in res:
        print(f"DAMAGE: {n} {'OK' if ok else 'FAILED'}")
    print(f"DAMAGE TOTAL: {len(res) - len(failed)}/{len(res)} OK")
    sys.exit(1 if failed else 0)
