//! Visual test example for the Rotland HUD over a plain sky-blue background.
//! Run with `--shot=shots/hud.png` to take a screenshot.

#[path = "../src/hud.rs"]
mod hud;

use engine::egui;
use engine::{EngineArgs, FrameTarget, Game, Gpu, Input};
use hud::{HotbarSlot, HudState};

struct HudView {
    state: HudState,
}

impl Game for HudView {
    fn init(&mut self, _gpu: &Gpu, _args: &EngineArgs) {}

    fn update(&mut self, _input: &Input, dt: f32) {
        // Advance clock slightly
        self.state.hour = (self.state.hour + dt * 0.05).rem_euclid(24.0);
    }

    fn render(&mut self, _gpu: &Gpu, target: &FrameTarget, encoder: &mut wgpu::CommandEncoder) {
        // Plain sky-blue background
        let sky_blue = wgpu::Color {
            r: 0.45,
            g: 0.65,
            b: 0.90,
            a: 1.0,
        };
        let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("clear_sky_blue"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &target.color,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(sky_blue),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &target.depth,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Discard,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
    }

    fn ui(&mut self, ctx: &egui::Context) {
        hud::draw(ctx, &self.state);
    }
}

fn main() {
    let state = HudState {
        health: 78.0,
        max_health: 100.0,
        stamina: 62.0,
        max_stamina: 100.0,
        hour: 14.5,
        day: 3,
        heading_yaw: 0.0,
        hotbar: vec![
            HotbarSlot {
                label: "Axe".into(),
                count: 1,
                selected: true,
            },
            HotbarSlot {
                label: "Pistol".into(),
                count: 1,
                selected: false,
            },
            HotbarSlot {
                label: "Ammo".into(),
                count: 24,
                selected: false,
            },
            HotbarSlot {
                label: "Bandage".into(),
                count: 3,
                selected: false,
            },
            HotbarSlot {
                label: "Canned Food".into(),
                count: 2,
                selected: false,
            },
            HotbarSlot {
                label: "Water".into(),
                count: 1,
                selected: false,
            },
        ],
        ammo: Some((7, 24)),
        fps: 60.0,
        message: Some("Press [E] to talk".into()),
        dead: false,
    };

    engine::run(
        HudView { state },
        "Rotland - HUD View",
        EngineArgs::from_env(),
    );
}
