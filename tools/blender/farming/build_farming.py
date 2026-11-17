"""Builds every farming model into assets/models/farming/ (plus previews).

Run: blender -b --python tools/blender/farming/build_farming.py
Each model must stay within 200 triangles (ModelTestUtil.Budgets).
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, HERE)
import common  # noqa: E402
import farming_parts as parts  # noqa: E402

CATEGORY, BUDGET = "farming", 200

MODELS = [("soil_plot", parts.soil_plot), ("watering_can", parts.watering_can), ("hoe", parts.hoe)]
for crop in ("wheat", "potato", "carrot"):
    for stage in ("sprout", "young", "ripe"):
        MODELS.append((f"{crop}_{stage}", parts.crop_builder(crop, stage)))


def main():
    ok = True
    for name, build in MODELS:
        within = common.run_model(name, CATEGORY, BUDGET, build)
        ok = within and ok
        print("MODEL", name, "OK" if within else "OVER BUDGET")
    print("FARMING MODELS", "OK" if ok else "FAILED")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
