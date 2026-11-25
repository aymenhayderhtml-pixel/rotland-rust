"""lily_mud_patch: flat raised swamp mud puddle with notched lily pads (<= 80 tris). No collider."""
import math
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import bmesh  # noqa: E402
import common  # noqa: E402

NAME, CATEGORY, BUDGET = "lily_mud_patch", "props", 80

PUDDLE_SCALES = (1.04, 0.94, 1.08, 0.92, 1.02, 0.95, 1.06, 0.93)
PAD_PROFILE = (
    (0.00, 0.30),   # V-notch tip
    (0.48, 0.96),
    (2.25, 1.02),
    (4.03, 1.02),
    (5.80, 0.96),
)


def _mud_puddle():
    """Closed 8-sided organic mud puddle (~2.6 x 2.3 m, sunk 5 cm, raised 3 cm; 28 tris)."""
    bm = common.new_bm()
    rx, ry = 1.32, 1.14
    n = len(PUDDLE_SCALES)
    bottom = []
    top = []
    for i, s in enumerate(PUDDLE_SCALES):
        a = 2.0 * math.pi * i / n
        x = math.cos(a) * rx * s
        y = math.sin(a) * ry * s
        bottom.append(bm.verts.new((x, y, -0.05)))
        top.append(bm.verts.new((x * 0.94, y * 0.94, 0.03)))

    for i in range(n):
        j = (i + 1) % n
        bm.faces.new([bottom[i], bottom[j], top[j], top[i]])
    bm.faces.new(list(reversed(bottom)))
    bm.faces.new(top)
    bmesh.ops.recalc_face_normals(bm, faces=list(bm.faces))
    common.paint(bm, "swamp_mud")
    return bm


def _lily_pad(cx, cy, radius, yaw):
    """Closed manifold notched lily pad sitting on top of the puddle (10 tris)."""
    bm = common.new_bm()
    rim = []
    for a_off, r_scale in PAD_PROFILE:
        a = yaw + a_off
        r = radius * r_scale
        rim.append(bm.verts.new((cx + math.cos(a) * r, cy + math.sin(a) * r, 0.033)))
    top_center = bm.verts.new((cx, cy, 0.044))
    bot_center = bm.verts.new((cx, cy, 0.024))

    n = len(rim)
    for i in range(n):
        j = (i + 1) % n
        bm.faces.new([rim[i], rim[j], top_center])
        bm.faces.new([rim[j], rim[i], bot_center])
    bmesh.ops.recalc_face_normals(bm, faces=list(bm.faces))
    common.paint(bm, "swamp_moss")
    return bm


def build(model):
    # Dark raised mud puddle (28 tris)
    model.add(_mud_puddle())

    # 5 notched swamp_moss lily pads resting on the puddle (5 * 10 = 50 tris)
    for cx, cy, radius, yaw in [
        (-0.48, 0.22, 0.24, 0.60),
        (0.36, 0.38, 0.21, 2.30),
        (0.52, -0.24, 0.25, 4.10),
        (-0.18, -0.44, 0.19, 5.40),
        (-0.04, 0.04, 0.17, 1.40),
    ]:
        model.add(_lily_pad(cx, cy, radius, yaw))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=(0.0, 0.0, 0.0)) else 1


if __name__ == "__main__":
    sys.exit(main())
