"""tree_swamp_lod2: far impostor LOD for cypress/mangrove swamp tree (<= 50 tris)."""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402

NAME, CATEGORY, BUDGET = "tree_swamp_lod2", "props", 50


def build(model):
    # Single flared trunk frustum sunk 6 cm below ground (12 tris)
    model.add(common.make_frustum(0.95, 0.22, -0.06, 6.80, segments=4, color="swamp_bark"))

    # Wide flat canopy blob (20 tris)
    model.add(common.make_icosphere(
        subdiv=1, jitter=0.10, seed=81, radius=(3.20, 2.90, 1.05),
        loc=(0.0, 0.0, 7.65), color="swamp_moss",
    ))

    # Single hanging moss silhouette accent (12 tris)
    strand = common.make_box((0.20, 0.14, 2.20), color="swamp_moss")
    common.transform(strand, rot=(0.04, -0.04, 0.30), loc=(1.75, -0.75, 5.75))
    model.add(strand)

    model.add_collider("trunk", common.make_frustum(0.60, 0.25, -0.05, 6.20, segments=6, color="swamp_bark"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=(0.0, 0.0, 0.0)) else 1


if __name__ == "__main__":
    sys.exit(main())
