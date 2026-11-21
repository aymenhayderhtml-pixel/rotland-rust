"""car_wreck_sedan: a burned-out sedan wreck with no wheels on one side (<= 2500 tris). Has a collider."""
import math
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402

NAME, CATEGORY, BUDGET = "car_wreck_sedan", "props", 2500

# Sedan dimensions (metres)
# Total length ~4.4 m (y from -2.2 to +2.2), width ~1.76 m (x from -0.88 to +0.88)
BODY_HALF_WIDTH = 0.88
HOOD_Y_FRONT = 2.15
HOOD_Y_REAR = 0.85
CABIN_Y_FRONT = 0.85
CABIN_Y_REAR = -0.85
TRUNK_Y_FRONT = -0.85
TRUNK_Y_REAR = -2.15

SILL_Z = 0.26
BELT_Z = 0.84
ROOF_Z = 1.38
WHEEL_RADIUS = 0.27
WHEEL_Y_FRONT = 1.40
WHEEL_Y_REAR = -1.40


def _sedan_body():
    """Main scorched chassis, lower body sills, front hood, cabin frame and rear trunk."""
    bms = []

    # 1. Floorpan / lower chassis frame: y from -2.10 to +2.10
    floor = common.make_box((2 * BODY_HALF_WIDTH - 0.16, 4.20, 0.08),
                            loc=(0.0, 0.0, SILL_Z + 0.04), color="gun_black")
    bms.append(floor)

    # Rocker sills between wheels (y from -1.05 to +1.05)
    rocker_l = common.make_box((0.08, 2.10, BELT_Z - SILL_Z),
                               loc=(-BODY_HALF_WIDTH + 0.04, 0.0, (SILL_Z + BELT_Z) / 2), color="rust")
    rocker_r = common.make_box((0.08, 2.10, BELT_Z - SILL_Z),
                               loc=(BODY_HALF_WIDTH - 0.04, 0.0, (SILL_Z + BELT_Z) / 2), color="rust")
    bms.extend([rocker_l, rocker_r])

    # 2. Front Hood / Engine Bay: y from 0.85 to 2.15
    # Front quarter panels above wheel wells (front wheel well from y=1.05 to 1.75, opening up to z=0.62)
    fq_top_l = common.make_box((0.08, 0.70, BELT_Z - 0.62),
                               loc=(-BODY_HALF_WIDTH + 0.04, WHEEL_Y_FRONT, (0.62 + BELT_Z) / 2), color="gun_black")
    fq_top_r = common.make_box((0.08, 0.70, BELT_Z - 0.62),
                               loc=(BODY_HALF_WIDTH - 0.04, WHEEL_Y_FRONT, (0.62 + BELT_Z) / 2), color="rust")
    # Front corners (ahead of wheel)
    fq_front_l = common.make_box((0.08, 0.40, BELT_Z - SILL_Z),
                                 loc=(-BODY_HALF_WIDTH + 0.04, 1.95, (SILL_Z + BELT_Z) / 2), color="gun_black")
    fq_front_r = common.make_box((0.08, 0.40, BELT_Z - SILL_Z),
                                 loc=(BODY_HALF_WIDTH - 0.04, 1.95, (SILL_Z + BELT_Z) / 2), color="rust")
    bms.extend([fq_top_l, fq_top_r, fq_front_l, fq_front_r])

    # Front fascia: radiator header, empty headlight sockets, blackened grille
    grille = common.make_box((2 * BODY_HALF_WIDTH - 0.20, 0.06, 0.32),
                             loc=(0.0, HOOD_Y_FRONT, SILL_Z + 0.26), color="gun_black")
    headlight_l = common.make_box((0.18, 0.07, 0.16),
                                  loc=(-BODY_HALF_WIDTH + 0.18, HOOD_Y_FRONT, SILL_Z + 0.32), color="rock_dark")
    headlight_r = common.make_box((0.18, 0.07, 0.16),
                                  loc=(BODY_HALF_WIDTH - 0.18, HOOD_Y_FRONT, SILL_Z + 0.32), color="metal")
    bms.extend([grille, headlight_l, headlight_r])

    # Scorched engine block, cylinder head and exhaust inside engine bay
    engine_block = common.make_box((0.55, 0.60, 0.38), loc=(0.0, 1.45, SILL_Z + 0.20), color="gun_black")
    engine_head = common.make_box((0.46, 0.52, 0.12), loc=(0.0, 1.45, SILL_Z + 0.45), color="rust")
    radiator = common.make_box((0.85, 0.06, 0.32), loc=(0.0, HOOD_Y_FRONT - 0.14, SILL_Z + 0.24), color="metal")
    bms.extend([engine_block, engine_head, radiator])

    # Warped, crumpled hood panel: skewed open at an angle
    hood = common.make_box((2 * BODY_HALF_WIDTH - 0.16, 1.25, 0.035), loc=(0.0, 0.0, 0.0), color="rust")
    common.transform(hood, rot=(-0.22, 0.14, 0.06), loc=(-0.04, 1.48, BELT_Z + 0.12))
    bms.append(hood)

    # 3. Passenger Cabin: y from -0.85 to +0.85
    # Slanted A-pillars (front windshield frame)
    pillar_a_l = common.make_box((0.06, 0.06, 0.66), loc=(0.0, 0.0, 0.0), color="gun_black")
    common.transform(pillar_a_l, rot=(0.58, 0.0, 0.0), loc=(-BODY_HALF_WIDTH + 0.12, 0.58, 1.10))
    pillar_a_r = common.make_box((0.06, 0.06, 0.66), loc=(0.0, 0.0, 0.0), color="gun_black")
    common.transform(pillar_a_r, rot=(0.58, 0.0, 0.0), loc=(BODY_HALF_WIDTH - 0.12, 0.58, 1.10))

    # Slanted C-pillars (rear window frame for sedan 3-box notchback)
    pillar_c_l = common.make_box((0.07, 0.07, 0.68), loc=(0.0, 0.0, 0.0), color="gun_black")
    common.transform(pillar_c_l, rot=(-0.56, 0.0, 0.0), loc=(-BODY_HALF_WIDTH + 0.12, -0.58, 1.10))
    pillar_c_r = common.make_box((0.07, 0.07, 0.68), loc=(0.0, 0.0, 0.0), color="gun_black")
    common.transform(pillar_c_r, rot=(-0.56, 0.0, 0.0), loc=(BODY_HALF_WIDTH - 0.12, -0.58, 1.10))

    # Center B-pillars
    pillar_b_l = common.make_box((0.05, 0.06, ROOF_Z - BELT_Z),
                                 loc=(-BODY_HALF_WIDTH + 0.06, 0.0, (BELT_Z + ROOF_Z) / 2), color="rust")
    pillar_b_r = common.make_box((0.05, 0.06, ROOF_Z - BELT_Z),
                                 loc=(BODY_HALF_WIDTH - 0.06, 0.0, (BELT_Z + ROOF_Z) / 2), color="rust")
    bms.extend([pillar_a_l, pillar_a_r, pillar_c_l, pillar_c_r, pillar_b_l, pillar_b_r])

    # Sagging, charred roof panel
    roof = common.make_box((2 * BODY_HALF_WIDTH - 0.22, 1.25, 0.035), loc=(0.0, 0.0, 0.0), color="gun_black")
    common.transform(roof, rot=(0.04, 0.0, 0.0), loc=(0.0, 0.0, ROOF_Z - 0.02))
    bms.append(roof)

    # Interior details: dashboard, steering column, burnt seats
    dash = common.make_box((2 * BODY_HALF_WIDTH - 0.22, 0.26, 0.18),
                           loc=(0.0, CABIN_Y_FRONT - 0.16, BELT_Z + 0.06), color="gun_black")
    col_rod = common.make_box((0.04, 0.32, 0.04), loc=(0.0, 0.0, 0.0), color="metal")
    common.transform(col_rod, rot=(0.58, 0.0, 0.0), loc=(-0.35, CABIN_Y_FRONT - 0.26, BELT_Z + 0.12))
    wheel_rim = common.make_frustum(0.15, 0.15, 0.0, 0.03, segments=8, color="gun_black")
    common.transform(wheel_rim, rot=(0.58, 0.0, 0.0), loc=(-0.35, CABIN_Y_FRONT - 0.38, BELT_Z + 0.20))

    # Charred seat frames
    seat_fl = common.make_box((0.38, 0.38, 0.20), loc=(-0.35, 0.22, SILL_Z + 0.18), color="bark_dark")
    seat_fr = common.make_box((0.38, 0.38, 0.20), loc=(0.35, 0.22, SILL_Z + 0.18), color="bark_dark")
    rear_bench = common.make_box((2 * BODY_HALF_WIDTH - 0.26, 0.42, 0.22),
                                 loc=(0.0, -0.48, SILL_Z + 0.18), color="gun_black")
    bms.extend([dash, col_rod, wheel_rim, seat_fl, seat_fr, rear_bench])

    # Doors:
    # Right passenger door: closed with faded red paint patch
    door_panel_r = common.make_box((0.04, 0.95, BELT_Z - SILL_Z - 0.02),
                                   loc=(BODY_HALF_WIDTH - 0.02, 0.35, (SILL_Z + BELT_Z) / 2), color="car_red")
    # Left driver door: crumpled, swung open at an angle
    door_panel_l = common.make_box((0.04, 0.92, BELT_Z - SILL_Z - 0.02), loc=(0.0, 0.0, 0.0), color="rust")
    common.transform(door_panel_l, rot=(0.08, 0.15, -0.55),
                     loc=(-BODY_HALF_WIDTH - 0.22, 0.25, (SILL_Z + BELT_Z) / 2))
    bms.extend([door_panel_r, door_panel_l])

    # 4. Rear Trunk (Sedan 3-box): y from -0.85 to -2.15
    # Rear quarter panels above wheel wells (rear wheel well from y=-1.05 to -1.75)
    rq_top_l = common.make_box((0.08, 0.70, BELT_Z - 0.62),
                               loc=(-BODY_HALF_WIDTH + 0.04, WHEEL_Y_REAR, (0.62 + BELT_Z) / 2), color="rust")
    rq_top_r = common.make_box((0.08, 0.70, BELT_Z - 0.62),
                               loc=(BODY_HALF_WIDTH - 0.04, WHEEL_Y_REAR, (0.62 + BELT_Z) / 2), color="rust")
    # Rear corners (behind wheel)
    rq_rear_l = common.make_box((0.08, 0.40, BELT_Z - SILL_Z),
                                loc=(-BODY_HALF_WIDTH + 0.04, -1.95, (SILL_Z + BELT_Z) / 2), color="rust")
    rq_rear_r = common.make_box((0.08, 0.40, BELT_Z - SILL_Z),
                                loc=(BODY_HALF_WIDTH - 0.04, -1.95, (SILL_Z + BELT_Z) / 2), color="rust")
    rear_slam = common.make_box((2 * BODY_HALF_WIDTH - 0.16, 0.08, BELT_Z - SILL_Z),
                                loc=(0.0, TRUNK_Y_REAR + 0.04, (SILL_Z + BELT_Z) / 2), color="gun_black")
    # Trunk lid: popped open slightly
    trunk_lid = common.make_box((2 * BODY_HALF_WIDTH - 0.16, 1.15, 0.035), loc=(0.0, 0.0, 0.0), color="rust")
    common.transform(trunk_lid, rot=(0.22, 0.06, 0.0), loc=(0.0, -1.45, BELT_Z + 0.09))
    bms.extend([rq_top_l, rq_top_r, rq_rear_l, rq_rear_r, rear_slam, trunk_lid])

    # Bumpers
    # Front bumper: dangling on one side
    bumper_f = common.make_box((1.82, 0.10, 0.14), loc=(0.0, 0.0, 0.0), color="metal")
    common.transform(bumper_f, rot=(0.20, 0.12, -0.08), loc=(0.0, HOOD_Y_FRONT + 0.06, SILL_Z + 0.08))
    # Rear bumper: hanging low
    bumper_r = common.make_box((1.80, 0.10, 0.14), loc=(0.0, 0.0, 0.0), color="rust")
    common.transform(bumper_r, rot=(-0.16, 0.04, 0.0), loc=(0.0, TRUNK_Y_REAR - 0.06, SILL_Z + 0.08))
    bms.extend([bumper_f, bumper_r])

    return common.combine(bms)


def _running_gear():
    """Right side has 2 flat tires; left side has NO wheels (bare brake rotors resting on concrete block/ground)."""
    bms = []

    # Right side (+X): 2 flat tires on crushed rims
    for wy in (WHEEL_Y_FRONT, WHEEL_Y_REAR):
        # Flat tire: squished cylinder
        tire = common.make_frustum(WHEEL_RADIUS, WHEEL_RADIUS, -0.09, 0.09, segments=8, color="tire_black")
        common.transform(tire, rot=(0.0, math.pi / 2, 0.0))
        # Rusted rim
        rim = common.make_frustum(0.15, 0.15, -0.092, 0.092, segments=6, color="rust")
        common.transform(rim, rot=(0.0, math.pi / 2, 0.0))
        hub = common.make_box((0.04, 0.08, 0.08), loc=(0.092, 0.0, 0.0), color="metal")
        w_comb = common.combine([tire, rim, hub])
        common.transform(w_comb, loc=(BODY_HALF_WIDTH - 0.08, wy, WHEEL_RADIUS - 0.05))
        bms.append(w_comb)

    # Left side (-X): NO WHEELS! Bare rusty brake rotors and calipers
    for wy in (WHEEL_Y_FRONT, WHEEL_Y_REAR):
        rotor = common.make_frustum(0.16, 0.16, -0.02, 0.02, segments=8, color="rust")
        common.transform(rotor, rot=(0.0, math.pi / 2, 0.0), loc=(-BODY_HALF_WIDTH + 0.06, wy, 0.16))
        caliper = common.make_box((0.08, 0.10, 0.08), loc=(-BODY_HALF_WIDTH + 0.06, wy + 0.08, 0.20), color="gun_black")
        axle = common.make_box((0.25, 0.06, 0.06), loc=(-BODY_HALF_WIDTH + 0.18, wy, 0.16), color="metal")
        bms.extend([rotor, caliper, axle])

    # Concrete block / cinder brick propped under the left front sill
    cinder = common.make_box((0.35, 0.25, 0.15), loc=(-BODY_HALF_WIDTH + 0.02, WHEEL_Y_FRONT, 0.075), color="rock_mid")
    bms.append(cinder)

    return common.combine(bms)


def build(model):
    chassis = _sedan_body()
    gear = _running_gear()
    car = common.combine([chassis, gear])

    # Tilt vehicle 4 degrees towards wheelless left side
    common.transform(car, rot=(0.0, math.radians(-4.0), 0.0))
    model.add(car)

    # Collider: convex box covering the vehicle
    model.add_collider("sedan", common.make_box((1.86, 4.45, 1.25), loc=(0.0, 0.0, 0.65), color="white"))


def main():
    # Preview camera looking from front-left to showcase wheelless side, open door, and exposed engine
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, view=(-1.1, 1.0, 0.55)) else 1


if __name__ == "__main__":
    sys.exit(main())
