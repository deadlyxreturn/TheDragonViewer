//! Login screen: animated prototype background, brand block and the login
//! card in the middle, version and news panels on the right, community
//! links at the bottom left. Cards and buttons use the surfaces of the
//! in-world windows (`widgets::overlay_frame`, `widgets::action_button`);
//! their blur needs a world behind, the prototype is soft enough without it.

use super::icons::Icons;
use super::news::{LoginInfo, VersionState};
use super::widgets::{self, ActionKind};
use crate::settings::{GridChoice, Settings};
use crate::theme::Palette;
use egui::{Color32, Frame, Margin, RichText, Sense, Stroke, TextEdit, Vec2};
mod camera_limits_notice;
mod carousel;
mod chrome;
mod credentials_form;
mod derez;
mod driver_notice;
mod fade;
mod layout;
pub(crate) mod manager;
mod manager_drag;
mod moon;
pub(crate) mod motion;
pub mod photo_quality_notice;
pub mod safe_graphics_notice;

/// Prototype's pills: the fields' and buttons' round ends, and their glass edge.
const PILL_RADIUS: u8 = 9;

/// Room between the login card's rim and its fields.
const CARD_SIDE: f32 = 54.0;

const REMEMBERED_PASSWORD_MASK: &str = "••••••••••••••••";
/// Second Life's own pages behind the card's links (Prototype).
const FORGOT_PASSWORD_URL: &str = "https://secondlife.com/my/account/request.php";
const REGISTER_URL: &str = "https://join.secondlife.com/";

#[derive(Default)]
pub struct LoginForm {
    pub accounts: Vec<crate::login_accounts::Account>,
    pub accounts_loaded: bool,
    pub remember_checked: bool,
    pub remember_mfa: bool,
    pub mfa_authorization: Option<crate::credentials::MfaAuthorization>,
    pub mfa_for: Option<(String, String)>,
    pub content_height: f32,
    pub editing: bool,
    pub adding: bool,
    pub add_return: Option<(String, String)>,
    pub places: crate::login_places::Cache,
    pub managing: bool,
    pub show_info: bool,
    pub page: usize,
    pub selection_shown_for: Option<(String, String)>,
    pub pending_connect: Option<usize>,
    pub pending_since: Option<f64>,
    pub wheel_scroll: f32,
    pub wheel_time: Option<f64>,
    pub motion: motion::State,
    pub manager: manager::State,
    pub transition: Option<derez::Transition>,
    pub portraits: std::collections::HashMap<uuid::Uuid, egui::TextureHandle>,
    pub portrait_rx: Option<crossbeam_channel::Receiver<(uuid::Uuid, egui::ColorImage)>>,
    pub password: String,
    pub mfa_token: String,
    pub show_mfa: bool,
    pub tos_message: Option<String>,
    pub agree_tos: bool,
    /// The login server's critical message (reason "critical"), to read
    /// before the next attempt; `read_critical` is its own acknowledgement,
    /// apart from the Terms of Service (LLLoginInstance::handleTOSResponse).
    pub critical_message: Option<String>,
    pub read_critical: bool,
    /// The other server the login server sends the login on to
    /// (`NetEvent::LoginFailed::next_server`), until the next attempt;
    /// `accept_next_server` is its own checkbox.
    pub next_server: Option<String>,
    pub accept_next_server: bool,
    /// The other servers agreed to for this account and grid, named in
    /// every attempt (`LoginRequest::accepted_next_servers`).
    pub accepted_next_servers: Vec<String>,
    pub error: Option<String>,
    /// The session ended with its region's restart: the login button reads
    /// "Reconnect" (region_restart.rs).
    pub reconnect: bool,
    pub busy: bool,
    /// A visible reason keeps the button and all login paths disabled together.
    pub block_reason: Option<&'static str>,
    /// Password shown in clear (eye button).
    pub show_password: bool,
    /// The first field to fill already has the keyboard focus (set again
    /// when the login screen comes back).
    pub focused: bool,
    /// Remembered login hash for the shown user (never displayed).
    pub stored: Option<String>,
    /// Grid and user name `stored` was read for.
    pub stored_for: Option<(String, String)>,
    /// A place given from outside (command line, a link of a web page):
    /// where the next login starts, whatever the account's choice, shown in
    /// the start menus (FSPanelLogin::onUpdateStartSLURL). Not saved.
    pub start_slurl: Option<crate::slurl::Location>,
    /// Every account's favourite landmarks, written by the sessions
    /// (`login_favorites.rs`), offered in the start menus.
    pub favorites: crate::login_favorites::Store,
    /// The favourite chosen in an account's start menu: where that
    /// account's next login starts. Not saved, like the Legacy Viewer's
    /// start combo (only Home and Last are kept as LoginLocation).
    pub start_favorite: Option<(crate::login_favorites::Key, uuid::Uuid)>,
    /// The graphics driver is out of date (`gpu_driver::check`), and the
    /// user has not dismissed the warning for this driver.
    pub driver_notice: Option<crate::gpu_driver::Notice>,
    /// The safe graphics offered after the GPU stopped answering, or the
    /// way back from them (`app/safe_graphics.rs`).
    pub safe_graphics: Option<safe_graphics_notice::SafeGraphics>,
    /// "Photo quality" was switched off after the last session crashed with
    /// it on (`app/snapshot.rs`, `photo_quality_at_start`).
    pub photo_quality_off: bool,
    /// The camera's limits went back on with the settings' version 33
    /// (`Settings::camera_limits_restored`).
    pub camera_limits_restored: bool,
}

impl LoginForm {
    /// Another account or grid: no other login server is agreed to any more.
    pub fn forget_next_server(&mut self) {
        self.next_server = None;
        self.accept_next_server = false;
        self.accepted_next_servers.clear();
    }
}

#[derive(Default)]
pub enum LoginAction {
    ReportBug,
    /// The bug button's right click, as in the top bar.
    CopyVersion,
    /// "Ignore for this driver" on the out of date driver warning.
    DismissDriverNotice,
    /// A button of the safe graphics card.
    SafeGraphics(safe_graphics_notice::Choice),
    /// A button of the photo quality card: turned back on (true) or read.
    PhotoQualityOff(bool),
    /// A button of the camera limits card: the camera preferences (true) or
    /// read.
    CameraLimits(bool),
    #[default]
    None,
    Login,
    Quit,
    Minimize,
    ToggleMaximize,
    /// A press and drag on the empty part of the top bar moves the window
    /// (Dragon Viewer has no Windows title bar on the login screen).
    DragWindow,
    /// A drag on one of the window's edges or corners resizes it.
    ResizeWindow(winit::window::ResizeDirection),
    CheckUpdates,
    /// The cogwheel: the preferences window over the login screen.
    Preferences,
    /// The accessibility button beside it: the preferences on their
    /// Accessibility page.
    Accessibility,
    Advanced,
}

/// "Second Life client · Rust · <graphics API>" under the name (login
/// screen, its carousel, About), with the API this build draws with.
pub(crate) fn client_tagline() -> &'static str {
    if cfg!(target_os = "macos") {
        prototype_i18n::tr!("ui-login-second-life-client-rust-metal")
    } else {
        prototype_i18n::tr!("ui-login-second-life-client-rust-vulkan")
    }
}

/// The Terms of Service page of Linden Lab's grids (the `real_url` of
/// LLFloaterTOS, floater_tos.xml).
const SECOND_LIFE_TERMS: &str = "https://secondlife.com/app/tos/";

/// The page of the Terms of Service a login asks to accept: the first web
/// address of the login server's `message` (an OpenSim grid names its page
/// there, LLFloaterTOS::postBuild), else Linden Lab's page on their grids.
pub(crate) fn terms_url(message: &str, login_uri: &str) -> Option<String> {
    let named = message.split_whitespace().find_map(|word| {
        let word = word.trim_matches(|c: char| matches!(c, '<' | '>' | '(' | ')' | '[' | ']' | '"' | '\'' | ',' | ';' | '.'));
        let url = reqwest::Url::parse(word).ok()?;
        matches!(url.scheme(), "http" | "https").then(|| String::from(url))
    });
    named.or_else(|| prototype_net::login::is_linden_login(login_uri).then(|| SECOND_LIFE_TERMS.to_owned()))
}

/// The link to the Terms of Service page, opened in the browser.
pub(crate) fn terms_link(ui: &mut egui::Ui, p: &Palette, url: &str) {
    let label = prototype_i18n::tr!("ui-login-terms-of-service-read");
    let size = Vec2::new(ui.available_width(), 28.0);
    if widgets::action_button(ui, p, "external-link-filled", label, ActionKind::Flat, size)
        .on_hover_text(url)
        .clicked()
    {
        ui.ctx().open_url(egui::OpenUrl::new_tab(url));
    }
}

/// Animated prototype background (also behind the loading screens without a
/// last view).
pub fn draw_background(ui: &egui::Ui, p: &Palette) {
    let ctx = ui.ctx();
    let rect = ctx.content_rect();
    let painter = ctx.layer_painter(egui::LayerId::background());
    // Prototype: the glass bear in the code rain; the prototype until it is
    // decoded (a few frames at start)
    if let Some(bg) = super::prototype_art::background(ctx) {
        super::prototype_art::paint_cover(&painter, &bg, rect, Color32::WHITE); // theme-guard: content — an image drawn untinted
        return;
    }
    let t = ctx.input(|i| i.time) as f32;
    super::prototype_bg::paint(&painter, p, rect, t, 1.0);
    // slow animation: ~30 images/s are plenty
    ctx.request_repaint_after(std::time::Duration::from_millis(33));
}

fn img(ui: &egui::Ui, icons: &Icons, name: &str, rect: egui::Rect, tint: Color32) {
    if let Some(t) = icons.get(name) {
        ui.painter().image(
            t.id(),
            rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            tint,
        );
    }
}

/// Text field with a leading icon (and an optional eye toggle for passwords).
#[allow(clippy::too_many_arguments)]
fn field(
    ui: &mut egui::Ui,
    p: &Palette,
    icons: &Icons,
    id: &str,
    icon: &str,
    hint: &str,
    text: &mut String,
    password: Option<&mut bool>,
) -> egui::Response {
    // Only the username and editable grid match ASCII-restricted combo
    // editors in panel_login.xml/panel_fs_login.xml. Passwords, MFA, region
    // locations and the account search retain their independent policies.
    let restriction = matches!(id, "login_user" | "new_account_user" | "login_grid").then_some(widgets::TextPolicy::Ascii);
    let id = egui::Id::unique(id);
    let focused = ui.memory(|m| m.has_focus(id));
    let edge = if focused { Color32::from_rgb(255, 242, 165) } else { p.amber };
    let ink = p.ink;
    let icon_tint = if icon == "user-filled" { p.teal } else { p.ink };
    let mut surface = None;
    let response = Frame::new()
        .fill(if p.high_contrast { p.field } else { Color32::TRANSPARENT })
        .stroke(if p.high_contrast { Stroke::new(1.5, edge) } else { Stroke::NONE })
        .corner_radius(PILL_RADIUS) // theme-guard: content — round glass fields
        .inner_margin(Margin { left: 13, right: 12, top: 6, bottom: 6 })
        .show(ui, |ui| {
            surface = Some((ui.painter().clone(), ui.painter().add(egui::Shape::Noop)));
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 9.0;
                let (r, _) = ui.allocate_exact_size(Vec2::splat(24.0), Sense::hover());
                img(ui, icons, icon, r, icon_tint);
                let eye_w = if password.is_some() { 26.0 } else { 0.0 };
                let w = ui.available_width() - eye_w;
                let remembered = password.is_some() && text.is_empty() && hint == REMEMBERED_PASSWORD_MASK;
                ui.visuals_mut().weak_text_color = Some(ink);
                let hidden = remembered || password.as_deref().is_some_and(|show| !*show);
                // Interactivity comes from the disabled UI and TextEdit.
                // Keep this form's explicit ink color while it is disabled.
                let resp = widgets::text_field_with_policy(ui, true, text, restriction, |b, ui| {
                    TextEdit::singleline(b)
                        .id(id)
                        .interactive(ui.is_enabled())
                        .frame(
                            egui::Frame::NONE
                                .inner_margin(crate::ui::widgets::field_text_margin(ui.ctx(), &egui::FontId::proportional(16.0))),
                        )
                        .password(hidden)
                        .hint_text(RichText::new(hint).color(ink.gamma_multiply(0.82)))
                        .font(egui::FontId::proportional(18.0))
                        .text_color(ink)
                        .desired_width(w)
                });
                if let Some(show) = password {
                    let (r, eye) = ui.allocate_exact_size(Vec2::splat(18.0), if remembered { Sense::hover() } else { Sense::click() });
                    let col = if !remembered && eye.hovered() {
                        ink
                    } else {
                        ink.gamma_multiply(0.8)
                    };
                    img(ui, icons, if *show && !remembered { "eye-off" } else { "eye-filled" }, r, col);
                    let tip = if remembered {
                        prototype_i18n::tr!("ui-login-password-remembered-enter-a-new-one-to-replace-it")
                    } else if *show {
                        prototype_i18n::tr!("ui-login-hide")
                    } else {
                        prototype_i18n::tr!("common-show")
                    };
                    if eye.on_hover_text(tip).clicked() && !remembered {
                        *show = !*show;
                    }
                }
                resp
            })
            .inner
        });
    if let Some((painter, slot)) = surface.filter(|_| !p.high_contrast) {
        painter.set(slot, moon::gold_field(ui.ctx(), response.response.rect, edge, focused));
    }
    response.inner
}

fn caption(ui: &mut egui::Ui, p: &Palette, text: &str) {
    ui.label(RichText::new(text).size(11.5).color(p.muted));
    ui.add_space(-2.0);
}

/// The favourites of one account's start menu and the one chosen (its
/// item), as FSPanelLogin::addFavoritesToStartLocation adds them after
/// Last, Home and the typed region.
pub(super) struct StartFavorites<'a> {
    pub list: &'a [crate::login_favorites::Favorite],
    pub chosen: Option<uuid::Uuid>,
}

impl<'a> StartFavorites<'a> {
    #[cfg(test)]
    pub fn none() -> Self {
        StartFavorites { list: &[], chosen: None }
    }

    /// The account `key`'s favourites, and its choice in `slot`.
    pub fn of(
        store: &'a crate::login_favorites::Store,
        slot: &Option<(crate::login_favorites::Key, uuid::Uuid)>,
        key: &crate::login_favorites::Key,
    ) -> Self {
        StartFavorites {
            list: store.favorites(key),
            chosen: slot.as_ref().filter(|(k, _)| k == key).map(|(_, id)| *id),
        }
    }

    /// Keep in `slot` a choice made in the account `key`'s menu; another
    /// account's choice stays until this one changes.
    pub fn keep(&self, slot: &mut Option<(crate::login_favorites::Key, uuid::Uuid)>, key: crate::login_favorites::Key) {
        let before = slot.as_ref().filter(|(k, _)| *k == key).map(|(_, id)| *id);
        if before != self.chosen {
            *slot = self.chosen.map(|id| (key, id));
        }
    }

    /// The chosen favourite, while the account still has it.
    pub fn chosen(&self) -> Option<&crate::login_favorites::Favorite> {
        self.chosen.and_then(|id| self.list.iter().find(|f| f.item == id))
    }
}

/// "Region, x, y, z" under a favourite of the start menu.
fn favorite_place(favorite: &crate::login_favorites::Favorite) -> String {
    let [x, y, z] = favorite.pos;
    format!("{} ({}, {}, {})", favorite.region, x.round(), y.round(), z.round())
}

/// PROTOTYPE_DEMO_LOGIN_FAVORITES=open: the first start menu with favourites
/// (the combo `salt` of `ui`) opens once, after the login screen's first
/// frames, for captures.
pub(super) fn demo_open_start_menu(ui: &egui::Ui, salt: &str, favorites: &StartFavorites) {
    static OPENED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    if !favorites.list.is_empty()
        && crate::login_favorites::demo() == Some(crate::login_favorites::Demo::Open)
        && ui.ctx().cumulative_frame_nr() >= 30
        && !OPENED.swap(true, std::sync::atomic::Ordering::Relaxed)
    {
        egui::Popup::open_id(ui.ctx(), ui.make_persistent_id(egui::IdSalt::new(salt)).with("popup"));
    }
}

/// The start location menu: last location, home or a region, then the
/// account's favourite landmarks; a place given from outside (`slurl`)
/// comes first and stays chosen until another entry is, as the Legacy
/// Viewer's location combo shows the start SLURL
/// (FSPanelLogin::onUpdateStartSLURL).
pub(super) fn start_menu(
    ui: &mut egui::Ui,
    combo: egui::ComboBox,
    start: &mut String,
    slurl: &mut Option<crate::slurl::Location>,
    favorites: &mut StartFavorites,
) -> egui::Response {
    let place = slurl.as_ref().map(crate::external_url::place_label);
    let chosen = favorites.chosen().cloned();
    let selected = match (&place, &chosen, start.as_str()) {
        (Some(place), ..) => place.clone(),
        (None, Some(favorite), _) => favorite.name.clone(),
        (None, None, "home") => prototype_i18n::tr!("common-home").to_owned(),
        (None, None, "region") => prototype_i18n::tr!("ui-login-region").to_owned(),
        _ => prototype_i18n::tr!("ui-login-last-location").to_owned(),
    };
    // the favourites unfold over up to half the screen before scrolling
    let combo = if favorites.list.is_empty() {
        combo
    } else {
        combo.height(ui.spacing().combo_height.max(ui.ctx().content_rect().height() * 0.5))
    };
    let shown = combo.icon(widgets::combo_chevron).selected_text(selected).show_ui(ui, |ui| {
        ui.visuals_mut().override_text_color = Some(crate::theme::Theme::current_palette(ui.ctx()).ink);
        if let Some(place) = &place {
            let _ = ui.selectable_label(true, place);
        }
        let usual = place.is_none() && chosen.is_none();
        for (value, text) in [
            ("last", prototype_i18n::tr!("ui-login-last-location")),
            ("home", prototype_i18n::tr!("common-home")),
            ("region", prototype_i18n::tr!("ui-login-region")),
        ] {
            if ui.selectable_label(usual && start == value, text).clicked() {
                *start = value.into();
                *slurl = None;
                favorites.chosen = None;
            }
        }
        if !favorites.list.is_empty() {
            ui.add_space(4.0);
            ui.label(egui::RichText::new(prototype_i18n::tr!("ui-login-favorites")).size(11.5).weak());
        }
        for favorite in favorites.list {
            let on = place.is_none() && chosen.as_ref().is_some_and(|c| c.item == favorite.item);
            if ui
                .selectable_label(on, &favorite.name)
                .on_hover_text(favorite_place(favorite))
                .clicked()
            {
                favorites.chosen = Some(favorite.item);
                *slurl = None;
            }
        }
    });
    // a card's narrow menu cuts the place short
    if let Some(place) = place {
        shown.response.on_hover_text(place)
    } else if let Some(favorite) = &chosen {
        shown
            .response
            .on_hover_text(format!("{}\n{}", favorite.name, favorite_place(favorite)))
    } else {
        shown.response
    }
}

/// Height of a link in the login and account cards.
const BAR_BUTTON: f32 = 30.0;

/// Community link with its icon and name; `url` empty = not configured.
fn link_button(ui: &mut egui::Ui, p: &Palette, icon: &str, name: &str, url: &str) {
    let enabled = !url.is_empty();
    let resp = ui
        .add_enabled_ui(enabled, |ui| {
            widgets::action_button(ui, p, icon, name, ActionKind::Flat, Vec2::new(0.0, BAR_BUTTON))
        })
        .inner;
    let resp = resp
        .on_hover_text(url)
        .on_disabled_hover_text(prototype_i18n::tr!("ui-login-pending-link", name = name.to_string()));
    if resp.clicked() && enabled {
        ui.ctx().open_url(egui::OpenUrl::new_tab(url));
    }
}

/// A strip whose color runs through `stops` (position 0..1, color) from left
/// to right.
pub(crate) fn strip(r: egui::Rect, stops: &[(f32, Color32)]) -> egui::Shape {
    let mut mesh = egui::Mesh::default();
    for (at, color) in stops {
        let x = egui::lerp(r.left()..=r.right(), *at);
        mesh.colored_vertex(egui::pos2(x, r.top()), *color);
        mesh.colored_vertex(egui::pos2(x, r.bottom()), *color);
    }
    for i in 0..stops.len().saturating_sub(1) as u32 {
        let n = i * 2;
        mesh.add_triangle(n, n + 2, n + 1);
        mesh.add_triangle(n + 1, n + 2, n + 3);
    }
    egui::Shape::mesh(mesh)
}

/// Card of the login screen: a pane of frosted glass. Under the tint of the
/// in-world windows it shows the sky behind it out of focus
/// (`prototype_bg::frosted`) and two lights of the prototype, which also spill
/// around the card; its top edge catches the light, and the card the screen
/// is for (`accent`) wears the brand line. An opaque skin (panel_alpha =
/// 255) keeps its plain surface.
fn card<R>(ui: &mut egui::Ui, p: &Palette, margin: i8, accent: bool, body: impl FnOnce(&mut egui::Ui) -> R) -> egui::InnerResponse<R> {
    card_hover(ui, p, margin, accent, 0.0, body)
}

fn card_hover<R>(
    ui: &mut egui::Ui,
    p: &Palette,
    margin: i8,
    accent: bool,
    hover: f32,
    body: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<R> {
    glass_card(ui, p, margin, accent, hover, None, body)
}

/// A card edged in the caution colour: a warning about the machine itself
/// (`driver_notice.rs`).
fn card_caution<R>(ui: &mut egui::Ui, p: &Palette, margin: i8, body: impl FnOnce(&mut egui::Ui) -> R) -> egui::InnerResponse<R> {
    glass_card(ui, p, margin, false, 0.0, Some(p.amber), body)
}

fn glass_card<R>(
    ui: &mut egui::Ui,
    p: &Palette,
    margin: i8,
    accent: bool,
    hover: f32,
    edge: Option<Color32>,
    body: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<R> {
    let behind = (ui.painter().clone(), ui.painter().add(egui::Shape::Noop));
    let mut pane = None;
    let mut frame = widgets::overlay_frame(ui.ctx(), p);
    frame.stroke.color = edge.unwrap_or_else(|| frame.stroke.color.lerp_to_gamma(p.violet_pale, hover * 0.7));
    let response = frame.inner_margin(Margin::same(margin)).show(ui, |ui| {
        pane = Some((ui.painter().clone(), ui.painter().add(egui::Shape::Noop)));
        body(ui)
    });
    if let Some((painter, slot)) = pane.filter(|_| p.panel_alpha < 255) {
        let ctx = ui.ctx();
        let outer = response.response.rect;
        // inside the border of the frame
        let rect = outer.shrink(1.0);
        let strength = if accent { 1.0 } else { 0.6 } + hover * 0.45;
        let lights = vec![
            super::prototype_bg::radial_glow_shape(
                outer.left_top() + Vec2::new(outer.width() * 0.18, outer.height() * 0.12),
                outer.width() * 0.95,
                p.violet,
                0.24 * strength,
            ),
            super::prototype_bg::radial_glow_shape(
                outer.right_bottom() - Vec2::new(outer.width() * 0.12, outer.height() * 0.14),
                outer.width() * 0.8,
                p.teal,
                0.13 * strength,
            ),
        ];
        behind.0.set(behind.1, egui::Shape::Vec(lights.clone()));
        let mut shapes = super::prototype_bg::frosted_pane(p, ctx.content_rect(), ctx.input(|i| i.time) as f32, rect, lights);
        let top = |height: f32| egui::Rect::from_min_size(rect.min, Vec2::new(rect.width(), height));
        if accent {
            shapes.push(strip(top(2.0), &[(0.0, p.violet), (1.0, p.teal)]));
        } else {
            let lit = p.ink.gamma_multiply(0.30);
            shapes.push(strip(
                top(1.0),
                &[(0.0, Color32::TRANSPARENT), (0.3, lit), (1.0, Color32::TRANSPARENT)],
            ));
        }
        painter
            .with_clip_rect(rect.intersect(painter.clip_rect()))
            .set(slot, egui::Shape::Vec(shapes));
    }
    response
}

/// The main glass panel on the login screen: deep night glass in a gold
/// rim (the skin's `login-frame` picture instead when it has one).
fn proto_card<R>(ui: &mut egui::Ui, p: &Palette, body: impl FnOnce(&mut egui::Ui) -> R) -> egui::InnerResponse<R> {
    let mut surface = None;
    let response = Frame::new()
        .fill(if p.high_contrast { p.panel } else { Color32::TRANSPARENT })
        .stroke(if p.high_contrast { widgets::border_stroke(p, Stroke::new(2.0, p.amber)) } else { Stroke::NONE })
        .corner_radius(39) // theme-guard: content — the login card's rim
        .inner_margin(Margin { left: CARD_SIDE as i8, right: CARD_SIDE as i8, top: 52, bottom: 26 })
        .show(ui, |ui| {
            surface = Some((ui.painter().clone(), ui.painter().add(egui::Shape::Noop)));
            body(ui)
        });
    if let Some((painter, slot)) = surface.filter(|_| !p.high_contrast) {
        painter.set(slot, moon::card_face(ui.ctx(), response.response.rect));
    }
    response
}

/// A labelled menu row of the card (Grid, Start Location): the label, then
/// the menu in a dark field with a gold rim and a gold arrow.
fn pill_menu(ui: &mut egui::Ui, p: &Palette, _icons: &Icons, _icon: &str, label: &str, menu: impl FnOnce(&mut egui::Ui) -> egui::Response) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        let (label_rect, _) = ui.allocate_exact_size(Vec2::new(111.0, 28.0), Sense::hover());
        let galley = ui.painter().layout_no_wrap(label.to_owned(), egui::FontId::proportional(16.0), p.ink);
        widgets::paint_ink_left(ui.painter(), galley, label_rect.left_center() + Vec2::new(2.0, 0.0), p.ink);
        ui.visuals_mut().override_text_color = Some(p.ink);
        let w = &mut ui.visuals_mut().widgets;
        for state in [&mut w.inactive, &mut w.hovered, &mut w.active, &mut w.open] {
            state.bg_fill = Color32::TRANSPARENT;
            state.weak_bg_fill = Color32::TRANSPARENT;
            state.bg_stroke = Stroke::NONE;
        }
        let value_surface = ui.painter().add(egui::Shape::Noop);
        let choice = menu(ui);
        if !p.high_contrast {
            let hot = choice.hovered() || choice.has_focus();
            ui.painter().set(value_surface, moon::gold_field(ui.ctx(), choice.rect.expand2(Vec2::new(0.0, 1.0)), if hot { Color32::from_rgb(255, 242, 165) } else { p.amber }, hot));
        }
    });
}

/// The width a `text_link` of `label` takes.
fn text_link_size(ui: &egui::Ui, p: &Palette, label: &str, _url: &str, _underline: bool) -> f32 {
    ui.painter()
        .layout_no_wrap(label.to_owned(), egui::FontId::proportional(14.5), p.ink)
        .size()
        .x
}

/// A plain underlined text link (the card's "Create account" and "Forgot
/// password?"), lavender-white, brighter while hovered: no button face.
fn text_link(ui: &mut egui::Ui, p: &Palette, label: &str, url: &str) -> egui::Response {
    let resp = ui.add(egui::Label::new(RichText::new(label).size(14.5).color(p.ink)).sense(Sense::click()));
    if resp.hovered() {
        ui.painter().hline(resp.rect.x_range(), resp.rect.bottom() - 1.0, Stroke::new(1.2, Color32::from_rgb(255, 230, 134)));
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    if resp.clicked() {
        ui.ctx().open_url(egui::OpenUrl::new_tab(url));
    }
    resp.on_hover_text(url)
}

/// A pill of frosted glass with its icon and label (the card's foot).
#[allow(dead_code)]
fn pill_button(ui: &mut egui::Ui, p: &Palette, icons: &Icons, icon: &str, label: &str, size: Vec2) -> egui::Response {
    let (r, resp) = ui.allocate_exact_size(size, Sense::click());
    let radius = r.height() * 0.5;
    let fill = if resp.hovered() {
        p.field.gamma_multiply(0.85)
    } else {
        p.field.gamma_multiply(0.5)
    };
    if p.high_contrast {
        ui.painter().rect_filled(r, radius, fill);
        ui.painter().rect_stroke(
            r,
            radius,
            widgets::border_stroke(p, Stroke::new(1.2, p.amber)),
            egui::StrokeKind::Inside,
        );
    } else {
        let tint = match icon {
            "settings-filled" => p.indigo_light,
            "star-filled" => p.violet_light,
            _ => p.teal,
        };
        let hot = resp.hovered() || resp.has_focus();
        if hot {
            // a soft gold halo while hovered
            ui.painter().rect_stroke(
                r.expand(3.0),
                radius + 3.0,
                Stroke::new(4.0, p.amber.gamma_multiply(0.20)),
                egui::StrokeKind::Outside,
            );
        }
        // glossy glass: a light lavender top fading to deep indigo, a
        // bright reflection and a gold rim (polished_pill)
        let top = p.indigo_light.lerp_to_gamma(p.ink, 0.18).lerp_to_gamma(tint, 0.25);
        let bottom = p.navy.lerp_to_gamma(tint, 0.18);
        ui.painter().add(moon::polished_pill(
            ui.ctx(),
            r,
            top.gamma_multiply(if hot { 0.78 } else { 0.62 }),
            bottom.gamma_multiply(if hot { 0.80 } else { 0.66 }),
            p.ink,
            p.amber.lerp_to_gamma(p.ink, 0.2),
        ));
    }
    let font_size = if size.x < 108.0 { 12.5 } else { 15.5 };
    let galley = ui
        .painter()
        .layout_no_wrap(label.to_owned(), egui::FontId::proportional(font_size), p.ink);
    let total = 22.0 + 8.0 + galley.size().x;
    let x0 = r.center().x - total * 0.5;
    img(
        ui,
        icons,
        icon,
        egui::Rect::from_center_size(egui::pos2(x0 + 11.0, r.center().y), Vec2::splat(20.0)),
        p.amber,
    );
    super::widgets::paint_ink_left(&ui.painter().with_clip_rect(r), galley, egui::pos2(x0 + 30.0, r.center().y), p.ink);
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp
}

/// Prototype's frosted card inside a panel (Preferences' content card).
fn info_card<R>(ui: &mut egui::Ui, p: &Palette, margin: i8, body: impl FnOnce(&mut egui::Ui) -> R) -> egui::InnerResponse<R> {
    Frame::new()
        .fill(p.field.gamma_multiply(0.5))
        .stroke(Stroke::new(1.2, p.amber.gamma_multiply(0.7)))
        .corner_radius(18) // theme-guard: content — Prototype's frosted cards
        .inner_margin(Margin::same(margin))
        .show(ui, body)
}

fn version_panel(ui: &mut egui::Ui, p: &Palette, icons: &Icons, info: &LoginInfo) -> bool {
    let mut check_updates = false;
    info_card(ui, p, 16, |ui| {
        ui.set_width(ui.available_width());
        widgets::section_header(ui, p, "tag-filled", prototype_i18n::tr!("ui-login-version"));
        let current = prototype_version::version();
        ui.horizontal(|ui| {
            ui.label(RichText::new(prototype_i18n::tr!("ui-login-installed")).size(12.5).color(p.muted));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // a beta's long number elides beside a longer translated label
                ui.add(egui::Label::new(RichText::new(format!("v{current}")).size(13.0).strong().color(p.ink)).truncate());
            });
        });
        ui.add_space(2.0);
        let (icon, text, col) = match &info.version {
            VersionState::UpToDate => (
                "circle-check-filled",
                prototype_i18n::tr!("ui-login-up-to-date").to_owned(),
                p.success,
            ),
            VersionState::Available { version, .. } => (
                "sparkles-filled",
                prototype_i18n::tr!("status-v-version-available", version = version.to_string()),
                p.amber,
            ),
            VersionState::Checking => ("refresh", prototype_i18n::tr!("ui-login-checking").to_owned(), p.muted),
            VersionState::Failed => ("refresh", prototype_i18n::tr!("ui-login-unable-to-check").to_owned(), p.muted),
            VersionState::NotConfigured => (
                "circle-check-filled",
                prototype_i18n::tr!("ui-login-development-version").to_owned(),
                p.muted,
            ),
        };
        ui.horizontal(|ui| {
            ui.label(RichText::new(prototype_i18n::tr!("ui-login-latest")).size(12.5).color(p.muted));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(RichText::new(text).size(12.5).color(col));
                let (r, _) = ui.allocate_exact_size(Vec2::splat(14.0), Sense::hover());
                img(ui, icons, icon, r, col);
            });
        });
        if info.can_check_updates {
            ui.add_space(8.0);
            let size = Vec2::new(ui.available_width(), 36.0);
            check_updates = widgets::action_button(ui, p, "refresh", prototype_i18n::tr!("update-check"), ActionKind::Flat, size)
                .on_hover_text(prototype_i18n::tr!("update-check-hint"))
                .clicked();
        }
        if let VersionState::Available { url, .. } = &info.version
            && !url.is_empty()
        {
            ui.add_space(8.0);
            let size = Vec2::new(ui.available_width(), 30.0);
            let label = prototype_i18n::tr!("ui-login-download-update");
            if widgets::action_button(ui, p, "download-filled", label, ActionKind::Primary, size).clicked() {
                ui.ctx().open_url(egui::OpenUrl::new_tab(url));
            }
        }
    });
    check_updates
}

/// Side of the mark after the title of a news that opens a link.
const LINK_MARK: f32 = 13.0;

/// News card, `height` tall whatever the number of news (the list scrolls).
fn news_panel(ui: &mut egui::Ui, p: &Palette, icons: &Icons, info: &LoginInfo, height: f32) {
    const MARGIN: i8 = 16;
    info_card(ui, p, MARGIN, |ui| {
        ui.set_width(ui.available_width());
        ui.set_height(height - 2.0 * f32::from(MARGIN));
        widgets::section_header(ui, p, "article-filled", prototype_i18n::tr!("ui-login-news"));
        let rest = ui.available_height();
        egui::ScrollArea::vertical()
            .max_height(rest)
            .min_scrolled_height(rest)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 4.0;
                for (i, n) in info.news.iter().enumerate() {
                    if i > 0 {
                        ui.add_space(4.0);
                        let r = ui.available_rect_before_wrap();
                        // lighter than in a window: the card is glass over the sky
                        let mut line = widgets::divider_stroke(p);
                        line.color = line.color.gamma_multiply(0.7);
                        ui.painter().hline(r.x_range(), r.top(), line);
                        ui.add_space(6.0);
                    }
                    let has_url = !n.url.is_empty();
                    let resp = ui
                        .scope(|ui| {
                            if !n.date.is_empty() {
                                ui.label(RichText::new(&n.date).size(11.0).color(p.muted_dim));
                            }
                            // a title of several lines leaves the link mark its
                            // place: it stood beyond the card, which grew wider
                            let mark = if has_url { LINK_MARK + ui.spacing().item_spacing.x } else { 0.0 };
                            let room = ui.available_width() - mark;
                            ui.horizontal(|ui| {
                                ui.scope(|ui| {
                                    ui.set_max_width(room);
                                    ui.add(egui::Label::new(RichText::new(&n.title).size(14.0).strong().color(p.ink)).wrap());
                                });
                                if has_url {
                                    let (r, _) = ui.allocate_exact_size(Vec2::splat(LINK_MARK), Sense::hover());
                                    img(ui, icons, "external-link-filled", r, p.muted);
                                }
                            });
                            if !n.summary.is_empty() {
                                ui.add(egui::Label::new(RichText::new(&n.summary).size(12.5).color(p.muted)).wrap());
                            }
                        })
                        .response;
                    if has_url {
                        let r = ui.interact(resp.rect, ui.scope_id().with(("news", i)), Sense::click());
                        if r.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                        if r.clicked() {
                            ui.ctx().open_url(egui::OpenUrl::new_tab(&n.url));
                        }
                    }
                }
            });
    });
}

#[allow(clippy::too_many_arguments)]
pub fn show(
    ui: &mut egui::Ui,
    p: &Palette,
    icons: &Icons,
    logo: Option<&egui::TextureHandle>,
    settings: &mut Settings,
    form: &mut LoginForm,
    info: &LoginInfo,
    progress: Option<(&str, f32)>,
) -> LoginAction {
    super::prototype_art::set_background_choice(settings.prototype_login_bg);
    // demo: PROTOTYPE_DEMO_LOGIN_INFO opens the news and version panel once
    static DEMO_INFO: std::sync::Once = std::sync::Once::new();
    if std::env::var_os("PROTOTYPE_DEMO_LOGIN_INFO").is_some() {
        DEMO_INFO.call_once(|| form.show_info = true);
    }
    // Prototype: one login card for everyone (the mockup's), the saved
    // accounts still filling it in; the account carousel stays for the
    // transition it is already playing.
    if form.accounts_loaded && form.transition.is_some() {
        return carousel::show(ui, p, icons, logo, settings, form, info, progress);
    }
    draw_background(ui, p);
    let mut action = LoginAction::None;
    let ctx = ui.ctx().clone();
    let screen = ctx.content_rect();
    let wide = screen.width() >= 1100.0;
    // Dragon Viewer's mockup (1672 × 941): the card at the lower left,
    // 436 wide, its crest overhanging its top edge
    let s = (screen.width() / 1672.0).min(screen.height() / 941.0).clamp(0.82, 1.3);
    let card_w = (436.0 * s).clamp(400.0, 560.0).min(screen.width() - 24.0);
    let card_x = if wide { 104.0 * s } else { (screen.width() - card_w) * 0.5 };
    let crest_room = 96.0;
    let card_y = (359.0 * s)
        .min(screen.height() - 470.0)
        .max(44.0 + crest_room);
    let room = (screen.height() - card_y - 26.0).max(190.0);
    let area = egui::Id::unique("login_area");
    let card_rect = egui::Area::new(area)
        .default_size(Vec2::new(card_w, room))
        .fixed_pos(screen.min + Vec2::new(card_x, card_y))
        .show(&ctx, |ui| {
            ui.set_width(card_w);
            proto_card(ui, p, |ui| {
                let content_width = card_w - 2.0 * CARD_SIDE;
                ui.set_width(content_width);
                // Area reuses the previous measured height. Let the form
                // grow again when the viewport grows or a challenge appears.
                ui.set_max_height(room - 40.0);
                egui::ScrollArea::vertical()
                    .id_salt("dragon_login_form")
                    .max_height(room - 40.0)
                    .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        ui.set_width(content_width);
                        ui.spacing_mut().item_spacing.y = 6.0;
                        let enabled = !form.busy && !super::grids::is_open(&ctx);
                        ui.add_enabled_ui(enabled, |ui| {
                            let user = field(
                                ui,
                                p,
                                icons,
                                "login_user",
                                "user-filled",
                                prototype_i18n::tr!("common-username"),
                                &mut settings.username,
                                None,
                            );
                            ui.add_space(4.0);
                            // LLPanelLogin::setFields (indra/newview/llpanellogin.cpp,
                            // originally LGPL 2.1): show 16 masked filler characters.
                            // Keep the marker out of the input so login still uses the
                            // saved hash, and typing replaces it without editing filler.
                            let hint = if form.stored.is_some() {
                                REMEMBERED_PASSWORD_MASK
                            } else {
                                prototype_i18n::tr!("ui-login-password")
                            };
                            let pass = field(
                                ui,
                                p,
                                icons,
                                "login_pass",
                                "lock-filled",
                                hint,
                                &mut form.password,
                                Some(&mut form.show_password),
                            );
                            let mut mfa = None;
                            if form.show_mfa {
                                ui.add_space(4.0);
                                ui.label(
                                    RichText::new(prototype_i18n::tr!("ui-login-authentication-code-2fa"))
                                        .size(11.5)
                                        .color(p.amber),
                                );
                                mfa = Some(field(
                                    ui,
                                    p,
                                    icons,
                                    "login_mfa",
                                    "shield-lock-filled",
                                    "000000",
                                    &mut form.mfa_token,
                                    None,
                                ));
                                widgets::checkbox(ui, p, &mut form.remember_mfa, prototype_i18n::tr!("login-accounts-remember-mfa"));
                            }

                            ui.add_space(6.0);
                            pill_menu(ui, p, icons, "world-filled", prototype_i18n::tr!("dragon-login-grid"), |ui| {
                                let mut uri = settings.login_uri();
                                let choice = super::grids::selector(ui, &settings.grids, &mut uri);
                                settings.grid = crate::login_accounts::grid(&uri);
                                if settings.grid == GridChoice::Custom && !uri.is_empty() {
                                    settings.custom_login_uri = uri;
                                }
                                choice
                            });
                            let key = crate::login_favorites::key(&settings.login_uri(), &settings.username);
                            let mut favorites = StartFavorites::of(&form.favorites, &form.start_favorite, &key);
                            pill_menu(ui, p, icons, "map-pin-filled", prototype_i18n::tr!("dragon-login-start"), |ui| {
                                let width = ui.available_width();
                                demo_open_start_menu(ui, "start", &favorites);
                                start_menu(
                                    ui,
                                    egui::ComboBox::from_id_salt("start").width(width).truncate(),
                                    &mut settings.start_location,
                                    &mut form.start_slurl,
                                    &mut favorites,
                                )
                            });
                            favorites.keep(&mut form.start_favorite, key);
                            let key = crate::login_favorites::key(&settings.login_uri(), &settings.username);
                            let favorite_chosen = StartFavorites::of(&form.favorites, &form.start_favorite, &key).chosen().is_some();
                            if settings.grid == GridChoice::Custom {
                                field(
                                    ui,
                                    p,
                                    icons,
                                    "login_grid",
                                    "world-filled",
                                    "http://grille.exemple:8002/",
                                    &mut settings.custom_login_uri,
                                    None,
                                );
                            }
                            if settings.start_location == "region" && form.start_slurl.is_none() && !favorite_chosen {
                                field(
                                    ui,
                                    p,
                                    icons,
                                    "login_region",
                                    "map-pin-filled",
                                    prototype_i18n::tr!("ui-login-region-name"),
                                    &mut settings.start_region,
                                    None,
                                );
                            }
                            ui.add_space(2.0);
                            ui.spacing_mut().item_spacing.y = 2.0;
                            widgets::checkbox(ui, p, &mut settings.remember_username, prototype_i18n::tr!("dragon-login-remember-username"));
                            if widgets::checkbox(ui, p, &mut settings.remember_password, prototype_i18n::tr!("dragon-login-remember-password"))
                                .changed()
                                && settings.remember_password
                            {
                                // a password is remembered with its account's name
                                settings.remember_username = true;
                            }
                            ui.spacing_mut().item_spacing.y = 6.0;
                            ui.add_space(2.0);
                            let can = form.block_reason.is_none()
                                && form.tos_message.is_none()
                                && !settings.login_uri().is_empty()
                                && (!form.show_mfa || !form.mfa_token.trim().is_empty())
                                && !settings.username.trim().is_empty()
                                && (!form.password.is_empty() || form.stored.is_some());
                            // the mockup's LOG IN reaches past the fields on both sides
                            // the mockup's 380 × 82 jewel (the skin art's own shape)
                            let (row, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 84.0), Sense::hover());
                            let jewel_w = content_width + 52.0;
                            let r = egui::Rect::from_center_size(row.center(), Vec2::new(jewel_w, (jewel_w / 4.63).clamp(60.0, 96.0)));
                            let resp = ui.interact(r, egui::Id::unique("dragon_log_in"), if can { Sense::click() } else { Sense::hover() });
                            // Requested UX: focus the first missing field, or Connect
                            // when credentials are ready. Firestorm giveFocus otherwise
                            // returns to the user-name field when both are present.
                            if !form.focused && enabled {
                                form.focused = true;
                                match mfa.as_ref() {
                                    Some(m) if form.mfa_token.is_empty() => m.request_focus(),
                                    _ if settings.username.trim().is_empty() => user.request_focus(),
                                    _ if can => resp.request_focus(),
                                    _ => pass.request_focus(),
                                }
                            }
                            let hot = can && (resp.hovered() || resp.has_focus());
                            let label = if form.reconnect {
                                prototype_i18n::tr!("ui-login-reconnect")
                            } else {
                                prototype_i18n::tr!("ui-login-log-in")
                            };
                            resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, can && enabled, label));
                            moon::connect_button(ui.painter(), r, p, can, hot, resp.is_pointer_button_down_on());
                            let ink = if p.high_contrast { p.on_accent } else { Color32::WHITE };
                            super::dragon_art::outlined_text(
                                ui.painter(),
                                r.center() + Vec2::new(0.0, -1.0),
                                &label.to_uppercase(),
                                33.0,
                                if can { ink } else { ink.gamma_multiply(0.75) },
                                Color32::from_rgb(63, 28, 14),
                            );
                            if hot {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                            }
                            let enter =
                                (user.lost_focus() || pass.lost_focus() || mfa.is_some_and(|field| field.lost_focus()) || resp.has_focus())
                                    && ui.input(|i| i.key_pressed(egui::Key::Enter));
                            if (resp.clicked() || enter) && can && enabled {
                                action = LoginAction::Login;
                            }
                            ui.add_space(2.0);
                            {
                                let a = text_link_size(ui, p, prototype_i18n::tr!("dragon-login-create-account"), REGISTER_URL, false);
                                let b = text_link_size(ui, p, prototype_i18n::tr!("ui-login-forgot-password"), FORGOT_PASSWORD_URL, false);
                                let (row, _) = ui.allocate_exact_size(Vec2::new(content_width, 22.0), Sense::hover());
                                let total = a + b + 34.0;
                                let x = row.left() + (content_width - total).max(0.0) * 0.5;
                                let first = egui::Rect::from_min_size(egui::pos2(x, row.top()), Vec2::new(a + 2.0, 22.0));
                                let second = egui::Rect::from_min_size(egui::pos2(x + a + 34.0, row.top()), Vec2::new(b + 2.0, 22.0));
                                text_link(&mut ui.new_child(egui::UiBuilder::new().max_rect(first)), p, prototype_i18n::tr!("dragon-login-create-account"), REGISTER_URL);
                                ui.painter().vline(
                                    x + a + 17.0,
                                    egui::Rangef::new(row.top() + 3.0, row.bottom() - 3.0),
                                    Stroke::new(1.0, p.ink.gamma_multiply(0.65)),
                                );
                                text_link(&mut ui.new_child(egui::UiBuilder::new().max_rect(second)), p, prototype_i18n::tr!("ui-login-forgot-password"), FORGOT_PASSWORD_URL);
                            }
                        });
                        if let Some(reason) = form.block_reason {
                            ui.add_space(8.0);
                            ui.label(RichText::new(reason).size(12.0).color(p.amber));
                        }
                        if let Some(msg) = form.tos_message.clone() {
                            ui.add_space(8.0);
                            widgets::divider(ui, p);
                            ui.label(
                                RichText::new(prototype_i18n::tr!("ui-login-terms-of-service"))
                                    .strong()
                                    .color(p.amber),
                            );
                            egui::ScrollArea::vertical().max_height(120.0).show(ui, |ui| {
                                ui.label(RichText::new(&msg).size(12.0).color(p.muted));
                            });
                            if let Some(url) = terms_url(&msg, &settings.login_uri()) {
                                terms_link(ui, p, &url);
                            }
                            if ui
                                .add_enabled(
                                    form.block_reason.is_none() && !form.busy,
                                    egui::Button::new(prototype_i18n::tr!("ui-login-i-accept-the-terms-and-want-to-log-in")),
                                )
                                .clicked()
                            {
                                form.agree_tos = true;
                                form.tos_message = None;
                                action = LoginAction::Login;
                            }
                        }
                        // a step of its own, as the Legacy Viewer's message_critical
                        // floater: reading it acknowledges this message only
                        if let Some(msg) = form.critical_message.clone() {
                            ui.add_space(8.0);
                            widgets::divider(ui, p);
                            ui.label(RichText::new(prototype_i18n::tr!("ui-login-critical-message")).strong().color(p.amber));
                            egui::ScrollArea::vertical()
                                .id_salt("critical_message")
                                .max_height(120.0)
                                .show(ui, |ui| {
                                    ui.label(RichText::new(msg).size(12.0).color(p.muted));
                                });
                            if ui
                                .add_enabled(
                                    form.block_reason.is_none() && !form.busy,
                                    egui::Button::new(prototype_i18n::tr!("ui-login-critical-message-read")),
                                )
                                .clicked()
                            {
                                form.read_critical = true;
                                form.critical_message = None;
                                action = LoginAction::Login;
                            }
                        }
                        // the login would go on to another server: the user decides
                        if let Some(server) = form.next_server.clone() {
                            ui.add_space(8.0);
                            widgets::divider(ui, p);
                            ui.label(RichText::new(prototype_i18n::tr!("ui-login-next-server")).strong());
                            ui.label(
                                RichText::new(prototype_i18n::tr!("ui-login-next-server-body", server = server.as_str()))
                                    .size(12.0)
                                    .color(p.muted),
                            );
                            if ui
                                .add_enabled(
                                    form.block_reason.is_none() && !form.busy,
                                    egui::Button::new(prototype_i18n::tr!("ui-login-next-server-accept")),
                                )
                                .clicked()
                            {
                                form.accepted_next_servers.push(server);
                                form.next_server = None;
                                action = LoginAction::Login;
                            }
                        }
                        if let Some((msg, frac)) = progress {
                            ui.add_space(10.0);
                            widgets::progress(ui, p, "login_progress", frac, ui.available_width(), "", None);
                            ui.label(RichText::new(msg).size(12.0).color(p.violet_pale));
                        }
                        if let Some(err) = &form.error {
                            ui.add_space(8.0);
                            ui.label(RichText::new(err).size(12.5).color(p.danger));
                        }
                    });
            })
            .response
            .rect
        })
        .inner;
    if !p.high_contrast {
        moon::card_ornaments(&ctx, card_rect);
    }

    let chrome_action = chrome::footer(&ctx, p, icons, info, wide, form);
    if !matches!(chrome_action, LoginAction::None) {
        action = chrome_action;
    }
    action
}

#[cfg(test)]
pub(crate) fn test_output(mut output: egui::FullOutput) -> egui::FullOutput {
    // Headless interaction tests inspect shapes and actions without a GPU uploader.
    output.textures_delta.clear();
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn required_update_disables_mouse_enter_and_terms_with_synthetic_credentials() {
        fn position(shape: &egui::Shape, label: &str) -> Option<egui::Pos2> {
            match shape {
                egui::Shape::Text(t) if t.galley.text() == label => Some(t.pos + egui::vec2(3.0, 5.0)),
                egui::Shape::Vec(shapes) => shapes.iter().find_map(|s| position(s, label)),
                _ => None,
            }
        }
        let ctx = egui::Context::default();
        let mut settings = Settings {
            username: "Demo Resident".into(),
            ..Default::default()
        };
        let mut form = LoginForm {
            password: "synthetic".into(),
            block_reason: Some(prototype_i18n::tr!("update-login-required")),
            ..Default::default()
        };
        let p = crate::theme::Theme::default().palette();
        let icons = Icons::default();
        let info = LoginInfo::preview_release("0.3.0");
        let draw = |events, form: &mut LoginForm, settings: &mut Settings| {
            let mut action = LoginAction::None;
            let mut output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, Vec2::new(1600.0, 900.0))),
                    events,
                    ..Default::default()
                },
                |ui| {
                    action = show(ui, &p, &icons, None, settings, form, &info, None);
                },
            );
            output.textures_delta.clear();
            (action, output)
        };
        draw(vec![], &mut form, &mut settings);
        let (_, output) = draw(vec![], &mut form, &mut settings);
        assert!(
            output
                .shapes
                .iter()
                .any(|s| position(&s.shape, prototype_i18n::tr!("update-login-required")).is_some())
        );
        let pos = output
            .shapes
            .iter()
            .find_map(|s| position(&s.shape, prototype_i18n::tr!("ui-login-log-in")))
            .expect("button");
        for pressed in [true, false] {
            let (action, _) = draw(
                vec![
                    egui::Event::PointerMoved(pos),
                    egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: Default::default(),
                    },
                ],
                &mut form,
                &mut settings,
            );
            assert!(matches!(action, LoginAction::None));
        }
        ctx.memory_mut(|m| m.request_focus(egui::Id::unique("login_pass")));
        let (action, _) = draw(
            vec![egui::Event::Key {
                key: egui::Key::Enter,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Default::default(),
            }],
            &mut form,
            &mut settings,
        );
        assert!(matches!(action, LoginAction::None));
        form.tos_message = Some("Synthetic terms".into());
        let (_, output) = draw(vec![], &mut form, &mut settings);
        let pos = output
            .shapes
            .iter()
            .find_map(|s| position(&s.shape, prototype_i18n::tr!("ui-login-i-accept-the-terms-and-want-to-log-in")))
            .expect("terms");
        for pressed in [true, false] {
            let (action, _) = draw(
                vec![
                    egui::Event::PointerMoved(pos),
                    egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: Default::default(),
                    },
                ],
                &mut form,
                &mut settings,
            );
            assert!(matches!(action, LoginAction::None));
        }
        assert!(!form.agree_tos);
        // Verify the same fixture can log in once the policy permits it.
        form.tos_message = None;
        form.block_reason = None;
        draw(vec![], &mut form, &mut settings);
        let (_, output) = draw(vec![], &mut form, &mut settings);
        let pos = output
            .shapes
            .iter()
            .find_map(|s| position(&s.shape, prototype_i18n::tr!("ui-login-log-in")))
            .expect("enabled button");
        draw(
            vec![
                egui::Event::PointerMoved(pos),
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: Default::default(),
                },
            ],
            &mut form,
            &mut settings,
        );
        let (action, _) = draw(
            vec![egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: Default::default(),
            }],
            &mut form,
            &mut settings,
        );
        assert!(matches!(action, LoginAction::Login));
    }

    /// The Terms of Service link: the page the server's message names, else
    /// Linden Lab's on their grids, else none.
    #[test]
    fn the_terms_link_is_the_page_the_server_names_or_linden_lab_s() {
        let grid = "http://grid.example.org:8002/";
        assert_eq!(
            terms_url("Please accept the new terms.", prototype_net::login::SL_MAIN_GRID).as_deref(),
            Some(SECOND_LIFE_TERMS)
        );
        assert_eq!(
            terms_url("Read them (https://grid.example.org/tos.html).", grid).as_deref(),
            Some("https://grid.example.org/tos.html")
        );
        assert_eq!(terms_url("Accept the terms.", grid), None);
        assert_eq!(terms_url("ftp://grid.example.org/tos", grid), None);
    }

    /// The critical message is a step of its own: reading it sets
    /// `read_critical` alone, and both flags start false.
    #[test]
    fn separate_step_for_the_login_critical_message() {
        fn position(shape: &egui::Shape, label: &str) -> Option<egui::Pos2> {
            match shape {
                egui::Shape::Text(t) if t.galley.text() == label => Some(t.pos + egui::vec2(3.0, 5.0)),
                egui::Shape::Vec(shapes) => shapes.iter().find_map(|s| position(s, label)),
                _ => None,
            }
        }
        let ctx = egui::Context::default();
        let mut settings = Settings {
            username: "Demo Resident".into(),
            ..Default::default()
        };
        let mut form = LoginForm {
            password: "synthetic".into(),
            critical_message: Some("Synthetic message".into()),
            ..Default::default()
        };
        assert!(!form.agree_tos && !form.read_critical);
        let p = crate::theme::Theme::default().palette();
        let icons = Icons::default();
        let info = LoginInfo::preview_release("0.3.0");
        let mut draw = |events, form: &mut LoginForm| {
            let mut action = LoginAction::None;
            let mut output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, Vec2::new(1600.0, 900.0))),
                    events,
                    ..Default::default()
                },
                |ui| {
                    action = show(ui, &p, &icons, None, &mut settings, form, &info, None);
                },
            );
            output.textures_delta.clear();
            (action, output)
        };
        draw(vec![], &mut form);
        let (_, output) = draw(vec![], &mut form);
        let pos = output
            .shapes
            .iter()
            .find_map(|s| position(&s.shape, prototype_i18n::tr!("ui-login-critical-message-read")))
            .expect("the critical message's button");
        let mut last = LoginAction::None;
        for pressed in [true, false] {
            last = draw(
                vec![
                    egui::Event::PointerMoved(pos),
                    egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: Default::default(),
                    },
                ],
                &mut form,
            )
            .0;
        }
        assert!(matches!(last, LoginAction::Login));
        assert!(form.read_critical);
        assert!(!form.agree_tos);
        assert!(form.critical_message.is_none());
    }

    /// Another login server is a step of its own: agreeing names that
    /// server in the next attempt, and nothing else.
    #[test]
    fn another_login_server_is_agreed_to_in_its_own_step() {
        fn position(shape: &egui::Shape, label: &str) -> Option<egui::Pos2> {
            match shape {
                egui::Shape::Text(t) if t.galley.text() == label => Some(t.pos + egui::vec2(3.0, 5.0)),
                egui::Shape::Vec(shapes) => shapes.iter().find_map(|s| position(s, label)),
                _ => None,
            }
        }
        let ctx = egui::Context::default();
        let mut settings = Settings {
            username: "Demo Resident".into(),
            ..Default::default()
        };
        let mut form = LoginForm {
            password: "synthetic".into(),
            next_server: Some("login.other-grid.example".into()),
            ..Default::default()
        };
        let p = crate::theme::Theme::default().palette();
        let icons = Icons::default();
        let info = LoginInfo::preview_release("0.3.0");
        let mut draw = |events, form: &mut LoginForm| {
            let mut action = LoginAction::None;
            let mut output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, Vec2::new(1600.0, 900.0))),
                    events,
                    ..Default::default()
                },
                |ui| {
                    action = show(ui, &p, &icons, None, &mut settings, form, &info, None);
                },
            );
            output.textures_delta.clear();
            (action, output)
        };
        draw(vec![], &mut form);
        let (_, output) = draw(vec![], &mut form);
        let body = prototype_i18n::tr!("ui-login-next-server-body", server = "login.other-grid.example");
        assert!(
            output.shapes.iter().any(|s| position(&s.shape, &body).is_some()),
            "the server is named"
        );
        let pos = output
            .shapes
            .iter()
            .find_map(|s| position(&s.shape, prototype_i18n::tr!("ui-login-next-server-accept")))
            .expect("the other server's button");
        let mut last = LoginAction::None;
        for pressed in [true, false] {
            last = draw(
                vec![
                    egui::Event::PointerMoved(pos),
                    egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: Default::default(),
                    },
                ],
                &mut form,
            )
            .0;
        }
        assert!(matches!(last, LoginAction::Login));
        assert_eq!(form.accepted_next_servers, ["login.other-grid.example"]);
        assert!(form.next_server.is_none());
        assert!(!form.agree_tos && !form.read_critical);
        form.forget_next_server();
        assert!(form.accepted_next_servers.is_empty());
    }

    /// At 1280 × 720 the news, as tall as the login card, stood over the
    /// version and the Quit button of the line along the bottom.
    #[test]
    fn the_login_card_fits_below_the_menu_at_supported_sizes() {
        let p = crate::theme::Theme::default().palette();
        let icons = Icons::default();
        let info = LoginInfo::preview_release("0.1.0");
        for size in [Vec2::new(800.0, 560.0), Vec2::new(1280.0, 720.0), Vec2::new(1672.0, 941.0)] {
            let ctx = egui::Context::default();
            let mut settings = Settings::default();
            let mut form = LoginForm::default();
            for _ in 0..3 {
                let mut output = ctx.run_ui(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size)),
                        ..Default::default()
                    },
                    |ui| {
                        show(ui, &p, &icons, None, &mut settings, &mut form, &info, None);
                    },
                );
                output.textures_delta.clear();
            }
            let (menu, card) = ctx.memory(|m| {
                (
                    m.area_rect(egui::Id::unique("moon_login_menu")).expect("menu"),
                    m.area_rect(egui::Id::unique("login_area")).expect("login card"),
                )
            });
            assert!(menu.bottom() <= card.top(), "{size:?}: menu {menu:?} intersects card {card:?}");
            assert!(card.right() <= size.x && card.bottom() <= size.y, "{size:?}: card {card:?}");
            if size.y >= 941.0 {
                assert!(card.height() >= 620.0, "{size:?}: login controls clipped in {card:?}");
            }
        }
    }

    /// The cogwheel opens the preferences before login, from the top right
    /// corner, beside the narrow window's update button without covering it.
    #[test]
    fn the_cogwheel_asks_for_the_preferences_from_the_top_right_corner() {
        let p = crate::theme::Theme::default().palette();
        let icons = Icons::default();
        let info = LoginInfo::preview_release("0.3.0");
        for size in [Vec2::new(1000.0, 700.0), Vec2::new(1600.0, 900.0)] {
            let ctx = egui::Context::default();
            let mut settings = Settings::default();
            let mut form = LoginForm::default();
            let mut frame = |events: Vec<egui::Event>| {
                let mut action = LoginAction::None;
                let output = ctx.run_ui(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size)),
                        events,
                        ..Default::default()
                    },
                    |ui| action = show(ui, &p, &icons, None, &mut settings, &mut form, &info, None),
                );
                test_output(output);
                action
            };
            for _ in 0..3 {
                frame(vec![]);
            }
            let rect = |name: &str| ctx.memory(|m| m.area_rect(egui::Id::unique(name)));
            let cog = rect("login_settings").expect("the cogwheel");
            let caption = rect("login_window_controls").expect("the window controls");
            assert!(
                caption.left() >= cog.right() && caption.right() >= size.x - 20.0 && cog.top() <= 25.0,
                "{size:?}: the cog sits left of the window controls ({cog:?} {caption:?})"
            );
            for other in ["login_check_updates", "login_news", "login_version", "login_area"] {
                if let Some(r) = rect(other) {
                    assert!(!r.intersects(cog), "{size:?}: {other} {r:?} under the cogwheel {cog:?}");
                }
            }
            let pos = cog.center();
            frame(vec![
                egui::Event::PointerMoved(pos),
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: Default::default(),
                },
            ]);
            let action = frame(vec![egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: Default::default(),
            }]);
            assert!(matches!(action, LoginAction::Preferences), "{size:?}");

            // the accessibility button, just left of it, opens their
            // Accessibility page
            let access = rect("login_accessibility").expect("the accessibility button");
            assert!(
                access.right() < cog.left() && cog.left() - access.right() <= 12.0 && (access.center().y - cog.center().y).abs() < 1.0,
                "{size:?}: left of the cogwheel ({access:?}, {cog:?})"
            );
            assert_eq!(access.size(), cog.size(), "{size:?}: the cogwheel's size");
            for other in ["login_check_updates", "login_news", "login_version", "login_area"] {
                if let Some(r) = rect(other) {
                    assert!(
                        !r.intersects(access),
                        "{size:?}: {other} {r:?} under the accessibility button {access:?}"
                    );
                }
            }
            let pos = access.center();
            frame(vec![
                egui::Event::PointerMoved(pos),
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: Default::default(),
                },
            ]);
            let action = frame(vec![egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: Default::default(),
            }]);
            assert!(matches!(action, LoginAction::Accessibility), "{size:?}");
        }
    }

    /// A news with a link and a title of several lines made its card wider
    /// than the version card above it: the link mark stood after a title
    /// that already took the whole width.
    #[test]
    fn a_long_news_title_with_a_link_keeps_the_card_width() {
        let ctx = egui::Context::default();
        let p = crate::theme::Theme::default().palette();
        let icons = Icons::default();
        let mut info = LoginInfo::preview_release("0.3.0");
        let plain = info.news.clone();
        info.news.insert(
            0,
            super::super::news::NewsItem {
                date: "2026-10-13".into(),
                title: "A title long enough to run over several lines of the news card of the login screen".into(),
                summary: "Text.".into(),
                url: "https://example.org/news".into(),
            },
        );
        let width = |info: &LoginInfo| {
            let mut used = 0.0;
            for _ in 0..2 {
                let mut output = ctx.run_ui(Default::default(), |ui| {
                    ui.set_width(330.0);
                    used = ui.scope(|ui| news_panel(ui, &p, &icons, info, 400.0)).response.rect.width();
                });
                output.textures_delta.clear();
            }
            used
        };
        let with_link = width(&info);
        info.news = plain;
        assert_eq!(with_link, width(&info), "the card is as wide with the link as without");
    }

    #[test]
    fn saved_password_marker_survives_focus_without_becoming_input() {
        let ctx = egui::Context::default();
        let p = crate::theme::Theme::default().palette();
        let icons = Icons::default();
        let mut password = String::new();
        let mut show_password = false;
        let mut draw = |hint: &str, events: Vec<egui::Event>| {
            let mut output = ctx.run_ui(
                egui::RawInput {
                    events,
                    ..Default::default()
                },
                |ui| {
                    ui.set_width(300.0);
                    field(
                        ui,
                        &p,
                        &icons,
                        "test_pass",
                        "lock-filled",
                        hint,
                        &mut password,
                        Some(&mut show_password),
                    );
                },
            );
            // This headless test has no renderer to consume texture updates.
            output.textures_delta.clear();
            output
        };
        let has_mask = |output: &egui::FullOutput| {
            output
                .shapes
                .iter()
                .any(|s| matches!(&s.shape, egui::Shape::Text(t) if t.galley.job.text == REMEMBERED_PASSWORD_MASK))
        };
        assert!(!has_mask(&draw("", vec![])));
        ctx.memory_mut(|m| m.request_focus(egui::Id::unique("test_pass")));
        assert!(has_mask(&draw(REMEMBERED_PASSWORD_MASK, vec![])));
        assert!(ctx.memory(|m| m.has_focus(egui::Id::unique("test_pass"))));
        draw(REMEMBERED_PASSWORD_MASK, vec![egui::Event::Text("replacement".into())]);
        // egui paints the empty-field hint for the entry frame; the next frame
        // must show the newly typed value rather than the remembered marker.
        assert!(!has_mask(&draw(REMEMBERED_PASSWORD_MASK, vec![])));
        assert_eq!(password, "replacement");
        assert!(!show_password);
    }
}
