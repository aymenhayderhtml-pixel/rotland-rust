"""Body props for the jam: body_seated (slumped in a car seat, head to one side, arms hanging) and body_sheet
(a readable body shape under a white sheet). No gore: one small dark spot on the shirt at most. Budget 8000.
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, os.path.join(HERE, "..", "vehicles"))

import car_parts  # noqa: E402
import common  # noqa: E402
from jam_util import box, pbox  # noqa: E402

BUDGET = 8000  # new: DECISIONS 2026-11-26 (props budget; body_seated 4k-8k)


def _seated(model):
    """Origin = middle of the seat cushion. The body faces +Y: the back leans on the seat back (-Y side).
    Torso slumped back and to the driver's side, head down and to one side, arms hanging down the side."""
    model.add(common.make_icosphere(subdiv=3, radius=(0.2, 0.22, 0.08), loc=(0.0, -0.04, 0.0), color="pants_denim"))
    for sx in (-1, 1):
        model.add(common.make_icosphere(subdiv=3, radius=(0.1, 0.26, 0.09), loc=(sx * 0.1, 0.34, 0.02),
                                        color="pants_denim"))  # thighs forward
        model.add(pbox((sx * 0.1 - 0.09, 0.52, -0.2), (sx * 0.1 + 0.09, 0.64, -0.02), "pants_dark", bevel=0.02))
        model.add(pbox((sx * 0.1 - 0.09, 0.6, -0.24), (sx * 0.1 + 0.09, 0.78, -0.19), "shoe_dark", bevel=0.02))
    torso = common.make_icosphere(subdiv=4, radius=(0.21, 0.13, 0.29), color="shirt_blue")
    common.transform(torso, rot=(0.35, 0.0, 0.2), loc=(0.0, -0.12, 0.34))
    model.add(torso)
    model.add(box((0.02, 0.1, 0.34), (0.09, 0.13, 0.4), "blood_dark"))  # one small spot on the shirt
    head = common.make_icosphere(subdiv=4, radius=(0.11, 0.12, 0.12), color="skin_pale")
    common.transform(head, rot=(0.4, 0.0, -0.45), loc=(0.1, -0.1, 0.66))  # head down, to one side
    model.add(head)
    hair = common.make_icosphere(subdiv=3, radius=(0.115, 0.125, 0.07), color="hair_dark")
    common.transform(hair, rot=(0.4, 0.0, -0.45), loc=(0.1, -0.1, 0.73))
    model.add(hair)
    for sx in (-1, 1):  # arms hang from the shoulders, hands near the floor
        model.add(car_parts._beam((sx * 0.24, -0.1, 0.5), (sx * 0.27, 0.05, -0.12), 0.05, color="shirt_blue",
                                  segments=8))
        model.add(common.make_icosphere(subdiv=3, radius=(0.05, 0.06, 0.05), loc=(sx * 0.27, 0.07, -0.16),
                                        color="skin_pale"))


def _sheet(model):
    """A body shape under a white sheet: head, shoulders, torso, hips and two legs, two shoes at one end (no gore).
    Smooth lumps (little jitter), so the shape reads as a body, not as rubble."""
    model.add(common.make_icosphere(subdiv=4, jitter=0.02, seed=7, radius=(0.44, 1.0, 0.1),
                                    loc=(0.0, 0.0, 0.1), color="white"))  # the sheet on the ground
    lumps = [((0.0, 0.84, 0.2), (0.14, 0.16, 0.14), 4),  # head
             ((0.0, 0.5, 0.2), (0.28, 0.2, 0.12), 3),  # shoulders
             ((0.0, 0.1, 0.22), (0.22, 0.35, 0.14), 4),  # torso
             ((0.0, -0.3, 0.2), (0.24, 0.2, 0.14), 3),  # hips
             ((-0.12, -0.8, 0.16), (0.1, 0.42, 0.1), 3),  # left leg
             ((0.12, -0.8, 0.16), (0.1, 0.42, 0.1), 3)]  # right leg
    for k, (loc, radius, sub) in enumerate(lumps):
        model.add(common.make_icosphere(subdiv=sub, jitter=0.02, seed=8 + k, radius=radius, loc=loc, color="white"))
    for sx in (-0.12, 0.12):
        model.add(pbox((sx - 0.06, -1.25, 0.0), (sx + 0.06, -1.02, 0.1), "shoe_dark", bevel=0.02))


def build_all():
    """Builds body_seated (origin at the seat point) and body_sheet. Returns [(name, ok)]."""
    seated_ok = common.run_model("body_seated", "props", BUDGET, _seated, grip=(0.0, 0.0, 0.0))
    sheet_ok = common.run_model("body_sheet", "props", BUDGET, _sheet)
    return [("body_seated", seated_ok), ("body_sheet", sheet_ok)]
