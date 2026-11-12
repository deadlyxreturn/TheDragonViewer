//! Preferences › Prototype: what Prototype does differently from the Legacy
//! Viewer, and for each redrawn element the way back to the Legacy look.
//! The entry of the sidebar carries the prototype gradient, the one exception
//! to the theme here (docs/BRANDING.md); the settings keep the rows of the
//! other pages.

use super::super::cx::{Cx, F, field};
use crate::build::MoveHologram;
use crate::settings::{ChatStyle, DestinationsStyle, NotificationsStyle, PhotoWindowStyle, Settings, WorldMapStyle};
use crate::theme::Palette;
use crate::world::lookat::{Aim, Style};
use egui::epaint::{Mesh, Vertex, WHITE_UV};
use egui::{Color32, FontId, Pos2, Rect, Sense, Shape, Vec2};

/// The two looks a redrawn element can take.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Look {
    Prototype,
    Legacy,
}

/// Every choice of the page set to one look (the colored sky ambient
/// included), and Prototype's water with it;
/// the avatar plates' own settings and the filing of received items are
/// left as they are, and so is a gaze on the point under the cursor: the
/// Prototype look does not choose to show others what the cursor is over.
pub fn set_all(s: &mut Settings, look: Look) {
    let prototype = look == Look::Prototype;
    s.chat_style = if prototype { ChatStyle::Prototype } else { ChatStyle::Legacy };
    s.classic_name_tags = !prototype;
    s.lookat.style = if prototype { Style::Prototype } else { Style::Legacy };
    if !prototype {
        s.lookat.aim = Aim::Direction;
    }
    s.notifications_style = if prototype {
        NotificationsStyle::Prototype
    } else {
        NotificationsStyle::Legacy
    };
    s.world_map_style = if prototype { WorldMapStyle::Prototype } else { WorldMapStyle::Legacy };
    s.classic_bars = !prototype;
    s.classic_people = !prototype;
    s.classic_profiles = !prototype;
    s.photo_window = if prototype {
        PhotoWindowStyle::Prototype
    } else {
        PhotoWindowStyle::Legacy
    };
    s.classic_about_land = !prototype;
    s.classic_search = !prototype;
    s.classic_region_estate = !prototype;
    s.classic_inventory = !prototype;
    s.classic_outfit_gallery = !prototype;
    s.build.legacy_window = !prototype;
    s.classic_places = !prototype;
    s.destinations_style = if prototype {
        DestinationsStyle::Prototype
    } else {
        DestinationsStyle::Legacy
    };
    s.classic_pay_window = !prototype;
    s.legacy_money_tracker = !prototype;
    s.classic_controls = !prototype;
    s.classic_restart_alert = !prototype;
    s.classic_chat_lines = !prototype;
    s.classic_rez_effect = !prototype;
    s.classic_hover_tips = !prototype;
    s.predict_own_motion = prototype;
    s.water_effects.enabled = prototype;
    s.synchronize_animations = prototype;
    s.colored_sky_ambient = prototype;
}

/// The prototype gradient at `t` (0 at its start, 1 at its end): the brand's
/// green, turquoise, indigo, violet and rose, taken from the palette.
fn prototype_color(p: &Palette, t: f32) -> Color32 {
    let stops = [p.success, p.teal, p.indigo_light, p.violet_light, p.rose];
    let x = t.clamp(0.0, 1.0) * (stops.len() - 1) as f32;
    let i = (x.floor() as usize).min(stops.len() - 2);
    stops[i].lerp_to_gamma(stops[i + 1], x - i as f32)
}

fn vertex(pos: Pos2, color: Color32) -> Vertex {
    Vertex { pos, uv: WHITE_UV, color }
}

/// A strip of the prototype gradient, from its top to its bottom.
fn prototype_strip(p: &Palette, r: Rect) -> Shape {
    const STEPS: usize = 16;
    let mut m = Mesh::default();
    for i in 0..=STEPS {
        let t = i as f32 / STEPS as f32;
        let c = prototype_color(p, t);
        let y = r.top() + r.height() * t;
        m.vertices.push(vertex(Pos2::new(r.left(), y), c));
        m.vertices.push(vertex(Pos2::new(r.right(), y), c));
        if i > 0 {
            let k = (i as u32) * 2;
            m.add_triangle(k - 2, k - 1, k);
            m.add_triangle(k - 1, k + 1, k);
        }
    }
    Shape::mesh(m)
}

/// A choice between Prototype's look (first, in the accent) and the Legacy
/// one, over a "classic" switch.
fn look(cx: &mut Cx, name: &str, help: &str, prototype: &str, legacy: &str, f: &F<bool>) {
    cx.wide().choice(name, help, f, &[(false, prototype, true), (true, legacy, false)]);
}

/// An experimental choice: its warning joins its help.
fn warned(help: &str, warning: &str) -> String {
    format!("{help}\n{warning}")
}

pub fn page(cx: &mut Cx, tab: usize) {
    use prototype_i18n::tr;
    let prototype = tr!("ui-options-prototype-about-land-prototype");
    let classic = tr!("ui-options-prototype-about-land-classic");
    match tab {
        0 => {
            // Prototype's recorder (the toolbar's Record button)
            cx.group(tr!("recorder-prefs"), |cx| {
                cx.toggle(tr!("recorder-hide-name-tags"), tr!("recorder-prefs-help"), &field!(recorder.hide_name_tags));
                cx.choice(
                    tr!("recorder-content"),
                    tr!("recorder-content-help"),
                    &field!(recorder.content),
                    &[
                        (0, tr!("recorder-content-world"), true),
                        (1, tr!("recorder-content-bars"), true),
                        (2, tr!("recorder-content-all"), true),
                    ],
                );
                cx.toggle(tr!("recorder-cursor"), "", &field!(recorder.cursor));
                cx.choice(
                    tr!("recorder-blur"),
                    tr!("recorder-blur-help"),
                    &field!(recorder.motion_blur),
                    &[
                        (0, tr!("recorder-blur-off"), true),
                        (1, tr!("recorder-blur-light"), true),
                        (2, tr!("recorder-blur-medium"), true),
                        (3, tr!("recorder-blur-strong"), true),
                    ],
                );
                cx.toggle(tr!("recorder-sound"), "", &field!(recorder.sound));
                cx.toggle(tr!("recorder-microphone"), tr!("recorder-microphone-help"), &field!(recorder.microphone));
                cx.choice(tr!("recorder-fps"), "", &field!(recorder.fps), &[(30, "30 fps", true), (60, "60 fps", true)]);
                cx.choice(
                    tr!("recorder-quality"),
                    "",
                    &field!(recorder.quality),
                    &[
                        (0, tr!("recorder-quality-standard"), true),
                        (1, tr!("recorder-quality-high"), true),
                        (2, tr!("recorder-quality-best"), true),
                    ],
                );
            });
            cx.group(tr!("ui-options-prototype-redrawn-windows"), |cx| {
            cx.wide().choice(
                tr!("ui-options-prototype-conversations"),
                tr!("ui-options-prototype-conversations-help"),
                &field!(chat_style),
                &[
                    (ChatStyle::Prototype, tr!("ui-options-prototype-workspace"), true),
                    (ChatStyle::Compact, tr!("ui-options-prototype-compact"), true),
                    (ChatStyle::Bubbles, tr!("ui-options-prototype-bubbles"), true),
                    (ChatStyle::Legacy, tr!("ui-options-prototype-lines"), false),
                ],
            );
            look(
                cx,
                tr!("ui-options-prototype-chat-lines"),
                tr!("ui-options-prototype-chat-lines-help"),
                tr!("ui-options-prototype-chat-lines-cards"),
                tr!("ui-options-prototype-chat-lines-classic"),
                &field!(classic_chat_lines),
            );
            cx.wide().choice(
                tr!("ui-options-prototype-world-map"),
                tr!("ui-options-prototype-world-map-help"),
                &field!(world_map_style),
                &[
                    (WorldMapStyle::Prototype, tr!("ui-options-prototype-world-map-full"), true),
                    (WorldMapStyle::Legacy, tr!("ui-options-prototype-world-map-side"), false),
                ],
            );
            look(
                cx,
                tr!("ui-options-prototype-places"),
                tr!("ui-options-prototype-places-help"),
                tr!("ui-options-prototype-places-prototype"),
                tr!("ui-options-prototype-places-classic"),
                &field!(classic_places),
            );
            cx.wide().choice(
                tr!("ui-options-prototype-destinations"),
                tr!("ui-options-prototype-destinations-help"),
                &field!(destinations_style),
                &[
                    (DestinationsStyle::Prototype, tr!("ui-options-prototype-destinations-prototype"), true),
                    (DestinationsStyle::Legacy, tr!("ui-options-prototype-destinations-classic"), false),
                ],
            );
            look(
                cx,
                tr!("ui-options-prototype-people"),
                tr!("ui-options-prototype-people-help"),
                tr!("ui-options-prototype-people-rows"),
                tr!("ui-options-prototype-people-table"),
                &field!(classic_people),
            );
            look(
                cx,
                tr!("ui-options-prototype-profiles"),
                tr!("ui-options-prototype-profiles-help"),
                tr!("ui-options-prototype-profiles-prototype"),
                tr!("ui-options-prototype-profiles-classic"),
                &field!(classic_profiles),
            );
            let classic_profiles = cx.s.classic_profiles;
            cx.subs(classic_profiles, |cx| {
                cx.toggle(
                    tr!("ui-options-prototype-profiles-rights"),
                    tr!("ui-options-prototype-profiles-rights-help"),
                    &field!(profile_rights),
                );
            });
            cx.wide().choice(
                tr!("ui-options-prototype-photo-window"),
                tr!("ui-options-prototype-photo-window-help"),
                &field!(photo_window),
                &[
                    (PhotoWindowStyle::Prototype, tr!("ui-options-prototype-photo-viewfinder"), true),
                    (PhotoWindowStyle::Panel, tr!("ui-options-prototype-photo-panel"), true),
                    (PhotoWindowStyle::Legacy, tr!("ui-options-prototype-photo-classic"), false),
                ],
            );
            let viewfinder = cx.s.photo_window == PhotoWindowStyle::Prototype;
            cx.subs(!viewfinder, |cx| {
                use crate::snapshot::viewfinder::OnScreen;
                let labels = [
                    (tr!("ui-options-prototype-vf-controls-always"), true),
                    (tr!("ui-options-prototype-vf-controls-hover"), true),
                    (tr!("ui-options-prototype-vf-controls-hidden"), false),
                ];
                let choices: Vec<(OnScreen, &str, bool)> = OnScreen::ALL.iter().zip(labels).map(|(o, (l, a))| (*o, l, a)).collect();
                cx.wide().choice(
                    tr!("ui-options-prototype-vf-controls"),
                    tr!("ui-options-prototype-vf-controls-help"),
                    &field!(snapshot.viewfinder.controls),
                    &choices,
                );
            });
            look(
                cx,
                tr!("ui-options-prototype-about-land"),
                tr!("ui-options-prototype-about-land-help"),
                prototype,
                classic,
                &field!(classic_about_land),
            );
            look(
                cx,
                tr!("search-title"),
                tr!("ui-options-prototype-search-help"),
                prototype,
                tr!("ui-options-prototype-search-classic"),
                &field!(classic_search),
            );
            look(
                cx,
                tr!("region-title"),
                tr!("ui-options-prototype-region-estate-help"),
                prototype,
                classic,
                &field!(classic_region_estate),
            );
            look(
                cx,
                tr!("common-inventory"),
                tr!("ui-options-prototype-inventory-help"),
                prototype,
                classic,
                &field!(classic_inventory),
            );
            look(
                cx,
                tr!("ui-options-prototype-outfit-gallery"),
                tr!("ui-options-prototype-outfit-gallery-help"),
                tr!("ui-options-prototype-outfit-gallery-prototype"),
                tr!("ui-options-prototype-outfit-gallery-classic"),
                &field!(classic_outfit_gallery),
            );
            look(
                cx,
                tr!("ui-options-prototype-build"),
                tr!("ui-options-prototype-build-help"),
                prototype,
                classic,
                &field!(build.legacy_window),
            );
            look(
                cx,
                tr!("ui-options-prototype-pay-window"),
                tr!("ui-options-prototype-pay-window-help"),
                tr!("ui-options-prototype-pay-prototype"),
                tr!("ui-options-prototype-pay-classic"),
                &field!(classic_pay_window),
            );
            look(
                cx,
                tr!("money-tracker-title"),
                tr!("ui-options-prototype-money-tracker-help"),
                prototype,
                classic,
                &field!(legacy_money_tracker),
            );
            look(
                cx,
                tr!("ui-options-prototype-controls"),
                tr!("ui-options-prototype-controls-help"),
                tr!("ui-options-prototype-controls-prototype"),
                tr!("ui-options-prototype-controls-classic"),
                &field!(classic_controls),
            );
        });
        }
        1 => {
            cx.group(tr!("ui-options-prototype-in-the-world"), |cx| {
                look(
                    cx,
                    tr!("ui-options-prototype-name-tags"),
                    tr!("ui-options-prototype-name-tags-help"),
                    tr!("ui-options-prototype-plates"),
                    tr!("ui-options-prototype-classic"),
                    &field!(classic_name_tags),
                );
                cx.wide().choice(
                    tr!("ui-options-prototype-look-at"),
                    tr!("ui-options-prototype-look-at-help"),
                    &field!(lookat.style),
                    &[
                        (Style::Prototype, tr!("ui-options-prototype-halo"), true),
                        (Style::Legacy, tr!("ui-options-prototype-crosshair"), false),
                    ],
                );
                look(
                    cx,
                    tr!("ui-options-prototype-rez-effect"),
                    tr!("ui-options-prototype-rez-effect-help"),
                    tr!("ui-options-prototype-rez-effect-prototype"),
                    tr!("ui-options-prototype-rez-effect-classic"),
                    &field!(classic_rez_effect),
                );
                look(
                    cx,
                    tr!("ui-options-prototype-hover-tips"),
                    tr!("ui-options-prototype-hover-tips-help"),
                    prototype,
                    tr!("ui-options-prototype-hover-tips-legacy"),
                    &field!(classic_hover_tips),
                );
                // also in Graphics › Sky and environment, the same setting
                cx.wide().experimental().choice(
                    tr!("ui-options-colored-sky-ambient"),
                    tr!("ui-options-colored-sky-ambient-help"),
                    &field!(colored_sky_ambient),
                    &[(true, prototype, true), (false, tr!("ui-options-prototype-classic"), false)],
                );
            });
            cx.group(tr!("common-interface"), |cx| {
                cx.wide().choice(
                    tr!("ui-options-prototype-notifications"),
                    tr!("ui-options-prototype-notifications-help"),
                    &field!(notifications_style),
                    &[
                        (NotificationsStyle::Prototype, tr!("ui-options-prototype-animated-bell"), true),
                        (NotificationsStyle::Legacy, tr!("ui-options-prototype-toasts"), false),
                    ],
                );
                look(
                    cx,
                    tr!("ui-options-prototype-bars"),
                    tr!("ui-options-prototype-bars-help"),
                    tr!("ui-options-prototype-bars-islands"),
                    tr!("ui-options-prototype-bars-classic"),
                    &field!(classic_bars),
                );
                look(
                    cx,
                    tr!("ui-options-prototype-restart-alert"),
                    tr!("ui-options-prototype-restart-alert-help"),
                    tr!("ui-options-prototype-restart-card"),
                    tr!("ui-options-prototype-restart-classic"),
                    &field!(classic_restart_alert),
                );
            });
        }
        _ => {
            cx.group(tr!("ui-options-prototype-building"), |cx| {
                cx.wide().choice(
                    tr!("ui-options-prototype-move-hologram"),
                    tr!("ui-options-prototype-move-hologram-help"),
                    &field!(build.move_hologram),
                    &[
                        (MoveHologram::Off, tr!("ui-options-prototype-move-hologram-off"), false),
                        (MoveHologram::Moving, tr!("ui-options-prototype-move-hologram-moving"), true),
                        (MoveHologram::Start, tr!("ui-options-prototype-move-hologram-start"), true),
                    ],
                );
            });
            cx.group(tr!("ui-options-prototype-bridge"), |cx| {
                cx.toggle(
                    tr!("ui-options-prototype-use-bridge"),
                    tr!("ui-options-prototype-use-bridge-help"),
                    &field!(use_bridge),
                );
                use crate::world::portals::Sources;
                cx.combo(
                    tr!("ui-options-prototype-portal-sources"),
                    tr!("ui-options-prototype-portal-sources-help"),
                    &field!(portal_sources),
                    &[
                        (Sources::Friends, tr!("ui-options-prototype-portal-sources-friends")),
                        (Sources::FriendsAndGroups, tr!("ui-options-prototype-portal-sources-groups")),
                        (Sources::Everyone, tr!("ui-options-prototype-portal-sources-everyone")),
                        (Sources::Nobody, tr!("ui-options-prototype-portal-sources-nobody")),
                    ],
                );
            });
            cx.group(tr!("ui-options-prototype-gaze"), |cx| {
                cx.wide().choice(
                    tr!("ui-options-prototype-aim"),
                    tr!("ui-options-prototype-aim-help"),
                    &field!(lookat.aim),
                    &[
                        (Aim::Point, tr!("ui-options-prototype-aim-point"), true),
                        (Aim::Direction, tr!("ui-options-prototype-aim-direction"), false),
                    ],
                );
            });
            // the Legacy Viewer's until the human has tried it on the grid,
            // marked experimental meanwhile
            cx.group(tr!("ui-options-prototype-movement"), |cx| {
                let help = warned(
                    tr!("ui-options-prototype-own-motion-help"),
                    tr!("ui-options-prototype-own-motion-experimental"),
                );
                cx.wide().experimental().choice(
                    tr!("ui-options-prototype-own-motion"),
                    &help,
                    &field!(predict_own_motion),
                    &[
                        (true, tr!("ui-options-prototype-own-motion-instant"), true),
                        (false, tr!("ui-options-prototype-own-motion-server"), false),
                    ],
                );
            });
            cx.group(tr!("ui-options-prototype-animations"), |cx| {
                let help = warned(
                    tr!("ui-options-prototype-animation-clock-help"),
                    tr!("ui-options-prototype-animation-clock-experimental"),
                );
                cx.wide().experimental().choice(
                    tr!("ui-options-prototype-animation-clock"),
                    &help,
                    &field!(synchronize_animations),
                    &[
                        (true, tr!("ui-options-prototype-animation-clock-synchronized"), true),
                        (false, tr!("ui-options-prototype-animation-clock-legacy"), false),
                    ],
                );
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn received_filing_stays_opt_in_when_the_looks_change() {
        let mut settings = Settings::default();
        set_all(&mut settings, Look::Prototype);
        assert!(!settings.sort_received_items);
        settings.sort_received_items = true;
        set_all(&mut settings, Look::Legacy);
        assert!(settings.sort_received_items);
    }

    #[test]
    fn the_two_buttons_set_the_colored_sky_ambient_too() {
        let mut s = Settings::default();
        assert!(!s.colored_sky_ambient, "off by default");
        set_all(&mut s, Look::Prototype);
        assert!(s.colored_sky_ambient);
        set_all(&mut s, Look::Legacy);
        assert!(!s.colored_sky_ambient);
    }

    #[test]
    fn the_prototype_button_keeps_the_gaze_choice() {
        let mut s = Settings::default();
        set_all(&mut s, Look::Prototype);
        assert_eq!(s.lookat.aim, Aim::Direction, "never turned to the point by the button");
        s.lookat.aim = Aim::Point;
        set_all(&mut s, Look::Prototype);
        assert_eq!(s.lookat.aim, Aim::Point, "a chosen point stays");
        set_all(&mut s, Look::Legacy);
        assert_eq!(s.lookat.aim, Aim::Direction);
    }

    #[test]
    fn the_two_buttons_set_every_look_and_leave_the_plates_alone() {
        let mut s = Settings {
            plates_friends_only_full: true,
            ..Settings::default()
        };
        set_all(&mut s, Look::Legacy);
        assert_eq!(s.chat_style, ChatStyle::Legacy);
        assert!(s.classic_name_tags);
        assert_eq!(s.lookat.style, Style::Legacy);
        assert_eq!(s.lookat.aim, Aim::Direction);
        assert_eq!(s.notifications_style, NotificationsStyle::Legacy);
        assert_eq!(s.world_map_style, WorldMapStyle::Legacy);
        assert!(s.classic_bars, "the full-width bars come back");
        assert!(s.classic_people, "People › Near you shows the radar table");
        assert!(s.classic_profiles);
        assert_eq!(s.photo_window, PhotoWindowStyle::Legacy);
        assert!(s.classic_about_land, "About Land opens the Legacy Viewer's nine tabs");
        assert!(s.classic_search, "Search opens the Legacy Viewer's tabs");
        assert!(s.classic_region_estate);
        assert!(s.classic_inventory, "the Legacy Viewer's single inventory tree");
        assert!(s.classic_outfit_gallery, "the Legacy Viewer's small outfit tiles");
        assert!(s.build.legacy_window, "the Legacy Viewer's build tools and tabs");
        assert!(s.classic_places, "Places opens the Legacy Viewer's three tabs");
        assert_eq!(s.destinations_style, DestinationsStyle::Legacy, "the guide's web page");
        assert!(s.classic_pay_window);
        assert!(s.legacy_money_tracker, "the Legacy Viewer's Time / Name / Amount list");
        assert!(s.classic_controls, "the Legacy Viewer's two control windows");
        assert!(s.classic_restart_alert, "the Legacy Viewer's yellow restart box");
        assert!(s.classic_chat_lines, "the Legacy Viewer's one-line chat lines");
        assert!(s.classic_rez_effect, "grey objects while they load");
        assert!(s.classic_hover_tips, "the Legacy Viewer's text tooltips");
        assert!(!s.predict_own_motion, "the avatar waits for the simulator");
        assert!(!s.synchronize_animations);
        assert!(!s.water_effects.enabled);
        assert!(s.plates_friends_only_full, "a plate setting is not a look");
        set_all(&mut s, Look::Prototype);
        let fresh = Settings::default();
        assert_eq!(s.chat_style, fresh.chat_style);
        assert_eq!(s.classic_name_tags, fresh.classic_name_tags);
        assert_eq!(s.lookat.style, fresh.lookat.style);
        assert_eq!(s.lookat.aim, fresh.lookat.aim);
        assert_eq!(s.notifications_style, NotificationsStyle::Prototype);
        assert_eq!(fresh.notifications_style, NotificationsStyle::Prototype);
        assert_eq!(s.world_map_style, fresh.world_map_style);
        assert!(!s.classic_bars && !fresh.classic_bars, "the islands are the default");
        assert!(
            !s.classic_people && !fresh.classic_people,
            "the rows grouped by range are the default"
        );
        assert_eq!(s.classic_profiles, fresh.classic_profiles);
        assert_eq!(s.photo_window, PhotoWindowStyle::Prototype);
        assert_eq!(fresh.photo_window, PhotoWindowStyle::Prototype);
        assert_eq!(s.classic_about_land, fresh.classic_about_land);
        assert!(!fresh.classic_about_land, "the Prototype About Land is the default");
        assert!(!s.classic_search && !fresh.classic_search, "the Prototype Search is the default");
        assert!(!s.classic_region_estate && !fresh.classic_region_estate);
        assert!(
            !s.classic_inventory && !fresh.classic_inventory,
            "the Prototype inventory is the default"
        );
        assert!(!s.classic_places && !fresh.classic_places, "Prototype's Places is the default");
        assert_eq!(s.destinations_style, DestinationsStyle::Prototype);
        assert_eq!(
            fresh.destinations_style,
            DestinationsStyle::Prototype,
            "Prototype's Destinations is the default"
        );
        assert!(
            !s.classic_outfit_gallery && !fresh.classic_outfit_gallery,
            "the showcase is the default"
        );
        assert!(!s.build.legacy_window && !fresh.build.legacy_window);
        assert!(!s.classic_pay_window && !fresh.classic_pay_window);
        assert!(!s.legacy_money_tracker && !fresh.legacy_money_tracker);
        assert!(!s.classic_controls && !fresh.classic_controls);
        assert!(!s.classic_restart_alert && !fresh.classic_restart_alert);
        assert!(!s.classic_chat_lines && !fresh.classic_chat_lines, "the cards are the default");
        assert!(!s.classic_rez_effect && !fresh.classic_rez_effect, "the rez effect is the default");
        assert!(!s.classic_hover_tips && !fresh.classic_hover_tips, "Prototype's tips are the default");
        assert!(s.predict_own_motion, "Prototype's movement");
        assert!(!fresh.predict_own_motion, "the Legacy movement until tried on the grid");
        assert!(s.synchronize_animations, "the Prototype bulk choice enables receipt clocks");
        assert!(s.water_effects.enabled, "the explicit Prototype bulk choice enables water effects");
        assert!(!fresh.water_effects.enabled, "enhanced water is disabled by default");
        assert!(!fresh.synchronize_animations, "Legacy animation timing is the default");
        assert!(s.plates_friends_only_full);
    }

    /// The notifications' style is saved; a file from before it was added
    /// gets the animated bell.
    #[test]
    fn the_notifications_style_is_saved() {
        let s = Settings {
            notifications_style: NotificationsStyle::Legacy,
            ..Settings::default()
        };
        let json = serde_json::to_string(&s).expect("settings serialize");
        let back: Settings = serde_json::from_str(&json).expect("settings read back");
        assert_eq!(back.notifications_style, NotificationsStyle::Legacy);
        let old: Settings = serde_json::from_str("{}").expect("empty settings");
        assert_eq!(old.notifications_style, NotificationsStyle::Prototype);
    }

    #[test]
    fn the_prototype_gradient_runs_from_green_to_rose() {
        let p = crate::theme::Theme::default().palette();
        assert_eq!(prototype_color(&p, 0.0), p.success);
        assert_eq!(prototype_color(&p, 1.0), p.rose);
        assert_eq!(prototype_color(&p, 0.5), p.indigo_light);
    }
}
