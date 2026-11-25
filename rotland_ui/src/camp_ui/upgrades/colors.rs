//! Colors, states, formatting, and text helpers for camp upgrades.

use engine::egui::Color32;
use game::camp_upgrades::UpgradeLine;

use crate::menus::theme;

use super::super::view::{
    CampUpgradesView, NextUpgradeView, UpgradeInProgressView, UpgradeLineView,
};

const WALL_HP_WARN: f32 = 0.5; // moved: rotland/src/camp_ui/draw_upgrades.rs
const WALL_HP_CRIT: f32 = 0.25; // moved: rotland/src/camp_ui/draw_upgrades.rs

/// Affordability state of a single cash or material cost row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CostRowState {
    Enough,
    Short,
}

pub fn cash_cost_state(have_cash: i32, need_cash: i32) -> CostRowState {
    if have_cash >= need_cash {
        CostRowState::Enough
    } else {
        CostRowState::Short
    }
}

pub fn item_cost_state(have: u32, need: u32) -> CostRowState {
    if have >= need {
        CostRowState::Enough
    } else {
        CostRowState::Short
    }
}

pub fn cost_row_color(state: CostRowState) -> Color32 {
    match state {
        CostRowState::Enough => theme::SAFE_HOVER,
        CostRowState::Short => theme::DANGER_HOVER,
    }
}

/// Formats in-game hours as e.g. `"3 h 20 min"`, `"2 h"`, `"30 min"`, or `"< 1 min"`.
pub fn format_hours(hours: f32) -> String {
    if !hours.is_finite() || hours <= 0.0 {
        return "0 min".to_string();
    }
    let total_mins = (hours * 60.0).round() as u32; // moved: rotland/src/camp_ui/draw_upgrades.rs
    if total_mins == 0 {
        return "< 1 min".to_string();
    }
    let h = total_mins / 60; // moved: rotland/src/camp_ui/draw_upgrades.rs
    let m = total_mins % 60; // moved: rotland/src/camp_ui/draw_upgrades.rs
    match (h, m) {
        (0, mins) => format!("{mins} min"),
        (hrs, 0) => format!("{hrs} h"),
        (hrs, mins) => format!("{hrs} h {mins} min"),
    }
}

pub fn format_building_banner(target_name: &str, hours_left: f32) -> String {
    format!("Building: {target_name} — {} left", format_hours(hours_left))
}

pub fn format_cash_cost(have_cash: i32, need_cash: i32) -> String {
    format!("${have_cash}/${need_cash}")
}

pub fn format_item_cost(have: u32, need: u32) -> String {
    format!("{have}/{need}")
}

pub fn upgrade_progress_fraction(hours_left: f32, total_hours: f32) -> f32 {
    if !total_hours.is_finite() || total_hours <= 0.0 {
        return if hours_left.is_finite() && hours_left <= 0.0 {
            1.0
        } else {
            0.0
        };
    }
    let remaining = if hours_left.is_finite() {
        hours_left.clamp(0.0, total_hours)
    } else {
        total_hours
    };
    ((total_hours - remaining) / total_hours).clamp(0.0, 1.0)
}

pub fn wall_hp_fraction(wall_hp: f32, wall_hp_max: f32) -> f32 {
    if !wall_hp_max.is_finite() || wall_hp_max <= 0.0 || !wall_hp.is_finite() {
        return 0.0;
    }
    (wall_hp / wall_hp_max).clamp(0.0, 1.0)
}

pub fn wall_hp_color(fraction: f32) -> Color32 {
    if fraction > WALL_HP_WARN {
        theme::SAFE
    } else if fraction > WALL_HP_CRIT {
        theme::ACCENT
    } else {
        theme::DANGER
    }
}

pub fn format_wall_hp_label(wall_hp: f32, wall_hp_max: f32) -> String {
    if !wall_hp_max.is_finite() || wall_hp_max <= 0.0 {
        return "No walls".to_string();
    }
    let cur = if wall_hp.is_finite() {
        wall_hp.clamp(0.0, wall_hp_max)
    } else {
        0.0
    };
    if cur <= 0.0 {
        format!("0 / {wall_hp_max:.0} (Breached)")
    } else {
        format!("{cur:.0} / {wall_hp_max:.0}")
    }
}

pub fn refund_percent(refund_ratio: f32) -> u32 {
    if !refund_ratio.is_finite() {
        return 0;
    }
    (refund_ratio.clamp(0.0, 1.0) * 100.0).round() as u32 // moved: rotland/src/camp_ui/draw_upgrades.rs
}

pub fn cancel_button_label(refund_ratio: f32) -> String {
    format!("Cancel (refund {}%)", refund_percent(refund_ratio))
}

pub fn line_display_name(line: UpgradeLine) -> &'static str {
    match line {
        UpgradeLine::Walls => "Walls & Gates",
        UpgradeLine::Guards => "Guards & Towers",
        UpgradeLine::Workshop => "Workshop",
        UpgradeLine::Traders => "Traders & Services",
    }
}

pub fn current_summary_text(line: &UpgradeLineView) -> String {
    let trimmed = line.current_summary.trim();
    if !trimmed.is_empty() {
        trimmed.to_string()
    } else if line.current_level == 0 {
        "None".to_string()
    } else {
        format!("Level {}", line.current_level)
    }
}

pub fn resolve_in_progress_name(
    prog: &UpgradeInProgressView,
    upgrades: &CampUpgradesView,
) -> String {
    let trimmed = prog.target_name.trim();
    if !trimmed.is_empty() {
        return trimmed.to_string();
    }
    if let Some(line_view) = upgrades.line(prog.line)
        && let Some(next) = &line_view.next
        && !next.name.trim().is_empty()
    {
        return next.name.trim().to_string();
    }
    format!("{} Lv.{}", line_display_name(prog.line), prog.target_level)
}

pub fn effective_cant_start_reason<'a>(
    next: &'a NextUpgradeView,
    in_progress: Option<&UpgradeInProgressView>,
    line: UpgradeLine,
) -> Option<&'a str> {
    if let Some(reason) = next.cant_start_reason.as_deref() {
        let trimmed = reason.trim();
        return Some(if trimmed.is_empty() {
            "Unavailable"
        } else {
            trimmed
        });
    }
    if let Some(prog) = in_progress {
        return Some(if prog.line == line {
            "Currently building"
        } else {
            "Another upgrade is in progress"
        });
    }
    None
}
