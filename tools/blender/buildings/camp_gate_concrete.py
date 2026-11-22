"""camp_gate_concrete: 6 m wide opening concrete pier gate with heavy blast doors.

Features:
- 6 m clear opening between inner faces of concrete piers (-3.0 to +3.0).
- Left pier (x: -4.2 to -3.0) and right pier (x: +3.0 to +4.2) matching camp_wall_concrete.
- Concrete blocks with chamfered joints, settling lean, and painted hazard stripe.
- Rebar showing at broken pier top corner.
- Heavy industrial steel blast-gate leaves (`gate_leaf_l` and `gate_leaf_r`) with viewing slits and top spikes.
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

NAME, CATEGORY, BUDGET = "camp_gate_concrete", "buildings", 2500
OPENING_W = 6.0
HALF_OPENING = OPENING_W / 2.0  # 3.0 m
PIER_W = 1.20
PIER_D = 0.90
PIER_H = 4.0
GATE_H = 3.2


def _make_concrete_gate_leaf(side="l"):
    """Build one ~2.95 m wide, 3.2 m tall heavy industrial steel blast gate leaf."""
    bm_leaf = common.new_bm()
    x_start = -HALF_OPENING + 0.04 if side == "l" else 0.02
    x_end = -0.02 if side == "l" else HALF_OPENING - 0.04
    leaf_w = x_end - x_start
    cen_x = (x_start + x_end) / 2.0

    # 1. Main armored blast-plate slab (fits inside frame channels)
    slab = common.make_box((leaf_w - 0.20, 0.08, GATE_H - 0.20),
                           loc=(cen_x, 0.0, GATE_H / 2.0),
                           color="metal")
    bm_leaf = common.combine([bm_leaf, slab])

    # 2. Outer structural steel frame channel (top, bottom, and vertical edges)
    # Top channel
    f_top = common.make_box((leaf_w, 0.12, 0.08), loc=(cen_x, 0.0, GATE_H - 0.04), color="gun_black")
    # Bottom channel
    f_bot = common.make_box((leaf_w, 0.12, 0.08), loc=(cen_x, 0.0, 0.04), color="gun_black")
    # Hinge-side vertical channel (fits between top and bottom channels)
    s_hinge_x = x_start + 0.05 if side == "l" else x_end - 0.05
    f_hinge = common.make_box((0.10, 0.12, GATE_H - 0.16), loc=(s_hinge_x, 0.0, GATE_H / 2.0), color="gun_black")
    # Meeting-side vertical channel (fits between top and bottom channels)
    s_meet_x = x_end - 0.05 if side == "l" else x_start + 0.05
    f_meet = common.make_box((0.10, 0.12, GATE_H - 0.16), loc=(s_meet_x, 0.0, GATE_H / 2.0), color="gun_black")
    bm_leaf = common.combine([bm_leaf, f_top, f_bot, f_hinge, f_meet])

    # 3. Diagonal stiffener ribs on front face
    rib_d = common.make_box((leaf_w * 0.85, 0.04, 0.10),
                            loc=(cen_x, 0.065, GATE_H * 0.35),
                            color="gun_black")
    rib_d2 = common.make_box((leaf_w * 0.85, 0.04, 0.10),
                             loc=(cen_x, 0.065, GATE_H * 0.75),
                             color="gun_black")
    bm_leaf = common.combine([bm_leaf, rib_d, rib_d2])

    # 4. Armored vision port / viewing slit at eye level (z = 1.7 m, proud of frame)
    slit_x = cen_x
    slit_frame = common.make_box((0.80, 0.16, 0.22), loc=(slit_x, 0.0, 1.70), color="gun_black")
    slit_open = common.make_box((0.65, 0.20, 0.12), loc=(slit_x, 0.0, 1.70), color="rock_dark")
    # Iron bars in viewing slit
    bar1 = common.make_box((0.02, 0.22, 0.14), loc=(slit_x - 0.15, 0.0, 1.70), color="metal")
    bar2 = common.make_box((0.02, 0.22, 0.14), loc=(slit_x + 0.15, 0.0, 1.70), color="metal")
    bm_leaf = common.combine([bm_leaf, slit_frame, slit_open, bar1, bar2])

    # 5. Painted hazard stripe across the gate leaf (z = 1.35 to 1.65, aligned with wall & pier)
    num_s = 6
    sw = leaf_w / num_s
    for si in range(num_s):
        sx = x_start + (si + 0.5) * sw
        scol = "red_paint" if (si % 2 == (0 if side == "l" else 1)) else "white"
        stripe = common.make_box((sw - 0.01, 0.02, 0.28), loc=(sx, 0.065, 1.45), color=scol)
        bm_leaf = common.combine([bm_leaf, stripe])

    # 6. Spiked anti-climb teeth along the top rail (z = 3.2 to 3.45)
    num_spikes = 7
    sp_step = leaf_w / num_spikes
    for spi in range(num_spikes):
        spx = x_start + (spi + 0.5) * sp_step
        spike = common.make_box((0.03, 0.03, 0.22), loc=(spx, 0.0, GATE_H + 0.10), color="gun_black")
        bm_leaf = common.combine([bm_leaf, spike])

    # 7. Heavy industrial pivot hinge blocks at hinge side
    for hz in (0.60, 2.60):
        h_block = common.make_box((0.14, 0.16, 0.25), loc=(s_hinge_x, 0.0, hz), color="gun_black")
        h_pin = common.make_box((0.06, 0.06, 0.35), loc=(s_hinge_x, 0.0, hz), color="metal")
        bm_leaf = common.combine([bm_leaf, h_block, h_pin])

    # 8. Heavy central locking bar mechanism at meeting edge
    latch_x = -0.18 if side == "l" else 0.18
    latch_bar = common.make_box((0.35, 0.08, 0.14), loc=(latch_x, 0.065, 1.45), color="gun_black")
    latch_handle = common.make_box((0.06, 0.12, 0.24), loc=(latch_x, 0.105, 1.45), color="rust")
    bm_leaf = common.combine([bm_leaf, latch_bar, latch_handle])

    return bm_leaf


def _build_concrete_pier(k, x_lo, x_hi, broken=False):
    """Build one massive concrete gate pier."""
    cx = (x_lo + x_hi) / 2.0
    half_w = (x_hi - x_lo) / 2.0
    half_d = PIER_D / 2.0

    # Base footing (z = 0.0 to 0.25)
    k.box((x_lo - 0.05, -half_d - 0.05, 0.0), (x_hi + 0.05, half_d + 0.05, 0.25), "rock_dark")

    # 4 tiers of heavy concrete blocks (z: 0.25..1.20, 1.20..2.15, 2.15..3.10, 3.10..4.00)
    tiers = [
        (0.25, 1.20, "rock_mid"),
        (1.20, 2.15, "rock_light"),
        (2.15, 3.10, "rock_mid"),
        (3.10, 4.00, "rock_dark"),
    ]

    for t_i, (z0, z1, col) in enumerate(tiers):
        top_z = z1
        # Broken corner with exposed rebar on Tier 3 (top) if broken=True
        if broken and t_i == 3:
            # Main intact block
            k.box((x_lo + 0.02, -half_d + 0.02, z0), (x_hi - 0.25, half_d - 0.02, 4.00), col)
            # Broken chipped section at outer corner
            k.box((x_hi - 0.25, -half_d + 0.02, z0), (x_hi - 0.02, half_d - 0.02, 3.45), "rock_dark")

            # Exposed twisted steel rebars sticking out of fracture
            # Rebar 1
            k.box((x_hi - 0.20, 0.0, 3.45), (x_hi - 0.16, 0.04, 4.15), "rust")
            k.box((x_hi - 0.20, 0.04, 4.15), (x_hi - 0.14, 0.12, 4.28), "gun_black")
            # Rebar 2
            k.box((x_hi - 0.10, -0.08, 3.45), (x_hi - 0.07, -0.05, 4.05), "rust")
            k.box((x_hi - 0.08, -0.05, 4.05), (x_hi - 0.02, 0.02, 4.20), "gun_black")
            continue

        k.box((x_lo + 0.02, -half_d + 0.02, z0), (x_hi - 0.02, half_d - 0.02, top_z), col)

    # Painted hazard stripe across Tier 1 (z = 1.35 to 1.65) matching wall
    num_sp = 3
    sp_w = (x_hi - x_lo) / num_sp
    for s_i in range(num_sp):
        sx0 = x_lo + s_i * sp_w
        sx1 = sx0 + sp_w
        scol = "red_paint" if (s_i % 2 == (0 if x_lo < 0 else 1)) else "white"
        k.box((sx0 + 0.01, half_d - 0.02, 1.35), (sx1 - 0.01, half_d + 0.01, 1.65), scol)

    # Embedded steel hinge mounting plates facing the 6m opening
    hinge_x = x_hi if x_lo < 0 else x_lo
    for hz in (0.60, 2.60):
        hx0 = hinge_x - 0.06 if x_lo < 0 else hinge_x + 0.01
        hx1 = hinge_x - 0.01 if x_lo < 0 else hinge_x + 0.06
        k.box((hx0, -0.20, hz - 0.15), (hx1, 0.20, hz + 0.15), "gun_black")


def build(model):
    k = kit.Kit(model)

    # 1. Left Concrete Pier (x: -4.2 to -3.0) with broken top rebar
    _build_concrete_pier(k, -HALF_OPENING - PIER_W, -HALF_OPENING, broken=True)

    # 2. Right Concrete Pier (x: +3.0 to +4.2)
    _build_concrete_pier(k, HALF_OPENING, HALF_OPENING + PIER_W, broken=False)

    # 3. Gate leaves as separate child meshes
    leaf_l = _make_concrete_gate_leaf(side="l")
    leaf_r = _make_concrete_gate_leaf(side="r")
    model.add_part("gate_leaf_l", leaf_l)
    model.add_part("gate_leaf_r", leaf_r)

    # 4. door_1 empty in the middle of opening
    k.empty("door_1", (0.0, 0.0, 0.0))

    # 5. Colliders on piers only
    k.collider((-HALF_OPENING - PIER_W, -PIER_D / 2.0, 0.0),
               (-HALF_OPENING, PIER_D / 2.0, PIER_H), base="pier_l")
    k.collider((HALF_OPENING, -PIER_D / 2.0, 0.0),
               (HALF_OPENING + PIER_W, PIER_D / 2.0, PIER_H), base="pier_r")


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, view=kit.FRONT_VIEW) else 1


if __name__ == "__main__":
    sys.exit(main())
