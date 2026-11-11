use crate::theme::Palette;
use crate::ui::{prototype_bg, widgets};
use egui::{Color32, Painter, Pos2, Rect, Shape, Stroke, StrokeKind, pos2, vec2};

/// Gold and pearl around the glass, with a crest above the draggable title bar.
pub(in crate::ui) fn paint_window_trim(painter: &Painter, p: &Palette, rect: Rect, radius: f32, minimized: bool) {
    if p.high_contrast {
        return;
    }
    // Dragon Viewer: the gold rim of the mockup's Preferences
    let _ = minimized;
    crate::ui::dragon_art::gold_rim(painter, rect.shrink(1.5), radius);
}

pub(super) fn glass_wash(painter: &Painter, p: &Palette, rect: Rect, radius: f32) {
    if !p.high_contrast {
        widgets::gradient_rect(
            painter,
            rect,
            radius,
            Color32::from_rgba_unmultiplied(40, 44, 80, 70),
            Color32::from_rgba_unmultiplied(8, 8, 24, 90),
        );
        // glossy reflection across the top of the glass
        let gloss = Rect::from_min_max(rect.min, pos2(rect.right(), rect.top() + rect.height() * 0.32));
        widgets::gradient_rect(painter, gloss, radius, p.ink.gamma_multiply(0.11), Color32::TRANSPARENT);
    }
}

pub(super) fn sparkle(painter: &Painter, center: Pos2, radius: f32, face: Color32, edge: Color32) {
    let mut mesh = egui::epaint::Mesh::default();
    mesh.colored_vertex(center, face);
    for i in 0..10 {
        let theta = std::f32::consts::TAU * i as f32 / 10.0 - std::f32::consts::FRAC_PI_2;
        let r = if i % 2 == 0 { radius } else { radius * 0.43 };
        mesh.colored_vertex(center + vec2(theta.cos(), theta.sin()) * r, edge);
        mesh.add_triangle(0, i + 1, (i + 1) % 10 + 1);
    }
    painter.add(Shape::mesh(mesh));
}

/// A small, purely decorative moon-phase strip below the navigation.
pub(super) fn moon_phases(painter: &Painter, p: &Palette, center: Pos2) {
    // Dragon Viewer: seven small orbs under the page list
    let _ = p;
    for i in 0..7 {
        let x = (i as f32 - 3.0) * 16.0;
        crate::ui::dragon_art::orb(painter, center + vec2(x, 0.0), 6.0, i as u8 + 1);
    }
}

/// Native Slider retains dragging, keyboard focus and AccessKit. This paints
/// the rail and star over its exact interaction rectangle.
pub(super) fn volume_rail(ui: &egui::Ui, p: &Palette, response: &egui::Response, position: f32) {
    if !ui.is_rect_visible(response.rect) || p.high_contrast {
        return;
    }
    let painter = ui.painter();
    let dim = if response.enabled() { 1.0 } else { 0.42 };
    let rail = Rect::from_center_size(response.rect.center(), vec2(response.rect.width(), 6.0));
    // egui's circular handle centers travel inset by height / 2.5.
    let reach = response.rect.x_range().shrink(response.rect.height() / 2.5);
    let center = pos2(egui::lerp(reach.min..=reach.max, position), rail.center().y);
    let hot = response.hovered() || response.dragged() || response.has_focus();
    // Dragon Viewer: the glowing rail and an orb for a knob
    let mut painter = painter.clone();
    painter.multiply_opacity(dim);
    widgets::dragon_rail(&painter, rail, center.x);
    if hot {
        crate::ui::dragon_art::glow(&painter, center, 17.0, egui::Color32::from_rgb(255, 150, 0), 0.9);
    }
    crate::ui::dragon_art::orb(&painter, center, if hot { 14.5 } else { 13.0 }, 4);
    if response.has_focus() {
        painter.rect_stroke(
            response.rect,
            response.rect.height() * 0.5,
            Stroke::new(1.0, p.ink.gamma_multiply(0.72)),
            StrokeKind::Inside,
        );
    }
}
