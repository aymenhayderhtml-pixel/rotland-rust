//! New Halden's streets on the HUD mini-map: the same stroke style as Ashford's streets.

use engine::egui::{self, Color32, Pos2, Stroke};
use rotland_ui::metro_ui::MinimapStreet;

/// Strokes the metro streets (pixel offsets from the mini-map centre), clipped to the disc.
pub(super) fn draw_metro_streets(
    painter: &egui::Painter,
    centre: Pos2,
    streets: &[MinimapStreet],
    disk_r: f32,
    scale: f32,
    color: Color32,
) {
    for street in streets {
        let Some((ca, cb)) = game::clip_segment_to_circle(street.a, street.b, disk_r) else {
            continue;
        };
        let pa = Pos2::new(centre.x + ca.x, centre.y + ca.y);
        let pb = Pos2::new(centre.x + cb.x, centre.y + cb.y);
        painter.line_segment([pa, pb], Stroke::new(street.width_px * scale, color));
    }
}
