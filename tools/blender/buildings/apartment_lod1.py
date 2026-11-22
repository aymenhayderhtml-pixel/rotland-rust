"""apartment_lod1: low-poly distant LOD for apartment.

Same footprint (18 x 12 m in 20 x 14 m lot), same silhouette (3 floors, parapet, water tank,
simplified fire escape, roof bulkhead), no interior, no colliders for inside.
Budget: 800 triangles.
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, HERE)

import common  # noqa: E402
import kit  # noqa: E402

NAME, CATEGORY, BUDGET = "apartment_lod1", "buildings", 800
WALL_TOP = 8.6
PARAPET_TOP = 9.2
COPING_TOP = 9.28


def build(model):
    k = kit.Kit(model)

    # 1. Main building solid box: x [-9.0, 8.2], y [-6.0, 6.0], z [0.0, 8.6]
    k.box((-9.0, -6.0, 0.0), (8.2, 6.0, WALL_TOP), "red_paint")
    k.collider((-9.0, -6.0, 0.0), (8.2, 6.0, WALL_TOP), "main_body")

    # Stone plinth base (z 0.02 to 0.55, proud by 2-3cm, butted at corners, split for door)
    k.box((-8.96, 5.98, 0.02), (-0.6, 6.03, 0.55), "rock_mid")
    k.box((0.6, 5.98, 0.02), (8.16, 6.03, 0.55), "rock_mid")
    k.box((-8.96, -6.03, 0.02), (8.16, -5.98, 0.55), "rock_mid")
    k.box((-9.03, -6.03, 0.02), (-8.98, 6.03, 0.55), "rock_mid")
    k.box((8.18, -6.03, 0.02), (8.23, 6.03, 0.55), "rock_mid")

    # Stone belt courses (proud, butted at corners)
    for zb in (2.95, 5.75):
        k.box((-8.96, 5.98, zb), (8.16, 6.025, zb + 0.15), "rock_light")
        k.box((-8.96, -6.025, zb), (8.16, -5.98, zb + 0.15), "rock_light")
        k.box((-9.025, -6.025, zb), (-8.98, 6.025, zb + 0.15), "rock_light")
        k.box((8.18, -6.025, zb), (8.225, 6.025, zb + 0.15), "rock_light")

    # Front door (flush dark opening with lintel)
    k.box((-0.6, 5.99, 0.02), (0.6, 6.02, kit.DOOR_H), "rock_dark")
    k.box((-0.75, 5.97, kit.DOOR_H), (0.75, 6.03, kit.DOOR_H + 0.15), "rock_light")

    # Simplified window patches (bands / separated blocks per floor)
    # Front: 2 blocks on ground floor (left of door, right of door), 1 long row on upper floors
    # Ground floor front
    k.box((-7.2, 5.98, 1.2), (-2.5, 6.01, 2.3), "tire_black")
    k.box((2.5, 5.98, 1.2), (7.0, 6.01, 2.3), "tire_black")
    # Upper floors front
    for z0, z1 in ((3.9, 5.0), (6.6, 7.7)):
        k.box((-7.2, 5.98, z0), (7.0, 6.01, z1), "tire_black")

    # Back windows
    for z0, z1 in ((1.2, 2.3), (3.9, 5.0), (6.6, 7.7)):
        k.box((-7.2, -6.01, z0), (7.0, -5.98, z1), "tire_black")

    # West windows
    for z0, z1 in ((1.2, 2.3), (3.9, 5.0), (6.6, 7.7)):
        k.box((-9.01, -4.5, z0), (-8.98, 4.5, z1), "tire_black")

    # East windows
    for z0, z1 in ((1.2, 2.3), (3.9, 5.0), (6.6, 7.7)):
        k.box((8.18, -4.5, z0), (8.21, 4.5, z1), "tire_black")

    # 2. Fire Escape on East side (simplified silhouette)
    for zp in (3.0, 5.8):
        # Platform
        k.box((8.2, -2.2, zp), (9.0, 2.2, zp + 0.06), "rust")
        # Railing plane (sits on top of platform)
        k.box((8.94, -2.2, zp + 0.06), (9.0, 2.2, zp + 0.85), "rust")
    # Diagonal stair bar
    k.box((8.45, -1.8, 3.06), (8.75, 1.8, 5.8), "rust")
    # Ladder strip
    k.box((8.8, 1.8, 1.4), (8.95, 1.9, 3.0), "rust")
    k.box((8.8, -1.8, 5.8), (8.95, -1.7, PARAPET_TOP + 0.35), "rust")

    # 3. Roof: slab, parapet, water tank, bulkhead
    k.box((-9.0, -6.0, WALL_TOP), (8.2, 6.0, WALL_TOP + 0.15), "rock_dark", roof=True)

    # Parapet walls (butted at corners)
    k.box((-8.85, 5.85, WALL_TOP + 0.15), (8.05, 6.0, PARAPET_TOP), "red_paint", roof=True)
    k.box((-8.85, -6.0, WALL_TOP + 0.15), (8.05, -5.85, PARAPET_TOP), "red_paint", roof=True)
    k.box((-9.0, -6.0, WALL_TOP + 0.15), (-8.85, 6.0, PARAPET_TOP), "red_paint", roof=True)
    k.box((8.05, -6.0, WALL_TOP + 0.15), (8.2, 6.0, PARAPET_TOP), "red_paint", roof=True)

    # Coping strip (butted at corners)
    k.box((-8.83, 5.83, PARAPET_TOP), (8.03, 6.02, COPING_TOP), "rock_light", roof=True)
    k.box((-8.83, -6.02, PARAPET_TOP), (8.03, -5.83, COPING_TOP), "rock_light", roof=True)
    k.box((-9.02, -6.02, PARAPET_TOP), (-8.83, 6.02, COPING_TOP), "rock_light", roof=True)
    k.box((8.03, -6.02, PARAPET_TOP), (8.22, 6.02, COPING_TOP), "rock_light", roof=True)

    # Simplified water tank (6 segments)
    tx, ty = -4.0, -2.0
    tz_base = WALL_TOP + 0.15
    tz_tank = tz_base + 1.2
    tz_top = tz_tank + 2.0
    tz_roof = tz_top + 0.8
    for lx in (tx - 0.8, tx + 0.8):
        for ly in (ty - 0.8, ty + 0.8):
            k.box((lx - 0.08, ly - 0.08, tz_base), (lx + 0.08, ly + 0.08, tz_tank), "bark_dark", roof=True)
    tank_bm = common.make_frustum(1.15, 1.15, tz_tank, tz_top, segments=6, center=(tx, ty), color="bark_dark")
    model.add(tank_bm, roof=True)
    cone_bm = common.make_frustum(1.25, 0.0, tz_top, tz_roof, segments=6, center=(tx, ty), color="rock_dark")
    model.add(cone_bm, roof=True)

    # Bulkhead box
    k.box((0.5, 0.5, tz_base), (3.0, 2.8, tz_base + 1.8), "red_paint", roof=True)
    k.box((0.45, 0.45, tz_base + 1.8), (3.05, 2.85, tz_base + 1.95), "rock_mid", roof=True)

    # Chimney
    k.box((5.2, -3.3, tz_base), (5.8, -2.7, tz_base + 1.5), "red_paint", roof=True)

    # 4. Markers: lot_size only
    k.empty("lot_size", (0.0, 0.0, 0.0), scale=(20.0, 14.0, 1.0))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, view=kit.FRONT_VIEW) else 1


if __name__ == "__main__":
    sys.exit(main())
