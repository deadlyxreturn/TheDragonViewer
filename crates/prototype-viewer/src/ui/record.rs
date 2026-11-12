//! Prototype's Record window (the toolbar's Record button): one big button
//! that starts and stops the video, its few choices under it, and the last
//! clip saved; while a video records, a small badge at the top of the
//! screen shows the time and stops it (`badge`).

use super::widgets::{self, ActionKind, Floater};
use crate::recorder::{Notice, Recorder, RecorderSettings};
use crate::theme::Palette;
use egui::{RichText, Sense, Vec2};
use std::time::Duration;

pub const WINDOW: &str = "record";

/// What the window or the badge asks of the app.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordAction {
    Toggle,
}

fn clock(d: Duration) -> String {
    let s = d.as_secs();
    if s >= 3600 {
        format!("{}:{:02}:{:02}", s / 3600, s / 60 % 60, s % 60)
    } else {
        format!("{:02}:{:02}", s / 60, s % 60)
    }
}

/// The big round button: a green bubble with the white record dot, or the
/// stop square while recording (its ring pulsing).
fn big_button(ui: &mut egui::Ui, p: &Palette, recording: bool, busy: bool) -> egui::Response {
    // Dragon Viewer: the skin's record button (`record-button`, and
    // `record-button-active` while recording), its own picture with its word
    let skin = if recording { "record-button-active" } else { "record-button" };
    if !p.high_contrast && super::dragon_art::has(skin) {
        let side = 150.0;
        let (rect, resp) = ui.allocate_exact_size(Vec2::splat(side), if busy { Sense::hover() } else { Sense::click() });
        let t = ui.input(|i| i.time) as f32;
        let hot = resp.hovered() && !busy;
        if recording {
            let pulse = 0.5 + 0.5 * (t * 3.0).sin();
            super::dragon_art::glow(ui.painter(), rect.center(), side * 0.55, egui::Color32::from_rgb(255, 40, 20), 0.5 + 0.5 * pulse);
            ui.ctx().request_repaint();
        } else if hot {
            super::dragon_art::glow(ui.painter(), rect.center(), side * 0.55, egui::Color32::from_rgb(255, 150, 0), 0.8);
        }
        let grow = if hot { 4.0 } else { 0.0 };
        let tint = if busy { egui::Color32::from_gray(150) } else { egui::Color32::WHITE };
        super::dragon_art::paint(ui.painter(), skin, rect.expand(grow), tint);
        if hot {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        return resp;
    }
    let side = 84.0;
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(side), if busy { Sense::hover() } else { Sense::click() });
    let painter = ui.painter();
    let c = rect.center();
    let t = ui.input(|i| i.time) as f32;
    if recording {
        let pulse = 0.5 + 0.5 * (t * 3.0).sin();
        painter.circle_filled(c, side * 0.5 + 4.0 + pulse * 3.0, p.violet.gamma_multiply(0.16 + 0.12 * pulse));
        ui.ctx().request_repaint();
    }
    let face = egui::Rect::from_center_size(c, Vec2::splat(side - 8.0));
    if p.high_contrast {
        widgets::bubble(painter, p, face, face.width() * 0.5, resp.hovered());
    } else {
        // Dragon Viewer: a glossy orb in a gold rim, red while recording
        painter.circle_filled(c, face.width() * 0.5 + 3.0, egui::Color32::from_rgb(110, 44, 4));
        painter.circle_stroke(c, face.width() * 0.5 + 1.5, egui::Stroke::new(2.5, egui::Color32::from_rgb(255, 210, 100)));
        super::dragon_art::orb(painter, c, face.width() * 0.5, if recording { 1 } else { 4 });
        if recording {
            painter.circle_filled(c, face.width() * 0.5, egui::Color32::from_rgba_unmultiplied(220, 20, 10, 90));
        }
    }
    if recording {
        let sq = egui::Rect::from_center_size(c, Vec2::splat(24.0));
        painter.rect_filled(sq, 5.0, p.field); // theme-guard: content — the stop square
    } else {
        // Prototype: the record dot in white on the green bubble
        painter.circle_filled(c, 14.0, p.field);
    }
    if resp.hovered() && !busy {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp
}

pub fn window(ctx: &egui::Context, p: &Palette, open: &mut bool, s: &mut RecorderSettings, rec: &Recorder) -> Option<RecordAction> {
    let mut action = None;
    let screen = ctx.content_rect();
    let pos = egui::pos2(screen.right() - 360.0, screen.top() + 120.0);
    let mut floater = Floater::new(WINDOW, prototype_i18n::tr!("recorder-title"), pos, Vec2::new(340.0, 0.0));
    floater.resizable = false;
    floater.show(ctx, p, open, |ui| {
        ui.set_width(320.0);
        ui.spacing_mut().item_spacing.y = 8.0;
        let recording = rec.recording();
        let busy = rec.finishing();
        ui.vertical_centered(|ui| {
            ui.add_space(6.0);
            if big_button(ui, p, recording, busy).clicked() {
                action = Some(RecordAction::Toggle);
            }
            let (line, color) = if recording {
                (clock(rec.elapsed()), p.ink)
            } else if busy {
                (prototype_i18n::tr!("recorder-saving").to_owned(), p.muted)
            } else {
                (prototype_i18n::tr!("recorder-start").to_owned(), p.ink)
            };
            if p.high_contrast {
                ui.label(RichText::new(line).size(16.0).strong().color(color));
            } else {
                let (r, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 28.0), Sense::hover());
                super::dragon_art::outlined_text(ui.painter(), r.center(), &line, 22.0, color, egui::Color32::from_rgb(60, 20, 4));
            }
        });
        ui.add_space(2.0);
        ui.add_enabled_ui(!recording, |ui| {
            // frames per second and quality, one click each (the photo
            // window's chips)
            ui.horizontal(|ui| {
                ui.label(RichText::new(prototype_i18n::tr!("recorder-fps")).size(12.5).color(p.muted));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let mut i = usize::from(s.fps >= 60);
                    if widgets::segmented(ui, p, &mut i, &[("30 fps", true), ("60 fps", true)]) {
                        s.fps = if i == 1 { 60 } else { 30 };
                    }
                });
            });
            ui.horizontal(|ui| {
                ui.label(RichText::new(prototype_i18n::tr!("recorder-quality")).size(12.5).color(p.muted));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let labels = [
                        prototype_i18n::tr!("recorder-quality-standard"),
                        prototype_i18n::tr!("recorder-quality-high"),
                        prototype_i18n::tr!("recorder-quality-best"),
                    ];
                    let mut i = usize::from(s.quality.min(2));
                    if widgets::segmented(ui, p, &mut i, &labels.map(|l| (l, true))) {
                        s.quality = i as u8;
                    }
                });
            });
            widgets::divider(ui, p);
            ui.label(RichText::new(prototype_i18n::tr!("recorder-content")).size(12.5).color(p.muted));
            let mut i = usize::from(s.content.min(2));
            if widgets::segmented(
                ui,
                p,
                &mut i,
                &[
                    (prototype_i18n::tr!("recorder-content-world"), true),
                    (prototype_i18n::tr!("recorder-content-bars"), true),
                    (prototype_i18n::tr!("recorder-content-all"), true),
                ],
            ) {
                s.content = i as u8;
            }
            widgets::switch_labeled(ui, p, &mut s.cursor, prototype_i18n::tr!("recorder-cursor"));
            ui.horizontal(|ui| {
                ui.label(RichText::new(prototype_i18n::tr!("recorder-blur")).size(12.5).color(p.muted));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let mut i = usize::from(s.motion_blur.min(3));
                    let labels = [
                        prototype_i18n::tr!("recorder-blur-off"),
                        prototype_i18n::tr!("recorder-blur-light"),
                        prototype_i18n::tr!("recorder-blur-medium"),
                        prototype_i18n::tr!("recorder-blur-strong"),
                    ];
                    if widgets::segmented(ui, p, &mut i, &labels.map(|l| (l, true))) {
                        s.motion_blur = i as u8;
                    }
                });
            });
            widgets::switch_labeled(ui, p, &mut s.hide_name_tags, prototype_i18n::tr!("recorder-hide-name-tags"));
            widgets::switch_labeled(ui, p, &mut s.sound, prototype_i18n::tr!("recorder-sound"));
            widgets::switch_labeled(ui, p, &mut s.microphone, prototype_i18n::tr!("recorder-microphone"));
        });
        widgets::divider(ui, p);
        let folder = s.folder();
        ui.horizontal(|ui| {
            ui.add(
                egui::Label::new(RichText::new(folder.display().to_string()).size(11.5).color(p.muted))
                    .truncate(),
            );
        });
        ui.horizontal(|ui| {
            let size = Vec2::new(142.0, 32.0);
            if widgets::action_button(ui, p, "folder-filled", prototype_i18n::tr!("recorder-open-folder"), ActionKind::Flat, size).clicked() {
                let _ = std::fs::create_dir_all(&folder);
                let _ = crate::open_path::open(&folder);
            }
            if let Some((Notice::Saved(path), _)) = &rec.notice
                && widgets::action_button(ui, p, "player-play-filled", prototype_i18n::tr!("recorder-play-last"), ActionKind::Flat, size).clicked()
            {
                let _ = crate::open_path::open(path);
            }
        });
        match &rec.notice {
            Some((Notice::Saved(path), _)) => {
                let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
                ui.label(RichText::new(prototype_i18n::tr!("recorder-saved", file = name)).size(12.0).color(p.violet_pale));
            }
            Some((Notice::Failed(e), _)) => {
                ui.label(RichText::new(e).size(12.0).color(p.danger));
            }
            None => {}
        }
    });
    action
}

/// While recording: a small glass pill at the top of the screen, the red
/// dot, the time and a stop button. Kept out of the video when only the 3D
/// view is recorded (it is interface).
pub fn badge(ctx: &egui::Context, p: &Palette, rec: &Recorder) -> Option<RecordAction> {
    if !rec.recording() {
        return None;
    }
    let mut action = None;
    let screen = ctx.content_rect();
    let shown = egui::Area::new(egui::Id::unique("record_badge"))
        .order(egui::Order::Foreground)
        .pivot(egui::Align2::CENTER_TOP)
        .fixed_pos(egui::pos2(screen.center().x, screen.top() + 44.0))
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(p.field.gamma_multiply(0.78))
                .stroke(egui::Stroke::new(1.0, p.field))
                .corner_radius(18) // theme-guard: content — the recording pill
                .inner_margin(egui::Margin::symmetric(12, 6))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let t = ui.input(|i| i.time) as f32;
                        let (dot, _) = ui.allocate_exact_size(Vec2::splat(12.0), Sense::hover());
                        let a = 0.55 + 0.45 * (t * 3.0).sin().abs();
                        ui.painter().circle_filled(dot.center(), 5.5, p.violet.gamma_multiply(a));
                        ui.label(
                            RichText::new(format!("REC  {}", clock(rec.elapsed())))
                                .size(13.0)
                                .strong()
                                .monospace()
                                .color(p.ink),
                        );
                        let stop = if p.high_contrast {
                            widgets::action_button(ui, p, "player-stop-filled", prototype_i18n::tr!("recorder-stop"), ActionKind::Primary, Vec2::new(0.0, 26.0))
                        } else {
                            // Dragon Viewer: Stop as the red jewel of Cancel
                            super::options::jewel_button(ui, p, prototype_i18n::tr!("recorder-stop"), Vec2::new(92.0, 30.0), true)
                        };
                        if stop.clicked()
                        {
                            action = Some(RecordAction::Toggle);
                        }
                    });
                });
            ui.ctx().request_repaint_after(Duration::from_millis(250));
        });
    // interface: left out of a "World + bars" video
    widgets::note_window_rect(ctx, shown.response.rect);
    action
}
