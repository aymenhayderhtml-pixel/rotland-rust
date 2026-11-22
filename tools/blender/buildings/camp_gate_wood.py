"""camp_gate_wood: 6 m wide opening palisade gate between two wooden towers.

Features:
- 6 m clear opening between inner faces of towers (-3.0 to +3.0).
- Left tower (x: -4.0 to -3.0) and right tower (x: +3.0 to +4.0) matching camp_wall_wood.
- Overhead lintel log at z = 3.65 m.
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

NAME, CATEGORY, BUDGET = "camp_gate_wood", "buildings", 2500
OPENING_W = 6.0
HALF_OPENING = OPENING_W / 2.0  # 3.0 m
TOWER_W = 1.0
TOWER_D = 1.0
TOWER_H = 3.8
GATE_H = 2.6
LOG_R = 0.12


def _make_gate_leaf(side="l"):
    """Build one ~2.95 m wide, 2.6 m tall gate leaf of sharpened logs with braces and iron hinges."""
    bm_leaf = common.new_bm()
    x_start = -HALF_OPENING + 0.04 if side == "l" else 0.02
    x_end = -0.02 if side == "l" else HALF_OPENING - 0.04
    leaf_w = x_end - x_start
    num_logs = 12
    step = leaf_w / num_logs
    colors = ["bark_light", "bark_dark"]

    for i in range(num_logs):
        cx = x_start + step * 0.5 + i * step
        cy = 0.0
        h = 2.55 + ((i * 3) % 5) * 0.03  # slight variation
        tip_h = 0.40
        body_h = h - tip_h
        color = colors[i % 2]
        rot = (i * 0.65) % math.pi

        body = common.make_frustum(LOG_R, LOG_R, 0.02, body_h, segments=6,
                                   center=(cx, cy), rot_z=rot, color=color)
        tip = common.make_frustum(LOG_R, 0.0, body_h, h, segments=6,
                                  center=(cx, cy), rot_z=rot, color=color)
        bm_leaf = common.combine([bm_leaf, body, tip])

    # Horizontal cross-timbers on inside face (-Y)
    y_rail0 = -LOG_R - 0.07
    y_rail1 = -LOG_R
    r1 = common.make_box((leaf_w - 0.04, y_rail1 - y_rail0, 0.12),
                          loc=(x_start + leaf_w / 2.0, (y_rail0 + y_rail1) / 2.0, 0.65),
                          color="bark_dark")
    r2 = common.make_box((leaf_w - 0.04, y_rail1 - y_rail0, 0.12),
                          loc=(x_start + leaf_w / 2.0, (y_rail0 + y_rail1) / 2.0, 1.95),
                          color="bark_dark")
    bm_leaf = common.combine([bm_leaf, r1, r2])

    # Diagonal Z-brace timber
    diag_cen_x = x_start + leaf_w / 2.0
    diag = common.make_box((leaf_w * 0.85, 0.06, 0.10),
                           loc=(diag_cen_x, (y_rail0 + y_rail1) / 2.0 - 0.02, 1.30),
                           color="bark_light")
    bm_leaf = common.combine([bm_leaf, diag])

    # Wrought-iron strap hinges on the hinge side
    hinge_x = -HALF_OPENING if side == "l" else HALF_OPENING
    strap_w = 0.80
    strap_cx = hinge_x + (strap_w / 2.0 if side == "l" else -strap_w / 2.0)
    for hz in (0.65, 1.95):
        strap = common.make_box((strap_w, 0.02, 0.08),
                                loc=(strap_cx, LOG_R + 0.03, hz),
                                color="gun_black")
        pin = common.make_box((0.08, 0.08, 0.16),
                              loc=(hinge_x, 0.0, hz),
                              color="gun_black")
        bm_leaf = common.combine([bm_leaf, strap, pin])

    # Center latch / drop bolt
    latch_x = -0.15 if side == "l" else 0.15
    latch = common.make_box((0.25, 0.04, 0.08),
                            loc=(latch_x, -LOG_R - 0.08, 1.30),
                            color="metal")
    bm_leaf = common.combine([bm_leaf, latch])

    return bm_leaf


def _build_tower(k, model, x_lo, x_hi):
    """Build one palisade lookout tower."""
    cx = (x_lo + x_hi) / 2.0
    half_w = (x_hi - x_lo) / 2.0
    half_d = TOWER_D / 2.0

    # 4 corner vertical posts
    corners = [
        (cx - half_w + 0.12, -half_d + 0.12),
        (cx + half_w - 0.12, -half_d + 0.12),
        (cx - half_w + 0.12, half_d - 0.12),
        (cx + half_w - 0.12, half_d - 0.12),
    ]
    for px, py in corners:
        post = common.make_frustum(0.12, 0.12, 0.0, TOWER_H, segments=6,
                                   center=(px, py), color="bark_dark")
        model.add(post)

    # Infill log palisade walls on the outer and back faces (0.03 to 2.68 m)
    for y_wall in (-half_d + 0.12, half_d - 0.12):
        k.box((x_lo + 0.05 if x_lo < 0 else x_lo, y_wall - 0.05, 0.03),
              (x_hi if x_lo < 0 else x_hi - 0.05, y_wall + 0.05, 2.68), "bark_light")
    # Outer side wall (away from opening)
    outer_x = x_lo if x_lo < 0 else x_hi - 0.10
    k.box((outer_x, -half_d + 0.05, 0.03), (outer_x + 0.10, half_d - 0.05, 2.68), "bark_dark")

    # Lookout deck at z = 2.8 m
    k.solid((x_lo + 0.05 if x_lo < 0 else x_lo, -half_d, 2.70),
            (x_hi if x_lo < 0 else x_hi - 0.05, half_d, 2.80), "wood_crate", base="deck")

    # Parapet log railings around deck (z = 2.82 to 3.70, inset to prevent corner coplanar pairs)
    rail_top = 3.70
    k.box((x_lo + 0.05 if x_lo < 0 else x_lo, -half_d, 2.82),
          (x_hi if x_lo < 0 else x_hi - 0.05, -half_d + 0.08, rail_top), "bark_light")
    k.box((x_lo + 0.05 if x_lo < 0 else x_lo, half_d - 0.08, 2.82),
          (x_hi if x_lo < 0 else x_hi - 0.05, half_d, rail_top), "bark_light")
    k.box((outer_x, -half_d + 0.08, 2.82), (outer_x + 0.08, half_d - 0.08, rail_top), "bark_light")

    # Hinge post facing the opening
    inner_post_x = (x_hi - 0.12) if x_lo < 0 else (x_lo + 0.12)
    hinge_post = common.make_frustum(0.15, 0.15, 0.0, TOWER_H, segments=6,
                                     center=(inner_post_x, 0.0), color="bark_dark")
    model.add(hinge_post)


def build(model):
    k = kit.Kit(model)

    # 1. Left Tower (x: -4.0 to -3.0)
    _build_tower(k, model, -HALF_OPENING - TOWER_W, -HALF_OPENING)

    # 2. Right Tower (x: +3.0 to +4.0)
    _build_tower(k, model, HALF_OPENING, HALF_OPENING + TOWER_W)

    # 3. Overhead lintel beam across the 6m opening (z = 3.82 to 4.02 on top of posts)
    # Spans from outer edge of left tower (-4.0) to outer edge of right tower (+4.0)
    k.box((-HALF_OPENING - TOWER_W, -0.12, 3.82),
          (HALF_OPENING + TOWER_W, 0.12, 4.02), "bark_dark")
    # Diagonal knee-braces supporting lintel at opening corners
    k.box((-HALF_OPENING - 0.20, -0.06, 3.30), (-HALF_OPENING + 0.35, 0.06, 3.82), "bark_light")
    k.box((HALF_OPENING - 0.35, -0.06, 3.30), (HALF_OPENING + 0.20, 0.06, 3.82), "bark_light")

    # Decorative wooden camp sign plate on lintel (centered)
    k.box((-1.2, -0.16, 3.82), (1.2, -0.12, 4.15), "wood_crate")
    k.box((-1.1, -0.17, 3.85), (1.1, -0.15, 4.10), "white")

    # 4. Gate leaves as separate child meshes
    leaf_l = _make_gate_leaf(side="l")
    leaf_r = _make_gate_leaf(side="r")
    model.add_part("gate_leaf_l", leaf_l)
    model.add_part("gate_leaf_r", leaf_r)

    # 5. door_1 empty in the middle of opening
    k.empty("door_1", (0.0, 0.0, 0.0))

    # 6. Colliders on towers only (opening is clear)
    k.collider((-HALF_OPENING - TOWER_W, -TOWER_D / 2.0, 0.0),
               (-HALF_OPENING, TOWER_D / 2.0, TOWER_H), base="tower_l")
    k.collider((HALF_OPENING, -TOWER_D / 2.0, 0.0),
               (HALF_OPENING + TOWER_W, TOWER_D / 2.0, TOWER_H), base="tower_r")


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, view=kit.FRONT_VIEW) else 1


if __name__ == "__main__":
    sys.exit(main())
