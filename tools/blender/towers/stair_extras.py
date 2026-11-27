"""R43 stairwell extras: floor digits 0-9, ROOF signs, roof ladder segment, roof hatch."""
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, os.path.join(HERE, "..", "buildings"))
sys.path.insert(0, os.path.join(HERE, "..", "metro"))
sys.path.insert(0, os.path.join(HERE, "..", "traces"))

import common  # noqa: E402
import graffiti_font  # noqa: E402
import kit  # noqa: E402

# Unit-cell stroke digits (0..9), same convention as graffiti_font.FONT
DIGITS = {
    "0": [(0, 0, 1, 0), (1, 0, 1, 1), (1, 1, 0, 1), (0, 1, 0, 0)],
    "1": [(0.5, 0, 0.5, 1), (0.25, 0.8, 0.5, 1), (0.2, 0, 0.8, 0)],
    "2": [(0, 1, 1, 1), (1, 1, 1, 0.5), (1, 0.5, 0, 0.5), (0, 0.5, 0, 0), (0, 0, 1, 0)],
    "3": [(0, 1, 1, 1), (1, 1, 1, 0), (0.2, 0.5, 1, 0.5), (0, 0, 1, 0)],
    "4": [(0, 1, 0, 0.45), (0, 0.45, 1, 0.45), (1, 1, 1, 0)],
    "5": [(1, 1, 0, 1), (0, 1, 0, 0.55), (0, 0.55, 1, 0.55), (1, 0.55, 1, 0), (1, 0, 0, 0)],
    "6": [(1, 1, 0, 1), (0, 1, 0, 0), (0, 0, 1, 0), (1, 0, 1, 0.5), (1, 0.5, 0, 0.5)],
    "7": [(0, 1, 1, 1), (1, 1, 0.35, 0)],
    "8": [(0, 0, 1, 0), (1, 0, 1, 1), (1, 1, 0, 1), (0, 1, 0, 0), (0, 0.5, 1, 0.5)],
    "9": [(0, 0, 1, 0), (1, 0, 1, 1), (1, 1, 0, 1), (0, 1, 0, 0.5), (0, 0.5, 1, 0.5)],
}


def build_digit(model, digit):
    """Stencil-painted digit 0.35 m tall on a flat 0.01 m plate. No collider; <100 tris."""
    model.add(common.make_box((0.24, 0.01, 0.35), loc=(0.0, 0.0, 0.175), color="concrete_stain"))
    w, h = 0.15, 0.25
    thick = 0.032
    for u0, v0, u1, v1 in DIGITS[digit]:
        x0 = -(u0 - 0.5) * w
        x1 = -(u1 - 0.5) * w
        z0 = 0.05 + v0 * h
        z1 = 0.05 + v1 * h
        model.add(graffiti_font.quad_bar(x0, z0, x1, z1, 0.007, thick, "white"))


def build_sign_roof(model):
    """Plywood board on two legs, hand-painted 'ROOF' with a big arrow up, 1.3 m tall."""
    # Two wooden legs + cross brace
    for sx in (-0.28, 0.28):
        model.add(common.make_box((0.06, 0.06, 1.25), loc=(sx, 0.0, 0.625), color="bark_dark"))
    model.add(common.make_box((0.66, 0.04, 0.06), loc=(0.0, 0.03, 0.35), color="bark_dark"))
    # Plywood board (z 0.65..1.30)
    model.add(common.make_box((0.86, 0.03, 0.65), loc=(0.0, 0.04, 0.975), color="wood_crate"))
    # "ROOF" painted in red on +Y face
    for bm in graffiti_font.text_bms("ROOF", 0.0, 0.74, 0.11, 0.18, 0.058, "red_paint", flat=True):
        model.add(bm)
    # Big upward arrow above "ROOF"
    model.add(graffiti_font.quad_bar(0.0, 0.96, 0.0, 1.22, 0.058, 0.045, "red_paint"))
    model.add(graffiti_font.quad_bar(0.0, 1.22, -0.14, 1.09, 0.058, 0.040, "red_paint"))
    model.add(graffiti_font.quad_bar(0.0, 1.22, 0.14, 1.09, 0.058, 0.040, "red_paint"))
    model.add_collider("sign", common.make_box((0.86, 0.12, 1.30), loc=(0.0, 0.02, 0.65), color="white"))


def build_arrow_spray(model):
    """Cheap spray-painted arrow plate for the lobby wall. No collider."""
    model.add(common.make_box((0.40, 0.008, 0.45), loc=(0.0, 0.0, 0.225), color="concrete_stain"))
    model.add(graffiti_font.quad_bar(0.0, 0.06, 0.0, 0.38, 0.006, 0.045, "red_paint"))
    model.add(graffiti_font.quad_bar(0.0, 0.38, -0.13, 0.24, 0.006, 0.040, "red_paint"))
    model.add(graffiti_font.quad_bar(0.0, 0.38, 0.13, 0.24, 0.006, 0.040, "red_paint"))


def build_ladder_seg(model):
    """Steel ladder segment: 1.0 m tall, 0.5 m wide, rungs ~0.33 m, wall brackets. Tiles seamlessly."""
    # Side rails (full 0.0..1.0 m so vertical tiling has no seam)
    for sx in (-0.25, 0.25):
        model.add(common.make_box((0.04, 0.04, 1.00), loc=(sx, 0.0, 0.50), color="metal"))
    # 3 rungs evenly spaced across 1.0 m (0.16, 0.50, 0.83 -> 0.33-0.34 m pitch across seam)
    for rz in (0.16, 0.50, 0.83):
        model.add(common.make_box((0.46, 0.03, 0.03), loc=(0.0, 0.0, rz), color="rust_dark"))
    # Wall brackets extending toward -Y
    for bz in (0.25, 0.75):
        for sx in (-0.25, 0.25):
            model.add(common.make_box((0.035, 0.18, 0.04), loc=(sx, -0.10, bz), color="gun_black"))


def build_roof_hatch(model):
    """Raised hatch curb (1.4 x 1.4 m, 0.5 m tall) with flipped-open lid and dark opening.

    Colliders only on the 4 curb sides, leaving the inner 1.2 x 1.2 m clear.
    """
    k = kit.Kit(model)
    outer = 0.70  # 1.4 x 1.4 m
    inner = 0.60  # 1.2 x 1.2 m clear
    h = 0.50

    # 4 curb walls (visual + collider)
    k.solid((-outer, -outer, 0.0), (outer, -inner, h), "concrete_grey", base="curb_b")
    k.solid((-outer, inner, 0.0), (outer, outer, h), "concrete_grey", base="curb_f")
    k.solid((-outer, -inner, 0.0), (-inner, inner, h), "concrete_grey", base="curb_l")
    k.solid((inner, -inner, 0.0), (outer, inner, h), "concrete_grey", base="curb_r")

    # Metal flashing rim on top of the curb
    k.box((-outer - 0.02, -outer - 0.02, h), (outer + 0.02, -inner + 0.02, h + 0.04), "metal")
    k.box((-outer - 0.02, inner - 0.02, h), (outer + 0.02, outer + 0.02, h + 0.04), "metal")
    k.box((-outer - 0.02, -inner + 0.02, h), (-inner + 0.02, inner - 0.02, h + 0.04), "metal")
    k.box((inner - 0.02, -inner + 0.02, h), (outer + 0.02, inner - 0.02, h + 0.04), "metal")

    # Dark plane just above the deck inside the curb so the opening reads as a deep shaft
    k.box((-inner, -inner, 0.01), (inner, inner, 0.03), "gun_black")

    # Flipped-open steel lid hinged on -Y edge (rotates ~105 deg backward)
    lid = common.make_box((1.38, 1.38, 0.05), color="metal")
    common.transform(lid, loc=(0.0, 0.69, 0.025))
    common.transform(lid, rot=(math.radians(105.0), 0.0, 0.0), loc=(0.0, -outer, h + 0.04))
    model.add(lid)
    # Hinge blocks on -Y edge
    for hx in (-0.42, 0.42):
        k.box((hx - 0.05, -outer - 0.03, h), (hx + 0.05, -outer + 0.03, h + 0.07), "gun_black")


def build_all():
    results = []
    for d in "0123456789":
        name = f"stair_digit_{d}"
        ok = common.run_model(name, "props", 100, lambda m, _d=d: build_digit(m, _d),
                              grip=(0.0, 0.0, 0.0), view=(0.3, 1.0, 0.2))
        results.append((name, ok))

    specs = [
        ("stair_sign_roof", 1500, build_sign_roof, (0.5, 1.0, 0.35)),
        ("stair_arrow_spray", 200, build_arrow_spray, (0.3, 1.0, 0.2)),
        ("roof_ladder_seg", 1000, build_ladder_seg, (0.8, 1.0, 0.4)),
        ("roof_hatch", 2500, build_roof_hatch, (1.0, 0.8, 0.7)),
    ]
    for name, budget, fn, view in specs:
        ok = common.run_model(name, "props", budget, fn, grip=(0.0, 0.0, 0.0), view=view)
        results.append((name, ok))
    return results


if __name__ == "__main__":
    failed = [n for n, ok in build_all() if not ok]
    if failed:
        print("STAIR EXTRAS FAILED:", ", ".join(failed))
        sys.exit(1)
    print("STAIR EXTRAS OK")
