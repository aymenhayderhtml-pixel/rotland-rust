"""swamp_stump: broken rotten swamp stump with jagged top, roots and moss (<= 150 tris). Has a collider."""
import math
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import bmesh  # noqa: E402
import common  # noqa: E402
import veg  # noqa: E402

NAME, CATEGORY, BUDGET = "swamp_stump", "props", 150

TOP_Z = (0.86, 0.72, 0.56, 0.44, 0.48, 0.62, 0.78, 0.84)
BOT_R = (0.52, 0.48, 0.51, 0.49, 0.53, 0.48, 0.50, 0.52)
TOP_R = (0.37, 0.39, 0.41, 0.38, 0.39, 0.40, 0.37, 0.36)


def _jagged_body():
    """Closed 8-sided stump body with jagged broken rim and recessed rotten core (30 tris)."""
    bm = common.new_bm()
    n = len(TOP_Z)
    bottom = []
    top = []
    for i in range(n):
        a = 2.0 * math.pi * i / n
        ca, sa = math.cos(a), math.sin(a)
        bottom.append(bm.verts.new((ca * BOT_R[i], sa * BOT_R[i], -0.05)))
        top.append(bm.verts.new((ca * TOP_R[i], sa * TOP_R[i], TOP_Z[i])))
    core = bm.verts.new((0.0, 0.0, 0.34))

    bark_faces = []
    core_faces = []
    for i in range(n):
        j = (i + 1) % n
        bark_faces.append(bm.faces.new([bottom[i], bottom[j], top[j], top[i]]))
        core_faces.append(bm.faces.new([top[i], top[j], core]))
    bark_faces.append(bm.faces.new(list(reversed(bottom))))

    bmesh.ops.recalc_face_normals(bm, faces=list(bm.faces))
    common.paint_faces(bm, bark_faces, "swamp_bark")
    common.paint_faces(bm, core_faces, "swamp_mud")
    return bm


def build(model):
    # Broken rotten stump trunk (30 tris)
    model.add(_jagged_body())

    # 2 jagged wood splinters rising from the high rim (2 * 4 = 8 tris)
    for angle, top_z, tilt in ((0.05, 0.86, (0.14, 0.10, 0.05)), (5.45, 0.84, (-0.12, 0.14, 5.45))):
        splinter = common.make_frustum(0.065, 0.0, 0.0, 0.24, segments=3, color="swamp_bark")
        common.transform(splinter, rot=tilt,
                         loc=(0.34 * math.cos(angle), 0.34 * math.sin(angle), top_z - 0.06))
        model.add(splinter)

    # 4 gnarled roots flaring into the ground (4 * 12 = 48 tris)
    for angle, z_start, reach in [
        (0.30, 0.36, 0.92),
        (1.85, 0.32, 0.86),
        (3.45, 0.34, 0.90),
        (5.05, 0.38, 0.88),
    ]:
        start = (0.36 * math.cos(angle), 0.36 * math.sin(angle), z_start)
        end = (reach * math.cos(angle + 0.10), reach * math.sin(angle + 0.10), -0.06)
        model.add(veg.limb(start, end, 0.14, 0.07, "swamp_bark", segs=4))

    # Swamp moss clumps on the low broken rim, root junction and shady flank (20 + 20 + 12 = 52 tris)
    model.add(common.make_icosphere(
        subdiv=1, jitter=0.18, seed=41, radius=(0.24, 0.20, 0.11),
        loc=(0.30 * math.cos(2.75), 0.30 * math.sin(2.75), 0.46), color="swamp_moss",
    ))
    model.add(common.make_icosphere(
        subdiv=1, jitter=0.18, seed=42, radius=(0.25, 0.20, 0.12),
        loc=(0.48 * math.cos(5.00), 0.48 * math.sin(5.00), 0.14), color="swamp_moss",
    ))
    moss_strip = common.make_box((0.08, 0.22, 0.36), color="swamp_moss")
    common.transform(moss_strip, rot=(0.0, 0.0, 1.20),
                     loc=(0.41 * math.cos(1.20), 0.41 * math.sin(1.20), 0.34))
    model.add(moss_strip)

    model.add_collider("body", common.make_frustum(0.52, 0.40, -0.05, 0.72, segments=6, color="swamp_bark"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=(0.0, 0.0, 0.0)) else 1


if __name__ == "__main__":
    sys.exit(main())
