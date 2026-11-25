# R34-03: speed (after the lead's Phase 0)

Owns: whatever the lead's measurement points to, inside engine/src/render/*, rotland/src/*, world/src/scatter.rs. Tell the lead which files you changed.
Files: engine/src/render/*, rotland/src/*, world/src/scatter.rs, docs/PERF_R34.md

Goal: forest scene back to >= 115 FPS avg (R31 was 121, now ~105) without making the world look emptier. Camp must not get slower.
Read ONLY the files the lead's numbers point to. Typical wins, in order: fewer draw calls for scattered props (instancing/batching per model), shadow casters culled by distance (small props cast no shadow beyond ~40 m), cheaper far trees (LOD). Every tuning number gets `// new: PERF R34`.
Write in `docs/PERF_R34.md` what you changed and why.
