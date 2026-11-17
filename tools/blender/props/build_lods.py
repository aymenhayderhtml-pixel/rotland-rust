"""Batch build all vegetation LOD models."""
import os
import subprocess
import sys

LOD_SCRIPTS = [
    "tree_pine_01_lod1.py",
    "tree_pine_01_lod2.py",
    "tree_pine_02_lod1.py",
    "tree_pine_02_lod2.py",
    "pine_tall_lod1.py",
    "pine_tall_lod2.py",
    "tree_broadleaf_01_lod1.py",
    "tree_broadleaf_01_lod2.py",
    "tree_oak_big_lod1.py",
    "tree_oak_big_lod2.py",
    "tree_birch_lod1.py",
    "tree_birch_lod2.py",
    "tree_autumn_lod1.py",
    "tree_autumn_lod2.py",
    "tree_dead_lod1.py",
    "tree_dead_lod2.py",
    "bush_01_lod1.py",
]

here = os.path.dirname(os.path.abspath(__file__))
for script in LOD_SCRIPTS:
    path = os.path.join(here, script)
    print(f"=== Building {script} ===")
    mod_name = os.path.splitext(script)[0]
    # Import and run directly within Blender
    import importlib.util
    spec = importlib.util.spec_from_file_location(mod_name, path)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    res = mod.main()
    if res != 0:
        print(f"FAILED {script}")
        sys.exit(1)
print("All vegetation LODs built successfully!")
