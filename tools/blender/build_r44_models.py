"""Builds the R44 safe house, rooftop camp and rope ladder models (R44-02).

Run: `flock /tmp/rotland-blender.lock blender -b --python tools/blender/build_r44_models.py`
Prints FAILED lines and exits 1 if a model is over budget, missing, or does not re-import.
Ends with `R44 BUILD OK` when everything passes.
"""
import os
import sys

import bpy

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, "towers"))

import common  # noqa: E402
import rooftop_camp  # noqa: E402
import safehouse  # noqa: E402

GROUPS = (("safehouse", safehouse.build_all), ("rooftop_camp", rooftop_camp.build_all))
NAMES = (
    "safehouse_shack", "safehouse_bed", "safehouse_chest", "travel_sign",
    "rope_ladder_seg", "rope_ladder_coil", "survivor_tent", "green_smoke_barrel",
    "green_smoke_plume", "rooftop_trader_stall",
)


def _reimports(path):
    """Opens the exported .glb again in a clean scene; True if it brings in at least one mesh."""
    common.reset_scene()
    bpy.ops.import_scene.gltf(filepath=path)
    return any(obj.type == "MESH" for obj in bpy.data.objects)


def main():
    failures = []
    for group, build_all in GROUPS:
        for name, ok in build_all():
            print(f"R44 {group}: {name} {'OK' if ok else 'FAILED'}")
            if not ok:
                failures.append(name)
    for name in NAMES:
        path = os.path.join(common.MODELS_DIR, "props", name + ".glb")
        if not os.path.exists(path):
            failures.append(name + " (missing)")
        elif not _reimports(path):
            failures.append(name + " (re-import)")
    if failures:
        print("R44 BUILD FAILED:", ", ".join(failures))
        return 1
    print("R44 BUILD OK")
    return 0


if __name__ == "__main__":
    sys.exit(main())
