//! Hint system dialog box widget.
//!
//! Port of Godot C# `scripts/UI/HintPanel.cs`.

use engine::egui::{self, vec2, Align2, Color32, Rect};

/// Draws the tip box at the left-middle of the screen: dark rounded panel with 18 px text.
pub fn draw_hint(ctx: &egui::Context, text: &str) {
    let screen: Rect = ctx.content_rect();
    egui::Area::new(egui::Id::unique("rotland_hint"))
        .order(egui::Order::Foreground)
        .anchor(Align2::LEFT_CENTER, vec2(16.0, 0.0))
        .interactable(false)
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(Color32::from_black_alpha(158))
                .corner_radius(6.0)
                .inner_margin(10.0)
                .show(ui, |ui| {
                    ui.set_width(280.0_f32.min(screen.width() - 52.0));
                    ui.label(
                        egui::RichText::new(text)
                            .size(18.0)
                            .color(Color32::from_rgb(242, 242, 230)),
                    );
                });
        });
}
