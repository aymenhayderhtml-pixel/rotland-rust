"""car_wreck_van: a rusty abandoned cargo van with flat tires and an open sliding side door (<= 2500 tris). Has a collider."""
import math
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402

NAME, CATEGORY, BUDGET = "car_wreck_van", "props", 2500

# Van dimensions (metres)
# Total length ~4.8 m (y from -2.38 to +2.38), width ~1.96 m (x from -0.98 to +0.98), height ~2.05 m
BODY_HALF_WIDTH = 0.96
LENGTH_FRONT = 2.36
LENGTH_REAR = -2.36
COWL_Y = 1.28           # hood-windshield junction
HEADER_Y = 0.68         # windshield-roof junction
CAB_PARTITION_Y = 0.48

FLOOR_Z = 0.42
BELT_Z = 1.15
ROOF_Z = 2.02
ARCH_TOP_Z = 0.68
WHEEL_RADIUS = 0.28
WHEEL_Y_FRONT = 1.55
WHEEL_Y_REAR = -1.45


def _chassis_and_wheels():
    """Heavy frame, floorpan, and 4 flat squashed tires with rusty rims."""
    bms = []

    # Floorpan: spanning under the whole van body
    floor = common.make_box((2 * BODY_HALF_WIDTH - 0.20, 4.60, 0.08),
                            loc=(0.0, 0.0, FLOOR_Z - 0.04), color="gun_black")
    bms.append(floor)

    # 4 flat wheels on the 4 corners
    for wx, sign_x in [(-BODY_HALF_WIDTH + 0.10, -1), (BODY_HALF_WIDTH - 0.10, 1)]:
        for wy in [WHEEL_Y_FRONT, WHEEL_Y_REAR]:
            tire = common.make_frustum(WHEEL_RADIUS, WHEEL_RADIUS, -0.10, 0.10, segments=10, color="tire_black")
            common.transform(tire, rot=(0.0, math.pi / 2, 0.0))
            rim = common.make_frustum(0.16, 0.16, -0.105, 0.105, segments=8, color="rust")
            common.transform(rim, rot=(0.0, math.pi / 2, 0.0))
            hub = common.make_box((0.04, 0.10, 0.10), loc=(sign_x * 0.10, 0.0, 0.0), color="metal")
            wheel = common.combine([tire, rim, hub])
            common.transform(wheel, loc=(wx, wy, WHEEL_RADIUS - 0.06))
            bms.append(wheel)

    # Bumpers:
    # Front heavy steel bumper
    front_bumper = common.make_box((2 * BODY_HALF_WIDTH + 0.04, 0.14, 0.18),
                                   loc=(0.0, LENGTH_FRONT + 0.06, FLOOR_Z + 0.05), color="metal")
    # Rear step bumper with tow hitch
    rear_bumper = common.make_box((2 * BODY_HALF_WIDTH + 0.02, 0.20, 0.14),
                                  loc=(0.0, LENGTH_REAR - 0.08, FLOOR_Z + 0.02), color="rust")
    hitch = common.make_box((0.08, 0.18, 0.08), loc=(0.0, LENGTH_REAR - 0.16, FLOOR_Z - 0.02), color="metal")
    bms.extend([front_bumper, rear_bumper, hitch])

    return common.combine(bms)


def _front_cab():
    """Slanted short hood, front grille, windshield frame, cab doors, dashboard, seats, and partition."""
    bms = []

    # Slanted front hood: from windshield base (cowl at y=1.28, z=1.12) down to grille (y=2.35, z=0.88)
    hood = common.make_box((2 * BODY_HALF_WIDTH - 0.16, 1.08, 0.04), loc=(0.0, 0.0, 0.0), color="rust")
    common.transform(hood, rot=(-0.24, 0.0, 0.0), loc=(0.0, 1.82, 1.00))
    bms.append(hood)

    # Front fenders with wheel arches:
    # Top fender strip above wheel well (y from 1.20 to 1.90, z from ARCH_TOP_Z to BELT_Z)
    for sign_x, col in [(-1, "rust"), (1, "car_white")]:
        fx = sign_x * (BODY_HALF_WIDTH - 0.04)
        f_top = common.make_box((0.08, 0.70, BELT_Z - ARCH_TOP_Z),
                                loc=(fx, WHEEL_Y_FRONT, (ARCH_TOP_Z + BELT_Z) / 2), color=col)
        # Front corner forward of wheel well (y from 1.90 to 2.36)
        f_front = common.make_box((0.08, 0.46, BELT_Z - FLOOR_Z),
                                  loc=(fx, 2.13, (FLOOR_Z + BELT_Z) / 2), color=col)
        # Rear corner behind front wheel well (y from 1.00 to 1.20)
        f_rear = common.make_box((0.08, 0.20, BELT_Z - FLOOR_Z),
                                 loc=(fx, 1.10, (FLOOR_Z + BELT_Z) / 2), color=col)
        bms.extend([f_top, f_front, f_rear])

    # Front grille & fascia
    grille = common.make_box((2 * BODY_HALF_WIDTH - 0.20, 0.08, 0.42),
                             loc=(0.0, LENGTH_FRONT - 0.04, FLOOR_Z + 0.24), color="gun_black")
    headlight_l = common.make_box((0.20, 0.09, 0.16),
                                  loc=(-BODY_HALF_WIDTH + 0.22, LENGTH_FRONT - 0.03, FLOOR_Z + 0.32), color="rock_dark")
    headlight_r = common.make_box((0.20, 0.09, 0.16),
                                  loc=(BODY_HALF_WIDTH - 0.22, LENGTH_FRONT - 0.03, FLOOR_Z + 0.32), color="metal")
    bms.extend([grille, headlight_l, headlight_r])

    # Windshield A-pillars from cowl (y=1.28, z=1.12) to header (y=0.68, z=ROOF_Z)
    a_pillar_l = common.make_box((0.08, 0.08, 1.08), loc=(0.0, 0.0, 0.0), color="metal")
    common.transform(a_pillar_l, rot=(0.59, 0.0, 0.0), loc=(-BODY_HALF_WIDTH + 0.08, 0.98, 1.57))
    a_pillar_r = common.make_box((0.08, 0.08, 1.08), loc=(0.0, 0.0, 0.0), color="metal")
    common.transform(a_pillar_r, rot=(0.59, 0.0, 0.0), loc=(BODY_HALF_WIDTH - 0.08, 0.98, 1.57))

    header = common.make_box((2 * BODY_HALF_WIDTH - 0.16, 0.10, 0.08),
                             loc=(0.0, HEADER_Y, ROOF_Z - 0.04), color="metal")
    cowl = common.make_box((2 * BODY_HALF_WIDTH - 0.16, 0.10, 0.08),
                           loc=(0.0, COWL_Y, BELT_Z - 0.02), color="rust")
    bms.extend([a_pillar_l, a_pillar_r, header, cowl])

    # Left driver door: closed with rusty paint and side mirror
    door_l = common.make_box((0.06, 0.52, BELT_Z - FLOOR_Z),
                             loc=(-BODY_HALF_WIDTH + 0.03, 0.74, (FLOOR_Z + BELT_Z) / 2), color="rust")
    b_pillar_l = common.make_box((0.08, 0.08, ROOF_Z - BELT_Z),
                                 loc=(-BODY_HALF_WIDTH + 0.04, CAB_PARTITION_Y, (BELT_Z + ROOF_Z) / 2), color="car_white")
    cant_rail_l = common.make_box((0.08, 0.30, 0.08),
                                  loc=(-BODY_HALF_WIDTH + 0.04, 0.58, ROOF_Z - 0.04), color="car_white")
    mirror_l = common.make_box((0.04, 0.14, 0.20),
                               loc=(-BODY_HALF_WIDTH - 0.06, 1.15, BELT_Z + 0.08), color="gun_black")
    mirror_arm_l = common.make_box((0.08, 0.04, 0.04),
                                   loc=(-BODY_HALF_WIDTH - 0.01, 1.15, BELT_Z + 0.08), color="metal")
    bms.extend([door_l, b_pillar_l, cant_rail_l, mirror_l, mirror_arm_l])

    # Right passenger cab door: closed with white paint and side mirror
    door_fr = common.make_box((0.06, 0.52, BELT_Z - FLOOR_Z),
                              loc=(BODY_HALF_WIDTH - 0.03, 0.74, (FLOOR_Z + BELT_Z) / 2), color="car_white")
    b_pillar_r = common.make_box((0.08, 0.08, ROOF_Z - BELT_Z),
                                 loc=(BODY_HALF_WIDTH - 0.04, CAB_PARTITION_Y, (BELT_Z + ROOF_Z) / 2), color="rust")
    cant_rail_r = common.make_box((0.08, 0.30, 0.08),
                                  loc=(BODY_HALF_WIDTH - 0.04, 0.58, ROOF_Z - 0.04), color="car_white")
    mirror_r = common.make_box((0.04, 0.14, 0.20),
                               loc=(BODY_HALF_WIDTH + 0.06, 1.15, BELT_Z + 0.08), color="metal")
    bms.extend([door_fr, b_pillar_r, cant_rail_r, mirror_r])

    # Cab interior: dashboard, steering column, driver/passenger seats
    dash = common.make_box((2 * BODY_HALF_WIDTH - 0.20, 0.32, 0.24),
                           loc=(0.0, COWL_Y - 0.14, BELT_Z - 0.05), color="gun_black")
    column = common.make_box((0.05, 0.35, 0.05), loc=(0.0, 0.0, 0.0), color="metal")
    common.transform(column, rot=(0.60, 0.0, 0.0), loc=(-0.42, 1.08, BELT_Z - 0.02))
    wheel = common.make_frustum(0.18, 0.18, 0.0, 0.03, segments=8, color="gun_black")
    common.transform(wheel, rot=(0.60, 0.0, 0.0), loc=(-0.42, 0.94, BELT_Z + 0.10))

    # Bucket seats
    seat_l = common.make_box((0.44, 0.44, 0.24), loc=(-0.42, 0.78, FLOOR_Z + 0.16), color="canvas_khaki")
    back_l = common.make_box((0.42, 0.10, 0.55), loc=(-0.42, 0.58, FLOOR_Z + 0.52), color="canvas_khaki")
    seat_r = common.make_box((0.44, 0.44, 0.24), loc=(0.42, 0.78, FLOOR_Z + 0.16), color="leather")
    back_r = common.make_box((0.42, 0.10, 0.55), loc=(0.42, 0.58, FLOOR_Z + 0.52), color="leather")
    bms.extend([dash, column, wheel, seat_l, back_l, seat_r, back_r])

    # Cab partition wall behind the seats (bulkhead separating cab from cargo area)
    part_l = common.make_box((0.68, 0.04, ROOF_Z - FLOOR_Z),
                             loc=(-BODY_HALF_WIDTH + 0.38, CAB_PARTITION_Y, (FLOOR_Z + ROOF_Z) / 2), color="metal")
    part_r = common.make_box((0.68, 0.04, ROOF_Z - FLOOR_Z),
                             loc=(BODY_HALF_WIDTH - 0.38, CAB_PARTITION_Y, (FLOOR_Z + ROOF_Z) / 2), color="metal")
    part_top = common.make_box((2 * BODY_HALF_WIDTH - 0.20, 0.04, 0.22),
                               loc=(0.0, CAB_PARTITION_Y, ROOF_Z - 0.13), color="metal")
    bms.extend([part_l, part_r, part_top])

    return common.combine(bms)


def _cargo_bay_and_sliding_door():
    """Hollow cargo box, left panel wall, open sliding door on the right, rear doors, and roof."""
    bms = []

    # 1. Left side body (-X): solid panel van wall with rear wheel arch opening
    # Wall from CAB_PARTITION_Y (0.48) to LENGTH_REAR (-2.36)
    # Rear wheel well at y = -1.45 (opening from y = -1.80 to -1.10)
    # Lower wall ahead of rear wheel (y from -1.10 to 0.48)
    wl_fwd = common.make_box((0.06, 1.58, BELT_Z - FLOOR_Z),
                             loc=(-BODY_HALF_WIDTH + 0.03, -0.31, (FLOOR_Z + BELT_Z) / 2), color="rust")
    # Lower wall above rear wheel arch
    wl_arch = common.make_box((0.06, 0.70, BELT_Z - ARCH_TOP_Z),
                              loc=(-BODY_HALF_WIDTH + 0.03, WHEEL_Y_REAR, (ARCH_TOP_Z + BELT_Z) / 2), color="rust")
    # Lower wall behind rear wheel (y from -2.36 to -1.80)
    wl_rear = common.make_box((0.06, 0.56, BELT_Z - FLOOR_Z),
                              loc=(-BODY_HALF_WIDTH + 0.03, -2.08, (FLOOR_Z + BELT_Z) / 2), color="rust")
    # Upper left wall (solid panel)
    wl_upper = common.make_box((0.06, 2.84, ROOF_Z - BELT_Z),
                               loc=(-BODY_HALF_WIDTH + 0.03, -0.94, (BELT_Z + ROOF_Z) / 2), color="car_white")
    rust_patch = common.make_box((0.07, 0.90, 0.60),
                                 loc=(-BODY_HALF_WIDTH + 0.03, -0.60, 1.20), color="rust")
    bms.extend([wl_fwd, wl_arch, wl_rear, wl_upper, rust_patch])

    # 2. Right side body (+X):
    # Sliding door opening from y = -0.75 to +0.48 (width 1.23 m, clear open opening into cargo bay!)
    # Rear quarter panel behind sliding door: y from -2.36 to -0.75 (length 1.61 m)
    # With rear wheel arch opening at y = -1.45
    wr_fwd = common.make_box((0.06, 0.35, BELT_Z - FLOOR_Z),
                             loc=(BODY_HALF_WIDTH - 0.03, -0.925, (FLOOR_Z + BELT_Z) / 2), color="rust")
    wr_arch = common.make_box((0.06, 0.70, BELT_Z - ARCH_TOP_Z),
                              loc=(BODY_HALF_WIDTH - 0.03, WHEEL_Y_REAR, (ARCH_TOP_Z + BELT_Z) / 2), color="rust")
    wr_rear = common.make_box((0.06, 0.56, BELT_Z - FLOOR_Z),
                              loc=(BODY_HALF_WIDTH - 0.03, -2.08, (FLOOR_Z + BELT_Z) / 2), color="rust")
    wr_upper = common.make_box((0.06, 1.61, ROOF_Z - BELT_Z),
                               loc=(BODY_HALF_WIDTH - 0.03, -1.555, (BELT_Z + ROOF_Z) / 2), color="car_white")
    # Exterior guide track rail running along the outside rear quarter
    track_rail = common.make_box((0.04, 1.55, 0.04),
                                 loc=(BODY_HALF_WIDTH + 0.02, -1.555, BELT_Z + 0.04), color="metal")
    bms.extend([wr_fwd, wr_arch, wr_rear, wr_upper, track_rail])

    # 3. The Sliding Side Door: OPEN!
    # Slid back over the rear quarter panel (resting at y = -1.25 to -0.05), tilted slightly outward
    door_bm = common.make_box((0.06, 1.20, ROOF_Z - FLOOR_Z - 0.06), loc=(0.0, 0.0, 0.0), color="car_white")
    d_handle = common.make_box((0.03, 0.12, 0.05), loc=(0.04, 0.45, 0.0), color="gun_black")
    d_rust = common.make_box((0.065, 0.60, 0.50), loc=(0.0, -0.20, -0.30), color="rust")
    door_combined = common.combine([door_bm, d_handle, d_rust])

    common.transform(door_combined, rot=(0.0, math.radians(3.0), -math.radians(2.0)),
                     loc=(BODY_HALF_WIDTH + 0.04, -0.65, (FLOOR_Z + ROOF_Z) / 2))
    bms.append(door_combined)

    # 4. Rear double doors (y = LENGTH_REAR):
    rear_door_l = common.make_box((BODY_HALF_WIDTH - 0.05, 0.06, ROOF_Z - FLOOR_Z),
                                  loc=(-BODY_HALF_WIDTH / 2 + 0.02, LENGTH_REAR + 0.03, (FLOOR_Z + ROOF_Z) / 2), color="rust")
    rear_door_r = common.make_box((BODY_HALF_WIDTH - 0.05, 0.06, ROOF_Z - FLOOR_Z),
                                  loc=(0.0, 0.0, 0.0), color="car_white")
    common.transform(rear_door_r, rot=(0.0, 0.0, math.radians(12.0)),
                     loc=(BODY_HALF_WIDTH / 2 - 0.02, LENGTH_REAR + 0.08, (FLOOR_Z + ROOF_Z) / 2))
    bms.extend([rear_door_l, rear_door_r])

    # 5. Roof panel: from rear doors (y = LENGTH_REAR = -2.36) to windshield header (y = HEADER_Y = 0.68)
    roof_len = HEADER_Y - LENGTH_REAR
    roof_center_y = (HEADER_Y + LENGTH_REAR) / 2
    roof_main = common.make_box((2 * BODY_HALF_WIDTH - 0.08, roof_len, 0.06),
                                loc=(0.0, roof_center_y, ROOF_Z - 0.02), color="car_white")
    bms.append(roof_main)

    # Longitudinal roof stiffening ribs
    for rx in [-0.55, -0.25, 0.25, 0.55]:
        rib = common.make_box((0.06, roof_len - 0.20, 0.03),
                              loc=(rx, roof_center_y, ROOF_Z + 0.025), color="rust")
        bms.append(rib)

    # 6. Cargo inside the bay:
    # A wooden loot crate and an old rusty barrel sitting inside visible through the open sliding door!
    crate = common.make_box((0.55, 0.55, 0.55), loc=(0.0, 0.0, 0.0), color="wood_crate")
    common.transform(crate, rot=(0.0, 0.0, 0.25), loc=(-0.15, -0.10, FLOOR_Z + 0.28))
    bms.append(crate)

    barrel = common.make_frustum(0.24, 0.24, 0.0, 0.72, segments=8, color="rust")
    common.transform(barrel, rot=(0.15, 0.1, 0.0), loc=(-0.35, -1.10, FLOOR_Z + 0.02))
    bms.append(barrel)

    # Ribbed cargo floor inside
    for fy in [-1.8, -1.3, -0.8, -0.3, 0.2]:
        ridge = common.make_box((2 * BODY_HALF_WIDTH - 0.26, 0.06, 0.02),
                                loc=(0.0, fy, FLOOR_Z + 0.01), color="metal")
        bms.append(ridge)

    return common.combine(bms)


def build(model):
    chassis = _chassis_and_wheels()
    cab = _front_cab()
    cargo = _cargo_bay_and_sliding_door()
    van = common.combine([chassis, cab, cargo])
    model.add(van)

    # Box collider covering the entire van
    model.add_collider("van", common.make_box((2.02, 4.88, 2.05), loc=(0.0, 0.0, 1.05), color="white"))


def main():
    # Preview camera looking from front-right to showcase the open sliding side door, front cab, and flat tires
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, view=(1.1, 0.9, 0.55)) else 1


if __name__ == "__main__":
    sys.exit(main())
