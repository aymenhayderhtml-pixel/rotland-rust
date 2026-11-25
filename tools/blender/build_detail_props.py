"""Builds all 28 R35 detail props (biome, town clutter, interiors) and one contact sheet.

Run: blender -b --python tools/blender/build_detail_props.py
Only writes sheet C (`assets/textures/palette_c.png`) and the new R35 detail props. Sheets A and B
and all existing models are never touched, so they stay byte-for-byte the same.
Exits with 1 if any model is over budget or fails to build.
"""
import importlib
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, "props"))

import palette  # noqa: E402
import build_all  # noqa: E402

MODELS = [
    # Biome detail props (R35-02)
    "tumbleweed",
    "bleached_bones",
    "fence_post_desert",
    "frozen_log",
    "ice_shards",
    "snow_bush_small",
    "cattails",
    "lily_pads",
    "swamp_moss_clump",
    "scree_patch",
    "alpine_flowers",
    "cairn",
    # Town clutter (R35-03)
    "laundry_line",
    "porch_chair",
    "water_barrel",
    "woodpile",
    "sled",
    "propane_tank",
    "bench_wood",
    "fire_barrel",
    # Interiors (R35-06)
    "int_table",
    "int_chair",
    "int_bed",
    "int_shelf",
    "int_stove",
    "int_fridge",
    "int_cabinet",
    "int_rug",
]


def main():
    palette.write_palette_png_c()  # only sheet C: sheets A and B must stay byte-identical
    failures = []
    for name in MODELS:
        module = importlib.import_module(name)
        if module.main() != 0:
            failures.append(name)
    build_all.write_contact_sheet(MODELS, "contact_sheet_detail.png")

    if failures:
        print("BUILD FAILED:", ", ".join(failures))
        return 1
    print("BUILD OK: all", len(MODELS), "detail props within budget")
    return 0


if __name__ == "__main__":
    sys.exit(main())
