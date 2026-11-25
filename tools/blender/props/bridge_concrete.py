"""bridge_concrete: 16 m span x 9 m wide concrete slab bridge with side walls and 2 piers (<= 300 tris).

Origin at deck centre at ground level (z=0). Deck top at z=0.6 (local y=0.6 in glTF/Godot).
Deck runs along Blender Y (-8.0 to +8.0), which is local +Z/-Z in glTF/Godot.
"""
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402

NAME, CATEGORY, BUDGET = "bridge_concrete", "props", 300
GRIP = (0.0, 0.0, 0.0)


def build(model):
    # Main concrete slab deck (16 m x 9 m, top at z=0.60, bottom at z=0.20)
    model.add(common.make_box((9.0, 16.0, 0.40), loc=(0.0, 0.0, 0.40), color="rock_mid"))

    # Subtle asphalt road surface strip down the middle (top flush at z=0.61)
    model.add(common.make_box((7.6, 16.0, 0.02), loc=(0.0, 0.0, 0.60), color="rock_dark"))

    # Low concrete side walls (parapets) and coping caps
    for x in (-4.25, 4.25):
        model.add(common.make_box((0.50, 16.0, 0.65), loc=(x, 0.0, 0.925), color="rock_light"))
        model.add(common.make_box((0.58, 16.06, 0.10), loc=(x, 0.0, 1.30), color="rock_mid"))

    # 2 concrete piers underneath (extending into riverbed to z=-2.0)
    for y in (-4.2, 4.2):
        model.add(common.make_box((7.8, 0.80, 2.20), loc=(0.0, y, -0.90), color="rock_dark"))

    # Colliders
    model.add_collider("deck", common.make_box((9.0, 16.0, 0.60), loc=(0.0, 0.0, 0.30), color="white"))
    model.add_collider("wall_left", common.make_box((0.55, 16.0, 0.75), loc=(-4.25, 0.0, 0.975), color="white"))
    model.add_collider("wall_right", common.make_box((0.55, 16.0, 0.75), loc=(4.25, 0.0, 0.975), color="white"))
    model.add_collider("pier_1", common.make_box((7.8, 0.80, 2.20), loc=(0.0, -4.2, -0.90), color="white"))
    model.add_collider("pier_2", common.make_box((7.8, 0.80, 2.20), loc=(0.0, 4.2, -0.90), color="white"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=GRIP) else 1


if __name__ == "__main__":
    sys.exit(main())
