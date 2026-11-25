# R35-05: terrain colour detail, shore foam, river flow

Owns: world/src/terrain_paint.rs (outer biomes + a light core variation layer), engine/src/render/water.wgsl, engine/src/render/water.rs
Files: world/src/terrain_paint.rs, engine/src/render/water.wgsl, engine/src/render/water.rs
Read only: these files, rotland/src/river_view.rs, rotland/src/water_view.rs, world/tests/golden.rs (to see what it checks).

- Terrain: add patchy colour variation (two noise scales) and darker damp ground near water, lighter dry patches in desert, blue shade in snow hollows. The core may get variation ONLY if golden.rs does not check colours; if it does, outer biomes only. Grass must stay green.
- Water: a thin white foam line where water meets the shore (from depth difference, cheap), and a flowing ripple pattern on rivers along their direction (pass flow dir per vertex or use mesh UV). Lakes: gentle ripple only.
- Report anything the lead must wire.
