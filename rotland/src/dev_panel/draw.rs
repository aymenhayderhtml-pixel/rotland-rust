//! Immediate-mode egui 0.37 layout and frame-time graph for the F1 developer overlay.

use engine::egui::{
    self, pos2, vec2, Align2, Color32, CornerRadius, FontId, RichText, Sense, Stroke, StrokeKind,
};
use game::dev_panel::{
    filter_items, parse_at_flag, FpsHistory, FPS_HISTORY_CAPACITY, FRAME_MS_30FPS, FRAME_MS_60FPS,
};
use glam::Vec3;
use world::time::Clock;

use super::actions::DevAction;
use super::{DevPanelState, DevPanelView};

const PANEL_DEFAULT_WIDTH: f32 = 340.0; // new: DECISIONS 2026-11-20
const GRAPH_HEIGHT: f32 = 76.0; // new: DECISIONS 2026-11-20
const GRAPH_MIN_WIDTH: f32 = 220.0; // new: DECISIONS 2026-11-20
const GRAPH_MIN_SCALE_MS: f32 = 40.0; // new: DECISIONS 2026-11-20
const GRAPH_SCALE_HEADROOM: f32 = 1.1; // new: DECISIONS 2026-11-20
const GRAPH_FONT_SIZE: f32 = 9.5; // new: DECISIONS 2026-11-20
const COORD_FIELD_WIDTH: f32 = 68.0; // new: DECISIONS 2026-11-20
const ITEM_SEARCH_WIDTH: f32 = 155.0; // new: DECISIONS 2026-11-20
const ITEM_LIST_HEIGHT: f32 = 135.0; // new: DECISIONS 2026-11-20
const STATUS_FONT_SIZE: f32 = 12.0; // new: DECISIONS 2026-11-20
const MAX_GIVE_COUNT: u32 = 999; // new: DECISIONS 2026-11-20
const HOURS_PER_DAY: f32 = 24.0; // new: DECISIONS 2026-11-20
const NOON_HOUR: f32 = 12.0; // new: DECISIONS 2026-11-20
const MS_PER_SECOND: f32 = 1000.0; // new: DECISIONS 2026-11-20

/// Draws the floating "Dev Panel" window and returns any actions triggered this frame.
pub fn draw_dev_panel(
    ctx: &egui::Context,
    state: &mut DevPanelState,
    view: &DevPanelView<'_>,
) -> Vec<DevAction> {
    let mut actions = Vec::new();
    let mut open = state.open;

    egui::Window::new("Dev Panel")
        .open(&mut open)
        .resizable(true)
        .default_width(PANEL_DEFAULT_WIDTH)
        .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-12.0, 56.0))
        .show(ctx, |ui| {
            draw_perf_section(ui, &state.history, view);
            ui.separator();
            draw_teleport_section(ui, state, view, &mut actions);
            ui.separator();
            draw_time_section(ui, view, &mut actions);
            ui.separator();
            draw_zombies_section(ui, state, view, &mut actions);
            ui.separator();
            draw_items_section(ui, state, view, &mut actions);
            ui.separator();

            let mut god = state.god_mode;
            if ui
                .checkbox(&mut god, "God Mode (invulnerable + full needs)")
                .changed()
            {
                actions.push(DevAction::SetGodMode(god));
            }
            if let Some(msg) = &state.status_msg {
                ui.add_space(4.0);
                ui.label(
                    RichText::new(msg)
                        .size(STATUS_FONT_SIZE)
                        .color(Color32::from_rgb(175, 225, 175)),
                );
            }
        });

    state.open = open;
    actions
}

fn draw_perf_section(ui: &mut egui::Ui, history: &FpsHistory, view: &DevPanelView<'_>) {
    ui.label(RichText::new("Performance & Stats").strong());
    let latest_ms = history.as_slice().last().copied().unwrap_or_else(|| {
        if view.fps > 0.0 {
            MS_PER_SECOND / view.fps
        } else {
            0.0
        }
    });
    ui.monospace(format!(
        "FPS: {:.1} ({:.1} ms) | Draws: {} | Shd: {}",
        view.fps, latest_ms, view.draws, view.shadow_draws
    ));
    ui.monospace(format!(
        "Tris: {} | Shadow tris: {}",
        view.triangles, view.shadow_triangles
    ));
    ui.monospace(format!(
        "Pos: ({:.1}, {:.1}, {:.1}) | Yaw: {:.0} deg",
        view.player_pos.x,
        view.player_pos.y,
        view.player_pos.z,
        view.player_yaw.to_degrees()
    ));
    draw_frame_graph(ui, history);
}

fn draw_frame_graph(ui: &mut egui::Ui, history: &FpsHistory) {
    let width = ui.available_width().max(GRAPH_MIN_WIDTH);
    let (rect, _) = ui.allocate_exact_size(vec2(width, GRAPH_HEIGHT), Sense::hover());
    let painter = ui.painter().with_clip_rect(rect);

    painter.rect(
        rect,
        CornerRadius::same(3),
        Color32::from_rgb(18, 20, 24),
        Stroke::new(1.0, Color32::from_rgb(65, 70, 78)),
        StrokeKind::Inside,
    );

    let y_max = (history.max_ms() * GRAPH_SCALE_HEADROOM).max(GRAPH_MIN_SCALE_MS);
    let ms_to_y = |ms: f32| rect.bottom() - (ms / y_max).clamp(0.0, 1.0) * rect.height();

    // Horizontal guide lines: 16.7 ms (60 FPS target) and 33.3 ms (30 FPS minimum).
    let guides = [
        (FRAME_MS_60FPS, "16.7ms", Color32::from_rgb(125, 215, 95)),
        (FRAME_MS_30FPS, "33.3ms", Color32::from_rgb(235, 110, 70)),
    ];
    for (ms, label, color) in guides {
        let y = ms_to_y(ms);
        painter.line_segment(
            [pos2(rect.left(), y), pos2(rect.right(), y)],
            Stroke::new(1.0, color),
        );
        painter.text(
            pos2(rect.right() - 3.0, y - 1.0),
            Align2::RIGHT_BOTTOM,
            label,
            FontId::monospace(GRAPH_FONT_SIZE),
            color,
        );
    }

    let samples = history.as_slice();
    if samples.len() >= 2 {
        let denom = (FPS_HISTORY_CAPACITY - 1) as f32;
        let start_slot = FPS_HISTORY_CAPACITY - samples.len();
        let points: Vec<egui::Pos2> = samples
            .iter()
            .enumerate()
            .map(|(i, &ms)| {
                let x = rect.left() + ((start_slot + i) as f32 / denom) * rect.width();
                pos2(x, ms_to_y(ms))
            })
            .collect();
        painter.line(points, Stroke::new(1.5, Color32::from_rgb(110, 195, 255)));
    }

    painter.text(
        pos2(rect.left() + 4.0, rect.top() + 2.0),
        Align2::LEFT_TOP,
        format!(
            "avg {:.1}ms | max {:.1}ms ({}/{})",
            history.avg_ms(),
            history.max_ms(),
            history.len(),
            FPS_HISTORY_CAPACITY
        ),
        FontId::monospace(GRAPH_FONT_SIZE),
        Color32::from_rgb(200, 206, 214),
    );
}

fn draw_teleport_section(
    ui: &mut egui::Ui,
    state: &mut DevPanelState,
    view: &DevPanelView<'_>,
    actions: &mut Vec<DevAction>,
) {
    ui.label(RichText::new("Teleport").strong());
    ui.horizontal(|ui| {
        let camp_btn = ui.add_enabled(view.camp_spawn.is_some(), egui::Button::new("Camp Spawn"));
        if camp_btn.clicked()
            && let Some(pos) = view.camp_spawn
        {
            actions.push(DevAction::Teleport(pos));
        }

        let poi_label = match &view.nearest_poi {
            Some((name, pos)) => {
                let dist = (pos.x - view.player_pos.x).hypot(pos.z - view.player_pos.z);
                format!("Nearest POI: {name} ({dist:.0}m)")
            }
            None => "Nearest POI (none)".to_string(),
        };
        let poi_btn = ui.add_enabled(view.nearest_poi.is_some(), egui::Button::new(poi_label));
        if poi_btn.clicked()
            && let Some((_, pos)) = &view.nearest_poi
        {
            actions.push(DevAction::Teleport(*pos));
        }
    });

    ui.horizontal(|ui| {
        ui.label("X:");
        let rx = ui.add(
            egui::TextEdit::singleline(&mut state.custom_x)
                .desired_width(COORD_FIELD_WIDTH)
                .hint_text("0.0"),
        );
        ui.label("Z:");
        let rz = ui.add(
            egui::TextEdit::singleline(&mut state.custom_z)
                .desired_width(COORD_FIELD_WIDTH)
                .hint_text("0.0"),
        );
        let enter_pressed =
            (rx.lost_focus() || rz.lost_focus()) && ui.input(|i| i.key_pressed(egui::Key::Enter));
        if ui.button("Go").clicked() || enter_pressed {
            let combined = format!("{},{}", state.custom_x, state.custom_z);
            if let Some((x, z)) = parse_at_flag(&combined) {
                actions.push(DevAction::Teleport(Vec3::new(x, 0.0, z)));
            } else {
                state.status_msg = Some("Invalid X/Z coordinates".to_string());
            }
        }
        if ui.button("Here").clicked() {
            state.custom_x = format!("{:.1}", view.player_pos.x);
            state.custom_z = format!("{:.1}", view.player_pos.z);
        }
    });
}

fn draw_time_section(ui: &mut egui::Ui, view: &DevPanelView<'_>, actions: &mut Vec<DevAction>) {
    ui.label(RichText::new("Time of Day").strong());
    let mut hour = view.current_hour;
    ui.horizontal(|ui| {
        ui.monospace(Clock::format_clock(hour));
        if ui
            .add(egui::Slider::new(&mut hour, 0.0..=HOURS_PER_DAY).text("h"))
            .changed()
        {
            actions.push(DevAction::SetTime(hour));
        }
    });
    ui.horizontal(|ui| {
        let mut frozen = view.clock_frozen;
        if ui.checkbox(&mut frozen, "Freeze time").changed() {
            actions.push(DevAction::SetFreezeTime(frozen));
        }
        if ui.button("Dawn").clicked() {
            actions.push(DevAction::SetTime(Clock::DAY_BREAK_HOUR as f32));
        }
        if ui.button("Noon").clicked() {
            actions.push(DevAction::SetTime(NOON_HOUR));
        }
        if ui.button("Night").clicked() {
            actions.push(DevAction::SetTime(Clock::NIGHT_START_HOUR as f32));
        }
    });
}

fn draw_zombies_section(
    ui: &mut egui::Ui,
    state: &mut DevPanelState,
    view: &DevPanelView<'_>,
    actions: &mut Vec<DevAction>,
) {
    ui.label(RichText::new("Zombies").strong());
    ui.horizontal(|ui| {
        let selected_label = state.selected_zombie_type.clone();
        egui::ComboBox::from_id_salt("dev_zombie_type")
            .selected_text(selected_label)
            .show_ui(ui, |ui| {
                for &kind in view.zombie_types {
                    ui.selectable_value(&mut state.selected_zombie_type, kind.to_string(), kind);
                }
            });
        if ui.button("Spawn (8m)").clicked() {
            actions.push(DevAction::SpawnZombie(state.selected_zombie_type.clone()));
        }
        if ui.button("Kill (<50m)").clicked() {
            actions.push(DevAction::KillZombiesNear);
        }
    });
}

fn draw_items_section(
    ui: &mut egui::Ui,
    state: &mut DevPanelState,
    view: &DevPanelView<'_>,
    actions: &mut Vec<DevAction>,
) {
    ui.label(RichText::new("Items").strong());
    ui.horizontal(|ui| {
        ui.add(
            egui::TextEdit::singleline(&mut state.item_search)
                .desired_width(ITEM_SEARCH_WIDTH)
                .hint_text("Search items..."),
        );
        if ui.button("-").clicked() {
            state.give_count = state.give_count.saturating_sub(1).max(1);
        }
        ui.add(egui::DragValue::new(&mut state.give_count).range(1..=MAX_GIVE_COUNT));
        if ui.button("+").clicked() {
            state.give_count = state.give_count.saturating_add(1).min(MAX_GIVE_COUNT);
        }
    });

    let id_refs: Vec<&str> = view.all_item_ids.iter().map(String::as_str).collect();
    let matches = filter_items(&id_refs, &state.item_search);

    egui::ScrollArea::vertical()
        .id_salt("dev_item_list")
        .max_height(ITEM_LIST_HEIGHT)
        .show(ui, |ui| {
            if matches.is_empty() {
                ui.label(RichText::new("No matching items").weak());
                return;
            }
            for item_id in matches {
                ui.horizontal(|ui| {
                    ui.monospace(item_id);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Give").clicked() {
                            actions.push(DevAction::GiveItem {
                                id: item_id.to_string(),
                                count: state.give_count.max(1),
                            });
                        }
                    });
                });
            }
        });
}
