//! Pure layout calculation for the Camp Board upgrade cards grid.

use engine::egui::{self, Rect, Vec2, vec2};

/// Calculated responsive layout for the upgrade cards grid.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CardLayout {
    pub columns: usize,
    pub card_width: f32,
    pub card_height: f32,
    pub font_size: f32,
    pub rows: usize,
    pub gap: f32,
}

impl CardLayout {
    pub fn card_size(&self) -> Vec2 {
        vec2(self.card_width, self.card_height)
    }

    pub fn total_width(&self) -> f32 {
        (self.columns as f32) * self.card_width
            + (self.columns.saturating_sub(1) as f32) * self.gap
    }

    pub fn total_height(&self) -> f32 {
        (self.rows as f32) * self.card_height
            + (self.rows.saturating_sub(1) as f32) * self.gap
    }

    pub fn fits_in(&self, avail: Vec2) -> bool {
        self.total_width() <= avail.x + 1e-3 && self.total_height() <= avail.y + 1e-3
    }
}

/// Computes card dimensions, column count, and font size purely from available space.
/// 4 upgrade lines are arranged as 4 columns when width >= 1400 px, or a 2x2 grid when under 1400 px.
pub fn card_layout(avail: Vec2) -> CardLayout {
    let w = if avail.x.is_finite() { avail.x.max(0.0) } else { 0.0 };
    let h = if avail.y.is_finite() { avail.y.max(0.0) } else { 0.0 };

    // 4 columns when width is at least 1400 px; 2x2 grid (2 columns) when under 1400 px
    let columns = if w >= 1400.0 { 4 } else { 2 }; // new: DECISIONS 2026-11-22
    let rows = 4_usize.div_ceil(columns); // new: DECISIONS 2026-11-22

    let gap = 10.0f32.min(w * 0.02).max(4.0); // new: DECISIONS 2026-11-22
    let col_f = columns as f32;
    let row_f = rows as f32;

    let avail_w = (w - (col_f - 1.0) * gap).max(0.0);
    let card_width = (avail_w / col_f).floor();

    let avail_h = (h - (row_f - 1.0) * gap).max(0.0);
    let card_height = (avail_h / row_f).floor();

    let font_size = if card_height < 300.0 {
        12.0 // new: DECISIONS 2026-11-22
    } else if card_height < 450.0 {
        13.5 // new: DECISIONS 2026-11-22
    } else {
        15.0 // new: DECISIONS 2026-11-22
    };

    CardLayout {
        columns,
        card_width,
        card_height,
        font_size,
        rows,
        gap,
    }
}

pub fn panel_dimensions(screen_rect: Rect, scale: f32) -> Vec2 {
    let margin_x = 24.0 * scale; // new: DECISIONS 2026-11-22
    let margin_y = 16.0 * scale; // new: DECISIONS 2026-11-22
    let max_w = (screen_rect.width() - margin_x * 2.0).max(300.0);
    let max_h = (screen_rect.height() - margin_y * 2.0).max(280.0);
    vec2(
        max_w.min(screen_rect.width()),
        max_h.min(screen_rect.height()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn card_layout_fits_in_1920x986_1280x720_and_1024x600() {
        // 1920x986 (wide): 4 columns, 1 row
        let layout_1920 = card_layout(vec2(1920.0, 986.0));
        assert_eq!(layout_1920.columns, 4);
        assert_eq!(layout_1920.rows, 1);
        assert!(layout_1920.card_width > 0.0);
        assert!(layout_1920.card_height > 0.0);
        assert!(layout_1920.total_width() <= 1920.0);
        assert!(layout_1920.total_height() <= 986.0);
        assert!(layout_1920.fits_in(vec2(1920.0, 986.0)));

        // 1280x720 (medium): 2 columns, 2 rows (2x2 grid)
        let layout_1280 = card_layout(vec2(1280.0, 720.0));
        assert_eq!(layout_1280.columns, 2);
        assert_eq!(layout_1280.rows, 2);
        assert!(layout_1280.card_width > 0.0);
        assert!(layout_1280.card_height > 0.0);
        assert!(layout_1280.total_width() <= 1280.0);
        assert!(layout_1280.total_height() <= 720.0);
        assert!(layout_1280.fits_in(vec2(1280.0, 720.0)));

        // 1024x600 (compact): 2 columns, 2 rows (2x2 grid)
        let layout_1024 = card_layout(vec2(1024.0, 600.0));
        assert_eq!(layout_1024.columns, 2);
        assert_eq!(layout_1024.rows, 2);
        assert!(layout_1024.card_width > 0.0);
        assert!(layout_1024.card_height > 0.0);
        assert!(layout_1024.total_width() <= 1024.0);
        assert!(layout_1024.total_height() <= 600.0);
        assert!(layout_1024.fits_in(vec2(1024.0, 600.0)));
    }

    #[test]
    fn card_layout_handles_zero_and_negative_sizes_gracefully() {
        let layout_zero = card_layout(vec2(0.0, 0.0));
        assert_eq!(layout_zero.columns, 2);
        assert_eq!(layout_zero.card_width, 0.0);
        assert_eq!(layout_zero.card_height, 0.0);

        let layout_nan = card_layout(vec2(f32::NAN, f32::NAN));
        assert_eq!(layout_nan.columns, 2);
        assert_eq!(layout_nan.card_width, 0.0);
        assert_eq!(layout_nan.card_height, 0.0);
    }

    #[test]
    fn panel_dimensions_fit_720p_to_1440p() {
        for (w, h) in [(1024.0, 600.0), (1280.0, 720.0), (1920.0, 1080.0), (2560.0, 1440.0)] {
            let rect = Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(w, h));
            let scale = crate::menus::theme::ui_scale(h);
            let panel = panel_dimensions(rect, scale);
            assert!(panel.x < w, "panel width {} must fit screen width {}", panel.x, w);
            assert!(panel.y < h, "panel height {} must fit screen height {}", panel.y, h);
        }
    }
}
