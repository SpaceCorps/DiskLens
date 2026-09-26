use egui::{Color32, CornerRadius, Pos2, Rect, Stroke, Ui, Vec2};

#[allow(dead_code)]
pub struct ThemeColors;

#[allow(dead_code)]
impl ThemeColors {
    // Backgrounds
    pub const BG_APP: Color32 = Color32::from_rgb(13, 17, 23);
    pub const BG_PANEL: Color32 = Color32::from_rgb(19, 27, 44);
    pub const BG_CARD: Color32 = Color32::from_rgb(26, 37, 58);
    pub const BG_HOVER: Color32 = Color32::from_rgb(34, 48, 74);
    pub const BG_ACTIVE: Color32 = Color32::from_rgb(45, 62, 94);

    // Accents
    pub const CYAN: Color32 = Color32::from_rgb(56, 189, 248);
    pub const BLUE: Color32 = Color32::from_rgb(59, 130, 246);
    pub const INDIGO: Color32 = Color32::from_rgb(99, 102, 241);

    // Status / Fill severity
    pub const CRITICAL_RED: Color32 = Color32::from_rgb(239, 68, 68);
    pub const WARNING_AMBER: Color32 = Color32::from_rgb(245, 158, 11);
    pub const MODERATE_BLUE: Color32 = Color32::from_rgb(56, 189, 248);
    pub const LIGHT_GREEN: Color32 = Color32::from_rgb(16, 185, 129);

    // Text
    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(243, 244, 246);
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(156, 163, 175);
    pub const TEXT_FAINT: Color32 = Color32::from_rgb(107, 114, 128);

    // Borders
    pub const BORDER: Color32 = Color32::from_rgb(36, 48, 73);
    pub const BORDER_BRIGHT: Color32 = Color32::from_rgb(56, 189, 248);

    pub fn color_for_disk_percent(percent: f64) -> Color32 {
        if percent >= 5.0 {
            Self::CRITICAL_RED
        } else if percent >= 1.0 {
            Self::WARNING_AMBER
        } else if percent >= 0.2 {
            Self::MODERATE_BLUE
        } else {
            Self::LIGHT_GREEN
        }
    }

    pub fn color_for_folder_percent(percent: f64) -> Color32 {
        if percent >= 40.0 {
            Self::CRITICAL_RED
        } else if percent >= 15.0 {
            Self::WARNING_AMBER
        } else if percent >= 5.0 {
            Self::MODERATE_BLUE
        } else {
            Self::LIGHT_GREEN
        }
    }
}

/// Custom painter for a sleek modern percentage bar.
pub fn draw_percent_bar(ui: &mut Ui, fill_fraction: f64, color: Color32, label: &str, height: f32) {
    let desired_width = ui.available_width().max(120.0);
    let (rect, _response) =
        ui.allocate_exact_size(Vec2::new(desired_width, height), egui::Sense::hover());

    let painter = ui.painter();

    // Background track
    let corner_radius = CornerRadius::same(height as u8 / 2);
    painter.rect_filled(rect, corner_radius, Color32::from_rgb(22, 30, 46));
    painter.rect_stroke(
        rect,
        corner_radius,
        Stroke::new(1.0, Color32::from_rgb(36, 48, 73)),
        egui::StrokeKind::Inside,
    );

    // Fill rect
    let clamped_fraction = fill_fraction.clamp(0.0, 1.0) as f32;
    if clamped_fraction > 0.001 {
        let fill_width = (rect.width() * clamped_fraction).max(height);
        let fill_rect = Rect::from_min_size(rect.min, Vec2::new(fill_width, rect.height()));
        painter.rect_filled(fill_rect, corner_radius, color);
    }

    // Label inside or beside
    let text_pos = Pos2::new(rect.min.x + 8.0, rect.center().y);
    let text_color = if clamped_fraction > 0.45 {
        Color32::WHITE
    } else {
        Color32::from_rgb(220, 225, 235)
    };

    painter.text(
        text_pos,
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(11.0),
        text_color,
    );
}
