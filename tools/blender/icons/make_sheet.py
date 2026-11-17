"""Labelled review grid of every item icon: assets/icons/_sheet.png. Needs Pillow (python3 -I).

Icons are shown at their real 128 px size on a dark background, like the hotbar.
Called by render_icons.py; can also be run by hand.
"""
import json
import os

from PIL import Image, ImageDraw, ImageFont

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
ICON_DIR = os.path.join(ROOT, "assets", "icons")
COLS = 6
CELL = 160
BG = (38, 40, 44)


def main():
    with open(os.path.join(ROOT, "data", "items.json"), encoding="utf-8") as f:
        ids = list(json.load(f).keys())
    rows = (len(ids) + COLS - 1) // COLS
    sheet = Image.new("RGB", (COLS * CELL, rows * CELL), BG)
    draw = ImageDraw.Draw(sheet)
    font = ImageFont.load_default(size=13)
    for n, item_id in enumerate(ids):
        x = (n % COLS) * CELL
        y = (n // COLS) * CELL
        path = os.path.join(ICON_DIR, item_id + ".png")
        if os.path.exists(path):
            icon = Image.open(path).convert("RGBA")
            sheet.paste(icon, (x + (CELL - icon.width) // 2, y + 6), icon)
        draw.text((x + 8, y + CELL - 22), item_id, fill=(230, 230, 230), font=font)
    out = os.path.join(ICON_DIR, "_sheet.png")
    sheet.save(out)
    print("SHEET", out)


if __name__ == "__main__":
    main()
