"""boardwalk_section: 4 m x 2 m wooden plank walkway on short posts (<= 120 tris).

Origin at deck centre at ground level (z=0). Walkway top at z=0.4 (local y=0.4 in glTF/Godot).
Walkway runs along Blender Y (-2.0 to +2.0), which is local +Z/-Z in glTF/Godot.
"""
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402

NAME, CATEGORY, BUDGET = "boardwalk_section", "props", 120
GRIP = (0.0, 0.0, 0.0)


def build(model):
    # 5 weathered planks across X (2.0 m wide), spaced along Y (-2.0 to +2.0), top at z=0.40
    colors = ("wood_crate", "bark_light", "bark_dark", "wood_crate", "bark_light")
    for i in range(5):
        y = -1.6 + i * 0.80
        z_shift = 0.005 * (1 if i % 2 == 0 else -1)
        model.add(common.make_box((2.0, 0.72, 0.06), loc=(0.0, y, 0.37 + z_shift), color=colors[i]))

    # 4 short corner posts extending down into swamp mud (z=-0.5 to z=0.34)
    for x in (-0.80, 0.80):
        for y in (-1.50, 1.50):
            model.add(common.make_box((0.14, 0.14, 0.84), loc=(x, y, -0.08), color="bark_dark"))

    model.add_collider("deck", common.make_box((2.0, 4.0, 0.40), loc=(0.0, 0.0, 0.20), color="white"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=GRIP) else 1


if __name__ == "__main__":
    sys.exit(main())
