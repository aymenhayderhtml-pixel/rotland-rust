"""laundry_line: two wooden posts with a sagging rope and hanging clothes (<= 300 tris)."""
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402

NAME, CATEGORY, BUDGET = "laundry_line", "props", 300


def build(model):
    # Two wooden posts
    for sx in (-1.05, 1.05):
        model.add(common.make_frustum(0.055, 0.045, 0.0, 1.85, segments=6, center=(sx, 0.0), color="bark_dark"))
        model.add_collider(
            f"post_{'l' if sx < 0 else 'r'}",
            common.make_box((0.12, 0.12, 1.85), loc=(sx, 0.0, 0.925), color="bark_dark"),
        )

    # Sagging rope in two angled halves
    for sx, tilt in ((-0.525, 0.11), (0.525, -0.11)):
        rope = common.make_box((1.07, 0.022, 0.022), color="reed_tan")
        common.transform(rope, loc=(sx, 0.0, 1.69), rot=(0.0, tilt, 0.0))
        model.add(rope)

    # Hanging clothes: blue shirt, denim pants, white towel, red shirt
    items = [
        (-0.62, 1.44, (0.36, 0.04, 0.44), 0.04, "shirt_blue"),
        (-0.18, 1.35, (0.28, 0.04, 0.56), -0.03, "pants_denim"),
        (0.24, 1.41, (0.28, 0.03, 0.44), 0.02, "white"),
        (0.64, 1.45, (0.34, 0.04, 0.42), -0.04, "shirt_red"),
    ]
    for x, z, size, tilt_x, color in items:
        cloth = common.make_box(size, color=color)
        common.transform(cloth, loc=(x, 0.0, z), rot=(tilt_x, 0.0, 0.0))
        model.add(cloth)
        # Small clothespin on top
        pin = common.make_box((0.03, 0.04, 0.05), loc=(x, 0.0, z + size[2] * 0.5 + 0.015), color="wood_crate")
        model.add(pin)


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
