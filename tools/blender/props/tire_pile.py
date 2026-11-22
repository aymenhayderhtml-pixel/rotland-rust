"""tire_pile: six old tyres stacked in a pyramid (<= 900 tris). Has a collider around the pile."""
import math
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402

NAME, CATEGORY, BUDGET = "tire_pile", "props", 900
TYRE_HEIGHT = 0.22
TYRE_RADIUS = 0.36

# (x, y, z0): three on the ground, two on them, one on top.
# Slight vertical offsets on upper tyres mimic real stacking and prevent shared bottom planes.
TYRES = [
    (-0.40, -0.20, 0.0),
    (0.40, -0.20, 0.0),
    (0.0, 0.60, 0.0),
    (-0.20, 0.20, TYRE_HEIGHT),
    (0.30, 0.45, TYRE_HEIGHT + 0.02),
    (0.05, 0.10, 2 * TYRE_HEIGHT + 0.02),
]


def _make_tyre(x, y, z0, segments=12):
    bm = common.new_bm()
    r_outer = TYRE_RADIUS
    r_shoulder = TYRE_RADIUS - 0.03
    r_inner = 0.20
    z_bot = z0
    z_sh = z0 + TYRE_HEIGHT - 0.04
    z_top = z0 + TYRE_HEIGHT
    z_hole = z0 + TYRE_HEIGHT - 0.06

    def ring(r, z):
        return [bm.verts.new((x + math.cos(a) * r, y + math.sin(a) * r, z))
                for a in (2 * math.pi * i / segments for i in range(segments))]

    v_bot = ring(r_outer, z_bot)
    v_sh = ring(r_outer, z_sh)
    v_top = ring(r_shoulder, z_top)
    v_in_top = ring(r_inner, z_top)
    v_in_bot = ring(r_inner, z_hole)

    # Bottom cap
    bm.faces.new(list(reversed(v_bot)))
    common.paint_faces(bm, list(bm.faces), "tire_black")

    # Side wall (bot to sh) and shoulder (sh to top)
    for i in range(segments):
        j = (i + 1) % segments
        f_side = bm.faces.new([v_bot[i], v_bot[j], v_sh[j], v_sh[i]])
        f_sh = bm.faces.new([v_sh[i], v_sh[j], v_top[j], v_top[i]])
        f_rim = bm.faces.new([v_top[i], v_top[j], v_in_top[j], v_in_top[i]])
        common.paint_faces(bm, [f_side, f_sh, f_rim], "tire_black")

        f_inner = bm.faces.new([v_in_top[i], v_in_top[j], v_in_bot[j], v_in_bot[i]])
        common.paint_faces(bm, [f_inner], "gun_black")

    # Inner recessed floor
    f_hole = bm.faces.new(v_in_bot)
    common.paint_faces(bm, [f_hole], "gun_black")

    return bm


def build(model):
    for x, y, z0 in TYRES:
        model.add(_make_tyre(x, y, z0, segments=12))
    model.add_collider("pile", common.make_box((1.6, 1.6, 0.66), loc=(0.0, 0.2, 0.33), color="white"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
