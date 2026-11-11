//! Shared login information, community links, version footer and the
//! preferences button.
use super::{LoginAction, LoginForm};
use crate::theme::Palette;
use crate::ui::{
    icons::Icons,
    news::LoginInfo,
    widgets::{self, ActionKind},
};
use egui::{Context, Id, Rect, RichText, Vec2, pos2, vec2};

fn menu_action(ui: &mut egui::Ui, label: &str, action: &mut LoginAction, next: LoginAction) {
    if ui.button(label).clicked() {
        *action = next;
        ui.close();
    }
}

fn menu_bar(ctx: &Context, p: &Palette, form: &mut LoginForm) -> LoginAction {
    let mut action = LoginAction::None;
    let screen = ctx.content_rect();
    egui::Area::new(Id::unique("moon_login_menu"))
        .fixed_pos(screen.min)
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(if p.high_contrast { p.bar } else { p.bar.gamma_multiply(0.58) })
                .inner_margin(egui::Margin::symmetric(15, 5))
                .show(ui, |ui| {
                    ui.set_width((screen.width() - 30.0).max(0.0));
                    ui.set_min_height(29.0);
                    // The bar is the window's title bar: drag its empty part
                    // to move the window, double-click it to maximize. The
                    // menus and buttons added after it take the clicks first.
                    let strip = Rect::from_min_size(screen.min, vec2(screen.width(), 39.0));
                    let grab = ui.interact(strip, Id::unique("moon_login_window_drag"), egui::Sense::click_and_drag());
                    if grab.double_clicked() {
                        action = LoginAction::ToggleMaximize;
                    } else if grab.drag_started() {
                        action = LoginAction::DragWindow;
                    }
                    if !p.high_contrast {
                        let strip = Rect::from_min_size(screen.min, vec2(screen.width(), 39.0));
                        widgets::gradient_rect(
                            ui.painter(),
                            strip,
                            0.0,
                            p.indigo_light.gamma_multiply(0.23),
                            p.navy.gamma_multiply(0.12),
                        );
                        // glass sheen: a pearl highlight over the bar's upper half
                        let sheen = Rect::from_min_size(screen.min, vec2(screen.width(), 18.0));
                        widgets::gradient_rect(
                            ui.painter(),
                            sheen,
                            0.0,
                            p.ink.gamma_multiply(0.16),
                            egui::Color32::TRANSPARENT,
                        );
                    }
                    egui::MenuBar::new().ui(ui, |ui| {
                        let (mark, _) = ui.allocate_exact_size(vec2(29.0, 28.0), egui::Sense::hover());
                        if !p.high_contrast {
                            super::super::prototype_bg::radial_glow(ui.painter(), mark.center(), 19.0, p.amber, 0.27);
                        }
                        super::super::dragon_art::orb(ui.painter(), mark.center(), 13.0, 4);
                        if screen.width() >= 650.0 {
                            let font = egui::FontId::new(22.0, super::super::fonts::display_bold(ui.ctx()));
                            let mut job = egui::text::LayoutJob::default();
                            let gold = egui::Color32::from_rgb(255, 231, 130);
                            job.append("DRAGON", 0.0, egui::TextFormat { font_id: font.clone(), color: gold, italics: true, ..Default::default() });
                            job.append(" VIEWER", 0.0, egui::TextFormat { font_id: font, color: egui::Color32::WHITE, italics: true, ..Default::default() });
                            ui.label(job);
                            ui.add_space(8.0);
                        }
                        ui.separator();
                        // The mockup's menus (bold italic, wide): what works before login
                        ui.style_mut()
                            .text_styles
                            .insert(egui::TextStyle::Button, egui::FontId::new(17.0, super::super::fonts::display_bold(ui.ctx())));
                        ui.spacing_mut().button_padding = vec2(16.0, 4.0);
                        ui.menu_button(prototype_i18n::tr!("dragon-menu-file"), |ui| {
                            menu_action(ui, prototype_i18n::tr!("common-preferences"), &mut action, LoginAction::Preferences);
                            menu_action(ui, prototype_i18n::tr!("moon-login-advanced"), &mut action, LoginAction::Advanced);
                            ui.separator();
                            menu_action(ui, prototype_i18n::tr!("ui-login-exit"), &mut action, LoginAction::Quit);
                        });
                        ui.menu_button(prototype_i18n::tr!("dragon-menu-edit"), |ui| {
                            menu_action(ui, prototype_i18n::tr!("common-preferences"), &mut action, LoginAction::Preferences);
                        });
                        ui.menu_button(prototype_i18n::tr!("dragon-menu-view"), |ui| {
                            menu_action(ui, prototype_i18n::tr!("common-maximize"), &mut action, LoginAction::ToggleMaximize);
                        });
                        for name in [prototype_i18n::tr!("dragon-menu-world"), prototype_i18n::tr!("dragon-menu-build")] {
                            ui.menu_button(name, |ui| {
                                ui.add_enabled(false, egui::Button::new(prototype_i18n::tr!("dragon-menu-after-login")));
                            });
                        }
                        // Help only before login: news, updates, bug reports
                        ui.menu_button(prototype_i18n::tr!("moon-menu-help"), |ui| {
                            if ui.button(prototype_i18n::tr!("ui-login-news")).clicked() {
                                form.show_info = true;
                                ui.close();
                            }
                            menu_action(
                                ui,
                                prototype_i18n::tr!("moon-menu-check-updates"),
                                &mut action,
                                LoginAction::CheckUpdates,
                            );
                            menu_action(ui, prototype_i18n::tr!("moon-menu-report-bug"), &mut action, LoginAction::ReportBug);
                        });
                    });
                });
            let edge = ui.min_rect().bottom();
            // the skin's long gold ornament across the bar, left of the
            // window buttons (the mockup's)
            if !p.high_contrast {
                let at = Rect::from_min_size(pos2(screen.right() - 215.0 - 285.0, screen.top() - 8.0), vec2(285.0, 58.0));
                crate::ui::dragon_art::paint_fill(ui.painter(), "gold-dragon", at, egui::Color32::WHITE);
            }
            if !p.high_contrast {
                // a soft gold glow under the bar's edge
                ui.painter().hline(screen.x_range(), edge + 1.0, egui::Stroke::new(4.0, p.amber.gamma_multiply(0.16)));
            }
            ui.painter().hline(screen.x_range(), edge, egui::Stroke::new(1.2, p.amber.gamma_multiply(0.85)));
            // the version stays in Debug › Copy version, out of the bar
        });
    action
}

fn caption_button(ui: &mut egui::Ui, p: &Palette, icon: &str, label: &str, close: bool) -> bool {
    widgets::glossy_caption_button(ui, p, icon, label, close, vec2(34.0, 28.0))
}

fn caption_controls(ctx: &Context, p: &Palette, action: &mut LoginAction) {
    let maximized = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
    let (maximize_icon, maximize_label) = if maximized {
        ("copy", prototype_i18n::tr!("ui-options-restore"))
    } else {
        ("square", prototype_i18n::tr!("common-maximize"))
    };
    egui::Area::new(Id::unique("login_window_controls"))
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::RIGHT_TOP, vec2(-10.0, 5.0))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 7.0;
                if caption_button(ui, p, "minus", prototype_i18n::tr!("region-restart-minimize"), false) {
                    *action = LoginAction::Minimize;
                }
                if caption_button(ui, p, maximize_icon, maximize_label, false) {
                    *action = LoginAction::ToggleMaximize;
                }
                if caption_button(ui, p, "x", prototype_i18n::tr!("common-close"), true) {
                    *action = LoginAction::Quit;
                }
            });
        });
}

/// The window's edge grips (no Windows frame on the login screen).
fn resize_edges(ctx: &Context, action: &mut LoginAction) {
    if let Some(dir) = widgets::window_resize_grips(ctx) {
        *action = LoginAction::ResizeWindow(dir);
    }
}

pub fn sidebar(ctx: &Context, p: &Palette, icons: &Icons, info: &LoginInfo, card: Rect, width: f32, lowest: f32) -> bool {
    let left = ctx.content_rect().right() - 32.0 - width;
    egui::Area::new(Id::unique("login_news"))
        .fixed_pos(egui::pos2(left, card.top()))
        .show(ctx, |ui| {
            ui.set_width(width);
            super::news_panel(ui, p, icons, info, card.height().min(lowest - card.top()).max(120.0));
        });
    egui::Area::new(Id::unique("login_version"))
        .pivot(egui::Align2::LEFT_BOTTOM)
        .fixed_pos(egui::pos2(left, card.top() - 14.0))
        .show(ctx, |ui| {
            ui.set_width(width);
            super::version_panel(ui, p, icons, info)
        })
        .inner
}

pub fn footer(ctx: &Context, p: &Palette, icons: &Icons, info: &LoginInfo, sidebar_visible: bool, form: &mut LoginForm) -> LoginAction {
    let mut action = menu_bar(ctx, p, form);
    if form.safe_graphics.is_some() || form.photo_quality_off || form.camera_limits_restored || form.driver_notice.is_some() {
        super::driver_notice::column(ctx, |ui, width| {
            if let Some(state) = form.safe_graphics
                && let Some(choice) = super::safe_graphics_notice::card(ui, p, icons, state, width)
            {
                action = LoginAction::SafeGraphics(choice);
            }
            if form.photo_quality_off
                && let Some(back_on) = super::photo_quality_notice::card(ui, p, icons, width)
            {
                action = LoginAction::PhotoQualityOff(back_on);
            }
            if form.camera_limits_restored
                && let Some(open) = super::camera_limits_notice::card(ui, p, icons, width)
            {
                action = LoginAction::CameraLimits(open);
            }
            if let Some(notice) = &form.driver_notice
                && super::driver_notice::card(ui, p, icons, notice, width)
            {
                action = LoginAction::DismissDriverNotice;
            }
        });
    }
    let _ = sidebar_visible;
    // Quick Preferences and Accessibility beside the caption buttons.
    egui::Area::new(Id::unique("login_settings"))
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::RIGHT_TOP, vec2(-138.0, 4.0))
        .show(ctx, |ui| {
            if widgets::action_button(ui, p, "settings-filled", "", ActionKind::Flat, Vec2::splat(30.0))
                .on_hover_text(prototype_i18n::tr!("common-preferences"))
                .clicked()
            {
                action = LoginAction::Preferences;
            }
        });
    // Left of it, the way to a larger, more readable interface before the
    // first login: Preferences › Accessibility.
    egui::Area::new(Id::unique("login_accessibility"))
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::RIGHT_TOP, vec2(-176.0, 4.0))
        .show(ctx, |ui| {
            if widgets::action_button(ui, p, "accessible-filled", "", ActionKind::Flat, Vec2::splat(30.0))
                .on_hover_text(prototype_i18n::tr!("login-accessibility"))
                .clicked()
            {
                action = LoginAction::Accessibility;
            }
        });
    caption_controls(ctx, p, &mut action);
    resize_edges(ctx, &mut action);
    if form.show_info {
        let id = Id::unique("accounts_news_dialog");
        prototype_render::blurred_backdrop(ctx, egui::Modal::default_area(id).layer());
        let response = widgets::Modal::new(id).transparent_backdrop().show(ctx, |ui| {
            ui.set_width(380.0);
            ui.spacing_mut().item_spacing.y = 8.0;
            // Prototype: the bear and the name, like Preferences' title row
            ui.horizontal(|ui| {
                if let Some(bear) = super::super::prototype_art::texture(ctx, super::super::prototype_art::Art::Bear) {
                    let size = super::super::prototype_art::size_at_width(&bear, 30.0);
                    ui.add(egui::Image::new(&bear).fit_to_exact_size(size));
                }
                ui.label(RichText::new(prototype_i18n::tr!("ui-login-news")).size(20.0).color(p.ink));
            });
            if super::version_panel(ui, p, icons, info) {
                action = LoginAction::CheckUpdates;
            }
            ui.add_space(8.0);
            super::news_panel(ui, p, icons, info, 235.0);
            ui.add_space(4.0);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if widgets::action_button(
                    ui,
                    p,
                    "",
                    prototype_i18n::tr!("common-close"),
                    ActionKind::Primary,
                    Vec2::new(130.0, 40.0),
                )
                .clicked()
                {
                    form.show_info = false;
                }
            });
        });
        if response.should_close() {
            form.show_info = false;
        }
    }
    action
}
