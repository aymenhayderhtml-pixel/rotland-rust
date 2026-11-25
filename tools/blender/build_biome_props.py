"""Builds all R31 biome props (desert, snow/mountains, swamp, plus tree LODs) and one contact sheet.

Run: blender -b --python tools/blender/build_biome_props.py
Only writes sheet C (`assets/textures/palette_c.png`) and the new R31 biome props. Sheets A and B
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
    # Desert (R31-06)
    "cactus_tall",
    "cactus_round",
    "desert_shrub",
    "rock_sandstone_01",
    "rock_sandstone_02",
    "cattle_skull",
    "tree_desert_dead",
    "tree_desert_dead_lod1",
    "tree_desert_dead_lod2",
    # Snow & Mountains (R31-07)
    "pine_snow",
    "pine_snow_lod1",
    "pine_snow_lod2",
    "rock_ice_01",
    "snow_drift",
    "snow_shrub",
    "boulder_big",
    "rock_cliff_01",
    # Swamp (R31-08)
    "tree_swamp",
    "tree_swamp_lod1",
    "tree_swamp_lod2",
    "reeds",
    "swamp_log_mossy",
    "swamp_stump",
    "lily_mud_patch",
    # R32-09
    "toxic_puff",
    # R33-08: bridges, signpost, boardwalk
    "bridge_wood",
    "bridge_concrete",
    "signpost",
    "boardwalk_section",
]


def main():
    palette.write_palette_png_c()  # only sheet C: sheets A and B must stay byte-identical
    failures = []
    for name in MODELS:
        module = importlib.import_module(name)
        if module.main() != 0:
            failures.append(name)
    build_all.write_contact_sheet(MODELS, "contact_sheet_biomes.png")

    if failures:
        print("BUILD FAILED:", ", ".join(failures))
        return 1
    print("BUILD OK: all", len(MODELS), "biome props within budget")
    return 0


if __name__ == "__main__":
    sys.exit(main())
