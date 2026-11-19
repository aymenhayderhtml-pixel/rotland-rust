//! 3D world and scene rendering, skeletal animation poses, batching, and lighting.

use engine::render::DrawItem;
use engine::FrameTarget;
use engine::Gpu;
use glam::{Mat4, Vec3};

use super::start_world::start_world;
use super::state::{ActorsState, AudioState, PlaySide, RenderState, SessionState, UiState, WorldState};
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
    // Cars: the hatchback model at the body's place (its wheels are part of the model for now)
    if let (Some(cars), Some(pack)) = (actors.cars.as_ref(), render.pack.as_ref()) {
        for d in cars.draw_states() {
            if let Some(slot) = render.library.get(&gpu.device, pack, &d.model_name, false) {
                let mut placed = render.library.place(slot, Vec3::ZERO, 0.0, 1.0);
                placed.centre = d.world_matrix.transform_point3(placed.centre);
                placed.matrix = d.world_matrix;
                beds.push(placed);
            }
        }
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
    let placed = streamer.chunks.values().flat_map(|c| c.placed.iter()).chain(beds.iter());
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
        let (_, left, right) = render.car_markers;
        let [l, r] = car_lights::headlight_spots(
            car.draw_state().world_matrix,
            left,
            right,
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
    drop((main, shadow, items));
    render.scratch_beds = beds;
}
