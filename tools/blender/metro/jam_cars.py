"""The five jam cars and their burnt variants (sedan, hatch, SUV, pickup, van).

Each car: hollow lower body (door holes), greenhouse with glass, doors (some open), flat tyres,
rust and dust, roof rack luggage (SUV, van) or bed load (pickup), dash, seats and steering wheel.
Seat points are the contract for the jam bodies (see SEATS, copied to data/metro.json jam.seatOffsets).
Origin: bottom centre of the bounding box (automatic). Budgets: 15000 each (owner target 8k-15k).
"""
import os
import random
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, ".."))

import common  # noqa: E402
from jam_car_extras import bed_load, grime, rack_and_luggage, wheels  # noqa: E402
from jam_car_interior import build_interior  # noqa: E402
from jam_car_shell import greenhouse_parts, lower_parts  # noqa: E402
from jam_util import add_bbox_collider, offset_for, to_build  # noqa: E402

# new: DECISIONS 2026-11-26 (car sizes in metres; doors are (side, hinge_y, back_y, open_deg), side -1 = driver side)
CARS = {
    "sedan": dict(name="car_jam_sedan", body="car_blue", budget=15000, seed=11, half_l=2.0, W=0.86,
                  wheel_y=1.3, sill=0.24, belt=0.92, roof=1.42, lid_open=40.0, flat=[2], shelf=True,
                  cab=dict(bf=0.85, bb=-0.9, tf=0.35, tb=-0.55, xb=0.84, xt=0.74),
                  doors=[(-1, 0.85, 0.05, 60.0), (1, 0.85, 0.05, 0.0),
                         (-1, -0.05, -0.9, 90.0), (1, -0.05, -0.9, 0.0)],
                  windows=[(1, (0.10, 0.44, 0.14, 0.86), "glass"), (1, (0.52, 0.88, 0.14, 0.86), "crack"),
                           (3, (0.12, 0.46, 0.14, 0.86), "glass"), (3, (0.56, 0.90, 0.14, 0.86), "hole"),
                           (2, (0.12, 0.88, 0.10, 0.92), "crack"), (0, (0.14, 0.86, 0.12, 0.84), "glass")]),
    "hatch": dict(name="car_jam_hatch", body="car_red", budget=15000, seed=12, half_l=2.0, W=0.84,
                  wheel_y=1.25, sill=0.24, belt=0.90, roof=1.40, lid_open=0.0, flat=[1], shelf=True,
                  cab=dict(bf=0.60, bb=-1.90, tf=0.25, tb=-1.70, xb=0.82, xt=0.72),
                  doors=[(-1, 0.60, -0.40, 60.0), (1, 0.60, -0.40, 0.0),
                         (-1, -0.40, -1.60, 90.0), (1, -0.40, -1.60, 0.0)],
                  windows=[(1, (0.10, 0.40, 0.14, 0.86), "glass"), (1, (0.46, 0.90, 0.14, 0.86), "glass"),
                           (3, (0.10, 0.40, 0.14, 0.86), "crack"), (3, (0.46, 0.90, 0.14, 0.86), "glass"),
                           (2, (0.12, 0.88, 0.10, 0.90), "glass"), (0, (0.12, 0.88, 0.14, 0.86), "hole")]),
    "suv": dict(name="car_jam_suv", body="car_white", budget=15000, seed=13, half_l=2.25, W=0.90,
                wheel_y=1.4, sill=0.28, belt=1.10, roof=1.78, lid_open=0.0, flat=[0, 3], rack=True,
                cab=dict(bf=0.60, bb=-2.20, tf=0.30, tb=-2.00, xb=0.88, xt=0.80),
                doors=[(-1, 0.55, -0.45, 60.0), (1, 0.55, -0.45, 0.0),
                       (-1, -0.45, -1.30, 0.0), (1, -0.45, -1.30, 90.0)],
                windows=[(1, (0.08, 0.36, 0.16, 0.90), "glass"), (1, (0.42, 0.70, 0.16, 0.90), "crack"),
                         (1, (0.76, 0.94, 0.16, 0.90), "glass"), (3, (0.10, 0.40, 0.16, 0.90), "hole"),
                         (3, (0.44, 0.72, 0.16, 0.90), "glass"), (2, (0.12, 0.88, 0.10, 0.90), "glass"),
                         (0, (0.12, 0.88, 0.20, 0.90), "glass")]),
    "pickup": dict(name="car_jam_pickup", body="car_red", budget=15000, seed=14, half_l=2.6, W=0.92,
                   wheel_y=1.8, sill=0.36, belt=1.02, roof=1.72, lid_open=0.0, flat=[3],
                   bed=dict(front=-0.60, top=0.86),
                   cab=dict(bf=1.20, bb=-0.60, tf=0.90, tb=-0.45, xb=0.90, xt=0.80),
                   doors=[(-1, 1.10, -0.50, 60.0), (1, 1.10, -0.50, 0.0)],
                   windows=[(1, (0.12, 0.46, 0.12, 0.86), "glass"), (1, (0.54, 0.92, 0.12, 0.86), "crack"),
                            (3, (0.08, 0.46, 0.12, 0.86), "hole"), (2, (0.12, 0.88, 0.12, 0.90), "glass"),
                            (0, (0.12, 0.88, 0.15, 0.85), "hole")]),
    "van": dict(name="car_jam_van", body="car_white", budget=15000, seed=15, half_l=2.5, W=0.95,
                wheel_y=1.7, sill=0.34, belt=1.50, roof=2.42, lid_open=0.0, flat=[1], rack=True,
                cab=dict(bf=2.40, bb=-2.50, tf=2.10, tb=-2.45, xb=0.93, xt=0.92),
                doors=[(-1, 1.60, 0.20, 60.0), (1, 1.60, 0.20, 0.0), (-1, -0.20, -1.60, 90.0)],
                windows=[(1, (0.62, 0.93, 0.20, 0.85), "glass"), (3, (0.07, 0.38, 0.20, 0.85), "crack"),
                         (2, (0.12, 0.88, 0.12, 0.90), "glass")]),
}

# Seat points in metres, exported origin: +X right, +Y front, +Z up. The first entry is the driver seat.
CAR_VIEW = (1.0, 1.0, 0.5)  # preview camera: front right, above (+Y is the front)

SEATS = {
    "sedan": [("driver", (-0.38, 0.45, 0.42)), ("passenger", (0.38, 0.45, 0.42)),
              ("rear_left", (-0.38, -0.45, 0.42)), ("rear_right", (0.38, -0.45, 0.42))],
    "hatch": [("driver", (-0.36, 0.42, 0.42)), ("passenger", (0.36, 0.42, 0.42)),
              ("rear_left", (-0.34, -0.50, 0.42))],
    "suv": [("driver", (-0.40, 0.55, 0.48)), ("passenger", (0.40, 0.55, 0.48)),
            ("rear_left", (-0.40, -0.25, 0.48)), ("rear_right", (0.40, -0.25, 0.48))],
    "pickup": [("driver", (-0.38, 0.30, 0.48)), ("passenger", (0.38, 0.30, 0.48))],
    "van": [("driver", (-0.38, 0.95, 0.46)), ("passenger", (0.38, 0.95, 0.46)),
            ("rear_left", (-0.38, -0.40, 0.46))],
}


def _build_car(key, burnt):
    spec = CARS[key]

    def build(model):
        rng = random.Random(spec["seed"] + (100 if burnt else 0))
        body = "gun_black" if burnt else spec["body"]
        exterior = [(bm, False) for bm in lower_parts(spec, body, burnt)]
        exterior += [(bm, True) for bm in greenhouse_parts(spec, body, burnt)]
        exterior += [(bm, False) for bm in wheels(spec, burnt)]
        exterior += grime(spec, rng, burnt)
        exterior += [(bm, False) for bm in rack_and_luggage(spec, rng, burnt)]
        exterior += [(bm, False) for bm in bed_load(spec, rng, burnt)]
        offset = offset_for([bm for bm, _ in exterior])
        seats = SEATS[key]
        inside = [(bm, False) for bm in build_interior(spec, seats, offset, burnt)]
        for bm, roof in exterior + inside:
            model.add(bm, roof=roof)
        for name, point in seats:
            model.add_empty("seat_" + name, to_build(point, offset))
        # Solid shell: the closed body only (open doors and roof loads stay walk-through)
        add_bbox_collider(model, lower_parts(spec, body, burnt) + greenhouse_parts(spec, body, burnt))

    return build


def build_all():
    """Builds the five cars and their burnt variants. Returns [(name, ok)]."""
    results = []
    for key, spec in CARS.items():
        for burnt in (False, True):
            name = spec["name"] + ("_burnt" if burnt else "")
            ok = common.run_model(name, "vehicles", spec["budget"], _build_car(key, burnt), top_preview=True, view=CAR_VIEW)
            results.append((name, ok))
    return results
