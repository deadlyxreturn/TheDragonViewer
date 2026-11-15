//! The pages of the preferences, one module each, in the order of the
//! sidebar. A page declares its rows through `Cx` for one of its tabs.

pub mod accessibility;
pub mod appearance;
pub mod prototype;
pub mod avatars;
pub mod build;
pub mod camera;
pub mod chat;
pub mod commands;
pub mod confirmations;
pub mod controls;
pub mod fun;
pub mod graphics;
pub mod inventory;
pub mod joystick;
pub mod layout;
pub mod media;
pub mod network;
pub mod privacy;
pub mod protections;
pub mod rlva;
pub mod shortcuts;
pub mod sound;
pub mod spelling;
pub mod system;
pub mod vr;

use super::Page;
use super::cx::Cx;

/// The tabs of a page.
pub fn tabs(page: Page) -> Vec<&'static str> {
    match page {
        Page::Prototype => vec![
            prototype_i18n::tr!("common-windows"),
            prototype_i18n::tr!("ui-options-tab-world-and-interface"),
            prototype_i18n::tr!("common-behavior"),
        ],
        Page::Graphics => vec![
            prototype_i18n::tr!("ui-options-quality"),
            prototype_i18n::tr!("common-light"),
            prototype_i18n::tr!("ui-options-tab-sky-and-environment"),
            prototype_i18n::tr!("ui-options-tab-reflections-and-water"),
            prototype_i18n::tr!("common-image"),
            prototype_i18n::tr!("ui-options-smoothness"),
            prototype_i18n::tr!("region-parcels"),
        ],
        Page::Camera => vec![
            prototype_i18n::tr!("common-viewpoint"),
            prototype_i18n::tr!("common-behavior"),
            prototype_i18n::tr!("common-mouselook"),
        ],
        Page::Controls => vec![prototype_i18n::tr!("ui-options-page-controls"), prototype_i18n::tr!("joystick-title")],
        Page::Gamepad => vec![
            prototype_i18n::tr!("joystick-gamepad-settings"),
            prototype_i18n::tr!("joystick-map-controls"),
        ],
        Page::Avatars => vec![
            prototype_i18n::tr!("ui-options-tab-name-tags"),
            prototype_i18n::tr!("ui-options-names"),
            prototype_i18n::tr!("ui-options-look-at-targets"),
            prototype_i18n::tr!("ui-options-prototype-animations"),
        ],
        Page::Chat => vec![
            prototype_i18n::tr!("common-nearby-chat"),
            prototype_i18n::tr!("ui-options-prototype-notifications"),
            prototype_i18n::tr!("ui-options-tab-keywords"),
            prototype_i18n::tr!("ui-options-tab-history"),
            prototype_i18n::tr!("ui-options-tab-away-and-replies"),
            prototype_i18n::tr!("ui-options-groups-and-blocking"),
            prototype_i18n::tr!("ui-options-tab-spelling"),
        ],
        Page::Sound => vec![
            prototype_i18n::tr!("ui-options-volumes"),
            prototype_i18n::tr!("ui-options-interface-sounds"),
            prototype_i18n::tr!("common-voice"),
        ],
        Page::Layout => vec![
            prototype_i18n::tr!("ui-options-tab-bars-and-windows"),
            prototype_i18n::tr!("ui-options-hover-tips"),
            prototype_i18n::tr!("ui-options-tab-startup"),
        ],
        Page::Appearance => vec![
            prototype_i18n::tr!("ui-options-font"),
            prototype_i18n::tr!("ui-options-colors"),
            prototype_i18n::tr!("extra-skin"),
        ],
        Page::Shortcuts => shortcuts::tabs(),
        _ => vec![page.name()],
    }
}

/// The rows of one tab of a page.
pub fn draw(cx: &mut Cx, page: Page, tab: usize) {
    match page {
        Page::Prototype => prototype::page(cx, tab),
        Page::Graphics => graphics::page(cx, tab),
        Page::Camera => camera::page(cx, tab),
        Page::Vr => vr::page(cx),
        Page::Avatars => avatars::page(cx, tab),
        Page::Build => build::page(cx),
        Page::Chat => chat::page(cx, tab),
        Page::Sound => sound::page(cx, tab),
        Page::Media => media::page(cx),
        Page::Privacy => privacy::page(cx),
        Page::Protections => protections::page(cx),
        Page::Rlva => rlva::page(cx),
        Page::Layout => layout::page(cx, tab),
        Page::Appearance => appearance::page(cx, tab),
        Page::Accessibility => accessibility::page(cx),
        Page::Confirmations => confirmations::page(cx),
        Page::Inventory => inventory::page(cx),
        Page::Controls => {
            if tab == 0 {
                controls::page(cx)
            } else {
                joystick::page(cx)
            }
        }
        Page::Gamepad => joystick::gamepad_page(cx, tab),
        Page::Shortcuts => shortcuts::page(cx, tab),
        Page::Commands => commands::page(cx),
        Page::System => system::page(cx),
        Page::Network => network::page(cx),
        Page::Fun => fun::page(cx),
    }
}
