"""camp_gate_metal: 6 m wide opening corrugated metal gate between two steel towers.

Features:
- 6 m clear opening between inner faces of towers (-3.0 to +3.0).
- Left tower (x: -4.0 to -3.0) and right tower (x: +3.0 to +4.0) matching camp_wall_metal.
- Overhead steel truss gantry at z = 3.90 m.
- Separate child meshes `gate_leaf_l` and `gate_leaf_r` (can swing later).
- `door_1` empty in the middle at (0.0, 0.0, 0.0).
- Budget: 2500 triangles.
"""
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, HERE)

import common  # noqa: E402
import kit  # noqa: E402

NAME, CATEGORY, BUDGET = "camp_gate_metal", "buildings", 2500
OPENING_W = 6.0
HALF_OPENING = OPENING_W / 2.0  # 3.0 m
TOWER_W = 1.0
TOWER_D = 0.90
TOWER_H = 4.0
GATE_H = 3.0


def _make_metal_gate_leaf(side="l"):
    """Build one 3.0 m wide, 3.0 m tall corrugated steel gate leaf with frame, patches, and barbed wire."""
    bm_leaf = common.new_bm()
    x_start = -HALF_OPENING if side == "l" else 0.0
    x_end = 0.0 if side == "l" else HALF_OPENING
    cen_x = (x_start + x_end) / 2.0

    # 1. Outer steel perimeter frame (tubular square steel 0.08 x 0.08)
    tf = 0.04
    # Bottom rail
    r_bot = common.make_box((HALF_OPENING, 0.08, 0.08), loc=(cen_x, 0.0, 0.08), color="gun_black")
    # Top rail
    r_top = common.make_box((HALF_OPENING, 0.08, 0.08), loc=(cen_x, 0.0, GATE_H - 0.04), color="gun_black")
    # Side rails
    s_hinge_x = x_start + tf if side == "l" else x_end - tf
    s_meet_x = x_end - tf if side == "l" else x_start + tf
    s1 = common.make_box((0.08, 0.08, GATE_H), loc=(s_hinge_x, 0.0, GATE_H / 2.0), color="gun_black")
    s2 = common.make_box((0.08, 0.08, GATE_H), loc=(s_meet_x, 0.0, GATE_H / 2.0), color="gun_black")
    # Mid-height horizontal girt
    r_mid = common.make_box((HALF_OPENING, 0.06, 0.06), loc=(cen_x, 0.0, GATE_H / 2.0), color="gun_black")
    bm_leaf = common.combine([bm_leaf, r_bot, r_top, s1, s2, r_mid])

    # 2. Corrugated / ribbed sheeting panels inside frame
    # 3 vertical panels per leaf (each 0.95 m wide)
    panel_w = (HALF_OPENING - 0.16) / 3.0
    for p_i in range(3):
        px0 = x_start + 0.08 + p_i * panel_w
        px1 = px0 + panel_w
        p_cen = (px0 + px1) / 2.0
        # Main sheet
        sheet = common.make_box((panel_w - 0.01, 0.02, GATE_H - 0.16),
                                loc=(p_cen, 0.01, GATE_H / 2.0),
                                color="metal")
        bm_leaf = common.combine([bm_leaf, sheet])

        # Vertical ribs on sheet
        for rib_i in range(3):
            rx = px0 + (rib_i + 0.5) * (panel_w / 3.0)
            rib = common.make_box((0.03, 0.02, GATE_H - 0.16),
                                  loc=(rx, 0.022, GATE_H / 2.0),
                                  color="rock_mid")
            bm_leaf = common.combine([bm_leaf, rib])

    # 3. Rusty patches and scrap plate repairs
    patch1_x = cen_x - 0.40 if side == "l" else cen_x + 0.40
    p1 = common.make_box((0.70, 0.02, 0.80), loc=(patch1_x, 0.025, 0.90), color="rust")
    patch2_x = cen_x + 0.35 if side == "l" else cen_x - 0.35
    p2 = common.make_box((0.60, 0.02, 0.70), loc=(patch2_x, 0.025, 2.10), color="red_paint")
    bm_leaf = common.combine([bm_leaf, p1, p2])

    # 4. Barbed top wire extension
    # Outrigger arms at hinge, middle, and meet side
    for ax in (s_hinge_x, cen_x, s_meet_x):
        arm = common.make_box((0.03, 0.10, 0.35), loc=(ax, 0.05, GATE_H + 0.15), color="gun_black")
        bm_leaf = common.combine([bm_leaf, arm])

    # 2 wire strands across leaf top
    for wz in (GATE_H + 0.15, GATE_H + 0.30):
        wire = common.make_box((HALF_OPENING, 0.012, 0.012), loc=(cen_x, 0.08, wz), color="metal")
        bm_leaf = common.combine([bm_leaf, wire])
        # Barbs every 0.6 m
        for b_i in range(5):
            bx = x_start + 0.30 + b_i * 0.60
            barb = common.make_box((0.03, 0.04, 0.04), loc=(bx, 0.08, wz), color="gun_black")
            bm_leaf = common.combine([bm_leaf, barb])

    # 5. Heavy steel hinge barrels on hinge side
    for hz in (0.60, 2.40):
        hinge_barrel = common.make_box((0.12, 0.12, 0.22), loc=(s_hinge_x, -0.02, hz), color="gun_black")
        hinge_strap = common.make_box((0.50, 0.02, 0.08),
                                      loc=(s_hinge_x + (0.25 if side == "l" else -0.25), 0.04, hz),
                                      color="rust")
        bm_leaf = common.combine([bm_leaf, hinge_barrel, hinge_strap])

    # 6. Locking bar mechanism at meeting edge
    latch_x = -0.15 if side == "l" else 0.15
    latch = common.make_box((0.30, 0.06, 0.10), loc=(latch_x, 0.04, 1.40), color="gun_black")
    bm_leaf = common.combine([bm_leaf, latch])

    return bm_leaf


def _build_metal_tower(k, model, x_lo, x_hi):
    """Build one industrial steel truss/column tower."""
    cx = (x_lo + x_hi) / 2.0
    half_w = (x_hi - x_lo) / 2.0
    half_d = TOWER_D / 2.0

    # 4 vertical steel square columns
    for px in (cx - half_w + 0.08, cx + half_w - 0.08):
        for py in (-half_d + 0.08, half_d - 0.08):
            k.box((px - 0.06, py - 0.06, 0.0), (px + 0.06, py + 0.06, TOWER_H), "gun_black")
            k.box((px - 0.10, py - 0.10, 0.0), (px + 0.10, py + 0.10, 0.04), "rock_dark")  # base plate

    # Horizontal girts on all four sides at z = 1.0, 2.2, 3.4
    for gz in (1.0, 2.2, 3.4):
        # Front and back
        k.box((x_lo, -half_d + 0.04, gz), (x_hi, -half_d + 0.10, gz + 0.08), "gun_black")
        k.box((x_lo, half_d - 0.10, gz), (x_hi, half_d - 0.04, gz + 0.08), "gun_black")
        # Outer side
        outer_x = x_lo if x_lo < 0 else x_hi - 0.10
        k.box((outer_x, -half_d, gz), (outer_x + 0.08, half_d, gz + 0.08), "gun_black")

    # Corrugated sheet cladding on outer side and back
    outer_wall_x = x_lo if x_lo < 0 else x_hi - 0.05
    k.box((outer_wall_x, -half_d + 0.08, 0.05), (outer_wall_x + 0.04, half_d - 0.08, 3.4), "metal")
    # Rust patch on outer wall
    k.box((outer_wall_x - 0.01, -0.20, 1.2), (outer_wall_x + 0.05, 0.25, 2.2), "rust")

    # Top platform at z = 3.4 m
    k.box((x_lo, -half_d, 3.40), (x_hi, half_d, 3.48), "rock_dark")
    # Handrails around top
    k.box((x_lo, -half_d, 3.48), (x_hi, -half_d + 0.05, TOWER_H), "gun_black")
    k.box((x_lo, half_d - 0.05, 3.48), (x_hi, half_d, TOWER_H), "gun_black")
    k.box((outer_x, -half_d, 3.48), (outer_x + 0.05, half_d, TOWER_H), "gun_black")

    # Barbed wire outriggers on tower top
    for px in (cx - half_w + 0.08, cx + half_w - 0.08):
        k.box((px - 0.02, half_d - 0.05, TOWER_H), (px + 0.02, half_d + 0.15, TOWER_H + 0.35), "gun_black")


def build(model):
    k = kit.Kit(model)

    # 1. Left Tower (x: -4.0 to -3.0)
    _build_metal_tower(k, model, -HALF_OPENING - TOWER_W, -HALF_OPENING)

    # 2. Right Tower (x: +3.0 to +4.0)
    _build_metal_tower(k, model, HALF_OPENING, HALF_OPENING + TOWER_W)

    # 3. Overhead steel truss gantry spanning between towers (z = 3.90 to 4.25)
    span_w = OPENING_W + 2 * TOWER_W
    # Top and bottom chords
    k.box((-HALF_OPENING - TOWER_W, -0.06, 3.90), (HALF_OPENING + TOWER_W, 0.06, 3.98), "gun_black")
    k.box((-HALF_OPENING - TOWER_W, -0.06, 4.18), (HALF_OPENING + TOWER_W, 0.06, 4.26), "gun_black")
    # Diagonal truss web members
    num_web = 12
    step_w = span_w / num_web
    for w_i in range(num_web):
        wx = -HALF_OPENING - TOWER_W + (w_i + 0.5) * step_w
        k.box((wx - 0.03, -0.04, 3.98), (wx + 0.03, 0.04, 4.18), "metal")

    # Center warning hazard stripe plate on gantry
    k.box((-1.0, 0.065, 3.96), (1.0, 0.08, 4.20), "red_paint")
    k.box((-0.6, 0.08, 3.98), (-0.2, 0.09, 4.18), "white")
    k.box((0.2, 0.08, 3.98), (0.6, 0.09, 4.18), "white")

    # 4. Gate leaves as separate child meshes
    leaf_l = _make_metal_gate_leaf(side="l")
    leaf_r = _make_metal_gate_leaf(side="r")
    model.add_part("gate_leaf_l", leaf_l)
    model.add_part("gate_leaf_r", leaf_r)

    # 5. door_1 empty in the middle of opening
    k.empty("door_1", (0.0, 0.0, 0.0))

    # 6. Colliders on towers only
    k.collider((-HALF_OPENING - TOWER_W, -TOWER_D / 2.0, 0.0),
               (-HALF_OPENING, TOWER_D / 2.0, TOWER_H), base="tower_l")
    k.collider((HALF_OPENING, -TOWER_D / 2.0, 0.0),
               (HALF_OPENING + TOWER_W, TOWER_D / 2.0, TOWER_H), base="tower_r")


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, view=kit.FRONT_VIEW) else 1


if __name__ == "__main__":
    sys.exit(main())
