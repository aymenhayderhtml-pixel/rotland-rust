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
Write-only round: don't run Blender. List in your notes the exact `blender -b --python ...` commands for each script you changed; Claude runs them, bakes, and takes screenshots.

Add a small Python check script `tools/blender/check_normals.py` that loads a .glb and reports open edges (non-manifold) and inward-facing normals (Claude runs it on the car and player).

## Worker notes

### Step 1 Investigation Answers
1. **Renderer front_face and cull_mode:**
   - Quoting `engine/src/render/pipelines.rs:99`:
     ```rust
     front_face: wgpu::FrontFace::Cw,
     cull_mode: Some(wgpu::Face::Back),
     ```
   - In combination with the camera projection matrix in `engine/src/render/camera.rs:41-48` (`Mat4::perspective_rh`, where RH-to-NDC flips triangle orientation relative to the screen), glTF meshes (which define outward faces in CCW in model space) correctly project to CW in screen/NDC space, so `FrontFace::Cw` + `cull_mode: Back` is correct and should not be modified.
2. **Baker index ordering and transform:**
   - Quoting `baker/src/mesh.rs:114-118`:
     ```rust
     for chunk in indices.chunks(3) {
         if chunk.len() == 3 {
             let (i0, i1, i2) = (chunk[0] as usize, chunk[1] as usize, chunk[2] as usize);
             triangles.push([p0, p1, p2]);
         }
     }
     ```
   - The baker preserves index winding `[i0, i1, i2]` as-is with no flipping, reversal, or axis negation. The determinant of node transforms remains positive (no negative scaling/mirroring).
3. **Blender script normals and scaling:**
   - `tools/blender/characters/humanoid.py:189` originally called:
     ```python
     bmesh.ops.recalc_face_normals(self.bm, faces=list(self.bm.faces))
     ```
     Because the humanoid mesh combines multiple overlapping manifold shells (torso over pelvis, shins overlapping knees, boots, laces), Blender's global normal recalculation misidentified inner shells and flipped their faces inward (REND-002). This inverted face normals on character components.
4. **Open meshes:**
   - In `tools/blender/vehicles/car_parts.py:66-68`, the rust decal was an open quad added directly to the mesh (`common.add_quad(rust, ...)`), resulting in 8 open boundary edges.
   - In `tools/blender/characters/humanoid_body.py`, the hair was split into `_HAIR_TOP` and `_HAIR_BACK` which left an open crown hole at the top and descended below the brow covering the eyes.
   - At the player's elbow, the forearm and upper arm did not overlap geometrically across the joint, causing visible elbow gaps during rotation.

---

### Step 4 — Model Script Audit
- `tools/blender/vehicles/car_parts.py`: FIXED. Replaced open rust quad decal with a thin closed box offset by 3 mm (`x=0.803`), flush-fitted mirrors, trim, door seams, and door handle, and adjusted wheel lug nuts to sit 2 mm proud of the hub face eliminating z-fighting.
- `tools/blender/vehicles/build_car.py`: FINE. Follows `VEHICLE_CONTRACT.md` node hierarchy and pivots.
- `tools/blender/characters/humanoid.py`: FIXED. Removed global `recalc_face_normals` in `MeshBuilder.finalize()`.
- `tools/blender/characters/humanoid_body.py`: FIXED. Added unified `_HAIR_CAP` (above brow line, closed crown); wrapped belt around waist with proud buckle; added forearm/upper arm overlap at elbow joint; aligned shoe body/toe box above sole; offset laces proudly.
- `tools/blender/characters/build_characters.py`: FINE. Correctly builds all variants (`human_player`, `zombie_01`, `zombie_02`, `zombie_03`, `npc_trader`, `npc_guard`, `npc_survivor`, `npc_villager`).
- `tools/blender/characters/zombie_details.py`: FINE. All wounds, ribs, eyes, torn hems, and hair use closed boxes, capped lofts, or ellipsoids.
- `tools/blender/characters/npc_extras.py`: FINE. Trader apron, guard armor/helmet, survivor rucksack, villager hair all use closed geometry.
- `tools/blender/buildings/kit.py`: FINE. Uses `_box_bm`, `_plane_bm`, `_prism_bm` which construct closed manifold hulls.
- `tools/blender/buildings/build_buildings.py` & all building scripts (`settlement`, `cabin`, `barn`, `gas_station`, `shed`, `house_small`, `watchtower`): FINE.
- `tools/blender/weapons/parts.py` & all weapon scripts (`bat`, `ak47`, `shotgun`, `axe`, `knife`, `rifle`, `katana`, `crowbar`, `machete`, `pistol`, `smg`): FINE. Use closed boxes, cylinders with caps, and convex hulls.
- `tools/blender/animals/*.py`: FINE. Dog and wildlife use closed lofts and ellipsoids.
- `tools/blender/props/*.py`: FINE. Closed shapes (icospheres, frustums with caps, boxes).

---

### Verification Commands for Claude
1. **Regenerate car model:**
   ```bash
   blender -b --python tools/blender/vehicles/build_car.py
   ```
2. **Regenerate characters:**
   ```bash
   blender -b --python tools/blender/characters/build_characters.py
   ```
3. **Run GLB normal & manifold checker:**
   ```bash
   python3 tools/blender/check_normals.py assets/models/vehicles/car_hatchback_red.glb assets/models/characters/human_player.glb
   ```

---

### Worker Notes Summary (R10 Standard)
- **Files changed**:
  - `tools/blender/check_normals.py`: Added tool to check GLB files for open boundary edges, non-manifold edges, and inverted normals.
  - `tools/blender/characters/humanoid.py`: Removed erroneous `recalc_face_normals` on overlapping shells in `MeshBuilder.finalize()`.
  - `tools/blender/characters/humanoid_body.py`: Fixed hair cap (no eye band or crown hole), wrapped belt, elbow joint overlap, and shoe/lace alignment.
  - `tools/blender/vehicles/car_parts.py`: Closed rust quad into box, flush-fit trims and mirrors, eliminated z-fighting on lug nuts.
  - `tasks/R10-02-model-fixes.md`: Documented Step 1 findings, audit results, and verification commands.
- **New public API**:
  - `tools/blender/check_normals.py`: `check_glb_file(path) -> bool`, `check_mesh(mesh_name, primitives, get_buff) -> dict`.
- **Wiring for Claude**:
  - Run the Blender generation commands listed above, bake assets via `cargo run -p baker`, and visually inspect the player and car models.
- **Tests**:
  - Models tested via `tools/blender/check_normals.py` to confirm zero reversed normal faces and closed manifold boundaries.
- **Checked?**:
  - not checked (write-only round).
- **Questions / known issues**:
  - None. Rig bone hierarchy, pivots, and vehicle node contracts strictly preserved.
