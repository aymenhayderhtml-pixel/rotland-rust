//! egui 0.37 controls / key rebinding panel.
//! Port of `scripts/UI/ControlsPanel.cs`.

use engine::egui::{self, vec2, Color32, CornerRadius, Frame, Margin, RichText, ScrollArea, Stroke};
use game::{KeyBindings, KEYBINDS_ALL_ACTIONS};
use winit::keyboard::KeyCode;

/// UI state for the key rebinding panel.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct KeysUiState {
    /// The action currently waiting for a key press, or None.
    pub waiting: Option<String>,
    /// Active conflict asking the player whether to swap: (action, other_action, new_code).
    pub conflict: Option<(String, String, u32)>,
}

impl KeysUiState {
    /// Creates a new default `KeysUiState`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Puts the UI into waiting mode for `action`.
    pub fn begin_rebind(&mut self, action: &str) {
        self.waiting = Some(action.to_string());
        self.conflict = None;
    }

    /// Cancels any active rebind wait or conflict dialog.
    pub fn cancel(&mut self) {
        self.waiting = None;
        self.conflict = None;
    }

    /// Returns true if currently waiting for a key press.
    pub fn is_waiting(&self) -> bool {
        self.waiting.is_some()
    }

    /// The action name currently waiting for a key press.
    pub fn waiting_action(&self) -> Option<&str> {
        self.waiting.as_deref()
    }

    /// Returns true if a conflict swap confirmation is open.
    pub fn has_conflict(&self) -> bool {
        self.conflict.is_some()
    }
}

/// Renders the key rebinding controls panel into an egui UI.
/// Returns true if any key binding was changed.
pub fn draw_keys(
    ui: &mut egui::Ui,
    bindings: &mut KeyBindings,
    state: &mut KeysUiState,
    pressed_key: Option<KeyCode>,
) -> bool {
    let mut changed = false;

    // Handle key press input if waiting for a key
    if let Some(key) = pressed_key {
        if key == KeyCode::Escape {
            state.cancel();
        } else if let Some(action) = state.waiting.take() {
            if let Some(code) = KeyBindings::key_to_godot_code(key, false) {
                if let Some(other) = bindings.conflict_with(&action, code) {
                    state.conflict = Some((action, other.to_string(), code));
                } else {
                    bindings.set(&action, code);
                    changed = true;
                }
            }
        }
    }

    Frame::NONE
        .fill(Color32::from_rgb(26, 26, 28))
        .inner_margin(Margin::same(20))
        .corner_radius(CornerRadius::same(8))
        .stroke(Stroke::new(1.0, Color32::from_rgb(60, 60, 65)))
        .show(ui, |ui| {
            ui.set_width(540.0);
            ui.vertical_centered(|ui| {
                ui.label(RichText::new("Controls").size(24.0).color(Color32::WHITE).strong());
            });
            ui.add_space(10.0);

            // Active conflict dialog
            if let Some((action, other, code)) = state.conflict.clone() {
                ui.vertical_centered(|ui| {
                    let desc = KeyBindings::describe(code);
                    let other_label = KeyBindings::label_of(&other);
                    ui.label(
                        RichText::new(format!("{desc} is already used by {other_label}: swap?"))
                            .size(15.0)
                            .color(Color32::from_rgb(240, 200, 100)),
                    );
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.set_width(280.0);
                        if ui.button(RichText::new("Swap").size(15.0)).clicked() {
                            bindings.swap(&action, &other);
                            state.conflict = None;
                            changed = true;
                        }
                        if ui.button(RichText::new("Cancel").size(15.0)).clicked() {
                            state.conflict = None;
                        }
                    });
                });
                ui.add_space(12.0);
            }

            ScrollArea::vertical()
                .id_salt("controls_keybinds_scroll")
                .max_height(340.0)
                .show(ui, |ui| {
                    let mut current_section = "";
                    for bind in KEYBINDS_ALL_ACTIONS {
                        if bind.section != current_section {
                            current_section = bind.section;
                            ui.add_space(6.0);
                            ui.label(
                                RichText::new(current_section)
                                    .size(16.0)
                                    .strong()
                                    .color(Color32::from_rgb(143, 194, 74)),
                            );
                            ui.add_space(4.0);
                        }

                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(bind.label)
                                    .size(15.0)
                                    .color(Color32::from_rgb(200, 200, 205)),
                            );

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let is_this_waiting = state.waiting.as_deref() == Some(bind.action);
                                let btn_text = if is_this_waiting {
                                    "Press a key...".to_string()
                                } else {
                                    bindings.key_name(bind.action)
                                };

                                let btn_color = if is_this_waiting {
                                    Color32::from_rgb(255, 215, 0)
                                } else {
                                    Color32::WHITE
                                };

                                let btn = egui::Button::new(
                                    RichText::new(btn_text).size(15.0).color(btn_color),
                                )
                                .min_size(vec2(140.0, 28.0));

                                if ui.add(btn).clicked() {
                                    if is_this_waiting {
                                        state.cancel();
                                    } else {
                                        state.begin_rebind(bind.action);
                                    }
                                }
                            });
                        });
                        ui.add_space(2.0);
                    }
                });

            ui.add_space(12.0);
            ui.label(
                RichText::new("Click a key, then press the new key. Esc cancels.")
                    .size(13.0)
                    .color(Color32::from_rgb(150, 150, 155)),
            );
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                if ui
                    .add(
                        egui::Button::new(RichText::new("Reset to defaults").size(16.0))
                            .min_size(vec2(180.0, 36.0)),
                    )
                    .clicked()
                {
                    bindings.reset_all();
                    state.cancel();
                    changed = true;
                }
            });
        });

    changed
}
