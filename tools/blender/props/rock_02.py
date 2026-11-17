"""rock_02: small rounded stone, about 1 m wide (<= 150 tris)."""
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402

NAME, CATEGORY, BUDGET = "rock_02", "nature", 400
RADIUS = (0.5, 0.42, 0.3)
SEED = 12
ROT = (0.0, 0.0, 1.3)


def build(model):
    body = common.make_icosphere(subdiv=2, jitter=0.22, seed=SEED, radius=RADIUS, rot=ROT, color="rock_mid")
    common.paint_faces(body, [f for f in body.faces if f.normal.z > 0.45], "rock_light")
    common.paint_faces(body, [f for f in body.faces if f.normal.z < -0.6], "rock_dark")
    common.paint_faces(body, [f for f in body.faces if f.normal.z > 0.8], "leaf_dark")  # moss on the very top
    model.add(body)
    # A few pebbles round the base
    for k, (dx, dy) in enumerate(((1.0, 0.3), (-0.8, 0.7), (0.2, -1.0))):
        s = 0.12 + 0.05 * k
        model.add(common.make_icosphere(subdiv=1, jitter=0.2, seed=SEED + 7 * k + 1, radius=(s, s * 0.9, s * 0.6),
                                        loc=(dx * RADIUS[0], dy * RADIUS[1], s * 0.3), color="rock_dark"))
    model.add_collider("hull", common.make_icosphere(subdiv=1, jitter=0.22, seed=SEED, radius=RADIUS, rot=ROT))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
