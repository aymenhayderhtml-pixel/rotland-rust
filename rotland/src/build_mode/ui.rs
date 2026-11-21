//! egui UI rendering for the build mode part picker, costs, status messages, and ghost tint overlay.

use engine::egui::{self, Align2, Color32, FontId, Pos2, Rect, RichText, Stroke, vec2};
use engine::render::Camera;
use game::Inventory;
use glam::Vec4;

use super::state::BuildModeState;
use super::types::part_display_name;

/// Draws the build mode egui UI: part picker, costs, status panel, and projected ghost tint overlay.
#[allow(dead_code)]
pub fn draw_build_mode_ui(
    ctx: &egui::Context,
    state: &BuildModeState,
    inventory: Option<&Inventory>,
    camera: &Camera,
) {
    if !state.active {
        return;
    }

    let screen_rect = ctx.content_rect();
    let aspect = screen_rect.width() / screen_rect.height().max(1.0);
    let view_proj = camera.view_proj(aspect);

    // 1. Projected 2D tint overlay for the ghost preview in the 3D world
    if let Some(placement) = &state.current_placement {
        let pos = placement.position;
        let clip = view_proj * Vec4::new(pos.x, pos.y + 0.5, pos.z, 1.0);
        if clip.w > 0.05 {
            let ndc_x = clip.x / clip.w;
            let ndc_y = clip.y / clip.w;
            if (-1.2..=1.2).contains(&ndc_x) && (-1.2..=1.2).contains(&ndc_y) {
                let sx = (ndc_x * 0.5 + 0.5) * screen_rect.width() + screen_rect.min.x;
                let sy = (-ndc_y * 0.5 + 0.5) * screen_rect.height() + screen_rect.min.y;

                let is_ok = state.last_error.is_none();
                let (fill, stroke_col) = if is_ok {
                    (
                        Color32::from_rgba_unmultiplied(40, 220, 80, 50),
                        Color32::from_rgb(60, 240, 100),
                    )
                } else {
                    (
                        Color32::from_rgba_unmultiplied(220, 40, 40, 50),
                        Color32::from_rgb(255, 70, 70),
                    )
                };

                let painter = ctx.layer_painter(egui::LayerId::new(
                    egui::Order::Middle,
                    egui::Id::unique("build_mode_ghost_overlay"),
                ));

                let box_size = 32.0;
                let rect = Rect::from_center_size(Pos2::new(sx, sy), vec2(box_size, box_size));
                painter.rect_filled(rect, 4.0, fill);
                painter.rect_stroke(rect, 4.0, Stroke::new(2.0, stroke_col), egui::StrokeKind::Middle);
            }
        }
    }

    // 2. Part picker and status panel (bottom-center of screen)
    let selected_id = state.selected_part();
    let part_name = part_display_name(selected_id);
    let def = state.config.get(selected_id);

    egui::Area::new(egui::Id::unique("build_mode_bottom_panel"))
        .anchor(Align2::CENTER_BOTTOM, vec2(0.0, -90.0))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            let frame = egui::Frame::window(ui.style())
                .fill(Color32::from_rgba_unmultiplied(20, 22, 28, 230))
                .stroke(Stroke::new(1.0, Color32::from_rgb(90, 95, 110)))
                .inner_margin(egui::Margin::symmetric(14, 8));

            frame.show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(
                        RichText::new("BUILD MODE")
                            .font(FontId::proportional(12.0))
                            .color(Color32::from_rgb(255, 200, 70))
                            .strong(),
                    );

                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(format!("<  {part_name}  >"))
                                .font(FontId::proportional(17.0))
                                .color(Color32::WHITE)
                                .strong(),
                        );
                        ui.label(
                            RichText::new(format!("(Rot: {}°)", state.rotation * 90))
                                .font(FontId::proportional(13.0))
                                .color(Color32::from_rgb(180, 185, 200)),
                        );
                    });

                    // Cost line
                    if let Some(def) = def {
                        let mut cost_parts = Vec::new();
                        for (item, qty) in &def.cost {
                            let available = inventory.map(|inv| inv.count(item)).unwrap_or(0);
                            let color = if available >= *qty {
                                Color32::from_rgb(120, 220, 120)
                            } else {
                                Color32::from_rgb(255, 100, 100)
                            };
                            cost_parts.push((format!("{item}: {qty} (have {available})"), color));
                        }

                        if !cost_parts.is_empty() {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Cost: ").color(Color32::from_rgb(170, 175, 190)));
                                for (text, col) in cost_parts {
                                    ui.label(RichText::new(text).color(col).strong());
                                }
                            });
                        }
                    }

                    // Placement status message
                    if let Some(err) = state.last_error {
                        ui.label(
                            RichText::new(format!("Cannot place: {err}"))
                                .font(FontId::proportional(13.0))
                                .color(Color32::from_rgb(255, 90, 90))
                                .strong(),
                        );
                    } else if state.current_placement.is_some() {
                        ui.label(
                            RichText::new("Ready to place (Left Click)")
                                .font(FontId::proportional(13.0))
                                .color(Color32::from_rgb(80, 240, 120))
                                .strong(),
                        );
                    }

                    // Toast message if active
                    if let Some(ref toast) = state.toast {
                        ui.label(
                            RichText::new(toast)
                                .font(FontId::proportional(12.0))
                                .color(Color32::from_rgb(240, 220, 130)),
                        );
                    }

                    // Controls cheat sheet
                    ui.label(
                        RichText::new("[Wheel / 1-9] Cycle  [R] Rotate  [LMB] Place  [RMB] Remove (refund)  [E] Door  [Esc/B] Exit")
                            .font(FontId::proportional(11.0))
                            .color(Color32::from_rgb(150, 155, 170)),
                    );
                });
            });
        });
}
