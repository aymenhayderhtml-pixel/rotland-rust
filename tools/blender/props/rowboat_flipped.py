"""rowboat_flipped: upside-down wooden rowboat on the beach (<= 400 tris). Has a collider."""
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import bmesh  # noqa: E402
from mathutils import Vector  # noqa: E402
import common  # noqa: E402

NAME, CATEGORY, BUDGET = "rowboat_flipped", "props", 400


def _build_flipped_hull():
    """Builds a continuous low-poly inverted hull resting gunwales down on sand."""
    bm = common.new_bm()

    # Stations: (y, keel_z, chine_x, chine_z, gunwale_x, gunwale_z)
    # Gunwales at ground level (Z ~0.04), Keel at top (Z ~0.50)
    stations = [
        (-1.70, 0.46, 0.42, 0.42, 0.54, 0.04),  # Transom stern
        (-0.60, 0.50, 0.54, 0.44, 0.65, 0.04),  # Aft mid
        (0.60, 0.50, 0.50, 0.44, 0.62, 0.04),   # Fore mid
        (1.65, 0.42, 0.00, 0.38, 0.00, 0.04),   # Bow stem
    ]

    v_keel = []
    v_chine_l = []
    v_chine_r = []
    v_gun_l = []
    v_gun_r = []

    for y, kz, cx, cz, gx, gz in stations:
        v_keel.append(bm.verts.new(Vector((0.0, y, kz))))
        v_chine_l.append(bm.verts.new(Vector((-cx, y, cz))))
        v_chine_r.append(bm.verts.new(Vector((cx, y, cz))))
        v_gun_l.append(bm.verts.new(Vector((-gx, y, gz))))
        v_gun_r.append(bm.verts.new(Vector((gx, y, gz))))

    # Bottom quads/tris (keel to chine)
    for i in range(2):
        face = bm.faces.new([v_keel[i], v_chine_l[i], v_chine_l[i + 1], v_keel[i + 1]])
        common.paint_faces(bm, [face], "wood_crate")
    face = bm.faces.new([v_keel[2], v_chine_l[2], v_keel[3]])
    common.paint_faces(bm, [face], "wood_crate")

    for i in range(2):
        face = bm.faces.new([v_keel[i], v_keel[i + 1], v_chine_r[i + 1], v_chine_r[i]])
        common.paint_faces(bm, [face], "wood_crate")
    face = bm.faces.new([v_keel[2], v_keel[3], v_chine_r[2]])
    common.paint_faces(bm, [face], "wood_crate")

    # Topsides (chine to gunwales)
    for i in range(2):
        face = bm.faces.new([v_chine_l[i], v_gun_l[i], v_gun_l[i + 1], v_chine_l[i + 1]])
        common.paint_faces(bm, [face], "bark_dark")
    face = bm.faces.new([v_chine_l[2], v_gun_l[2], v_gun_l[3], v_keel[3]])
    common.paint_faces(bm, [face], "bark_dark")

    for i in range(2):
        face = bm.faces.new([v_chine_r[i], v_chine_r[i + 1], v_gun_r[i + 1], v_gun_r[i]])
        common.paint_faces(bm, [face], "bark_dark")
    face = bm.faces.new([v_chine_r[2], v_keel[3], v_gun_r[3], v_gun_r[2]])
    common.paint_faces(bm, [face], "bark_dark")

    # Transom at stern (-Y)
    face_transom = bm.faces.new([v_keel[0], v_chine_r[0], v_gun_r[0], v_gun_l[0], v_chine_l[0]])
    common.paint_faces(bm, [face_transom], "bark_dark")

    bmesh.ops.recalc_face_normals(bm, faces=list(bm.faces))
    return bm


def build(model):
    hull = _build_flipped_hull()
    # Tilt slightly as it rests on uneven beach sand
    common.transform(hull, rot=(0.04, 0.05, 0.0))
    model.add(hull)

    # Inverted keel strip along top ridge
    keel = common.make_box((0.08, 3.40, 0.08), loc=(0.0, -0.05, 0.50), color="bark_dark")
    common.transform(keel, rot=(0.04, 0.05, 0.0))
    model.add(keel)

    # Inverted stem post at bow (+Y)
    stem = common.make_box((0.08, 0.12, 0.52), loc=(0.0, 1.66, 0.26), color="bark_dark")
    common.transform(stem, rot=(0.04, 0.05, 0.0))
    model.add(stem)

    # Piece of driftwood log propping up the boat on one side
    log = common.make_frustum(0.08, 0.06, -0.35, 0.35, segments=5, color="bark_dark")
    common.transform(log, rot=(0.2, 0.3, 0.4), loc=(0.58, -1.3, 0.10))
    model.add(log)

    # Weathered mooring rope trailing into the sand
    rope = common.make_box((0.04, 0.70, 0.03), loc=(0.18, 1.85, 0.02), color="wheat_gold")
    common.transform(rope, rot=(0.0, 0.0, 0.40))
    model.add(rope)

    # Collider
    model.add_collider("hull", common.make_box((1.35, 3.65, 0.56), loc=(0.0, 0.0, 0.28), color="white"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
