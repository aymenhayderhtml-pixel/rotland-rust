"""bridge_wood: 12 m span x 5 m wide wooden plank bridge with log railings and posts (<= 400 tris).

Origin at deck centre at ground level (z=0). Deck top at z=0.5 (local y=0.5 in glTF/Godot).
Deck runs along Blender Y (-6.0 to +6.0), which is local +Z/-Z in glTF/Godot.
"""
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402

NAME, CATEGORY, BUDGET = "bridge_wood", "props", 400
GRIP = (0.0, 0.0, 0.0)


def build(model):
    # Two longitudinal stringer beams underneath (along Y, spanning 12 m)
    for x in (-1.8, 1.8):
        model.add(common.make_box((0.24, 12.0, 0.24), loc=(x, 0.0, 0.30), color="bark_dark"))

    # 15 wooden planks across X (5.0 m wide), spaced along Y (-6.0 to +6.0), top at z=0.50
    colors = ("wood_crate", "bark_light", "bark_dark", "wood_crate", "bark_light")
    for i in range(15):
        y = -5.6 + i * 0.80
        z_shift = 0.006 * (1 if i % 3 == 0 else -1)
        model.add(common.make_box((5.0, 0.72, 0.08), loc=(0.0, y, 0.46 + z_shift), color=colors[i % len(colors)]))

    # 4 posts per side (8 total), extending below ground (z=-0.5) to railing height (z=1.35)
    post_ys = (-5.4, -1.8, 1.8, 5.4)
    for x in (-2.38, 2.38):
        for y in post_ys:
            model.add(common.make_box((0.18, 0.18, 1.85), loc=(x, y, 0.425), color="bark_dark"))

    # 2 log railings (1 per side, along Y)
    for x in (-2.38, 2.38):
        model.add(common.make_box((0.14, 11.4, 0.14), loc=(x, 0.0, 1.25), color="bark_light"))

    # Colliders: deck and side railings
    model.add_collider("deck", common.make_box((5.0, 12.0, 0.50), loc=(0.0, 0.0, 0.25), color="white"))
    model.add_collider("rail_left", common.make_box((0.22, 12.0, 0.85), loc=(-2.38, 0.0, 0.925), color="white"))
    model.add_collider("rail_right", common.make_box((0.22, 12.0, 0.85), loc=(2.38, 0.0, 0.925), color="white"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=GRIP) else 1


if __name__ == "__main__":
    sys.exit(main())
