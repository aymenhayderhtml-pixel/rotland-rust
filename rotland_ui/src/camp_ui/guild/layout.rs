//! Responsive layout calculation for the Guild camp board screen.

use engine::egui::Vec2;

/// Computed dimensions and sizing for the three-column guild screen.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GuildLayout {
    pub panel_width: f32,
    pub panel_height: f32,
    pub column_width: f32,
    pub body_height: f32,
    pub spacing: f32,
    pub is_compact: bool,
}

/// Minimum and maximum panel bounds.
pub const MIN_PANEL_WIDTH: f32 = 740.0;
pub const MAX_PANEL_WIDTH: f32 = 1180.0;
pub const MIN_PANEL_HEIGHT: f32 = 540.0;
pub const MAX_PANEL_HEIGHT: f32 = 680.0;

/// Column layout spacing.
pub const COLUMN_SPACING: f32 = 12.0;
pub const HEADER_MARGIN: f32 = 140.0;

/// Calculates responsive guild layout from the available viewport rect.
pub fn calc_guild_layout(viewport_size: Vec2) -> GuildLayout {
    let screen_w = viewport_size.x.max(320.0);
    let screen_h = viewport_size.y.max(240.0);

    // Margins around the centered panel (24px each side)
    let max_w = (screen_w - 48.0).max(MIN_PANEL_WIDTH);
    let max_h = (screen_h - 48.0).max(MIN_PANEL_HEIGHT);

    let panel_width = max_w.clamp(MIN_PANEL_WIDTH, MAX_PANEL_WIDTH);
    let panel_height = max_h.clamp(MIN_PANEL_HEIGHT, MAX_PANEL_HEIGHT);

    // Inner content area after panel margin (14px * 2 = 28px)
    let inner_width = (panel_width - 28.0).max(100.0);
    let body_height = (panel_height - HEADER_MARGIN).max(200.0);

    let is_compact = inner_width < 960.0;
    let spacing = COLUMN_SPACING;
    let column_width = ((inner_width - (spacing * 2.0)) / 3.0).max(80.0);

    GuildLayout {
        panel_width,
        panel_height,
        column_width,
        body_height,
        spacing,
        is_compact,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_1920x986() {
        let layout = calc_guild_layout(Vec2::new(1920.0, 986.0));
        assert_eq!(layout.panel_width, MAX_PANEL_WIDTH);
        assert_eq!(layout.panel_height, MAX_PANEL_HEIGHT);
        assert!(!layout.is_compact);
        assert!(layout.column_width >= 350.0);
        assert!(layout.body_height >= 400.0);
    }

    #[test]
    fn test_layout_1280x720() {
        let layout = calc_guild_layout(Vec2::new(1280.0, 720.0));
        assert!(layout.panel_width <= 1280.0 - 48.0);
        assert!(layout.panel_height <= 720.0 - 48.0);
        assert!(layout.column_width >= 320.0);
        assert!(layout.body_height >= 350.0);
    }

    #[test]
    fn test_layout_1024x600() {
        let layout = calc_guild_layout(Vec2::new(1024.0, 600.0));
        assert!(layout.panel_width <= 1024.0);
        assert!(layout.panel_height <= 600.0);
        assert!(layout.column_width > 0.0);
        assert!(layout.body_height > 0.0);
    }
}
