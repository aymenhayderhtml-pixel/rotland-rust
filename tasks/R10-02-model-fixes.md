# R10-02: Fix broken model surfaces and the player character (GitHub #12, #1) — PHASE A

Read `tasks/R10-common.md` first. Then read `docs/ART_PIPELINE.md` and `docs/RIG_CONTRACT.md` (joint names must not change). Open the two GitHub issues for the screenshots: `gh issue view 12`, `gh issue view 1`.

## You own
- `tools/blender/**` (the model scripts)
- `baker/src/**` (only if the cause is there)
- The render pipeline's cull/winding setting in `engine/src/render/` — **only the one line(s)** for `cull_mode` / `front_face`, and only if step 1 proves it is the cause.
- Regenerated `.glb` files the scripts export (Claude runs the baker).

## Step 1 — find the shared cause first (one fix may solve many models)
Answer these in your notes, quoting the code lines:
1. What `front_face` and `cull_mode` does the renderer use? (search `engine/src/render/` for `FrontFace` and `cull_mode`)
2. Does the baker flip, reorder or re-wind triangle indices when it reads .glb (glTF is counter-clockwise front faces)? Does it change handedness / mirror any axis (a mirror flips winding)?
3. Do the Blender scripts call `bmesh.ops.recalc_face_normals` (or `bpy.ops.mesh.normals_make_consistent`) before export? Do any use negative scale (mirroring) without applying it?
4. Are any meshes open (a box with no lid = missing faces, not flipped faces)?

Then fix at the right place:
- If winding is flipped for **all** models → fix it once in the baker or renderer, not in every script.
- If only some scripts → add a shared helper in `tools/blender/` (e.g. `finalize_mesh(obj)`: apply scale, recalc normals outward, remove doubles) and call it in every script before export.

## Step 2 — the car (red pickup, #12)
- Close the open top of the body. Add the windows as real faces (dark tinted glass material) instead of holes.
- Tyres: round cylinders (enough sides to look round in low-poly, e.g. 12-16), no black spikes. The spikes are probably a bad cap fill or flipped faces on the tyre caps.
- Small parts (handles, trim strips) must touch the body, not float.
- Keep the car's node names / pivots the same (see `docs/VEHICLE_CONTRACT.md`).

## Step 3 — the player character (#1)
- Hair: a proper cap on top of the head; no band across the eyes; no hole on top.
- Belt: wraps around the waist (a ring the size of the hips), not a flat ring in front.
- Right forearm: no gap at the elbow; the forearm and upper arm overlap slightly at the joint.
- Shoes: sit on the feet (match foot position), no spikes.
- Parts look glued: make sure every part's underside is closed and normals point out, so the camera from below doesn't see inside.
- Joint names, bone count, and rest pose must not change (RIG_CONTRACT).

## Step 4 — check the rest
List every other model script and say for each: fine / fixed (what). At least check zombies, buildings, camp NPCs, weapons.

## Verify
If you can run Blender (`blender -b --python tools/blender/<script>.py`), regenerate the .glb files you changed. Claude runs `cargo run --release -p baker` and takes screenshots. If you can, render a quick Blender preview image of the car and character from front, side and below and list the image paths in your notes.

Add a small Python check script `tools/blender/check_normals.py` that loads a .glb and reports open edges (non-manifold) and inward-facing normals, and run it on the car and player.

## Worker notes
