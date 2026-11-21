//! egui 0.37 tabbed Settings menu (Video, Audio, Controls, Gameplay).

use engine::egui::{
    self, Align, Frame, Id, Layout, RichText, ScrollArea, Stroke, TextWrapMode, vec2,
};

use super::theme::{self, ButtonTone};
use crate::menus::{MenuAction, Menus};
use crate::settings::{
    MAX_FOV, MAX_RENDER_SCALE, MAX_SENSITIVITY, MIN_FOV, MIN_RENDER_SCALE, MIN_SENSITIVITY,
};

const SAVED_FLASH_SECS: f64 = 2.0; // new: DECISIONS 2026-11-21

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Hash)]
enum SettingsTab {
    #[default]
    Video,
    Audio,
    Controls,
    Gameplay,
}

impl SettingsTab {
    const ALL: [Self; 4] = [Self::Video, Self::Audio, Self::Controls, Self::Gameplay];

    fn label(self) -> &'static str {
        match self {
            Self::Video => "Video",
            Self::Audio => "Audio",
            Self::Controls => "Controls",
            Self::Gameplay => "Gameplay",
        }
    }
}

pub fn draw_settings(
    menus: &mut Menus,
    ui: &mut egui::Ui,
    screen_height: f32,
    scale: f32,
) -> Option<MenuAction> {
    let mut changed = false;

    let tab_id = Id::unique("rotland_settings_tab");
    let saved_id = Id::unique("rotland_settings_saved_at");
    let prev_tab = ui
        .ctx()
        .data_mut(|d| *d.get_temp_mut_or_default::<SettingsTab>(tab_id));
    let mut tab = prev_tab;
    let now = ui.ctx().input(|i| i.time);
    let saved_at = ui.ctx().data(|d| d.get_temp::<f64>(saved_id));
    let show_saved = saved_at.is_some_and(|t| now >= t && (now - t) < SAVED_FLASH_SECS);

    // Wide enough to comfortably contain keys_ui's 540px inner frame plus scrollbars at all scales.
    let inner_w = (640.0 * scale).max(600.0); // new: DECISIONS 2026-11-21
    let content_h = (screen_height * 0.54).clamp(340.0 * scale, 540.0 * scale); // new: DECISIONS 2026-11-21
    let row_w = inner_w - 18.0 * scale; // new: DECISIONS 2026-11-21

    let frame_resp = theme::panel_frame(scale, theme::BORDER).show(ui, |ui| {
        ui.set_width(inner_w);
        ui.vertical(|ui| {
            // Header row
            ui.horizontal(|ui| {
                ui.set_width(inner_w);
                ui.add(
                    egui::Label::new(
                        RichText::new("SETTINGS")
                            .size(theme::FONT_HEADING * scale)
                            .color(theme::TEXT_PRIMARY)
                            .strong(),
                    )
                    .wrap_mode(TextWrapMode::Extend),
                );
                if show_saved {
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.add(
                            egui::Label::new(
                                RichText::new("Settings saved")
                                    .size(theme::FONT_HELP * scale)
                                    .color(theme::SAFE)
                                    .strong(),
                            )
                            .wrap_mode(TextWrapMode::Extend),
                        );
                    });
                }
            });

            ui.add_space(12.0 * scale);

            // Top tab bar: Video | Audio | Controls | Gameplay
            let tab_gap = 6.0 * scale; // new: DECISIONS 2026-11-21
            let tab_w = (inner_w - tab_gap * (SettingsTab::ALL.len() as f32 - 1.0))
                / SettingsTab::ALL.len() as f32;
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = tab_gap;
                for candidate in SettingsTab::ALL {
                    if theme::tab_button(ui, candidate.label(), tab_w, scale, tab == candidate)
                        .clicked()
                    {
                        tab = candidate;
                    }
                }
            });

            if tab != prev_tab {
                menus.keys_ui.cancel();
                ui.ctx().data_mut(|d| d.insert_temp(tab_id, tab));
            }

            theme::paint_divider(ui, inner_w, scale);
            ui.add_space(12.0 * scale);

            // Fixed-height scrollable tab body prevents panel jumps between tabs
            ScrollArea::vertical()
                .id_salt(("settings_tab_scroll", tab))
                .max_height(content_h)
                .show(ui, |ui| {
                    ui.set_width(row_w);
                    ui.set_min_height(content_h);
                    match tab {
                        SettingsTab::Video => changed |= draw_video_tab(menus, ui, row_w, scale),
                        SettingsTab::Audio => changed |= draw_audio_tab(menus, ui, row_w, scale),
                        SettingsTab::Controls => {
                            changed |= draw_controls_tab(menus, ui, row_w, scale);
                        }
                        SettingsTab::Gameplay => {
                            changed |= draw_gameplay_tab(menus, ui, row_w, scale);
                        }
                    }
                });

            ui.add_space(12.0 * scale);
            theme::paint_divider(ui, inner_w, scale);
            ui.add_space(14.0 * scale);

            // Bottom action bar: Back (left) and Apply (right)
            let btn_size = vec2(180.0 * scale, theme::BTN_HEIGHT * scale); // new: DECISIONS 2026-11-21
            ui.horizontal(|ui| {
                ui.set_width(inner_w);
                if theme::menu_button(ui, "Back (Esc)", btn_size, scale, ButtonTone::Default, true)
                    .clicked()
                {
                    menus.press_back();
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if theme::menu_button(ui, "Apply", btn_size, scale, ButtonTone::Safe, true)
                        .clicked()
                    {
                        menus.settings.clamp();
                        menus.save_settings();
                        ui.ctx().data_mut(|d| d.insert_temp(saved_id, now));
                        changed = true;
                    }
                });
            });
        });
    });

    theme::paint_top_accent(ui, frame_resp.response.rect, theme::ACCENT, scale);

    if changed {
        menus.settings.clamp();
        Some(MenuAction::SettingsChanged(menus.settings.clone()))
    } else {
        None
    }
}

fn draw_video_tab(menus: &mut Menus, ui: &mut egui::Ui, row_w: f32, scale: f32) -> bool {
    let mut changed = false;

    setting_row(
        ui,
        row_w,
        scale,
        "Graphics",
        Some("Balances shadow, foliage, and draw-distance detail."),
        |ui| {
            let presets = ["Low", "Medium", "High"];
            let mut cur_idx = match menus.settings.graphics_preset.to_ascii_lowercase().as_str() {
                "low" => 0,
                "high" => 2,
                _ => 1,
            };
            let prev_idx = cur_idx;
            egui::ComboBox::from_id_salt("graphics_preset")
                .width(160.0 * scale)
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
        },
    );

    setting_row(
        ui,
        row_w,
        scale,
        "Field of view",
        Some("Wider angles show more of the world around you."),
        |ui| {
            if ui
                .add(
                    egui::Slider::new(&mut menus.settings.field_of_view, MIN_FOV..=MAX_FOV)
                        .step_by(1.0)
                        .custom_formatter(|v, _| format!("{v:.0}°")),
                )
                .changed()
            {
                changed = true;
            }
        },
    );

    setting_row(
        ui,
        row_w,
        scale,
        "Render scale",
        Some("Lower values boost frame rate while keeping menus crisp."),
        |ui| {
            if ui
                .add(
                    egui::Slider::new(
                        &mut menus.settings.render_scale,
                        MIN_RENDER_SCALE..=MAX_RENDER_SCALE,
                    )
                    .step_by(0.05)
                    .custom_formatter(|v, _| format!("{:.0}%", v * 100.0)),
                )
                .changed()
            {
                changed = true;
            }
        },
    );

    setting_row(
        ui,
        row_w,
        scale,
        "V-Sync",
        Some("Syncs frame rate to your display to prevent tearing."),
        |ui| {
            if ui.checkbox(&mut menus.settings.vsync, "").changed() {
                changed = true;
            }
        },
    );

    changed
}

fn draw_audio_tab(menus: &mut Menus, ui: &mut egui::Ui, row_w: f32, scale: f32) -> bool {
    let mut changed = false;

    setting_row(
        ui,
        row_w,
        scale,
        "Master volume",
        Some("Overall level for all game audio channels."),
        |ui| {
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
        },
    );

    setting_row(ui, row_w, scale, "Music and ambience", None, |ui| {
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
    });

    setting_row(ui, row_w, scale, "Sound effects", None, |ui| {
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
    });

    changed
}

fn draw_controls_tab(menus: &mut Menus, ui: &mut egui::Ui, row_w: f32, scale: f32) -> bool {
    let mut changed = false;

    setting_row(
        ui,
        row_w,
        scale,
        "Mouse sensitivity",
        Some("Adjusts camera turn speed when looking around."),
        |ui| {
            if ui
                .add(
                    egui::Slider::new(
                        &mut menus.settings.mouse_sensitivity,
                        MIN_SENSITIVITY..=MAX_SENSITIVITY,
                    )
                    .step_by(0.1)
                    .custom_formatter(|v, _| format!("{v:.1}x")),
                )
                .changed()
            {
                changed = true;
            }
        },
    );

    setting_row(ui, row_w, scale, "Invert Y axis", None, |ui| {
        if ui.checkbox(&mut menus.settings.invert_y, "").changed() {
            changed = true;
        }
    });

    ui.add_space(10.0 * scale);
    if crate::keys_ui::draw_keys(ui, &mut menus.keys, &mut menus.keys_ui, menus.pressed_key) {
        menus.settings.key_bindings = menus.keys.overrides();
        changed = true;
    }

    changed
}

fn draw_gameplay_tab(menus: &mut Menus, ui: &mut egui::Ui, row_w: f32, scale: f32) -> bool {
    let mut changed = false;

    setting_row(
        ui,
        row_w,
        scale,
        "Show FPS",
        Some("Displays the live frame-rate counter on the HUD."),
        |ui| {
            if ui.checkbox(&mut menus.settings.show_fps, "").changed() {
                changed = true;
            }
        },
    );

    setting_row(
        ui,
        row_w,
        scale,
        "Show mini-map",
        Some("Shows the navigation mini-map while exploring."),
        |ui| {
            if ui.checkbox(&mut menus.settings.show_minimap, "").changed() {
                changed = true;
            }
        },
    );

    ui.add_space(16.0 * scale);
    Frame::NONE
        .fill(theme::CARD_BG)
        .stroke(Stroke::new(1.0, theme::BORDER))
        .corner_radius(theme::radius(theme::CORNER_CONTROL, scale))
        .inner_margin(theme::margin(14.0, scale))
        .show(ui, |ui| {
            ui.set_width(row_w - 28.0 * scale);
            ui.add(
                egui::Label::new(
                    RichText::new("DEVELOPER SHORTCUTS")
                        .size(12.0 * scale) // new: DECISIONS 2026-11-21
                        .color(theme::ACCENT)
                        .strong(),
                )
                .wrap_mode(TextWrapMode::Extend),
            );
            ui.add_space(4.0 * scale);
            ui.label(
                RichText::new(
                    "F5 save · F9 load · F8 new game · K hurt · H heal · Z spawn zombie · \
                     N noise · T skip 3 hours · F6 test map · F7 kit.",
                )
                .size(theme::FONT_HELP * scale)
                .color(theme::TEXT_MUTED),
            );
        });

    changed
}

/// Renders one settings row: label on the left, control on the right, optional muted help line below.
fn setting_row(
    ui: &mut egui::Ui,
    row_w: f32,
    scale: f32,
    label: &str,
    help: Option<&str>,
    control: impl FnOnce(&mut egui::Ui),
) {
    ui.vertical(|ui| {
        ui.set_width(row_w);
        ui.horizontal(|ui| {
            ui.set_width(row_w);
            ui.set_min_height(30.0 * scale); // new: DECISIONS 2026-11-21
            ui.add(
                egui::Label::new(
                    RichText::new(label)
                        .size(theme::FONT_BODY * scale)
                        .color(theme::TEXT_PRIMARY),
                )
                .wrap_mode(TextWrapMode::Extend),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), control);
        });

        if let Some(help_text) = help {
            ui.add(
                egui::Label::new(
                    RichText::new(help_text)
                        .size(theme::FONT_HELP * scale)
                        .color(theme::TEXT_MUTED),
                )
                .wrap_mode(TextWrapMode::Truncate),
            );
        }

        ui.add_space(6.0 * scale);
        theme::paint_divider(ui, row_w, scale);
        ui.add_space(6.0 * scale);
    });
}
