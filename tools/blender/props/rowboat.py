"""rowboat: upright wooden rowboat with seats and oars (<= 500 tris). Has a collider."""
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import bmesh  # noqa: E402
from mathutils import Vector  # noqa: E402
import common  # noqa: E402

NAME, CATEGORY, BUDGET = "rowboat", "props", 500


def _build_hull():
    """Builds a continuous low-poly clinker/dinghy hull with 4 stations along Y."""
    bm = common.new_bm()

    # Stations: (y, keel_z, chine_x, chine_z, gunwale_x, gunwale_z)
    stations = [
        (-1.70, 0.08, 0.42, 0.12, 0.54, 0.52),  # Transom stern
        (-0.60, 0.04, 0.54, 0.08, 0.65, 0.50),  # Aft mid
        (0.60, 0.04, 0.50, 0.08, 0.62, 0.50),   # Fore mid
        (1.65, 0.14, 0.00, 0.16, 0.00, 0.60),   # Bow stem
    ]

    v_keel = []
    v_chine_l = []
    v_chine_r = []
    v_gun_l = []
    v_gun_r = []

    for i, (y, kz, cx, cz, gx, gz) in enumerate(stations):
        v_keel.append(bm.verts.new(Vector((0.0, y, kz))))
        v_chine_l.append(bm.verts.new(Vector((-cx, y, cz))))
        v_chine_r.append(bm.verts.new(Vector((cx, y, cz))))
        if i == 3:
            # Single shared vertex at bow stem to eliminate hole/split seam
            v_stem = bm.verts.new(Vector((0.0, y, gz)))
            v_gun_l.append(v_stem)
            v_gun_r.append(v_stem)
        else:
            v_gun_l.append(bm.verts.new(Vector((-gx, y, gz))))
            v_gun_r.append(bm.verts.new(Vector((gx, y, gz))))

    # Bottom quads/tris (keel to chine)
    # Port (-X) bottom
    for i in range(2):
        face = bm.faces.new([v_keel[i], v_chine_l[i], v_chine_l[i + 1], v_keel[i + 1]])
        common.paint_faces(bm, [face], "wood_crate")
    # Bow port bottom triangle
    face = bm.faces.new([v_keel[2], v_chine_l[2], v_keel[3]])
    common.paint_faces(bm, [face], "wood_crate")

    # Starboard (+X) bottom
    for i in range(2):
        face = bm.faces.new([v_keel[i], v_keel[i + 1], v_chine_r[i + 1], v_chine_r[i]])
        common.paint_faces(bm, [face], "wood_crate")
    # Bow stbd bottom triangle
    face = bm.faces.new([v_keel[2], v_keel[3], v_chine_r[2]])
    common.paint_faces(bm, [face], "wood_crate")

    # Topside quads/tris (chine to gunwale)
    # Port (-X) topsides
    for i in range(2):
        face = bm.faces.new([v_chine_l[i], v_gun_l[i], v_gun_l[i + 1], v_chine_l[i + 1]])
        common.paint_faces(bm, [face], "bark_light")
    face = bm.faces.new([v_chine_l[2], v_gun_l[2], v_gun_l[3], v_keel[3]])
    common.paint_faces(bm, [face], "bark_light")

    # Starboard (+X) topsides
    for i in range(2):
        face = bm.faces.new([v_chine_r[i], v_chine_r[i + 1], v_gun_r[i + 1], v_gun_r[i]])
        common.paint_faces(bm, [face], "bark_light")
    face = bm.faces.new([v_chine_r[2], v_keel[3], v_gun_r[3], v_gun_r[2]])
    common.paint_faces(bm, [face], "bark_light")

    # Transom stern face at -Y
    face_transom = bm.faces.new([v_keel[0], v_chine_r[0], v_gun_r[0], v_gun_l[0], v_chine_l[0]])
    common.paint_faces(bm, [face_transom], "bark_dark")

    bmesh.ops.recalc_face_normals(bm, faces=list(bm.faces))

    # Solidify hull into closed manifold solid with thickness so inside faces exist and are visible from above
    layer = bm.faces.layers.int[common.LAYER]
    orig_faces = list(bm.faces)
    res = bmesh.ops.solidify(bm, geom=orig_faces, thickness=0.04)
    new_faces = [f for f in res["geom"] if isinstance(f, bmesh.types.BMFace)]
    # Color inner faces to match original shell
    for orig_f, new_f in zip(orig_faces, new_faces):
        new_f[layer] = orig_f[layer]
    # Color rim faces along gunwales and transom
    all_known = set(orig_faces) | set(new_faces)
    rim_faces = [f for f in bm.faces if f not in all_known]
    common.paint_faces(bm, rim_faces, "bark_dark")

    bmesh.ops.recalc_face_normals(bm, faces=list(bm.faces))
    return bm


def build(model):
    model.add(_build_hull())

    # Keel rib along bottom center
    keel = common.make_box((0.08, 3.40, 0.08), loc=(0.0, -0.05, 0.04), color="bark_dark")
    model.add(keel)

    # Gunwale rails along rim
    gw_l = common.make_box((0.06, 2.30, 0.06), loc=(-0.60, -0.55, 0.52), color="bark_dark")
    gw_r = common.make_box((0.06, 2.30, 0.06), loc=(0.60, -0.55, 0.52), color="bark_dark")
    model.add(gw_l)
    model.add(gw_r)

    # Stem post at bow (+Y)
    stem = common.make_box((0.08, 0.12, 0.60), loc=(0.0, 1.66, 0.32), color="bark_dark")
    model.add(stem)

    # Wooden interior seats (thwarts)
    # Stern bench
    seat_stern = common.make_box((0.92, 0.35, 0.04), loc=(0.0, -1.45, 0.36), color="wood_crate")
    # Center rowing thwart
    seat_mid = common.make_box((1.18, 0.30, 0.04), loc=(0.0, 0.0, 0.34), color="wood_crate")
    # Bow triangle seat
    seat_bow = common.make_box((0.50, 0.32, 0.04), loc=(0.0, 1.35, 0.38), color="wood_crate")
    model.add(seat_stern)
    model.add(seat_mid)
    model.add(seat_bow)

    # Floor bottom boards
    floor = common.make_box((0.60, 2.20, 0.03), loc=(0.0, -0.40, 0.12), color="bark_dark")
    model.add(floor)

    # Pair of wooden oars resting inside across seats
    for sign in (-1, 1):
        oar = common.new_bm()
        shaft = common.make_box((0.04, 1.80, 0.04), color="bark_light")
        blade = common.make_box((0.14, 0.45, 0.02), loc=(0.0, -0.80, 0.0), color="bark_light")
        oar = common.combine([shaft, blade])
        common.transform(oar, rot=(0.06 * sign, 0.0, 0.14 * sign), loc=(sign * 0.28, -0.10, 0.40))
        model.add(oar)
        # Oarlock on gunwale
        lock = common.make_box((0.04, 0.06, 0.08), loc=(sign * 0.62, 0.05, 0.54), color="metal")
        model.add(lock)

    # Mooring painter rope coiled at bow
    rope = common.make_frustum(0.14, 0.14, 0.40, 0.44, segments=6, center=(0.0, 1.25), color="wheat_gold")
    model.add(rope)

    # Collider
    model.add_collider("hull", common.make_box((1.35, 3.65, 0.58), loc=(0.0, 0.0, 0.29), color="white"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, top_preview=True) else 1


if __name__ == "__main__":
    sys.exit(main())
