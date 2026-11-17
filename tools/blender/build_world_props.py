"""Builds the V11-13 world art pass (trees, ground cover, man-made props) and one contact sheet of them.

Run: blender -b --python tools/blender/build_world_props.py
Only writes the new models and the new sheet B colours. The older models are not rebuilt, so their
.glb files stay byte-for-byte the same. Exits with 1 if a model is over budget or fails to build.
"""
import importlib
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, "props"))

import palette  # noqa: E402
import build_all  # noqa: E402

MODELS = ["tree_birch", "tree_oak_big", "tree_dead", "pine_tall", "tree_autumn",
          "grass_tuft_a", "grass_tuft_b", "flowers_white", "flowers_yellow", "fern", "mushrooms",
          "stump", "log_fallen",
          "fence_wood_section", "car_wreck", "road_sign", "barrel_rusty", "tire_pile"]


def main():
    palette.write_palette_png_b()  # only sheet B: sheet A must stay byte-identical
    failures = []
    for name in MODELS:
        module = importlib.import_module(name)
        if module.main() != 0:
            failures.append(name)
    build_all.write_contact_sheet(MODELS, "contact_sheet_world.png")

    if failures:
        print("BUILD FAILED:", ", ".join(failures))
        return 1
    print("BUILD OK: all", len(MODELS), "world props within budget")
    return 0


if __name__ == "__main__":
    sys.exit(main())
