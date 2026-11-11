//! Dragon Viewer's skin pictures: the login background, the orbs, the
//! button faces and the ornaments of the login card and Preferences.
//!
//! A picture is looked up by its name (`background`, `balls/ball-4`,
//! `login-crest`, ...) in, first come first served:
//! - the `skin` folder of the config folder (the user's own),
//! - the `skin` folder next to the exe,
//! - the repository's `skin/` folder, embedded in the exe at build time.
//!
//! SVG, PNG, JPEG and WebP are read, in that order of preference. Pictures
//! decode on a background thread; until one is ready (or when there is none)
//! the painters fall back to drawn shapes.

use egui::{Color32, Context, Painter, Pos2, Rect, Stroke, StrokeKind, TextureHandle, TextureOptions, Vec2, pos2, vec2};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};

mod embedded {
    include!(concat!(env!("OUT_DIR"), "/skin_files.rs"));
}

const EXTENSIONS: [&str; 5] = ["svg", "png", "jpg", "jpeg", "webp"];

/// Largest side a vector picture is drawn at (its texture).
const SVG_PX: u32 = 1024;

fn runtime_dirs() -> Vec<PathBuf> {
    let mut dirs = vec![crate::settings::config_dir().join("skin")];
    if let Some(exe) = std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.to_path_buf())) {
        dirs.push(exe.join("skin"));
    }
    dirs
}

/// The bytes of the picture `name` and whether it is an SVG.
fn find(name: &str) -> Option<(Vec<u8>, bool)> {
    for dir in runtime_dirs() {
        for ext in EXTENSIONS {
            let path = dir.join(format!("{name}.{ext}"));
            if let Ok(bytes) = std::fs::read(&path) {
                return Some((bytes, ext == "svg"));
            }
        }
    }
    for ext in EXTENSIONS {
        let file = format!("{name}.{ext}");
        if let Some((_, bytes)) = embedded::SKIN_FILES.iter().find(|(n, _)| *n == file) {
            return Some((bytes.to_vec(), ext == "svg"));
        }
    }
    None
}

/// Whether the skin has a picture called `name`.
pub fn has(name: &str) -> bool {
    static KNOWN: OnceLock<Mutex<HashMap<String, bool>>> = OnceLock::new();
    let known = KNOWN.get_or_init(Default::default);
    if let Some(v) = known.lock().ok().and_then(|k| k.get(name).copied()) {
        return v;
    }
    let v = find(name).is_some();
    if let Ok(mut k) = known.lock() {
        k.insert(name.to_owned(), v);
    }
    v
}

/// The files of the skin's `icons/` folder, embedded or beside the exe:
/// (icon name without extension, bytes, is SVG).
pub fn icon_files() -> Vec<(String, Vec<u8>, bool)> {
    let mut out: Vec<(String, Vec<u8>, bool)> = Vec::new();
    let mut push = |file: &str, bytes: Vec<u8>| {
        let Some((stem, ext)) = file.rsplit_once('.') else { return };
        let ext = ext.to_ascii_lowercase();
        if !EXTENSIONS.contains(&ext.as_str()) || out.iter().any(|(n, _, _)| n == stem) {
            return;
        }
        out.push((stem.to_owned(), bytes, ext == "svg"));
    };
    for dir in runtime_dirs() {
        if let Ok(rd) = std::fs::read_dir(dir.join("icons")) {
            for e in rd.flatten() {
                if let (Some(name), Ok(bytes)) = (e.file_name().to_str().map(str::to_owned), std::fs::read(e.path())) {
                    push(&name, bytes);
                }
            }
        }
    }
    for (file, bytes) in embedded::SKIN_FILES {
        if let Some(name) = file.strip_prefix("icons/") {
            push(name, bytes.to_vec());
        }
    }
    out
}

/// Decodes a picture to an image, an SVG drawn `max_px` on its longer side.
pub fn decode(bytes: &[u8], svg: bool, max_px: u32) -> Option<egui::ColorImage> {
    if svg {
        let opt = resvg::usvg::Options::default();
        let tree = resvg::usvg::Tree::from_data(bytes, &opt).ok()?;
        let size = tree.size();
        let scale = max_px as f32 / size.width().max(size.height()).max(1.0);
        let (w, h) = (
            (size.width() * scale).round().max(1.0) as u32,
            (size.height() * scale).round().max(1.0) as u32,
        );
        let mut pixmap = resvg::tiny_skia::Pixmap::new(w, h)?;
        resvg::render(&tree, resvg::tiny_skia::Transform::from_scale(scale, scale), &mut pixmap.as_mut());
        Some(egui::ColorImage::from_rgba_premultiplied([w as usize, h as usize], pixmap.data()))
    } else {
        let img = image::load_from_memory(bytes).ok()?;
        let img = if img.width().max(img.height()) > 4096 {
            img.resize(4096, 4096, image::imageops::FilterType::Triangle)
        } else {
            img
        };
        let rgba = img.to_rgba8();
        Some(egui::ColorImage::from_rgba_unmultiplied(
            [rgba.width() as usize, rgba.height() as usize],
            rgba.as_raw(),
        ))
    }
}

enum Slot {
    Loading,
    Ready(Option<Arc<egui::ColorImage>>),
}

fn slots() -> &'static Mutex<HashMap<String, Slot>> {
    static SLOTS: OnceLock<Mutex<HashMap<String, Slot>>> = OnceLock::new();
    SLOTS.get_or_init(Default::default)
}

/// The texture of the picture `name`, once decoded; None while it decodes
/// or when the skin has none.
pub fn texture(ctx: &Context, name: &str) -> Option<TextureHandle> {
    let id = egui::Id::unique(("dragon_skin_art", name));
    if let Some(t) = ctx.data(|d| d.get_temp::<TextureHandle>(id)) {
        return Some(t);
    }
    let mut table = slots().lock().ok()?;
    match table.get(name) {
        None => {
            table.insert(name.to_owned(), Slot::Loading);
            let (ctx, name) = (ctx.clone(), name.to_owned());
            std::thread::spawn(move || {
                let image = find(&name).and_then(|(bytes, svg)| {
                    let image = decode(&bytes, svg, SVG_PX);
                    if image.is_none() {
                        log::warn!("skin picture {name}: unreadable");
                    }
                    image
                });
                if let Ok(mut slots) = slots().lock() {
                    slots.insert(name, Slot::Ready(image.map(Arc::new)));
                }
                ctx.request_repaint();
            });
            None
        }
        Some(Slot::Loading) | Some(Slot::Ready(None)) => None,
        Some(Slot::Ready(Some(image))) => {
            let t = ctx.load_texture(format!("dragon-skin-{name}"), (**image).clone(), TextureOptions::LINEAR);
            ctx.data_mut(|d| d.insert_temp(id, t.clone()));
            Some(t)
        }
    }
}

fn full_uv() -> Rect {
    Rect::from_min_max(Pos2::ZERO, pos2(1.0, 1.0))
}

/// Draws `t` fitted inside `rect` (aspect kept, centred).
pub fn paint_contain(painter: &Painter, t: &TextureHandle, rect: Rect, tint: Color32) -> Rect {
    let [w, h] = t.size();
    let scale = (rect.width() / w as f32).min(rect.height() / h as f32);
    let r = Rect::from_center_size(rect.center(), vec2(w as f32 * scale, h as f32 * scale));
    painter.image(t.id(), r, full_uv(), tint);
    r
}

/// Draws the picture `name` fitted in `rect`; false when the skin has none.
pub fn paint(painter: &Painter, name: &str, rect: Rect, tint: Color32) -> bool {
    match texture(painter.ctx(), name) {
        Some(t) => {
            paint_contain(painter, &t, rect, tint);
            true
        }
        None => false,
    }
}

/// Draws the picture `name` stretched over `rect`; false when there is none.
pub fn paint_fill(painter: &Painter, name: &str, rect: Rect, tint: Color32) -> bool {
    match texture(painter.ctx(), name) {
        Some(t) => {
            painter.image(t.id(), rect, full_uv(), tint);
            true
        }
        None => false,
    }
}

/// A glowing orb, `index` 1 to 7: the skin's `balls/ball-<index>` picture,
/// else a plain gold orb.
pub fn orb(painter: &Painter, center: Pos2, radius: f32, index: u8) {
    let rect = Rect::from_center_size(center, Vec2::splat(radius * 2.0));
    if paint(painter, &format!("balls/ball-{}", index.clamp(1, 7)), rect, Color32::WHITE) {
        return;
    }
    // drawn orb: deep orange rim to a pale gold core, a white highlight
    let steps = 14;
    for i in 0..steps {
        let t = i as f32 / (steps - 1) as f32;
        let r = radius * (1.0 - t * 0.92);
        let c = Color32::from_rgb(200, 84, 0).lerp_to_gamma(Color32::from_rgb(255, 236, 140), t.powf(1.4));
        let off = vec2(-0.26, -0.3) * radius * t;
        painter.circle_filled(center + off, r, c);
    }
    painter.circle_stroke(center, radius, Stroke::new((radius * 0.06).max(1.0), Color32::from_rgb(130, 46, 0)));
    painter.circle_filled(
        center + vec2(-0.3, -0.42) * radius,
        radius * 0.2,
        Color32::from_rgba_unmultiplied(255, 255, 240, 150),
    );
}

/// A soft round glow (additive-looking halo) of `color` around `center`.
pub fn glow(painter: &Painter, center: Pos2, radius: f32, color: Color32, strength: f32) {
    let rings = 10;
    for i in 0..rings {
        let t = i as f32 / rings as f32;
        let r = radius * (1.0 - t * 0.7);
        painter.circle_filled(center, r, color.gamma_multiply(strength * 0.09 * (0.3 + t)));
    }
}

/// The gold jewel face of a button (`button-gold`, or `button-red` for a
/// red one): the skin's picture stretched to the rect, else a drawn bevel.
pub fn jewel(painter: &Painter, rect: Rect, red: bool, hot: bool, pressed: bool) {
    let name = if red { "button-red" } else { "button-gold" };
    let tint = if pressed {
        Color32::from_gray(220)
    } else if hot {
        Color32::WHITE
    } else {
        Color32::from_gray(240)
    };
    if paint_fill(painter, name, rect, tint) {
        if hot {
            painter.rect_filled(rect.shrink(rect.height() * 0.18), rect.height() * 0.3, Color32::from_white_alpha(18));
        }
        return;
    }
    drawn_jewel(painter, rect, red, hot, pressed);
}

/// The drawn jewel button: a dark outer rim, a bright metal rim, a glossy
/// gradient face and a highlight across its top.
pub fn drawn_jewel(painter: &Painter, rect: Rect, red: bool, hot: bool, pressed: bool) {
    let radius = rect.height() * 0.32;
    let (top, mid, bottom, rim, dark) = if red {
        (
            Color32::from_rgb(255, 120, 96),
            Color32::from_rgb(214, 30, 18),
            Color32::from_rgb(130, 8, 4),
            Color32::from_rgb(255, 196, 160),
            Color32::from_rgb(70, 4, 0),
        )
    } else {
        (
            Color32::from_rgb(255, 246, 170),
            Color32::from_rgb(255, 160, 20),
            Color32::from_rgb(200, 80, 0),
            Color32::from_rgb(255, 240, 180),
            Color32::from_rgb(110, 44, 0),
        )
    };
    let lift = if pressed { 0.85 } else if hot { 1.12 } else { 1.0 };
    let shade = |c: Color32| {
        Color32::from_rgb(
            (c.r() as f32 * lift).min(255.0) as u8,
            (c.g() as f32 * lift).min(255.0) as u8,
            (c.b() as f32 * lift).min(255.0) as u8,
        )
    };
    if hot && !pressed {
        let c = if red { Color32::from_rgb(255, 60, 20) } else { Color32::from_rgb(255, 150, 0) };
        painter.rect_stroke(rect.expand(3.0), radius + 3.0, Stroke::new(5.0, c.gamma_multiply(0.3)), StrokeKind::Outside);
    }
    painter.rect_filled(rect, radius, dark);
    let face = rect.shrink(2.5);
    let upper = Rect::from_min_max(face.min, pos2(face.right(), face.center().y));
    let lower = Rect::from_min_max(pos2(face.left(), face.center().y), face.max);
    super::widgets::gradient_rect(painter, face, radius - 2.0, shade(top), shade(bottom));
    super::widgets::gradient_rect(painter, upper, radius - 2.0, shade(top), shade(mid));
    super::widgets::gradient_rect(painter, lower, 0.0, shade(mid), shade(bottom));
    painter.rect_stroke(face, radius - 2.0, Stroke::new(1.6, rim), StrokeKind::Inside);
    let shine = Rect::from_min_max(
        face.min + vec2(radius * 0.6, 2.5),
        pos2(face.right() - radius * 0.6, face.top() + face.height() * 0.38),
    );
    if shine.is_positive() {
        super::widgets::gradient_rect(
            painter,
            shine,
            shine.height() * 0.5,
            Color32::from_white_alpha(120),
            Color32::from_white_alpha(8),
        );
    }
}

/// Bold italic text with a dark outline and a soft shadow (the jewel
/// buttons' labels and the window titles).
pub fn outlined_text(painter: &Painter, center: Pos2, text: &str, size: f32, color: Color32, outline: Color32) -> Rect {
    let family = super::fonts::display_bold(painter.ctx());
    let mut job = egui::text::LayoutJob::simple_singleline(text.to_owned(), egui::FontId::new(size, family.clone()), color);
    job.sections[0].format.italics = family == egui::FontFamily::Proportional;
    let galley = painter.layout_job(job);
    let pos = center - galley.size() * 0.5;
    let w = (size * 0.06).max(1.0);
    for (dx, dy) in [(-w, 0.0), (w, 0.0), (0.0, -w), (0.0, w), (-w, -w), (w, w), (-w, w), (w, -w), (0.0, w * 2.0)] {
        painter.galley_with_override_text_color(pos + vec2(dx, dy), galley.clone(), outline);
    }
    painter.galley(pos, galley.clone(), color);
    Rect::from_min_size(pos, galley.size())
}

/// The gold rim of Dragon Viewer's glass panels: an outer dark ring, a
/// bright gold ring and an orange glow outside.
pub fn gold_rim(painter: &Painter, rect: Rect, radius: f32) {
    painter.rect_stroke(rect.expand(2.0), radius + 2.0, Stroke::new(7.0, Color32::from_rgba_unmultiplied(255, 130, 0, 40)), StrokeKind::Outside);
    painter.rect_stroke(rect, radius, Stroke::new(3.0, Color32::from_rgb(110, 46, 4)), StrokeKind::Outside);
    painter.rect_stroke(rect, radius, Stroke::new(2.2, Color32::from_rgb(240, 160, 30)), StrokeKind::Inside);
    painter.rect_stroke(
        rect.shrink(2.2),
        radius - 2.2,
        Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 236, 170, 150)),
        StrokeKind::Inside,
    );
}

/// The bytes of the skin file at `path` (with its extension), from the
/// same places as the pictures.
pub fn file_bytes(path: &str) -> Option<Vec<u8>> {
    for dir in runtime_dirs() {
        if let Ok(bytes) = std::fs::read(dir.join(path)) {
            return Some(bytes);
        }
    }
    embedded::SKIN_FILES.iter().find(|(n, _)| *n == path).map(|(_, b)| b.to_vec())
}

#[derive(serde::Deserialize, Default)]
#[serde(default)]
struct ToolbarLayout {
    face_cap_px: f32,
    orbs: HashMap<String, u8>,
}

fn toolbar_layout() -> &'static ToolbarLayout {
    static LAYOUT: OnceLock<ToolbarLayout> = OnceLock::new();
    LAYOUT.get_or_init(|| {
        file_bytes("toolbar/layout.json")
            .and_then(|b| serde_json::from_slice(&b).map_err(|e| log::warn!("skin toolbar/layout.json: {e}")).ok())
            .unwrap_or_default()
    })
}

/// The orb (1 to 7) of a toolbar command, from `toolbar/layout.json`.
pub fn toolbar_orb(command: &str) -> u8 {
    toolbar_layout().orbs.get(command).copied().unwrap_or(4).clamp(1, 7)
}

/// The skin's toolbar button face (`toolbar/face`, `toolbar/face-active`)
/// drawn over `rect` in three slices, its rounded ends unstretched. False
/// when the skin has no face.
pub fn toolbar_face(painter: &Painter, rect: Rect, active: bool, hot: bool) -> bool {
    let name = if active { "toolbar/face-active" } else { "toolbar/face" };
    let Some(t) = texture(painter.ctx(), name).or_else(|| texture(painter.ctx(), "toolbar/face")) else {
        return false;
    };
    let [w, h] = t.size();
    let (w, h) = (w as f32, h as f32);
    let cap_px = if toolbar_layout().face_cap_px > 0.0 { toolbar_layout().face_cap_px } else { h * 0.47 };
    let scale = rect.height() / h;
    let cap = (cap_px * scale).min(rect.width() * 0.5);
    let u = cap_px / w;
    let tint = if hot && !active { Color32::from_gray(255) } else if active { Color32::WHITE } else { Color32::from_gray(235) };
    let pieces = [
        (Rect::from_min_max(rect.min, pos2(rect.left() + cap, rect.bottom())), Rect::from_min_max(pos2(0.0, 0.0), pos2(u, 1.0))),
        (
            Rect::from_min_max(pos2(rect.left() + cap, rect.top()), pos2(rect.right() - cap, rect.bottom())),
            Rect::from_min_max(pos2(u, 0.0), pos2(1.0 - u, 1.0)),
        ),
        (Rect::from_min_max(pos2(rect.right() - cap, rect.top()), rect.max), Rect::from_min_max(pos2(1.0 - u, 0.0), pos2(1.0, 1.0))),
    ];
    for (r, uv) in pieces {
        if r.width() > 0.0 {
            painter.image(t.id(), r, uv, tint);
        }
    }
    if hot {
        painter.rect_filled(rect.shrink2(vec2(cap * 0.5, rect.height() * 0.2)), rect.height() * 0.3, Color32::from_white_alpha(14));
    }
    true
}
