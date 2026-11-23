//! 3D world and scene rendering, skeletal animation poses, batching, and lighting.

use engine::render::DrawItem;
use engine::FrameTarget;
use engine::Gpu;
use glam::{Mat4, Vec3};

use super::start_world::start_world;
use super::state::{ActorsState, AudioState, PlaySide, RenderState, SessionState, UiState, WorldState};
use super::frame_input::build_mode;
use crate::{car_lights, fp_weapon, light};

/// Renders the 3D scene: updates streamed chunks, prepares skeletal instances, sets lighting, and draws passes.
pub fn render_frame(
    gpu: &Gpu,
    target: &FrameTarget,
    encoder: &mut wgpu::CommandEncoder,
    render: &mut RenderState,
    world: &mut WorldState,
    actors: &mut ActorsState,
    play: &mut PlaySide,
    ui: &mut UiState,
    session: &mut SessionState,
    audio: &mut AudioState,
) {
    // New game / Continue: build the picked world here, where the GPU is
    if let Some((seed, file)) = session.pending_world.take() {
        session.seed = seed;
        start_world(gpu, file, render, world, actors, play, ui, session, audio);
    }
    if let Some(bench) = session.bench.as_mut() {
        if bench.fixed_camera.is_none() {
            bench.init_fixed(render.camera, actors.player);
        }
        if let Some(cam) = bench.fixed_camera {
            render.camera = cam;
        }
        if let Some(pos) = bench.fixed_player_pos {
            actors.player = pos;
            if let Some(p) = actors.player_ctrl.as_mut() {
                p.position = pos;
                p.velocity = Vec3::ZERO;
            }
        }
        bench.update_instances(gpu, &mut render.library, render.pack.as_ref(), world);
    }
    let player_pose = actors.player_pose();
    if let (Some(model), Some(pv), Some(pack)) = (
        render.pending_weapon_model.take(),
        actors.player_visual.as_mut(),
        render.pack.as_ref(),
    ) {
        pv.set_weapon(&gpu.device, pack, model.as_deref());
    }
    // Chunk uploads need the GPU, so streaming runs here rather than in update
    if let (Some(streamer), Some(pack)) = (world.streamer.as_mut(), render.pack.as_ref()) {
        streamer.update(
            &gpu.device,
            pack,
            &mut render.library,
            &mut world.physics,
            actors.player,
        );
    }
    // Camp upgrades: walls, gates, towers, workshop, doctor, mechanic
    let camp_site = world
        .world_gen
        .as_ref()
        .and_then(|g| g.settlements().first().cloned());
    if let (Some(play_state), Some(pack), Some(site)) = (
        play.play.as_mut(),
        render.pack.as_ref(),
        camp_site.as_ref(),
    ) {
        let levels = play_state.camp.levels();
        play_state.camp.upgrade_view.update(
            &gpu.device,
            pack,
            &mut render.library,
            &mut world.physics,
            site,
            levels,
        );
    }
    // Farm beds: the soil, and the crop's model for its stage (Godot PlotNode)
    let mut beds = std::mem::take(&mut render.scratch_beds);
    beds.clear();
    if let (Some(play_state), Some(pack)) = (play.play.as_ref(), render.pack.as_ref()) {
        for v in play_state.camp.farm.plot_views() {
            if let Some(slot) = render.library.get(&gpu.device, pack, "soil_plot", false) {
                beds.push(render.library.place(slot, v.position, 0.0, 1.0));
            }
            if let Some(model) = &v.stage_model
                && let Some(slot) = render.library.get(&gpu.device, pack, model, false)
            {
                beds.push(render.library.place(slot, v.position, 0.0, 1.0));
            }
        }
    }
    // Cars: the 4x4 body plus its four wheels, driver door and tailgate as separate parts.
    // Also collects this frame's draw states for the emissive lamps and smoke VFX below.
    let car_draw_states: Vec<game::CarDrawState> = if let (Some(cars), Some(pack)) = (actors.cars.as_ref(), render.pack.as_ref()) {
        let door_hinge = render.car_markers.door_hinge_l;
        let gate_hinge = render.car_markers.tailgate_hinge;
        let states = cars.draw_states();
        let mut parts: Vec<(String, Mat4, Vec3)> = Vec::with_capacity(7);
        for d in &states {
            parts.clear();
            parts.push((d.model_name.clone(), d.world_matrix, Vec3::new(0.0, 0.85, 0.0)));
            for wheel in d.wheel_matrices {
                parts.push(("car_wheel".to_string(), wheel, Vec3::ZERO));
            }
            let door = d.world_matrix
                * Mat4::from_translation(door_hinge)
                * Mat4::from_rotation_y(-d.door_open * 1.10);
            parts.push((format!("car_door_{}", d.colour), door, Vec3::new(0.0, 0.25, 0.45)));
            let gate = d.world_matrix
                * Mat4::from_translation(gate_hinge)
                * Mat4::from_rotation_y(d.trunk_open * 1.35);
            parts.push((format!("car_tailgate_{}", d.colour), gate, Vec3::new(-0.74, 0.15, 0.0)));
            for (name, matrix, centre) in &parts {
                if let Some(slot) = render.library.get(&gpu.device, pack, name, false) {
                    let mut placed = render.library.place(slot, Vec3::ZERO, 0.0, 1.0);
                    placed.centre = matrix.transform_point3(*centre);
                    placed.matrix = *matrix;
                    beds.push(placed);
                }
            }
        }
        states
    } else {
        Vec::new()
    };
    // Base building: placed parts and ghost preview model
    if let Some(pack) = render.pack.as_ref() {
        build_mode::render_placed_and_ghost(
            &mut play.build_mode,
            &gpu.device,
            pack,
            &mut render.library,
            &mut beds,
            &mut world.physics,
        );
    }
    let (Some(renderer), Some(streamer)) = (render.renderer.as_mut(), world.streamer.as_ref()) else {
        render.scratch_beds = beds;
        return;
    };
    let items: Vec<DrawItem> = streamer
        .chunks
        .values()
        .map(|c| DrawItem {
            mesh: &c.ground,
            offset: c.origin,
        })
        .collect();
    // Joint 0 is the identity: static instanced props use it
    render.bones.clear();
    render.bones.push(Mat4::IDENTITY);
    let empty_bench = [];
    let bench_placed: &[crate::props::Placed] = session
        .bench
        .as_ref()
        .map(|b| b.placed_instances.as_slice())
        .unwrap_or(&empty_bench);
    let empty_upgrade = [];
    let upgrade_placed: &[crate::props::Placed] = play
        .play
        .as_ref()
        .map(|p| p.camp.upgrade_view.placed())
        .unwrap_or(&empty_upgrade);
    let placed = streamer
        .chunks
        .values()
        .flat_map(|c| c.placed.iter())
        .chain(beds.iter())
        .chain(upgrade_placed.iter())
        .chain(bench_placed.iter());
    let (mut main, mut shadow) = render.library.batches(
        placed,
        &render.camera,
        target.aspect(),
        renderer.shadows.distance,
    );
    let driving = actors.cars.as_ref().is_some_and(|c| c.driven_car_id.is_some());
    if let (Some(pv), Some(pose), false) = (actors.player_visual.as_mut(), player_pose, driving) {
        let loco = humanoid::Locomotion {
            speed: pose.horizontal_speed,
            on_floor: pose.on_floor,
            crouching: pose.crouching,
        };
        pv.animator.set_move_velocity(pose.velocity, pose.yaw);
        if pose.on_floor && !pose.first_person {
            let physics = &world.physics;
            let (up, down) = {
                let c = humanoid::foot_ik::FootIkConfig::load_shared();
                (c.ray_up, c.ray_down)
            };
            pv.update_foot_ground(pose.position, pose.yaw, |p| {
                physics
                    .raycast(Vec3::new(p.x, pose.position.y + up, p.z), Vec3::NEG_Y, up + down)
                    .map(|h| humanoid::foot_ik::GroundHit { point: h.point, normal: h.normal })
            });
        }
        // The player's yaw already uses the model's facing (checked in a screenshot: no extra turn)
        let (body, weapon) = pv.pose(session.last_dt, pose.position, pose.yaw, loco, &mut render.bones);
        // First person: the body only casts its shadow (like Godot's shadows-only body)
        if !pose.first_person {
            main.push(engine::render::SkinnedBatch {
                mesh: body.mesh,
                instances: body.instances.clone(),
            });
            if let Some(w) = &weapon {
                main.push(engine::render::SkinnedBatch {
                    mesh: w.mesh,
                    instances: w.instances.clone(),
                });
            }
        } else if let Some(w) = &weapon
            && let Some(def) = play
                .play
                .as_ref()
                .and_then(|p| p.gun.as_ref())
                .or_else(|| actors.combat.as_ref().and_then(|c| c.weapon_def()))
        {
            // First person: the weapon sits in front of the camera (Godot FirstPersonMount)
            let at = fp_weapon::fp_weapon_matrix_with_physics(
                &render.camera,
                def,
                actors.fp_sway,
                &world.physics,
            );
            main.push(engine::render::SkinnedBatch {
                mesh: w.mesh,
                instances: vec![engine::render::SkinInstance::new(at, 0)],
            });
        }
        shadow.push(body);
        shadow.extend(weapon);
    }
    let people = actors
        .camp_npcs
        .as_ref()
        .map(|c| c.pose(Some(actors.player), &mut render.bones))
        .unwrap_or_default();
    shadow.extend(
        people
            .iter()
            .map(|b| engine::render::SkinnedBatch {
                mesh: b.mesh,
                instances: b.instances.clone(),
            }),
    );
    main.extend(people);
    let zombies = actors
        .zombies
        .as_ref()
        .map(|z| z.pose(&mut render.bones))
        .unwrap_or_default();
    let mut animals = actors
        .wildlife
        .as_ref()
        .map(|w| w.pose(&mut render.bones))
        .unwrap_or_default();
    animals.extend(
        actors
            .dogs
            .as_ref()
            .map(|d| d.pose(&mut render.bones))
            .unwrap_or_default(),
    );
    if let Some(models) = actors.team_models.as_ref() {
        animals.extend(models.pose(&play.team.figures, &mut render.bones));
    }
    shadow.extend(zombies.iter().map(|b| engine::render::SkinnedBatch {
        mesh: b.mesh,
        instances: b.instances.clone(),
    }));
    main.extend(zombies);
    shadow.extend(animals.iter().map(|b| engine::render::SkinnedBatch {
        mesh: b.mesh,
        instances: b.instances.clone(),
    }));
    main.extend(animals);
    // Campfires: placed ones get stones and logs (the camp's are in its model); lit ones a flame
    if let (Some(meshes), Some(play_state)) = (render.campfire_meshes.as_ref(), play.play.as_ref()) {
        let fires = play_state.camp.campfires.campfires();
        // The fire's position is lifted 0.4 m above the ground (Godot ROOT_LIFT)
        let ground = |f: &game::Campfire| f.position - Vec3::Y * game::ROOT_LIFT;
        let bases: Vec<_> = fires
            .iter()
            .filter(|f| !f.permanent)
            .map(|f| engine::render::SkinInstance::new(Mat4::from_translation(ground(f)), 0))
            .collect();
        let flames: Vec<_> = fires
            .iter()
            .filter(|f| f.is_lit())
            .map(|f| {
                engine::render::SkinInstance::new(
                    Mat4::from_translation(ground(f))
                        * Mat4::from_scale(f.visuals.flame_scale.max(Vec3::splat(0.2))),
                    0,
                )
            })
            .collect();
        if !bases.is_empty() {
            shadow.push(engine::render::SkinnedBatch {
                mesh: &meshes.base,
                instances: bases.clone(),
            });
            main.push(engine::render::SkinnedBatch {
                mesh: &meshes.base,
                instances: bases,
            });
        }
        if !flames.is_empty() {
            main.push(engine::render::SkinnedBatch {
                mesh: &meshes.flame,
                instances: flames,
            });
        }
    }
    // Car emissives (self-lit lenses) and smoke VFX: non-shadow-casting batches in the main pass
    if !car_draw_states.is_empty() || render.car_smoke.is_active() {
        let markers = render.car_markers;
        let emissives = render.car_emissives.get_or_insert_with(|| crate::car_visuals::CarEmissiveMeshes::new(&gpu.device, &markers));
        let instance = |matrix: Mat4| engine::render::SkinInstance::new(matrix, 0);
        let mut headlights = Vec::new();
        let mut taillights = Vec::new();
        let mut brakelights = Vec::new();
        let mut reverselights = Vec::new();
        for d in &car_draw_states {
            if d.lights_on {
                headlights.push(instance(d.world_matrix));
                if !d.braking {
                    taillights.push(instance(d.world_matrix));
                }
            }
            if d.braking {
                brakelights.push(instance(d.world_matrix));
            }
            if d.reversing {
                reverselights.push(instance(d.world_matrix));
            }
        }
        for (instances, mesh) in [
            (headlights, &emissives.headlights),
            (taillights, &emissives.taillights),
            (brakelights, &emissives.brakelights),
            (reverselights, &emissives.reverselights),
        ] {
            if !instances.is_empty() {
                main.push(engine::render::SkinnedBatch { mesh, instances });
            }
        }
        // Smoke and embers from the pooled system (grow, spin, fade over the puff's life)
        let (light_puffs, dark_puffs, embers) = render.car_smoke.instances();
        for (instances, mesh) in [
            (light_puffs, &emissives.smoke_light),
            (dark_puffs, &emissives.smoke_dark),
            (embers, &emissives.ember),
        ] {
            if !instances.is_empty() {
                main.push(engine::render::SkinnedBatch { mesh, instances });
            }
        }
    }
    let mut lighting = light::at_hour(
        play.hour,
        render.flashlight_on.then(|| {
            // Held at the camera, pointing where it looks (Godot hangs it on the camera)
            light::flashlight(render.camera.position, render.camera.forward())
        }),
    );
    // The nearest car with its lights on lights the road (Godot CarLights); the renderer has 3 spots
    if let Some(cars) = &actors.cars
        && let Some(car) = cars
            .live_cars()
            .filter(|c| c.lights_on)
            .min_by(|a, b| {
                a.body
                    .position
                    .distance(actors.player)
                    .total_cmp(&b.body.position.distance(actors.player))
            })
    {
        let [l, r] = car_lights::headlight_spots(
            car.draw_state().world_matrix,
            render.car_markers.headlight_l,
            render.car_markers.headlight_r,
            true,
            &cars.config,
        );
        lighting.spots[1] = l;
        lighting.spots[2] = r;
    }
    renderer.draw(
        gpu,
        target,
        encoder,
        &render.camera,
        &lighting,
        &items,
        &main,
        &shadow,
        &render.bones,
    );
    crate::water_view::draw(
        gpu,
        target,
        encoder,
        &render.camera,
        &lighting,
        session.time,
        world,
        actors.player,
    );
    drop((main, shadow, items));
    if let (Some(bench), Some(r)) = (session.bench.as_mut(), render.renderer.as_ref()) {
        bench.step_frame(session.last_dt, &r.last, &mut session.quit);
    }
    render.scratch_beds = beds;
}
