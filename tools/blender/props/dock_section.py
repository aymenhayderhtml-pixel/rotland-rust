"""dock_section: 4 x 4 m weathered plank deck on posts going 3 m down into water (<= 400 tris).

Repeats 5 times for a 20 m dock. Origin is centered on the deck at z=0,
with posts extending down to z=-3.0 m so they penetrate into the lake bed.
"""
import math
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402

NAME, CATEGORY, BUDGET = "dock_section", "props", 400
GRIP = (0.0, 0.0, 0.0)  # origin centered at deck surface


def build(model):
    # Longitudinal stringer beams underneath (along Y)
    for x in (-1.75, 0.0, 1.75):
        beam = common.make_box((0.18, 4.0, 0.16), loc=(x, 0.0, -0.08), color="bark_dark")
        model.add(beam)

    # 10 weathered wooden planks across X (width 3.88 m to fit inside rub rails), spaced along Y (-2.0 to 2.0)
    colors = ("wood_crate", "bark_light", "bark_dark", "wood_crate", "bark_light")
    for i in range(10):
        y = -1.8 + i * 0.40
        col = colors[i % len(colors)]
        # Subtle organic shifts in width and x-offset for weathered look
        x_shift = 0.015 * (1 if i % 2 == 0 else -1)
        z_shift = 0.008 * (1 if i % 3 == 0 else -1)
        plank = common.make_box((3.88, 0.36, 0.08), loc=(x_shift, y, 0.04 + z_shift), color=col)
        model.add(plank)

    # 4 corner pilings (posts) extending down to -3.0 m, top protruding at +0.3 m
    posts_pos = [(-1.75, -1.3), (1.75, -1.3), (-1.75, 1.3), (1.75, 1.3)]
    for px, py in posts_pos:
        # Main wooden post (6-segment frustum)
        post = common.make_frustum(0.13, 0.13, -3.0, 0.32, segments=6, center=(px, py), color="bark_dark")
        model.add(post)
        # Rope / metal collar near the top of the post
        collar = common.make_frustum(0.15, 0.15, 0.18, 0.26, segments=6, center=(px, py), color="wheat_gold")
        model.add(collar)

    # Underwater cross-brace between left and right posts (underwater structural stability)
    for py in (-1.3, 1.3):
        brace = common.make_box((3.5, 0.12, 0.12), loc=(0.0, py, -1.6), color="bark_dark")
        model.add(brace)

    # Edge bumper / rub-rail planks along the outer sides at deck level (top at z=0.11 proud of planks)
    for x_edge in (-1.99, 1.99):
        rail = common.make_box((0.06, 3.98, 0.16), loc=(x_edge, 0.0, 0.03), color="bark_dark")
        model.add(rail)

    # Colliders
    model.add_collider("deck", common.make_box((4.0, 4.0, 0.24), loc=(0.0, 0.0, 0.0), color="white"))
    for idx, (px, py) in enumerate(posts_pos):
        model.add_collider(f"post_{idx+1}", common.make_box((0.3, 0.3, 3.32), loc=(px, py, -1.34), color="white"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, grip=GRIP) else 1


if __name__ == "__main__":
    sys.exit(main())
