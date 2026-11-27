"""Builds all R43 New Halden climbing models:
- stairs (R43-03)
- stair_extras (R43-03)
- interiors (R43-04)
- rooftop_kit (R43-05)
- brick tower rebuild (R43-05)

Run: `flock /tmp/rotland-blender.lock blender -b --python tools/blender/build_r43_models.py`
Prints FAILED lines and exits 1 if any model fails or goes over budget.
Every group is optional: a group whose script is not written yet is skipped with a printed note.
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, "towers"))

GROUPS = []

for _group in ("stairs", "stair_extras", "interiors", "rooftop_kit", "brick"):
    try:
        _mod = __import__(_group)
    except ImportError as err:
        print(f"R43 {_group}: skipped (not present yet: {err})")
        continue
    if not hasattr(_mod, "build_all"):
        print(f"R43 {_group}: skipped (no build_all)")
        continue
    GROUPS.append((_group, _mod.build_all))


def _results(results, group):
    if isinstance(results, int):
        return [(group, results == 0)]
    out = []
    for item in results:
        if len(item) == 2:
            out.append((item[0], item[1]))
        else:
            out.append((item[0], item[2]))
    return out


def main():
    if not GROUPS:
        print("R43 BUILD FAILED: no model group found")
        return 1

    failures = []
    for group, build_all in GROUPS:
        try:
            results = build_all()
        except Exception as err:  # noqa: BLE001
            frame = err.__traceback__
            where = "?"
            while frame is not None:
                where = f"{frame.tb_frame.f_code.co_filename}:{frame.tb_lineno}"
                frame = frame.tb_next
            print(f"ERROR GROUP: {group} at {where}: {type(err).__name__}: {err}")
            failures.append(group)
            continue
        for name, ok in _results(results, group):
            if not ok:
                print(f"FAILED: {name} (over budget or not written)")
                failures.append(name)
            print(f"R43 {group}: {name} {'OK' if ok else 'FAILED'}")

    if failures:
        print("R43 BUILD FAILED:", ", ".join(failures))
        return 1
    print("R43 BUILD OK")
    return 0


if __name__ == "__main__":
    sys.exit(main())
