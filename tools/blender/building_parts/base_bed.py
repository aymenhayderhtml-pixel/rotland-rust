"""base_bed: mattress on a wooden pallet frame, blanket, and pillow (<= 400 tris). Has a collider."""
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import common  # noqa: E402

NAME, CATEGORY, BUDGET = "base_bed", "building_parts", 400

# Dimensions fit 2m cell: pallet is 1.10m x 1.90m, sits on z=0
PALLET_W = 1.10
PALLET_L = 1.90
STRINGER_H = 0.04
BLOCK_H = 0.06
SLAT_H = 0.025
PALLET_H = STRINGER_H + BLOCK_H + SLAT_H  # 0.125m

MATTRESS_W = 1.00
MATTRESS_L = 1.80
MATTRESS_H = 0.22


def build(model):
    parts = []

    # 1. Pallet bottom stringer boards (3 boards running lengthwise along Y)
    for sx in (-1, 0, 1):
        x = sx * 0.48
        # Stringer board: 0.10m wide, 1.90m long, 0.04m thick
        parts.append(common.make_box((0.10, PALLET_L, STRINGER_H),
                                     loc=(x, 0.0, STRINGER_H / 2),
                                     color="bark_light" if sx == 0 else "wood_crate"))

        # 3 wooden blocks along each stringer (at back, center, front)
        for sy in (-1, 0, 1):
            y = sy * 0.84
            parts.append(common.make_box((0.10, 0.12, BLOCK_H),
                                         loc=(x, y, STRINGER_H + BLOCK_H / 2),
                                         color="bark_dark"))

    # 2. Pallet top deck slats (8 slats across X)
    num_slats = 8
    slat_w = 0.12
    step_y = (PALLET_L - slat_w) / (num_slats - 1)
    slat_z = STRINGER_H + BLOCK_H + SLAT_H / 2
    for i in range(num_slats):
        y = -PALLET_L / 2 + slat_w / 2 + i * step_y
        color = "wood_crate" if i % 3 != 0 else "bark_light"
        parts.append(common.make_box((PALLET_W, slat_w, SLAT_H),
                                     loc=(0.0, y, slat_z),
                                     color=color))

    # 3. Mattress on top of pallet deck
    mattress_z = PALLET_H + MATTRESS_H / 2
    parts.append(common.make_box((MATTRESS_W, MATTRESS_L, MATTRESS_H),
                                 loc=(0.0, 0.0, mattress_z),
                                 color="canvas_khaki"))

    # Mattress edge piping / border trim around the top rim
    top_rim_z = PALLET_H + MATTRESS_H - 0.015
    parts.append(common.make_box((MATTRESS_W + 0.02, MATTRESS_L + 0.02, 0.03),
                                 loc=(0.0, 0.0, top_rim_z),
                                 color="white"))

    # 4. Pillow at the head end (+Y)
    pillow_y = 0.60
    pillow_z = PALLET_H + MATTRESS_H + 0.045
    # Main pillow body
    parts.append(common.make_box((0.68, 0.36, 0.09),
                                 loc=(0.0, pillow_y, pillow_z),
                                 color="white"))
    # Pillow inner loft / puff
    parts.append(common.make_box((0.54, 0.26, 0.03),
                                 loc=(0.0, pillow_y, pillow_z + 0.045),
                                 color="skin_pale"))

    # 5. Blanket covering bottom 60% of bed (-Y)
    blanket_y_min = -0.92
    blanket_y_max = 0.20
    blanket_len = blanket_y_max - blanket_y_min
    blanket_y_center = (blanket_y_min + blanket_y_max) / 2
    blanket_top_z = PALLET_H + MATTRESS_H + 0.015

    # Main blanket sheet
    parts.append(common.make_box((MATTRESS_W + 0.03, blanket_len, 0.03),
                                 loc=(0.0, blanket_y_center, blanket_top_z),
                                 color="shirt_blue"))

    # Folded cuff / turned-down sheet lip at the upper blanket edge
    parts.append(common.make_box((MATTRESS_W + 0.04, 0.14, 0.04),
                                 loc=(0.0, blanket_y_max - 0.07, blanket_top_z + 0.02),
                                 color="white"))

    # Left and right blanket drapes hanging down the sides
    drape_h = MATTRESS_H + 0.04
    drape_z = blanket_top_z - drape_h / 2
    # Left drape (-X)
    parts.append(common.make_box((0.03, blanket_len, drape_h),
                                 loc=(-(MATTRESS_W / 2 + 0.015), blanket_y_center, drape_z),
                                 color="shirt_blue"))
    # Right drape (+X)
    parts.append(common.make_box((0.03, blanket_len, drape_h),
                                 loc=(MATTRESS_W / 2 + 0.015, blanket_y_center, drape_z),
                                 color="shirt_blue"))
    # Foot drape (-Y)
    parts.append(common.make_box((MATTRESS_W + 0.03, 0.03, drape_h),
                                 loc=(0.0, blanket_y_min - 0.015, drape_z),
                                 color="shirt_blue"))

    model.add(common.combine(parts))

    # Convex collision box for the bed
    model.add_collider("bed", common.make_box((PALLET_W, PALLET_L, PALLET_H + MATTRESS_H + 0.10),
                                              loc=(0.0, 0.0, (PALLET_H + MATTRESS_H + 0.10) / 2),
                                              color="wood_crate"))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build) else 1


if __name__ == "__main__":
    sys.exit(main())
