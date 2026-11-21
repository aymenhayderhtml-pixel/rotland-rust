"""base_barricade: 2 m wide makeshift survivor barricade with planks, tyres, and a door piece (<= 500 tris). Has a collider."""
import math
import os
import sys

import bmesh

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402

NAME, CATEGORY, BUDGET = "base_barricade", "building_parts", 500

WIDTH = 2.00
HEIGHT = 1.20


def _make_hollow_tyre(r_out=0.28, r_in=0.14, thick=0.18, segments=8, color="tire_black"):
    """Makes a low-poly hollow car tyre with a visible hole in the center."""
    bm = common.new_bm()
    layer = bm.faces.layers.int[common.LAYER]
    key = common.palette.key_of(color)

    z_top = thick / 2
    z_bot = -thick / 2

    v_out_top = []
    v_out_bot = []
    v_in_top = []
    v_in_bot = []
    for i in range(segments):
        a = 2 * math.pi * i / segments
        ca = math.cos(a)
        sa = math.sin(a)
        v_out_top.append(bm.verts.new((ca * r_out, sa * r_out, z_top)))
        v_out_bot.append(bm.verts.new((ca * r_out, sa * r_out, z_bot)))
        v_in_top.append(bm.verts.new((ca * r_in, sa * r_in, z_top)))
        v_in_bot.append(bm.verts.new((ca * r_in, sa * r_in, z_bot)))

    for i in range(segments):
        j = (i + 1) % segments
        # Outer tread quad
        f_out = bm.faces.new([v_out_bot[i], v_out_bot[j], v_out_top[j], v_out_top[i]])
        f_out[layer] = key
        # Inner hole quad
        f_in = bm.faces.new([v_in_top[i], v_in_top[j], v_in_bot[j], v_in_bot[i]])
        f_in[layer] = key
        # Top sidewall quad
        f_top = bm.faces.new([v_out_top[i], v_out_top[j], v_in_top[j], v_in_top[i]])
        f_top[layer] = key
        # Bottom sidewall quad
        f_bot = bm.faces.new([v_out_bot[j], v_out_bot[i], v_in_bot[i], v_in_bot[j]])
        f_bot[layer] = key

    bmesh.ops.recalc_face_normals(bm, faces=list(bm.faces))
    return bm


def build(model):
    parts = []

    # 1. Two heavy vertical wooden posts (X = +/-0.85, Z = 0.0 to 1.20m)
    post_x = 0.85
    post_thick = 0.10
    for sx in (-1, 1):
        parts.append(common.make_box((post_thick, post_thick, HEIGHT),
                                     loc=(sx * post_x, 0.0, HEIGHT / 2),
                                     color="bark_dark"))

        # Rear angled kickstand / brace leg (from Z = 0.90 down to ground Y = +0.44)
        brace = common.make_box((0.07, 0.07, 0.68), color="bark_dark")
        common.transform(brace, rot=(math.radians(35), 0.0, 0.0),
                         loc=(sx * post_x, 0.22, 0.46))
        parts.append(brace)

    # 2. Horizontal framing cross-rails connecting the two vertical posts
    # Bottom rail at Z = 0.24
    parts.append(common.make_box((WIDTH - 0.15, 0.05, 0.08),
                                 loc=(0.0, 0.0, 0.24),
                                 color="bark_dark"))
    # Top rail at Z = 0.94
    parts.append(common.make_box((WIDTH - 0.15, 0.05, 0.08),
                                 loc=(0.0, 0.0, 0.94),
                                 color="bark_dark"))

    # 3. Old residential door piece (salvaged, covering the right half from X = -0.04 to X = 0.88)
    door_w = 0.90
    door_h = 1.14
    door_x = 0.41
    door_y = -0.065
    door_z = door_h / 2
    # Main red wooden door slab
    parts.append(common.make_box((door_w, 0.035, door_h),
                                 loc=(door_x, door_y, door_z),
                                 color="red_paint"))

    # Two recessed panel indentations on the door
    for pz in (0.32, 0.80):
        parts.append(common.make_box((0.36, 0.015, 0.38),
                                     loc=(door_x, door_y - 0.018, pz),
                                     color="bark_dark"))

    # Brass doorknob and plate
    parts.append(common.make_box((0.04, 0.02, 0.08),
                                 loc=(0.02, door_y - 0.02, 0.58),
                                 color="brass"))
    parts.append(common.make_frustum(0.025, 0.025, 0.0, 0.02, segments=6,
                                     center=(0.02, door_y - 0.03), color="brass"))

    # 4. Rough wooden planks nailed across the left side and overlapping the door
    # Plank 1: Bottom kicker plank (Z = 0.10)
    parts.append(common.make_box((0.95, 0.03, 0.14),
                                 loc=(-0.52, -0.08, 0.10),
                                 color="wood_crate"))

    # Plank 2: Lower horizontal plank bridging left post and door (Z = 0.44)
    parts.append(common.make_box((1.15, 0.03, 0.15),
                                 loc=(-0.40, -0.085, 0.44),
                                 color="wood_crate"))

    # Plank 3: Upper horizontal plank (Z = 0.80)
    parts.append(common.make_box((1.25, 0.03, 0.14),
                                 loc=(-0.35, -0.09, 0.80),
                                 color="bark_light"))

    # Plank 4: Diagonal cross brace plank across the left side
    diag_len = 1.10
    diag = common.make_box((diag_len, 0.03, 0.12), color="wood_crate")
    common.transform(diag, rot=(0.0, math.radians(-42), 0.0),
                     loc=(-0.48, -0.10, 0.60))
    parts.append(diag)

    # Plank 5: Jagged top board along upper edge (Z = 1.16)
    parts.append(common.make_box((1.65, 0.03, 0.09),
                                 loc=(0.12, -0.08, 1.16),
                                 color="bark_dark"))

    # 5. Tyres stacked and leaning at the front left
    # Tyre 1: flat on ground at bottom-left front
    tyre1 = _make_hollow_tyre(r_out=0.28, r_in=0.14, thick=0.18, segments=8)
    common.transform(tyre1, loc=(-0.55, -0.22, 0.09))
    parts.append(tyre1)

    # Tyre 2: leaning against the left post and bottom tyre
    tyre2 = _make_hollow_tyre(r_out=0.28, r_in=0.14, thick=0.18, segments=8)
    common.transform(tyre2, rot=(math.radians(35), math.radians(20), 0.0),
                     loc=(-0.72, -0.14, 0.36))
    parts.append(tyre2)

    # 6. Rusty metal reinforcement plate nailed across the center seam
    parts.append(common.make_box((0.20, 0.015, 0.12),
                                 loc=(-0.08, -0.11, 0.62),
                                 color="rust"))
    # Metal nail / bolt studs
    for nx, nz in ((-0.85, 0.80), (-0.85, 0.44), (-0.05, 0.80), (-0.05, 0.44)):
        parts.append(common.make_box((0.025, 0.02, 0.025),
                                     loc=(nx, -0.105, nz),
                                     color="metal"))

    model.add(common.combine(parts))

    # Convex collision box covering the barricade
    model.add_collider("barrier", common.make_box((WIDTH, 0.65, HEIGHT),
                                                  loc=(0.0, 0.05, HEIGHT / 2),
                                                  color="wood_crate"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
