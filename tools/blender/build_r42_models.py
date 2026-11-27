"""Builds all R42 New Halden big-place models (hospital, police, fire station, army line) and the
four R42 zombie looks.

Run: `flock /tmp/rotland-blender.lock blender -b --python tools/blender/build_r42_models.py`
Prints FAILED lines and exits 1 if any model fails or goes over budget.
Every group is optional: a group whose script is not written yet is skipped with a printed note,
so one subagent's missing file never blocks the others.
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, "places"))
sys.path.insert(0, os.path.join(HERE, "characters"))

GROUPS = []

# Build order is the task order: the three big places, then the army line, then the zombie looks.
for _group in ("hospital", "police", "fire_station", "army_line", "zombie_looks"):
    try:
        _mod = __import__(_group)
    except ImportError as err:
        print(f"R42 {_group}: skipped (not present yet: {err})")
        continue
    if not hasattr(_mod, "build_all"):
        print(f"R42 {_group}: skipped (no build_all)")
        continue
    GROUPS.append((_group, _mod.build_all))


def _results(results, group):
    """Normalise a group's `build_all()` return value into [(name, ok)].

    Most groups return a list of (name, ok); a group that only prints its own summary (the zombie
    looks) returns an exit code (0 = fine). Both are accepted so a group can pick either style.
    """
    if isinstance(results, int):
        return [(group, results == 0)]
    out = []
    for item in results:
        if len(item) == 2:
            out.append((item[0], item[1]))
        else:  # (name, tris, ok)
            out.append((item[0], item[2]))
    return out


def main():
    if not GROUPS:
        print("R42 BUILD FAILED: no model group found")
        return 1

    failures = []
    for group, build_all in GROUPS:
        # One group crashing must not stop the others: print one error line, mark the group failed.
        # (No traceback in the log: the lead greps the log for `Traceback` to spot real failures.)
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
            print(f"R42 {group}: {name} {'OK' if ok else 'FAILED'}")

    if failures:
        print("R42 BUILD FAILED:", ", ".join(failures))
        return 1
    print("R42 BUILD OK")
    return 0


if __name__ == "__main__":
    sys.exit(main())
