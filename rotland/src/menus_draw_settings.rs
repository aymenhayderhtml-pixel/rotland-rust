//! egui 0.37 rendering for the Settings menu (graphics preset, sensitivity, FOV, scale, audio, FPS).

use engine::egui::{self, vec2, Color32, CornerRadius, Frame, Margin, RichText, ScrollArea, Stroke};

use super::{MenuAction, Menus};

pub fn draw_settings(menus: &mut Menus, ui: &mut egui::Ui) -> Option<MenuAction> {
    let mut changed = false;

    Frame::NONE
        .fill(Color32::from_rgb(26, 26, 28))
        .inner_margin(Margin::same(20))
        .corner_radius(CornerRadius::same(8))
        .stroke(Stroke::new(1.0, Color32::from_rgb(60, 60, 65)))
        .show(ui, |ui| {
            ui.set_width(520.0);
            ui.vertical_centered(|ui| {
                ui.label(RichText::new("Settings").size(28.0).color(Color32::WHITE).strong());
            });
            ui.add_space(14.0);

            ScrollArea::vertical().max_height(380.0).show(ui, |ui| {
                egui::Grid::new("settings_grid")
                    .num_columns(2)
                    .spacing(vec2(24.0, 10.0))
                    .show(ui, |ui| {
                        ui.label(RichText::new("Graphics").size(16.0));
                        let presets = ["Low", "Medium", "High"];
                        let mut cur_idx = match menus.settings.graphics_preset.to_ascii_lowercase().as_str() {
                            "low" => 0,
                            "high" => 2,
                            _ => 1,
                        };
                        let prev_idx = cur_idx;
                        egui::ComboBox::from_id_salt("graphics_preset")
                            .selected_text(presets[cur_idx])
                            .show_ui(ui, |ui| {
                                for (i, name) in presets.iter().enumerate() {
                                    ui.selectable_value(&mut cur_idx, i, *name);
                                }
                            });
                        if cur_idx != prev_idx {
                            menus.settings.graphics_preset = match cur_idx {
                                0 => "low".to_string(),
                                2 => "high".to_string(),
                                _ => "medium".to_string(),
                            };
                            changed = true;
                        }
                        ui.end_row();

                        ui.label(RichText::new("Mouse sensitivity").size(16.0));
                        if ui
                            .add(
                                egui::Slider::new(&mut menus.settings.mouse_sensitivity, 0.2..=3.0)
                                    .step_by(0.1)
                                    .custom_formatter(|v, _| format!("{v:.1}x")),
                            )
                            .changed()
                        {
                            changed = true;
                        }
                        ui.end_row();

                        ui.label(RichText::new("Invert Y axis").size(16.0));
                        if ui.checkbox(&mut menus.settings.invert_y, "").changed() {
                            changed = true;
                        }
                        ui.end_row();

                        ui.label(RichText::new("Field of view").size(16.0));
                        if ui
                            .add(
                                egui::Slider::new(&mut menus.settings.field_of_view, 60.0..=100.0)
                                    .step_by(1.0)
                                    .custom_formatter(|v, _| format!("{v:.0}°")),
                            )
                            .changed()
                        {
                            changed = true;
                        }
                        ui.end_row();

                        ui.label(RichText::new("Render scale").size(16.0));
                        if ui
                            .add(
                                egui::Slider::new(&mut menus.settings.render_scale, 0.5..=1.0)
                                    .step_by(0.05)
                                    .custom_formatter(|v, _| format!("{:.0}%", v * 100.0)),
                            )
                            .changed()
                        {
                            changed = true;
                        }
                        ui.end_row();

                        ui.label(RichText::new("V-Sync").size(16.0));
                        if ui.checkbox(&mut menus.settings.vsync, "").changed() {
                            changed = true;
                        }
                        ui.end_row();

                        ui.label(RichText::new("Master volume").size(16.0));
                        if ui
                            .add(
                                egui::Slider::new(&mut menus.settings.master_volume, 0.0..=1.0)
                                    .step_by(0.05)
                                    .custom_formatter(|v, _| format!("{:.0}%", v * 100.0)),
                            )
                            .changed()
                        {
                            changed = true;
                        }
                        ui.end_row();

                        ui.label(RichText::new("Music and ambience").size(16.0));
                        if ui
                            .add(
                                egui::Slider::new(&mut menus.settings.music_volume, 0.0..=1.0)
                                    .step_by(0.05)
                                    .custom_formatter(|v, _| format!("{:.0}%", v * 100.0)),
                            )
                            .changed()
                        {
                            changed = true;
                        }
                        ui.end_row();

                        ui.label(RichText::new("Sound effects").size(16.0));
                        if ui
                            .add(
                                egui::Slider::new(&mut menus.settings.sfx_volume, 0.0..=1.0)
                                    .step_by(0.05)
                                    .custom_formatter(|v, _| format!("{:.0}%", v * 100.0)),
                            )
                            .changed()
                        {
                            changed = true;
                        }
                        ui.end_row();

                        ui.label(RichText::new("Show FPS").size(16.0));
                        if ui.checkbox(&mut menus.settings.show_fps, "").changed() {
                            changed = true;
                        }
                        ui.end_row();
                    });

                ui.add_space(14.0);
                ui.label(
                    RichText::new(
                        "Developer keys: F5 save, F9 load, F8 new game, K hurt, H heal, Z spawn zombie, N noise, \
                         T skip 3 hours, F6 test map, F7 kit.",
                    )
                    .size(12.0)
                    .color(Color32::from_rgb(140, 140, 140)),
                );
            });

            ui.add_space(14.0);
            ui.vertical_centered(|ui| {
                if ui
                    .add(egui::Button::new(RichText::new("Back (Esc)").size(18.0)).min_size(vec2(180.0, 40.0)))
                    .clicked()
                {
                    menus.press_back();
                }
            });
        });

    if changed {
        menus.settings.clamp();
        Some(MenuAction::SettingsChanged(menus.settings.clone()))
    } else {
        None
    }
}
