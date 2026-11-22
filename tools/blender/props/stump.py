"""stump: a cut tree stump with a pale top ring and three roots (<= 200 tris). Has a collider."""
import math
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402

NAME, CATEGORY, BUDGET = "stump", "props", 200
RADIUS = 0.45
HEIGHT = 0.55


def build(model):
    body = common.make_frustum(RADIUS, RADIUS * 1.06, 0.0, HEIGHT, segments=8, color="bark_dark")
    body.normal_update()
    common.paint_faces(body, [f for f in body.faces if f.normal.z > 0.9], "bark_light")  # cut top
    model.add(body)
    for angle in (0.0, 2.1, 4.2):
        center = (RADIUS * 0.8 * math.cos(angle), RADIUS * 0.8 * math.sin(angle))
        # Start root base at z = -0.01 so root bottom cap does not coplanar with trunk bottom cap at z = 0
        model.add(common.make_frustum(0.12, 0.08, -0.01, 0.25, segments=5, center=center, color="bark_dark"))
    model.add_collider("body", common.make_frustum(RADIUS + 0.02, RADIUS * 1.06, 0.0, HEIGHT, segments=8,
                                                   color="bark_dark"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
