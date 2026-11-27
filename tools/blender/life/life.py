"""Builds all R41-04 silent life models (poles, wires, billboards, flappers, crows)."""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)

import billboards  # noqa: E402
import crows  # noqa: E402
import flappers  # noqa: E402
import poles  # noqa: E402


def build_all():
    """Runs all silent life model groups. Returns [(name, ok)]."""
    results = []
    results.extend(poles.build_all())
    results.extend(billboards.build_all())
    results.extend(flappers.build_all())
    results.extend(crows.build_all())
    return results
