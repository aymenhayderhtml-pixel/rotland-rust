"""Builds all 18 R40-05 survivor trace and lobby dressing models."""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)

import camp  # noqa: E402
import car_wall  # noqa: E402
import decals  # noqa: E402
import lobby_dressing  # noqa: E402


def build_all():
    """Runs all survivor trace model groups. Returns [(name, ok)]."""
    results = []
    results.extend(car_wall.build_all())
    results.extend(decals.build_all())
    results.extend(camp.build_all())
    results.extend(lobby_dressing.build_all())
    return results
