"""sled: wooden snow sled with metal runners and red front bar (<= 300 tris)."""
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402

NAME, CATEGORY, BUDGET = "sled", "props", 300


def build(model):
    # Two runners and curved front tips
    for sx in (-0.21, 0.21):
        model.add(common.make_box((0.035, 1.05, 0.04), loc=(sx, 0.04, 0.02), color="metal"))
        tip = common.make_box((0.035, 0.24, 0.04), color="metal")
        common.transform(tip, loc=(sx, -0.56, 0.09), rot=(0.55, 0.0, 0.0))
        model.add(tip)
        # Four riser struts
        for sy in (-0.34, 0.36):
            model.add(common.make_box((0.04, 0.05, 0.14), loc=(sx, sy, 0.11), color="bark_dark"))

    # Crossbars
    for sy in (-0.34, 0.36):
        model.add(common.make_box((0.50, 0.05, 0.03), loc=(0.0, sy, 0.175), color="bark_dark"))

    # Five longitudinal deck slats
    for i, sx in enumerate((-0.20, -0.10, 0.0, 0.10, 0.20)):
        col = "red_paint" if i in (0, 4) else "wood_crate"
        model.add(common.make_box((0.075, 1.08, 0.025), loc=(sx, 0.04, 0.20), color=col))

    # Front tow bar
    model.add(common.make_box((0.46, 0.04, 0.04), loc=(0.0, -0.64, 0.16), color="red_paint"))

    model.add_collider("box", common.make_box((0.52, 1.30, 0.22), loc=(0.0, -0.02, 0.11), color="wood_crate"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
