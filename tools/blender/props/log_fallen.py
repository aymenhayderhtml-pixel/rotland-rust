"""log_fallen: a cut log lying flat on the ground, pale cut ends (<= 300 tris). Has a collider."""
import math
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402

NAME, CATEGORY, BUDGET = "log_fallen", "nature", 600
RADIUS = 0.24
LENGTH = 3.0


def _log(segments, color):
    """Log standing along Z, with the two cut ends painted pale. Turned flat by the caller."""
    log = common.make_frustum(RADIUS, RADIUS * 0.92, 0.0, LENGTH, segments=segments, color=color)
    log.normal_update()
    common.paint_faces(log, [f for f in log.faces if abs(f.normal.z) > 0.9], "wood_crate")
    return log


def build(model):
    # Turn the log to lie along X: rotate about Y, so Z becomes X
    log = _log(12, "bark_light")
    common.transform(log, rot=(0.0, math.pi / 2, 0.3))
    model.add(log)
    # A snapped branch stub, a moss strip along the top and dark bark rings
    stub = common.make_frustum(0.08, 0.05, 0.0, 0.45, segments=6, color="bark_dark")
    common.transform(stub, rot=(-0.6, 0.0, 0.3), loc=(1.2 * math.cos(0.3), 1.2 * math.sin(0.3), 0.15))
    model.add(stub)
    moss = common.make_box((1.6, 0.18, 0.04), loc=(0.0, 0.0, 0.0), color="leaf_dark")
    common.transform(moss, rot=(0.0, 0.0, 0.3), loc=(1.4 * math.cos(0.3), 1.4 * math.sin(0.3), RADIUS * 0.95))
    model.add(moss)
    for t in (0.5, 2.2):
        ring = common.make_frustum(RADIUS + 0.01, RADIUS + 0.01, t, t + 0.06, segments=12, color="bark_dark")
        common.transform(ring, rot=(0.0, math.pi / 2, 0.3))
        model.add(ring)
    collider = _log(6, "bark_light")
    common.transform(collider, rot=(0.0, math.pi / 2, 0.3))
    model.add_collider("log", collider)


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
