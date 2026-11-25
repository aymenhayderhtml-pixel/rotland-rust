//! Rotland (Rust). R0: the real world with real trees and 25 zombies, for the go/no-go speed test.
//! Options: --scene=forest|camp (forest = Godot's forest25 benchmark walk), --seed=N (42), --hour=H (12),
//! --free (fly with mouse + WASD instead of the scripted walk), plus the engine's --shot / --stats / --seconds.

mod app;
mod bench;
mod bubbles;
mod camp;
mod camp_npcs;
mod campfire_mesh;
mod car_lights;
mod car_solid;
mod car_visuals;
mod combat;
mod combat_bash;
mod combat_damage;
mod companion_model;
mod companion_view;
mod crates;
mod dev_panel;
mod dog_view;
mod downed_play;
mod fishing_view;
mod fp_weapon;
mod light;
mod play;
mod player_visual;
mod poi_links;
mod props;
mod region_banner;
mod rescue_play;
mod rescue_view;
mod river_view;
mod session;
mod siege_view;
mod guild_task_view;
mod swim_view;
mod temperature_play;
mod toxic_cloud_view;
mod weather_play;
mod shooting;
mod streaming;
mod terrain;
mod trunk_play;
mod wildlife;
mod water_view;
mod zombies;

pub use rotland_ui::{camp_ui, display, hub_ui, hud, keys_ui, map_ui, menus, settings};

use engine::{EngineArgs, FrameTarget, Game, GameStats, Gpu, Input};

pub use app::{PlayerPose, R0};

impl Game for R0 {
    fn init(&mut self, gpu: &Gpu, args: &EngineArgs) {
        crate::water_view::init(args);
        app::startup::init(
            gpu,
            args,
            &mut self.render,
            &mut self.world,
            &mut self.actors,
            &mut self.play,
            &mut self.ui,
            &mut self.session,
            &mut self.audio,
        );
    }

    fn update(&mut self, input: &Input, dt: f32) {
        self.session.last_dt = dt;
        self.session.time += dt;

        if self.session.scene == "play" && !self.session.free_cam {
            let outcome = app::frame_input::process_input(
                input,
                dt,
                &mut self.render,
                &self.world,
                &mut self.actors,
                &mut self.play,
                &mut self.ui,
                &mut self.session,
            );
            let (actions, attack, dead) = match outcome {
                app::frame_input::InputOutcome::MenuOpen => return,
                app::frame_input::InputOutcome::Active {
                    actions,
                    attack,
                    dead,
                } => (actions, attack, dead),
            };

            app::frame_actors::update_actors(
                input,
                dt,
                &actions,
                attack,
                dead,
                &mut self.render,
                &mut self.world,
                &mut self.actors,
                &mut self.play,
                &mut self.ui,
                &mut self.session,
                &mut self.audio.audio,
            );
            app::frame_water::update_water_play(
                dt,
                attack,
                f64::from(self.session.time),
                &mut self.render,
                &mut self.play,
                &mut self.actors,
                &self.world,
            );
        } else {
            app::frame_actors::update_walk_or_free(
                input,
                dt,
                &mut self.render,
                &self.world,
                &mut self.actors,
                &mut self.session,
            );
        }

        app::frame_actors::check_scripted_timers(
            &mut self.session,
            &mut self.render,
            &self.world,
            &mut self.actors,
            &mut self.play,
            &mut self.audio,
        );

        app::frame_world::update_needs(
            dt,
            &mut self.play,
            &mut self.actors,
            &mut self.audio.audio,
        );

        app::frame_warmth::update_weather_and_warmth(dt, &mut self.play, &self.world, &mut self.actors);

        app::frame_world::update_world_interactions(
            input,
            dt,
            &self.render,
            &mut self.world,
            &mut self.actors,
            &mut self.play,
            &mut self.ui,
            &self.session,
            &mut self.audio.audio,
        );

        app::frame_audio::update_audio(
            dt,
            &self.render,
            &self.actors,
            &self.play,
            &self.ui,
            &mut self.audio,
        );

        app::frame_camp::update_camp_and_stats(
            dt,
            &mut self.render,
            &mut self.world,
            &mut self.actors,
            &mut self.play,
            &mut self.ui,
            &self.session,
            &mut self.audio,
        );

        app::frame_sim::update_sim(
            dt,
            &mut self.render,
            &mut self.world,
            &mut self.actors,
            &mut self.play,
            &mut self.ui,
            &self.session,
            &mut self.audio,
        );
        crate::downed_play::tick(
            dt,
            &mut self.actors,
            &mut self.play,
            &self.session,
            &mut self.audio.audio,
        );
    }

    fn render(&mut self, gpu: &Gpu, target: &FrameTarget, encoder: &mut wgpu::CommandEncoder) {
        app::render_frame::render_frame(
            gpu,
            target,
            encoder,
            &mut self.render,
            &mut self.world,
            &mut self.actors,
            &mut self.play,
            &mut self.ui,
            &mut self.session,
            &mut self.audio,
        );
    }

    fn ui(&mut self, ctx: &engine::egui::Context) {
        app::frame_ui::draw_ui(
            ctx,
            &mut self.render,
            &mut self.world,
            &mut self.actors,
            &mut self.play,
            &mut self.ui,
            &mut self.session,
        );
    }

    fn display(&self) -> Option<engine::DisplayRequest> {
        app::stats::display(&self.ui)
    }

    fn cursor_grab(&self) -> Option<bool> {
        app::stats::cursor_grab(&self.ui, &self.session)
    }

    fn should_quit(&self) -> bool {
        app::stats::should_quit(&self.session)
    }

    fn stats(&self) -> GameStats {
        app::stats::get_stats(
            &self.render,
            &self.world,
            &self.actors,
            &self.play,
            &self.ui,
            &self.session,
            &self.audio,
        )
    }
}

fn main() {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info,wgpu_core=warn,wgpu_hal=warn"),
    )
    .init();
    engine::run(
        R0::default(),
        "Rotland (Rust)",
        EngineArgs::from_env(),
    );
}
