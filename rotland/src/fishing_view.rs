//! Fishing UI view rendering the bite reaction bar.

use engine::egui::{self, vec2, Align2, Color32, Pos2, Rect};
use game::fishing::{FishingMachine, FishingPhase};

/// Draws the bite timing bar at the bottom center of the screen when fishing.
pub fn draw_fishing_bar(ctx: &egui::Context, machine: &FishingMachine) {
    if machine.phase == FishingPhase::Idle {
        return;
    }

    egui::Area::new(egui::Id::new("rotland_fishing_bar"))
        .order(egui::Order::Foreground)
        .anchor(Align2::CENTER_BOTTOM, vec2(0.0, -100.0))
        .interactable(false)
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(Color32::from_black_alpha(180))
                .corner_radius(6.0)
                .inner_margin(12.0)
                .show(ui, |ui| {
                    match machine.phase {
                        FishingPhase::Waiting => {
                            ui.label(
                                egui::RichText::new("Fishing... waiting for a bite")
                                    .size(16.0)
                                    .color(Color32::from_rgb(200, 220, 240)),
                            );
                        }
                        FishingPhase::Bite => {
                            ui.label(
                                egui::RichText::new("FISH ON! Press Attack in the green zone!")
                                    .size(16.0)
                                    .color(Color32::from_rgb(255, 220, 50)),
                            );
                            ui.add_space(4.0);

                            let (rect, _response) =
                                ui.allocate_exact_size(vec2(220.0, 20.0), egui::Sense::hover());
                            let painter = ui.painter();

                            // Bar background
                            painter.rect_filled(rect, 3.0, Color32::from_black_alpha(200));

                            // Sweet spot zone
                            let sweet_min_x = rect.min.x + machine.sweet_spot_min * rect.width();
                            let sweet_max_x = rect.min.x + machine.sweet_spot_max * rect.width();
                            let sweet_rect = Rect::from_min_max(
                                Pos2::new(sweet_min_x, rect.min.y),
                                Pos2::new(sweet_max_x, rect.max.y),
                            );
                            painter.rect_filled(sweet_rect, 2.0, Color32::from_rgb(46, 204, 113));

                            // Indicator needle
                            let needle_x = rect.min.x + machine.progress() * rect.width();
                            let needle_rect = Rect::from_center_size(
                                Pos2::new(needle_x, rect.center().y),
                                vec2(4.0, rect.height() + 4.0),
                            );
                            painter.rect_filled(needle_rect, 1.0, Color32::WHITE);
                        }
                        FishingPhase::Caught => {
                            ui.label(
                                egui::RichText::new("Caught a fish!")
                                    .size(18.0)
                                    .color(Color32::from_rgb(46, 204, 113)),
                            );
                        }
                        FishingPhase::Escaped => {
                            ui.label(
                                egui::RichText::new("The fish got away...")
                                    .size(16.0)
                                    .color(Color32::from_rgb(231, 76, 60)),
                            );
                        }
                        FishingPhase::Idle => {}
                    }
                });
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fishing_view_draw() {
        let ctx = egui::Context::default();
        let mut machine = FishingMachine::new(&game::FishingConfig::default());
        draw_fishing_bar(&ctx, &machine);
        machine.phase = FishingPhase::Waiting;
        draw_fishing_bar(&ctx, &machine);
        machine.phase = FishingPhase::Bite;
        draw_fishing_bar(&ctx, &machine);
        machine.phase = FishingPhase::Caught;
        draw_fishing_bar(&ctx, &machine);
        machine.phase = FishingPhase::Escaped;
        draw_fishing_bar(&ctx, &machine);
    }
}
