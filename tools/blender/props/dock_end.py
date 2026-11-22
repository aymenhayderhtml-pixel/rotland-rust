"""dock_end: 4 x 4 m dock end piece with a bench and a lamp post (<= 600 tris).

Repeats at the terminus of a 20 m dock. Origin is centered on the deck at z=0,
with posts extending down to z=-3.0 m. Features a weathered bench, nautical lamp post,
and life ring.
"""
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402

NAME, CATEGORY, BUDGET = "dock_end", "props", 600
GRIP = (0.0, 0.0, 0.0)  # origin centered at deck surface


def build(model):
    # Longitudinal stringer beams underneath (along Y)
    for x in (-1.75, 0.0, 1.75):
        beam = common.make_box((0.18, 4.0, 0.16), loc=(x, 0.0, -0.08), color="bark_dark")
        model.add(beam)

    # 8 weathered wooden planks across X (width 3.88 m to fit inside rub rails), spaced along Y (-2.0 to 2.0)
    colors = ("wood_crate", "bark_light", "bark_dark", "wood_crate")
    for i in range(8):
        y = -1.75 + i * 0.50
        col = colors[i % len(colors)]
        x_shift = 0.015 * (1 if i % 2 == 0 else -1)
        z_shift = 0.008 * (1 if i % 3 == 0 else -1)
        plank = common.make_box((3.88, 0.44, 0.08), loc=(x_shift, y, 0.04 + z_shift), color=col)
        model.add(plank)

    # 4 corner pilings (posts) extending down to -3.0 m, top protruding at +0.35 m
    posts_pos = [(-1.75, -1.3), (1.75, -1.3), (-1.75, 1.75), (1.75, 1.75)]
    for px, py in posts_pos:
        top_z = 0.40 if py > 0 else 0.30
        post = common.make_frustum(0.13, 0.13, -3.0, top_z, segments=5, center=(px, py), color="bark_dark")
        model.add(post)

    # Underwater cross-braces
    for py in (-1.3, 1.75):
        brace = common.make_box((3.5, 0.12, 0.12), loc=(0.0, py, -1.6), color="bark_dark")
        model.add(brace)

    # Side and end rub-rails (taller curb profile, top at z=0.11 proud of planks; end bumper trimmed to fit inside side rails)
    for x_edge in (-1.99, 1.99):
        rail = common.make_box((0.06, 3.98, 0.16), loc=(x_edge, 0.0, 0.03), color="bark_dark")
        model.add(rail)
    end_bumper = common.make_box((3.80, 0.08, 0.16), loc=(0.0, 2.00, 0.03), color="bark_dark")
    model.add(end_bumper)

    # Weathered wooden bench on the left (X=-1.2, Y=0.2, facing +X)
    # Bench legs
    for by in (-0.35, 0.75):
        leg_f = common.make_box((0.07, 0.07, 0.40), loc=(-1.42, by, 0.24), color="bark_dark")
        leg_b = common.make_box((0.07, 0.07, 0.85), loc=(-0.98, by, 0.47), color="bark_dark")
        # Brace y-thickness 0.04 to give 15 mm clearance from leg faces
        brace_b = common.make_box((0.44, 0.04, 0.06), loc=(-1.20, by, 0.43), color="bark_dark")
        model.add(leg_f)
        model.add(leg_b)
        model.add(brace_b)
    # Bench seat slats (2 wide wooden planks)
    for bx in (-1.35, -1.12):
        slat = common.make_box((0.18, 1.30, 0.04), loc=(bx, 0.20, 0.48), color="wood_crate")
        model.add(slat)
    # Bench backrest slat (offset proud of back leg front face at x=-0.945)
    back_slat = common.make_box((0.03, 1.30, 0.16), loc=(-0.92, 0.20, 0.76), color="bark_light")
    model.add(back_slat)

    # Lamp post on the front right (X=1.4, Y=1.2)
    lamp_base = common.make_box((0.24, 0.24, 0.12), loc=(1.4, 1.2, 0.10), color="metal")
    model.add(lamp_base)
    lamp_pole = common.make_box((0.08, 0.08, 2.60), loc=(1.4, 1.2, 1.36), color="metal")
    model.add(lamp_pole)

    # Horizontal arm extending inwards towards X=1.0
    arm = common.make_box((0.42, 0.06, 0.06), loc=(1.18, 1.2, 2.62), color="metal")
    model.add(arm)

    # Lantern housing
    lantern_cap = common.make_frustum(0.12, 0.02, 2.70, 2.82, segments=4, center=(1.0, 1.2), rot_z=0.785, color="metal")
    lantern_glass = common.make_box((0.16, 0.16, 0.18), loc=(1.0, 1.2, 2.58), color="wheat_gold")
    lantern_base = common.make_box((0.18, 0.18, 0.04), loc=(1.0, 1.2, 2.47), color="metal")
    model.add(lantern_cap)
    model.add(lantern_glass)
    model.add(lantern_base)

    # Life ring hanging on the lamp post (facing -X)
    ring_outer = common.make_frustum(0.22, 0.22, -0.04, 0.04, segments=5, color="red_paint")
    common.transform(ring_outer, rot=(0.0, 1.57, 0.0), loc=(1.34, 1.2, 1.6))
    ring_white = common.make_frustum(0.23, 0.23, -0.02, 0.02, segments=4, rot_z=0.39, color="white")
    common.transform(ring_white, rot=(0.0, 1.57, 0.0), loc=(1.34, 1.2, 1.6))
    model.add(ring_outer)
    model.add(ring_white)

    # Mooring bollards on dock edges (low-poly 4-sided boxes)
    for bx, by in [(-1.8, 1.2), (1.8, -0.5)]:
        bollard = common.make_box((0.10, 0.10, 0.20), loc=(bx, by, 0.14), color="metal")
        crossbar = common.make_box((0.04, 0.22, 0.04), loc=(bx, by, 0.20), color="metal")
        model.add(bollard)
        model.add(crossbar)

    # Colliders
    model.add_collider("deck", common.make_box((4.0, 4.0, 0.24), loc=(0.0, 0.0, 0.0), color="white"))
    model.add_collider("bench", common.make_box((0.65, 1.40, 0.85), loc=(-1.20, 0.20, 0.46), color="white"))
    model.add_collider("lamp", common.make_box((0.28, 0.28, 2.70), loc=(1.40, 1.20, 1.35), color="white"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=GRIP) else 1


if __name__ == "__main__":
    sys.exit(main())
