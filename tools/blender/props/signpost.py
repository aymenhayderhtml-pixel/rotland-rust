"""signpost: 2.5 m wooden trailhead signpost with 3 angled arrow boards (<= 120 tris)."""
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402

NAME, CATEGORY, BUDGET = "signpost", "props", 120


def _arrow_board(length, width, thick, color, rot_z, z):
    """Rectangular sign plank with a pointed tip on +X, rotated around Z at height `z`."""
    bm = common.new_bm()
    hx = length / 2
    hy = width / 2
    hz = thick / 2
    tip = hx + 0.16
    # 5-sided prism in XY extruded in Z
    pts = [(-hx, -hy), (hx, -hy), (tip, 0.0), (hx, hy), (-hx, hy)]
    bot = [bm.verts.new((x, y, -hz)) for x, y in pts]
    top = [bm.verts.new((x, y, hz)) for x, y in pts]
    bm.faces.new(list(reversed(bot)))
    bm.faces.new(top)
    for i in range(len(pts)):
        j = (i + 1) % len(pts)
        bm.faces.new([bot[i], bot[j], top[j], top[i]])
    common.paint(bm, color)
    common.transform(bm, loc=(0.12, 0.0, z), rot=(0.0, 0.0, rot_z))
    return bm


def build(model):
    # Main wooden post, 2.5 m tall
    model.add(common.make_box((0.14, 0.14, 2.5), loc=(0.0, 0.0, 1.25), color="bark_dark"))

    # 3 arrow boards at different angles and heights
    model.add(_arrow_board(0.75, 0.18, 0.04, "wood_crate", 0.35, 2.28))
    model.add(_arrow_board(0.80, 0.18, 0.04, "bark_light", -0.55, 2.02))
    model.add(_arrow_board(0.70, 0.18, 0.04, "white", 2.35, 1.76))

    model.add_collider("pole", common.make_box((0.20, 0.20, 2.5), loc=(0.0, 0.0, 1.25), color="white"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
