"""int_table: wooden dining/work table (0.75 m high, 1.2 x 0.75 m)."""
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402

NAME, CATEGORY, BUDGET = "int_table", "props", 300


def build(model):
    # Tabletop: 1.20 x 0.75 x 0.05 m (top at z = 0.75)
    model.add(common.make_box((1.20, 0.75, 0.05), loc=(0.0, 0.0, 0.725), color="wood_crate"))
    # Apron rails under tabletop
    for sy in (-0.31, 0.31):
        model.add(common.make_box((1.04, 0.04, 0.07), loc=(0.0, sy, 0.665), color="bark_light"))
    for sx in (-0.52, 0.52):
        model.add(common.make_box((0.04, 0.58, 0.07), loc=(sx, 0.0, 0.665), color="bark_light"))
    # Four legs
    for sx in (-0.52, 0.52):
        for sy in (-0.31, 0.31):
            model.add(common.make_box((0.06, 0.06, 0.70), loc=(sx, sy, 0.35), color="bark_dark"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
