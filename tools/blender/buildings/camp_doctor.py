"""camp_doctor: Medical tent 5x4 m (white canvas, red cross, cot, medicine shelf, npc_1 empty).

Military field hospital tent with white canvas, prominent red cross markings,
patient cot with IV drip stand, stocked medicine supply shelving, doctor's consultation desk,
and npc_1 marker for the camp doctor.
"""
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, HERE)

import common  # noqa: E402
import kit  # noqa: E402

NAME, CATEGORY, BUDGET = "camp_doctor", "buildings", 3000


def build(model):
    k = kit.Kit(model)

    # Duckboard / ground floor (5 x 4 m)
    k.solid((-2.5, -2.0, 0.0), (2.5, 2.0, 0.06), "canvas_khaki", base="tent_floor")

    # Corner tent frame posts
    for px in (-2.45, 2.45):
        for py in (-1.95, 1.95):
            k.box((px - 0.04, py - 0.04, 0.06), (px + 0.04, py + 0.04, 1.75), "bark_dark")
    # Rear ridge support post (at back wall, y = -1.95)
    k.box((-0.04, -1.95, 0.06), (0.04, -1.87, 2.75), "bark_dark")
    # Ridge beam along Y (z = 2.75 m)
    k.box((-0.05, -2.05, 2.70), (0.05, 2.05, 2.80), "bark_dark")

    # Front entrance doorframe posts (x = ±0.85, leaving a clear 1.7 m wide doorway in center!)
    k.box((-0.89, 1.88, 0.06), (-0.81, 1.96, 2.10), "bark_dark")
    k.box((0.81, 1.88, 0.06), (0.89, 1.96, 2.10), "bark_dark")
    k.box((-0.89, 1.88, 2.05), (0.89, 1.96, 2.12), "bark_dark")        # entrance lintel

    # Tent Canvas Walls (height 1.75 m)
    # West wall (full length)
    k.solid((-2.50, -2.0, 0.06), (-2.42, 2.0, 1.75), "white", base="wall_west")
    # East wall (full length)
    k.solid((2.42, -2.0, 0.06), (2.50, 2.0, 1.75), "white", base="wall_east")
    # South wall (rear, full width)
    k.solid((-2.50, -2.0, 0.06), (2.50, -1.92, 1.75), "white", base="wall_south")
    # Rear gable triangle (z from 1.75 to 2.75 m)
    k.gable([(-2.50, -1.96, 1.75), (2.50, -1.96, 1.75), (0.0, -1.96, 2.75)],
            (0.0, 0.08, 0.0), "white", roof=True)

    # Front wall (+Y: y = 2.0 m, entrance opening x: -0.85 to 0.85, open to z=2.1 m)
    # Left front wall
    k.solid((-2.50, 1.92, 0.06), (-0.85, 2.00, 1.75), "white", base="wall_north_w")
    # Right front wall
    k.solid((0.85, 1.92, 0.06), (2.50, 2.00, 1.75), "white", base="wall_north_e")
    # Tied-back canvas door flaps
    k.box((-0.90, 1.88, 0.06), (-0.80, 2.04, 1.95), "white")
    k.box((-0.92, 1.86, 0.95), (-0.78, 2.06, 1.05), "bark_dark")     # tie band
    k.box((0.80, 1.88, 0.06), (0.90, 2.04, 1.95), "white")
    k.box((0.78, 1.86, 0.95), (0.92, 2.06, 1.05), "bark_dark")      # tie band

    # Front gable triangle above entrance (z from 1.75 to 2.75 m)
    k.gable([(-2.50, 1.92, 1.75), (2.50, 1.92, 1.75), (0.0, 1.92, 2.75)],
            (0.0, 0.08, 0.0), "white", roof=True)

    # Sloped Canvas Roof (ridge along Y at z=2.75 m to eaves at z=1.75 m)
    ridge = (0.0, 0.0, 2.75)
    k.roof(ridge, (-2.58, 0.0, 1.72), span=(0.0, 1.0, 0.0), width=4.15, color="white")
    k.roof(ridge, (2.58, 0.0, 1.72), span=(0.0, 1.0, 0.0), width=4.15, color="white")

    # ================= RED CROSS MARKINGS =================
    # 1. Bold Red Cross on front gable (above entrance, y = 2.01)
    k.box((-0.10, 2.01, 2.10), (0.10, 2.03, 2.60), "red_paint")      # vertical bar
    k.box((-0.30, 2.01, 2.27), (0.30, 2.03, 2.43), "red_paint")      # horizontal bar

    # 2. Red Cross on rear gable (y = -2.01)
    k.box((-0.10, -2.03, 2.10), (0.10, -2.01, 2.60), "red_paint")     # vertical bar
    k.box((-0.30, -2.03, 2.27), (0.30, -2.01, 2.43), "red_paint")     # horizontal bar

    # 3. Red Cross on West canvas wall (x = -2.52)
    k.box((-2.52, -0.10, 0.85), (-2.50, 0.10, 1.45), "red_paint")     # vertical bar
    k.box((-2.52, -0.35, 1.05), (-2.50, 0.35, 1.25), "red_paint")     # horizontal bar

    # 4. Red Cross on East canvas wall (x = 2.50)
    k.box((2.50, -0.10, 0.85), (2.52, 0.10, 1.45), "red_paint")      # vertical bar
    k.box((2.50, -0.35, 1.05), (2.52, 0.35, 1.25), "red_paint")      # horizontal bar

    # ================= INTERIOR: MEDICAL COT (x: -2.2 to -1.1, y: -0.7 to 1.1) =================
    # Cot tubular steel frame
    for cx in (-2.15, -1.15):
        for cy in (-0.65, 1.05):
            k.box((cx - 0.02, cy - 0.02, 0.06), (cx + 0.02, cy + 0.02, 0.42), "metal")
    # Frame perimeter rails
    k.box((-2.17, -0.67, 0.38), (-1.13, -0.63, 0.42), "metal")
    k.box((-2.17, 1.03, 0.38), (-1.13, 1.07, 0.42), "metal")
    k.box((-2.17, -0.67, 0.38), (-2.13, 1.07, 0.42), "metal")
    k.box((-1.17, -0.67, 0.38), (-1.13, 1.07, 0.42), "metal")
    # Mattress (solid collider)
    k.solid((-2.15, -0.65, 0.40), (-1.15, 1.05, 0.52), "white", base="cot_mattress")
    # Pillow at head (+Y)
    k.box((-2.05, 0.65, 0.52), (-1.25, 1.00, 0.60), "white")
    # Folded olive wool blanket at foot (-Y)
    k.box((-2.14, -0.64, 0.52), (-1.16, 0.05, 0.56), "cloth_olive")

    # ================= IV DRIP STAND (at x = -0.9, y = 0.95) =================
    # 4-prong base on floor
    k.box((-1.05, 0.93, 0.06), (-0.75, 0.97, 0.10), "metal")
    k.box((-0.92, 0.80, 0.06), (-0.88, 1.10, 0.10), "metal")
    # Tall upright pole to z=1.95 m
    k.box((-0.91, 0.94, 0.10), (-0.89, 0.96, 1.95), "metal")
    # Top curved hanger hook
    k.box((-0.98, 0.94, 1.93), (-0.89, 0.96, 1.97), "metal")
    # Hanging IV saline fluid bag
    k.box((-1.00, 0.93, 1.70), (-0.94, 0.97, 1.90), "water_blue")
    k.box((-0.98, 0.94, 1.72), (-0.96, 0.96, 1.88), "white")
    # Flexible IV drip tube running toward cot
    k.box((-1.08, 0.94, 0.58), (-0.96, 0.96, 1.70), "water_blue")

    # ================= MEDICINE SHELF (x: -0.6 to 0.7, y: -1.90 to -1.55) =================
    # Shelving frame
    for sx in (-0.55, 0.65):
        for sy in (-1.88, -1.58):
            k.box((sx, sy, 0.06), (sx + 0.04, sy + 0.04, 1.80), "white")
    # 4 shelf boards (solid collider)
    for sz in (0.15, 0.65, 1.15, 1.65):
        k.solid((-0.55, -1.88, sz), (0.69, -1.54, sz + 0.04), "white", base="med_shelf")

    # Shelf stock:
    # Tier 1 (z=0.19): Wooden medical supply crates
    k.box((-0.50, -1.85, 0.19), (-0.05, -1.58, 0.55), "wood_crate")
    k.box((-0.30, -1.57, 0.32), (-0.25, -1.56, 0.42), "red_paint")
    k.box((-0.35, -1.57, 0.35), (-0.20, -1.56, 0.39), "red_paint")
    k.box((0.05, -1.85, 0.19), (0.60, -1.58, 0.48), "white")          # white medical box
    # Tier 2 (z=0.69): Medicine bottles and jars
    for i, bx in enumerate((-0.45, -0.32, -0.19, -0.06, 0.08, 0.22, 0.36, 0.50)):
        color = "water_blue" if i % 3 == 0 else ("brass" if i % 3 == 1 else "glass_tint")
        h = 0.15 if i % 2 == 0 else 0.22
        k.box((bx, -1.80, 0.69), (bx + 0.08, -1.68, 0.69 + h), color)
        k.box((bx + 0.02, -1.78, 0.69 + h), (bx + 0.06, -1.70, 0.69 + h + 0.03), "white")
    # Tier 3 (z=1.19): Bandage rolls and pill boxes
    for bx in (-0.48, -0.30, -0.12):
        k.box((bx, -1.82, 1.19), (bx + 0.14, -1.65, 1.35), "white")
    for bx in (0.10, 0.30, 0.48):
        k.box((bx, -1.80, 1.19), (bx + 0.14, -1.64, 1.30), "red_paint")
        k.box((bx + 0.02, -1.81, 1.23), (bx + 0.12, -1.63, 1.26), "white")
    # Tier 4 (z=1.69): Surgical instrument trays
    k.box((-0.45, -1.82, 1.69), (0.05, -1.60, 1.74), "metal")
    k.box((-0.40, -1.80, 1.74), (-0.02, -1.62, 1.76), "metal")
    k.box((0.15, -1.80, 1.69), (0.55, -1.62, 1.78), "glass_tint")

    # ================= DOCTOR'S DESK (x: 1.1 to 2.2, y: -0.6 to 0.4) =================
    # Desk body (solid collider)
    k.solid((1.15, -0.55, 0.06), (2.15, 0.35, 0.78), "wood_crate", base="doctor_desk")
    # Desk kneehole cutout
    k.box((1.15, -0.55, 0.06), (1.20, 0.35, 0.74), "bark_dark")
    k.box((2.10, -0.55, 0.06), (2.15, 0.35, 0.74), "bark_dark")
    # Stool behind desk (x = 1.65, y = -0.9)
    k.solid((1.50, -1.05, 0.06), (1.80, -0.75, 0.48), "bark_dark", base="doctor_stool")

    # Items on doctor's desk:
    # First aid box with red cross
    k.box((1.25, -0.45, 0.78), (1.65, -0.15, 0.95), "white")
    k.box((1.42, -0.46, 0.82), (1.48, -0.44, 0.91), "red_paint")
    k.box((1.37, -0.46, 0.85), (1.53, -0.44, 0.88), "red_paint")
    # Medical clipboard
    k.box((1.75, -0.35, 0.78), (2.05, 0.10, 0.82), "bark_light")
    k.box((1.78, -0.32, 0.82), (2.02, 0.07, 0.83), "white")
    # Stethoscope on desk
    k.box((1.30, 0.05, 0.78), (1.60, 0.28, 0.81), "gun_black")
    k.box((1.55, 0.12, 0.81), (1.62, 0.20, 0.83), "metal")
    # Desk lantern
    k.box((1.90, 0.15, 0.78), (2.05, 0.30, 0.98), "brass")

    # Empties
    k.empty("npc_1", (1.65, -0.85, 0.06), rot_z=0.0)
    k.empty("door_1", (0.0, 2.0, 0.0))
    k.empty("lot_size", (0.0, 0.0, 0.0), scale=(5.0, 4.0, 1.0))


def main():
    return 0 if common.run_model(NAME, CATEGORY, BUDGET, build, view=kit.FRONT_VIEW) else 1


if __name__ == "__main__":
    sys.exit(main())
