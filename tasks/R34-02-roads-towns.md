# R34-02: roads reach the home land, Blackwater walkways

Owns: world/src/roads/build.rs, world/src/towns/lots.rs, data/roads.json, data/towns.json
Files: world/src/roads/build.rs, world/src/towns/lots.rs, data/roads.json, data/towns.json
Read only: these 4 files, plus `world/src/towns/mod.rs` and `world/src/generator_contents.rs` (`keep_scattered_prop`, `append_town_contents`) to see how lots are placed.

1. **Roads:** the radial roads start ~2.6 km from camp, so from the home land they can't be seen. Make each trailhead sit at `core_radius + trailhead_margin` (set `trailhead_margin` to 30 in data) so the road runs through the blend band up to the home land edge. The road must NOT enter `core_radius` (test it). Golden test must still pass: the road paint/carving must not reach inside `core_radius`.
2. **Blackwater boardwalks:** today `append_boardwalk_lots` places many `boardwalk_section` pieces in rows on both sides of the street, at right angles, so they look like benches. Instead: one straight line of sections end-to-end (each 4 m long along its local Z, check the yaw) running parallel to the street, in front of each building's door side, from the street edge to the door. At most ~3 sections per building. Keep them off the street itself.
3. Tests: no trailhead closer than `core_radius + 20` or farther than `core_radius + 60`; every boardwalk lot is parallel to the street (yaw matches `street_dir` within 1°) and outside the street's 4 m half width.
