"""Shared color palettes for every generated model.

Three 64x128 PNGs, each a 4-column by 8-row grid of 16x16 cells (32 colors per sheet).
Each face samples the center of its cell, so a color's UV depends on its cell number.

- Sheet A (`palette.png`) is full and frozen: old models sample its cell centers. Never change it.
- Sheet B (`palette_b.png`) is full and frozen: vehicles, animals, farming, autumn trees. Never change it.
- Sheet C (`palette_c.png`) is for R31 biome props (desert, snow/mountains, swamp). Its first 16 cells
  are named below; the other 16 are a neutral grey, free for future colors.

Growing sheet A or B would move old UVs, so sheet C is a third sheet with its own material
(`palette_c`). Models that use only sheet A or sheet B export byte-for-byte the same as before.
Run `python3 tools/blender/palette.py` to rewrite all palette PNGs.
"""
import os
import struct
import zlib

CELL = 16
GRID_COLS = 4
GRID_ROWS = 8
GRID = GRID_COLS
WIDTH = CELL * GRID_COLS
HEIGHT = CELL * GRID_ROWS
SIZE = WIDTH
CELLS = GRID_COLS * GRID_ROWS  # colors per sheet

# Order matters: the index is the cell number (left to right, top to bottom).
# Never reorder existing cells (old models depend on their UVs).
COLORS = [
    ("bark_dark", (74, 59, 46)),
    ("bark_light", (120, 96, 72)),
    ("leaf_dark", (48, 79, 46)),
    ("leaf_mid", (76, 112, 62)),
    ("leaf_light", (110, 144, 84)),
    ("pine_dark", (36, 62, 48)),
    ("pine_mid", (58, 88, 66)),
    ("rock_dark", (84, 86, 84)),
    ("rock_mid", (120, 122, 116)),
    ("rock_light", (158, 158, 150)),
    ("wood_crate", (150, 116, 78)),
    ("metal", (110, 118, 124)),
    ("dirt", (110, 88, 62)),
    ("grass", (104, 130, 70)),
    ("red_paint", (152, 56, 46)),
    ("white", (226, 222, 212)),
    # Characters (V1-02)
    ("skin_light", (210, 172, 142)),
    ("skin_zombie", (112, 134, 102)),
    ("hair_dark", (40, 34, 30)),
    ("shirt_blue", (68, 96, 128)),
    ("shirt_zombie", (115, 98, 76)),
    ("pants_denim", (54, 72, 98)),
    ("pants_dark", (44, 46, 48)),
    ("shoe_dark", (32, 28, 26)),
    ("blood_dark", (110, 28, 28)),
    # Zombie variants (V1-06)
    ("shirt_red", (120, 52, 48)),
    ("skin_pale", (186, 180, 166)),
    ("gown_blue", (134, 160, 170)),
    # Weapons and items (V1-09)
    ("gun_black", (34, 34, 38)),
    ("water_blue", (70, 130, 196)),
    ("brass", (200, 150, 50)),
    ("soda_purple", (120, 50, 140)),
]

# Sheet B (V1-P3): named colors for the after-v1 backlog. Cells 16-31 stay grey (GREY_FILL).
COLORS_B = [
    ("car_red", (160, 40, 36)),
    ("car_blue", (52, 84, 140)),
    ("car_white", (206, 210, 214)),
    ("tire_black", (28, 28, 30)),
    ("chrome", (176, 182, 188)),
    ("glass_tint", (96, 124, 132)),
    ("horse_brown", (110, 72, 44)),
    ("horse_dark", (52, 38, 30)),
    ("dog_tan", (176, 134, 84)),
    ("dog_dark", (74, 56, 40)),
    ("leather", (92, 58, 36)),
    ("canvas_khaki", (160, 146, 104)),
    ("crop_green", (92, 150, 52)),
    ("wheat_gold", (206, 170, 84)),
    ("water_lake", (58, 110, 128)),
    ("cloth_olive", (96, 100, 58)),
    # Vehicles and animals (V11-08): rust patches on the car, pale chest on the dog
    ("rust", (138, 74, 42)),
    ("dog_cream", (222, 204, 170)),
    # Wild animals (V12-01): deer, rabbit, boar. Appended at the end of sheet B.
    ("deer_fur", (140, 104, 70)),
    ("deer_pale", (214, 200, 176)),
    ("antler", (196, 184, 160)),
    ("rabbit_fur", (160, 132, 100)),
    ("rabbit_pale", (222, 212, 200)),
    ("boar_hide", (118, 98, 80)),
    ("boar_bristle", (62, 52, 46)),
    ("boar_tusk", (232, 226, 206)),
    # Farming (V12-03): tilled soil, carrot tops and roots, potato skin, young sprouts. Appended at the end.
    ("soil_dark", (62, 44, 30)),
    ("carrot_orange", (214, 112, 40)),
    ("potato_brown", (156, 118, 78)),
    ("sprout_green", (126, 182, 74)),
    # World art pass (V11-13): autumn trees
    ("leaf_orange", (196, 110, 40)),
    ("leaf_red", (160, 48, 36)),
]

# Sheet C (R31): biome props (desert, snow/mountains, swamp). Cells 16-31 stay grey (GREY_FILL).
COLORS_C = [
    ("sand_light", (226, 200, 148)),
    ("sand_dark", (196, 165, 110)),
    ("sandstone", (190, 120, 80)),
    ("sandstone_dark", (150, 90, 60)),
    ("cactus_green", (88, 130, 70)),
    ("cactus_dark", (60, 95, 50)),
    ("dry_brush", (150, 125, 80)),
    ("bone_white", (225, 215, 190)),
    ("snow_white", (236, 240, 245)),
    ("snow_shadow", (190, 200, 215)),
    ("ice_blue", (170, 205, 230)),
    ("pine_snow_dark", (40, 75, 60)),
    ("swamp_moss", (70, 90, 45)),
    ("swamp_mud", (75, 62, 42)),
    ("reed_tan", (170, 150, 95)),
    ("swamp_bark", (70, 58, 45)),
]
GREY_FILL = (128, 128, 128)  # unused cells of sheet B and sheet C

NAMES = [name for name, _ in COLORS]
NAMES_B = [name for name, _ in COLORS_B]
NAMES_C = [name for name, _ in COLORS_C]
PNG_PATH = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", "assets", "textures", "palette.png"))
PNG_PATH_B = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", "assets", "textures", "palette_b.png"))
PNG_PATH_C = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", "assets", "textures", "palette_c.png"))

# A face stores one value: its cell index on sheet A, index + 32 for sheet B, or index + 64 for sheet C.
# Sheet A and B values are unchanged, so existing faces and UVs stay byte-for-byte identical.
KEY_B_OFFSET = CELLS
KEY_C_OFFSET = CELLS * 2


def sheet_of(color_name):
    """"A", "B" or "C". Raises ValueError for an unknown name."""
    if color_name in NAMES:
        return "A"
    if color_name in NAMES_B:
        return "B"
    if color_name in NAMES_C:
        return "C"
    raise ValueError(f"unknown palette color: {color_name}")


def index_of(color_name):
    """Cell number inside the color's own sheet (0-31). For a face value, use key_of."""
    sheet = sheet_of(color_name)
    if sheet == "A":
        return NAMES.index(color_name)
    if sheet == "B":
        return NAMES_B.index(color_name)
    return NAMES_C.index(color_name)


def key_of(color_name):
    """The value a face stores for this color (A: index; B: index + 32; C: index + 64)."""
    sheet = sheet_of(color_name)
    if sheet == "A":
        return index_of(color_name)
    if sheet == "B":
        return index_of(color_name) + KEY_B_OFFSET
    return index_of(color_name) + KEY_C_OFFSET


def name_of_key(key):
    if key < KEY_B_OFFSET:
        return NAMES[key]
    if key < KEY_C_OFFSET:
        return NAMES_B[key - KEY_B_OFFSET]
    return NAMES_C[key - KEY_C_OFFSET]


def sheet_of_key(key):
    if key < KEY_B_OFFSET:
        return "A"
    if key < KEY_C_OFFSET:
        return "B"
    return "C"


def uv_center(color_name):
    """UV of the middle of a color's cell. Row 0 is the top row of the PNG, which is glTF's V=0 side."""
    i = index_of(color_name)
    col = i % GRID_COLS
    row = i // GRID_COLS
    u = (col + 0.5) / GRID_COLS
    v_top_based = (row + 0.5) / GRID_ROWS
    # Blender's V runs bottom-up, so flip it here; the glTF exporter flips it back.
    return (u, 1.0 - v_top_based)


def _png_bytes(width, height, rgb_rows):
    def chunk(tag, data):
        return struct.pack(">I", len(data)) + tag + data + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)

    raw = b"".join(b"\x00" + row for row in rgb_rows)  # filter byte 0 per scanline
    header = struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0)  # 8-bit RGB
    return (b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", header)
            + chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b""))


def _write_sheet(colors, path, fill):
    rows = []
    for y in range(HEIGHT):
        row = bytearray()
        for x in range(WIDTH):
            cell_x = x // CELL
            cell_y = y // CELL
            idx = cell_y * GRID_COLS + cell_x
            if idx < len(colors):
                _, rgb = colors[idx]
            else:
                rgb = fill
            row += bytes(rgb)
        rows.append(bytes(row))
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "wb") as f:
        f.write(_png_bytes(WIDTH, HEIGHT, rows))
    return path


def write_palette_png(path=PNG_PATH):
    return _write_sheet(COLORS, path, (0, 0, 0))


def write_palette_png_b(path=PNG_PATH_B):
    return _write_sheet(COLORS_B, path, GREY_FILL)


def write_palette_png_c(path=PNG_PATH_C):
    return _write_sheet(COLORS_C, path, GREY_FILL)


if __name__ == "__main__":
    print("wrote", write_palette_png())
    print("wrote", write_palette_png_b())
    print("wrote", write_palette_png_c())
