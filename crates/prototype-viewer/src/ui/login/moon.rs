//! Dragon Viewer's login card pieces: the card's glass and gold rim, its
//! fields, the LOG IN jewel and the ornaments around the card. Hit targets
//! and actions stay in the login form. Each piece shows the skin's picture
//! when it has one (`login-frame`, `login-button`, `login-crest`,
//! `cloud-scroll`, `balls/ball-*`), else a drawn version.
use crate::theme::Palette;
use crate::ui::dragon_art;
use egui::{Color32, Painter, Rect, Shape, Stroke, StrokeKind, pos2, vec2};

use super::super::widgets::gradient_rect_shape;

/// A second, inset reflection gives the rounded face a visible glass
/// surface without covering the text or changing the widget's hit target.
pub(super) fn polished_pill(ctx: &egui::Context, rect: Rect, top: Color32, bottom: Color32, light: Color32, edge: Color32) -> Shape {
    let radius = rect.height() * 0.5;
    let mut face = vec![gradient_rect_shape(ctx, rect, radius, top, bottom)];
    let shine = Rect::from_min_max(
        rect.min + vec2(radius * 0.52, 2.0),
        pos2(rect.right() - radius * 0.52, rect.top() + rect.height() * 0.43),
    );
    if shine.is_positive() {
        face.push(gradient_rect_shape(
            ctx,
            shine,
            shine.height() * 0.5,
            light.gamma_multiply(0.68),
            light.gamma_multiply(0.01),
        ));
    }
    face.push(Shape::rect_stroke(rect, radius, Stroke::new(1.4, edge), StrokeKind::Inside));
    Shape::Vec(face)
}

/// A dark field with a gold rim and a faint glow under it (the card's text
/// fields and menus).
pub(super) fn gold_field(ctx: &egui::Context, rect: Rect, edge: Color32, hot: bool) -> Shape {
    let radius = 7.0;
    let mut v = vec![
        gradient_rect_shape(
            ctx,
            rect,
            radius,
            Color32::from_rgba_unmultiplied(28, 26, 52, 225),
            Color32::from_rgba_unmultiplied(12, 12, 30, 235),
        ),
        Shape::rect_stroke(rect, radius, Stroke::new(1.6, edge), StrokeKind::Inside),
        // a light line under the top edge: the bevel
        Shape::line_segment(
            [rect.left_top() + vec2(radius, 2.0), rect.right_top() + vec2(-radius, 2.0)],
            Stroke::new(1.0, Color32::from_white_alpha(30)),
        ),
    ];
    if hot {
        v.insert(
            0,
            Shape::rect_stroke(rect.expand(2.0), radius + 2.0, Stroke::new(4.0, Color32::from_rgba_unmultiplied(255, 183, 44, 70)), StrokeKind::Outside),
        );
    }
    // the warm glow under the field's foot (the mockup's)
    let foot = Rect::from_center_size(pos2(rect.center().x, rect.bottom() + 1.0), vec2(rect.width() * 0.74, 3.0));
    v.push(gradient_rect_shape(
        ctx,
        foot,
        1.5,
        Color32::from_rgba_unmultiplied(255, 170, 40, 60),
        Color32::TRANSPARENT,
    ));
    Shape::Vec(v)
}

/// The login card's face: the skin's `login-frame` stretched over it, or
/// deep night glass with a soft top sheen in a gold rim.
pub(super) fn card_face(ctx: &egui::Context, rect: Rect) -> Shape {
    let radius = 39.0;
    if let Some(t) = dragon_art::texture(ctx, "login-frame") {
        return Shape::image(t.id(), rect, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
    }
    let mut v = vec![
        // the warm outer glow
        Shape::rect_stroke(rect.expand(4.0), radius + 4.0, Stroke::new(9.0, Color32::from_rgba_unmultiplied(255, 120, 0, 34)), StrokeKind::Outside),
        gradient_rect_shape(
            ctx,
            rect,
            radius,
            Color32::from_rgba_unmultiplied(46, 34, 84, 200),
            Color32::from_rgba_unmultiplied(14, 10, 32, 225),
        ),
    ];
    let sheen = Rect::from_min_max(rect.min + vec2(6.0, 6.0), pos2(rect.right() - 6.0, rect.top() + rect.height() * 0.3));
    v.push(gradient_rect_shape(ctx, sheen, radius - 6.0, Color32::from_white_alpha(22), Color32::TRANSPARENT));
    v.push(Shape::rect_stroke(rect, radius, Stroke::new(4.0, Color32::from_rgb(105, 44, 4)), StrokeKind::Outside));
    v.push(Shape::rect_stroke(rect, radius, Stroke::new(3.0, Color32::from_rgb(240, 160, 30)), StrokeKind::Inside));
    v.push(Shape::rect_stroke(rect.shrink(3.0), radius - 3.0, Stroke::new(1.2, Color32::from_rgba_unmultiplied(255, 236, 170, 170)), StrokeKind::Inside));
    v.push(Shape::rect_stroke(rect.shrink(7.0), radius - 7.0, Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 190, 80, 60)), StrokeKind::Inside));
    Shape::Vec(v)
}

/// The LOG IN jewel: the skin's `login-button` face (else the gold jewel),
/// with an orb at each end.
pub(super) fn connect_button(painter: &Painter, rect: Rect, p: &Palette, active: bool, hot: bool, pressed: bool) {
    if p.high_contrast {
        painter.rect_filled(rect.shrink2(vec2(18.0, 6.0)), 12.0, p.violet);
        return;
    }
    let face = rect.shrink2(vec2(0.0, 4.0));
    let lit = active && hot;
    if !dragon_art::paint_fill(
        painter,
        "login-button",
        rect,
        if pressed { Color32::from_gray(220) } else { Color32::WHITE },
    ) {
        // a hexagonal-ended jewel: pointed ends, a bright rim
        if lit {
            dragon_art::glow(painter, face.center(), face.width() * 0.45, Color32::from_rgb(252, 109, 4), 0.6);
        }
        let tip = face.height() * 0.5;
        let points = |r: Rect, t: f32| {
            vec![
                pos2(r.left(), r.center().y),
                pos2(r.left() + t, r.top()),
                pos2(r.right() - t, r.top()),
                pos2(r.right(), r.center().y),
                pos2(r.right() - t, r.bottom()),
                pos2(r.left() + t, r.bottom()),
            ]
        };
        painter.add(Shape::convex_polygon(points(face, tip), Color32::from_rgb(110, 44, 0), Stroke::NONE));
        let inner = face.shrink(3.0);
        painter.add(Shape::convex_polygon(points(inner, tip - 2.0), Color32::from_rgb(255, 214, 120), Stroke::NONE));
        let body = inner.shrink(3.0);
        dragon_art::drawn_jewel(painter, body.shrink2(vec2(tip * 0.55, 0.0)), false, lit, pressed);
        painter.add(Shape::closed_line(points(inner, tip - 2.0), Stroke::new(1.0, Color32::from_rgb(255, 246, 200))));
    }
    // the two orbs inside its ends
    let r = (face.height() * 0.4).min(30.0);
    let inset = face.height() * 0.62;
    dragon_art::orb(painter, pos2(face.left() + inset, face.center().y), r, 4);
    dragon_art::orb(painter, pos2(face.right() - inset, face.center().y), r, 1);
    if !active {
        painter.rect_filled(face.shrink2(vec2(face.height() * 1.2, 6.0)), 10.0, Color32::from_black_alpha(40));
    }
}

/// The crest over the card's top edge and the clouds at its lower corners
/// (the skin's `login-crest` and `cloud-scroll`; a large orb when the skin
/// has no crest).
pub(super) fn card_ornaments(ctx: &egui::Context, card: Rect) {
    let painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Foreground, egui::Id::unique("dragon_login_ornaments")));
    let crest = Rect::from_center_size(pos2(card.center().x, card.top() - 26.0), vec2(card.width() * 0.73, 155.0));
    if !dragon_art::paint(&painter, "login-crest", crest, Color32::WHITE) {
        let c = pos2(card.center().x, card.top() - 6.0);
        dragon_art::glow(&painter, c, 66.0, Color32::from_rgb(255, 150, 0), 1.0);
        painter.circle_filled(c, 44.0, Color32::from_rgb(70, 26, 0));
        painter.circle_stroke(c, 44.0, Stroke::new(3.0, Color32::from_rgb(255, 200, 80)));
        dragon_art::orb(&painter, c, 38.0, 4);
    }
    if let Some(t) = dragon_art::texture(ctx, "cloud-scroll") {
        let [w, h] = t.size();
        let size = vec2(140.0, 140.0 * h as f32 / w.max(1) as f32);
        let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
        let left = Rect::from_min_size(pos2(card.left() - 5.0, card.bottom() + 4.0 - size.y), size);
        painter.image(t.id(), left, uv, Color32::WHITE);
        let right = Rect::from_min_size(pos2(card.right() + 5.0 - size.x, card.bottom() + 4.0 - size.y), size);
        painter.image(t.id(), right, Rect::from_min_max(pos2(1.0, 0.0), pos2(0.0, 1.0)), Color32::WHITE);
    }
}

