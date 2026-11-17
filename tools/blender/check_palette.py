"""Checks both palette sheets. Plain `python3 tools/blender/check_palette.py` (no Blender needed).

Sheet A's 32 names must stay in this exact order (old models depend on their cell numbers).
Every name must be unique across both sheets. Both PNGs must be 64x128, and the center pixel
of each named cell must equal its RGB. The PNG is decoded here with the standard library only.
"""
import os
import struct
import sys
import zlib

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import palette  # noqa: E402

SHEET_A_NAMES = [
    "bark_dark", "bark_light", "leaf_dark", "leaf_mid", "leaf_light", "pine_dark", "pine_mid", "rock_dark",
    "rock_mid", "rock_light", "wood_crate", "metal", "dirt", "grass", "red_paint", "white",
    "skin_light", "skin_zombie", "hair_dark", "shirt_blue", "shirt_zombie", "pants_denim", "pants_dark",
    "shoe_dark", "blood_dark", "shirt_red", "skin_pale", "gown_blue", "gun_black", "water_blue", "brass",
    "soda_purple",
]


def read_png_rgb(path):
    """Returns (width, height, pixel) where pixel(x, y) gives (r, g, b). 8-bit RGB PNG only."""
    with open(path, "rb") as f:
        data = f.read()
    if data[:8] != b"\x89PNG\r\n\x1a\n":
        raise ValueError("not a PNG")
    pos = 8
    width = height = None
    idat = b""
    while pos < len(data):
        length = struct.unpack(">I", data[pos:pos + 4])[0]
        tag = data[pos + 4:pos + 8]
        body = data[pos + 8:pos + 8 + length]
        if tag == b"IHDR":
            width, height, depth, color = struct.unpack(">IIBB", body[:10])
            if (depth, color) != (8, 2):
                raise ValueError("only 8-bit RGB PNGs are supported")
        elif tag == b"IDAT":
            idat += body
        pos += 12 + length
    raw = zlib.decompress(idat)
    stride = width * 3
    rows = []
    prev = bytearray(stride)
    p = 0
    for _ in range(height):
        kind = raw[p]
        line = bytearray(raw[p + 1:p + 1 + stride])
        p += 1 + stride
        for i in range(stride):
            a = line[i - 3] if i >= 3 else 0
            b = prev[i]
            c = prev[i - 3] if i >= 3 else 0
            if kind == 1:
                line[i] = (line[i] + a) & 0xFF
            elif kind == 2:
                line[i] = (line[i] + b) & 0xFF
            elif kind == 3:
                line[i] = (line[i] + (a + b) // 2) & 0xFF
            elif kind == 4:
                pa, pb, pc = abs(b - c), abs(a - c), abs(a + b - 2 * c)
                pred = a if (pa <= pb and pa <= pc) else (b if pb <= pc else c)
                line[i] = (line[i] + pred) & 0xFF
            elif kind != 0:
                raise ValueError(f"unknown PNG filter {kind}")
        rows.append(line)
        prev = line

    def pixel(x, y):
        o = x * 3
        return tuple(rows[y][o:o + 3])

    return width, height, pixel


def cell_center(index):
    col = index % palette.GRID_COLS
    row = index // palette.GRID_COLS
    return col * palette.CELL + palette.CELL // 2, row * palette.CELL + palette.CELL // 2


def main():
    problems = []

    if palette.NAMES != SHEET_A_NAMES:
        problems.append("sheet A names or order changed (old models depend on them)")
    if len(palette.NAMES) != palette.CELLS:
        problems.append(f"sheet A has {len(palette.NAMES)} colors, expected {palette.CELLS}")

    all_names = palette.NAMES + palette.NAMES_B
    dupes = sorted({n for n in all_names if all_names.count(n) > 1})
    if dupes:
        problems.append(f"names used twice across sheets: {', '.join(dupes)}")

    sheets = [
        ("A", palette.PNG_PATH, palette.COLORS),
        ("B", palette.PNG_PATH_B, palette.COLORS_B),
    ]
    for sheet, path, colors in sheets:
        if not os.path.exists(path):
            problems.append(f"sheet {sheet} PNG is missing: {path}")
            continue
        width, height, pixel = read_png_rgb(path)
        if (width, height) != (palette.WIDTH, palette.HEIGHT):
            problems.append(f"sheet {sheet} is {width}x{height}, expected {palette.WIDTH}x{palette.HEIGHT}")
            continue
        for index, (name, rgb) in enumerate(colors):
            x, y = cell_center(index)
            if pixel(x, y) != rgb:
                problems.append(f"sheet {sheet} cell {index} ({name}) center is {pixel(x, y)}, expected {rgb}")
        if sheet == "B":
            for index in range(len(colors), palette.CELLS):
                x, y = cell_center(index)
                if pixel(x, y) != palette.GREY_FILL:
                    problems.append(f"sheet B spare cell {index} is not grey")
        print(f"sheet {sheet}: {width}x{height}, {len(colors)} named colors checked")

    if problems:
        for p in problems:
            print("FAIL:", p)
        return 1
    print("PALETTE OK")
    return 0


if __name__ == "__main__":
    sys.exit(main())
