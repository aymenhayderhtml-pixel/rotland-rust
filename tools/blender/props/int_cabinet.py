"""int_cabinet: wooden cabinet/dresser facing +Y (1.0 x 0.48 x 0.90 m)."""
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402

NAME, CATEGORY, BUDGET = "int_cabinet", "props", 300


def build(model):
    # Four short feet
    for sx in (-0.43, 0.43):
        for sy in (-0.18, 0.18):
            model.add(common.make_box((0.06, 0.06, 0.08), loc=(sx, sy, 0.04), color="bark_dark"))
    # Main body
    model.add(common.make_box((0.96, 0.46, 0.77), loc=(0.0, 0.0, 0.465), color="bark_light"))
    # Top slab
    model.add(common.make_box((1.02, 0.50, 0.05), loc=(0.0, 0.0, 0.875), color="bark_dark"))
    # Two cabinet door fronts on +Y with handles
    for sx in (-0.23, 0.23):
        model.add(common.make_box((0.42, 0.03, 0.66), loc=(sx, 0.235, 0.465), color="wood_crate"))
        model.add(common.make_box((0.03, 0.03, 0.14), loc=(sx * 0.35, 0.255, 0.50), color="metal"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
