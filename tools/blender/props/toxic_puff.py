"""toxic_puff: lumpy low-poly cloud ball, ~1 m wide, origin at centre (<= 80 tris)."""
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402

NAME, CATEGORY, BUDGET = "toxic_puff", "nature", 80

# Three overlapping subdiv=0 icosahedra (3 * 20 = 60 tris <= 80 budget) centered around (0, 0, 0)
LOBES = (
    (31, (0.36, 0.35, 0.34), (-0.12, -0.06, 0.02)),
    (37, (0.33, 0.32, 0.31), (0.15, 0.08, 0.10)),
    (43, (0.31, 0.33, 0.29), (0.04, -0.14, -0.11)),
)


def build(model):
    for seed, radius, loc in LOBES:
        puff = common.make_icosphere(
            subdiv=0,
            jitter=0.14,
            seed=seed,
            radius=radius,
            loc=loc,
            color="swamp_moss",
        )
        common.paint_faces(
            puff,
            [f for f in puff.faces if f.normal.z > 0.15 or f.normal.x > 0.25],
            "crop_green",
        )
        model.add(puff)


def main():
    # Pass grip=(0, 0, 0) so Model.finalize_and_export keeps the origin at the cloud's centre
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=(0.0, 0.0, 0.0)) else 1


if __name__ == "__main__":
    sys.exit(main())
