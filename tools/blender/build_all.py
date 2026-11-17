"""Builds every prop model, then a contact sheet of all previews.

Run: blender -b --python tools/blender/build_all.py
Exits with 1 if any model is over its triangle budget or fails to build.
"""
import array
import importlib
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, "props"))
sys.path.insert(0, os.path.join(HERE, "characters"))
sys.path.insert(0, os.path.join(HERE, "weapons"))
sys.path.insert(0, os.path.join(HERE, "icons"))
sys.path.insert(0, os.path.join(HERE, "buildings"))

import bpy  # noqa: E402
import palette  # noqa: E402
import common  # noqa: E402
import build_characters  # noqa: E402
import build_weapons  # noqa: E402
import build_buildings  # noqa: E402
import render_icons  # noqa: E402

MODELS = ["tree_pine_01", "tree_pine_02", "tree_broadleaf_01", "rock_01", "rock_02", "rock_03",
          "bush_01", "crate_loot"]
COLUMNS = 4


def write_contact_sheet(names, file_name="contact_sheet.png"):
    size = common.PREVIEW_SIZE
    rows = (len(names) + COLUMNS - 1) // COLUMNS
    width, height = COLUMNS * size, rows * size
    sheet_pixels = array.array("f", [0.78, 0.78, 0.78, 1.0]) * (width * height)
    for n, name in enumerate(names):
        path = os.path.join(common.PREVIEW_DIR, name + ".png")
        if not os.path.exists(path):
            continue
        img = bpy.data.images.load(path, check_existing=False)
        src = array.array("f", [0.0]) * (size * size * 4)
        img.pixels.foreach_get(src)
        tile_col, tile_row = n % COLUMNS, n // COLUMNS
        for y in range(size):  # Blender rows run bottom-up, so both images are flipped the same way
            src_row = (size - 1 - y) * size * 4
            dst_row = ((height - 1) - (tile_row * size + y)) * width * 4 + tile_col * size * 4
            sheet_pixels[dst_row:dst_row + size * 4] = src[src_row:src_row + size * 4]
        bpy.data.images.remove(img)
    sheet = bpy.data.images.new("contact_sheet", width=width, height=height, alpha=True)
    sheet.pixels.foreach_set(sheet_pixels)
    sheet.filepath_raw = os.path.join(common.PREVIEW_DIR, file_name)
    sheet.file_format = "PNG"
    sheet.save()
    print("contact sheet:", sheet.filepath_raw)


def main():
    palette.write_palette_png()
    palette.write_palette_png_b()
    failures = []
    for name in MODELS:
        module = importlib.import_module(name)
        if not module.main() == 0:
            failures.append(name)
    write_contact_sheet(MODELS)

    if build_characters.main() != 0:
        failures.append("characters")

    if build_weapons.main() != 0:
        failures.append("weapons")

    if build_buildings.main() != 0:
        failures.append("buildings")

    # Icons last: they reset the Blender scene for every render
    if render_icons.main() != 0:
        failures.append("icons")

    if failures:
        print("BUILD FAILED:", ", ".join(failures))
        return 1
    print("BUILD OK: all", len(MODELS), "models, characters, weapons and icons within budget")
    return 0


if __name__ == "__main__":
    sys.exit(main())
