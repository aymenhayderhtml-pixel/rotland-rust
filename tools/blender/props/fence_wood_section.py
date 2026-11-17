"""fence_wood_section: a 2 m wooden fence panel, two posts and two rails (<= 200 tris). Has a collider."""
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402

NAME, CATEGORY, BUDGET = "fence_wood_section", "props", 400


def build(model):
    for x in (-0.98, 0.98):
        model.add(common.make_box((0.12, 0.12, 1.2), loc=(x, 0.0, 0.6), color="bark_dark"))
        model.add(common.make_frustum(0.085, 0.0, 1.2, 1.32, segments=4, center=(x, 0.0), rot_z=0.785, color="bark_dark"))
    for i, z in enumerate((0.3, 0.62, 0.94)):
        rail = common.make_box((2.0, 0.05, 0.09), loc=(0.0, 0.0, 0.0), color="bark_light")
        common.transform(rail, rot=(0.0, 0.03 * (1 if i % 2 else -1), 0.0), loc=(0.0, 0.035, z))
        model.add(rail)
        for x in (-0.95, 0.95):
            model.add(common.make_box((0.02, 0.02, 0.02), loc=(x, 0.065, z), color="metal"))
    model.add_collider("panel", common.make_box((2.0, 0.12, 1.2), loc=(0.0, 0.0, 0.6), color="white"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
