//! Preferences floater: the pages in sections down a sidebar (with the
//! search of every setting and the modified ones at its top), the page's
//! tabs, its settings in two columns, the reset of the page at the bottom.
//! Every setting is a row of `cx::Cx`: one declaration draws it, finds it,
//! marks it when it differs from its default and resets it.

pub mod backup;
pub(crate) mod cx;
mod finish;
mod pages;
mod saved_graphics;
mod scale;

pub(super) use finish::paint_window_trim;
pub use pages::privacy;
pub(crate) use pages::sound::{ear_choice, noise_choice};
pub(crate) use scale::keyboard as scale_keyboard;

use super::fonts::SystemFont;
use super::icons::Icons;
use super::widgets::{Floater, keycap};
use crate::settings::Settings;
use crate::theme::Palette;
use crate::ui::widgets;
use cx::{Counts, Cx, Mode};
use egui::{Rect, RichText, Sense, Vec2, pos2, vec2};
use std::collections::{HashMap, HashSet};

/// The preferences' window (its layout is saved under this id).
pub const WINDOW: &str = "preferences";

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Capture {
    Binding(crate::keybinds::Action, usize),
}

/// A part of the Sound and voice page another window opens it on (the
/// microphone menu of the bars).
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AudioSection {
    Voice,
    Processing,
    Indicators,
}

/// The pages of the window, in the order of the sidebar.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum Page {
    Prototype,
    #[default]
    Graphics,
    Camera,
    Avatars,
    Build,
    Chat,
    Sound,
    Media,
    Privacy,
    Protections,
    Rlva,
    Layout,
    Appearance,
    Accessibility,
    Confirmations,
    Inventory,
    Controls,
    Shortcuts,
    Commands,
    System,
    Network,
    Fun,
}

impl Page {
    pub const ALL: [Page; 22] = [
        Page::Prototype,
        Page::Graphics,
        Page::Camera,
        Page::Avatars,
        Page::Build,
        Page::Chat,
        Page::Sound,
        Page::Media,
        Page::Privacy,
        Page::Protections,
        Page::Rlva,
        Page::Layout,
        Page::Appearance,
        Page::Accessibility,
        Page::Confirmations,
        Page::Inventory,
        Page::Controls,
        Page::Shortcuts,
        Page::Commands,
        Page::System,
        Page::Network,
        Page::Fun,
    ];

    /// Its name in `PROTOTYPE_DEMO_OPTIONS`.
    pub fn key(self) -> &'static str {
        match self {
            Page::Prototype => "prototype",
            Page::Graphics => "graphics",
            Page::Camera => "camera",
            Page::Avatars => "avatars",
            Page::Build => "build",
            Page::Chat => "chat",
            Page::Sound => "sound",
            Page::Media => "media",
            Page::Privacy => "privacy",
            Page::Protections => "protections",
            Page::Rlva => "rlva",
            Page::Layout => "layout",
            Page::Appearance => "appearance",
            Page::Accessibility => "accessibility",
            Page::Confirmations => "confirmations",
            Page::Inventory => "inventory",
            Page::Controls => "controls",
            Page::Shortcuts => "shortcuts",
            Page::Commands => "commands",
            Page::System => "system",
            Page::Network => "network",
            Page::Fun => "fun",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Page::Prototype => prototype_i18n::tr!("ui-options-prototype"),
            Page::Graphics => prototype_i18n::tr!("ui-options-graphics"),
            Page::Camera => prototype_i18n::tr!("common-camera"),
            Page::Avatars => prototype_i18n::tr!("ui-options-avatars"),
            Page::Build => prototype_i18n::tr!("common-build"),
            Page::Chat => prototype_i18n::tr!("common-chat"),
            Page::Sound => prototype_i18n::tr!("ui-options-sound-and-voice"),
            Page::Media => prototype_i18n::tr!("ui-options-page-media"),
            Page::Privacy => prototype_i18n::tr!("ui-options-privacy"),
            Page::Protections => prototype_i18n::tr!("ui-options-protections"),
            Page::Rlva => prototype_i18n::tr!("ui-options-rlva"),
            Page::Layout => prototype_i18n::tr!("ui-options-layout"),
            Page::Appearance => prototype_i18n::tr!("ui-options-page-appearance"),
            Page::Accessibility => prototype_i18n::tr!("ui-options-accessibility"),
            Page::Confirmations => prototype_i18n::tr!("ui-options-page-confirmations"),
            Page::Inventory => prototype_i18n::tr!("common-inventory"),
            Page::Controls => prototype_i18n::tr!("ui-options-page-controls"),
            Page::Shortcuts => prototype_i18n::tr!("ui-options-shortcuts"),
            Page::Commands => prototype_i18n::tr!("ui-options-commands"),
            Page::System => prototype_i18n::tr!("ui-options-page-system"),
            Page::Network => prototype_i18n::tr!("ui-options-network-and-cache"),
            Page::Fun => prototype_i18n::tr!("ui-options-fun"),
        }
    }

    /// The line under the page's title.
    pub fn about(self) -> &'static str {
        match self {
            Page::Prototype => prototype_i18n::tr!("ui-options-prototype-intro"),
            Page::Graphics => prototype_i18n::tr!("ui-options-about-graphics"),
            Page::Camera => prototype_i18n::tr!("ui-options-about-camera"),
            Page::Avatars => prototype_i18n::tr!("ui-options-about-avatars"),
            Page::Build => prototype_i18n::tr!("ui-options-about-build"),
            Page::Chat => prototype_i18n::tr!("ui-options-about-chat"),
            Page::Sound => prototype_i18n::tr!("ui-options-about-sound"),
            Page::Media => prototype_i18n::tr!("ui-options-about-media"),
            Page::Privacy => prototype_i18n::tr!("ui-options-about-privacy"),
            Page::Protections => prototype_i18n::tr!("ui-options-about-protections"),
            Page::Rlva => prototype_i18n::tr!("ui-options-about-rlva"),
            Page::Layout => prototype_i18n::tr!("ui-options-about-layout"),
            Page::Appearance => prototype_i18n::tr!("ui-options-about-appearance"),
            Page::Accessibility => prototype_i18n::tr!("ui-options-about-accessibility"),
            Page::Confirmations => prototype_i18n::tr!("ui-options-about-confirmations"),
            Page::Inventory => prototype_i18n::tr!("ui-options-about-inventory"),
            Page::Controls => prototype_i18n::tr!("ui-options-about-controls"),
            Page::Shortcuts => prototype_i18n::tr!("ui-options-click-a-shortcut-then-press-the-desired-key-or"),
            Page::Commands => prototype_i18n::tr!("ui-options-about-commands"),
            Page::System => prototype_i18n::tr!("ui-options-about-system"),
            Page::Network => prototype_i18n::tr!("ui-options-about-network"),
            Page::Fun => prototype_i18n::tr!("ui-options-about-fun"),
        }
    }

    pub fn icon(self) -> &'static str {
        match self {
            Page::Prototype => "sparkles-filled",
            Page::Graphics => "device-desktop-filled",
            Page::Camera => "camera-filled",
            Page::Avatars => "user-filled",
            // the build tools' icon (Command_Build_Icon)
            Page::Build => "box",
            Page::Chat => "messages-filled",
            Page::Sound => "headset-filled",
            Page::Media => "player-play-filled",
            Page::Privacy => "lock-filled",
            Page::Protections => "shield-filled",
            Page::Rlva => "lock-square-rounded-filled",
            Page::Layout => "layout-filled",
            Page::Appearance => "palette-filled",
            Page::Accessibility => "accessible-filled",
            Page::Confirmations => "circle-check-filled",
            Page::Inventory => "box",
            Page::Controls => "mouse-filled",
            Page::Shortcuts => "keyboard-filled",
            Page::Commands => "square-rounded-chevron-right-filled",
            Page::System => "world-filled",
            Page::Network => "cloud-data-connection-filled",
            Page::Fun => "confetti-filled",
        }
    }

    /// A page whose whole feature is still experimental: its sidebar entry
    /// carries the experimental mark, and the page opens on a notice.
    pub fn experimental(self) -> bool {
        matches!(self, Page::Rlva)
    }

    /// The title of the sidebar section the page opens (the first page of
    /// each section; Prototype sits above them).
    fn section(self) -> Option<&'static str> {
        match self {
            Page::Graphics => Some(prototype_i18n::tr!("ui-options-section-world")),
            Page::Chat => Some(prototype_i18n::tr!("ui-options-section-communication")),
            Page::Layout => Some(prototype_i18n::tr!("common-interface")),
            Page::Controls => Some(prototype_i18n::tr!("ui-options-controls")),
            Page::System => Some(prototype_i18n::tr!("ui-options-section-system")),
            Page::Fun => Some(prototype_i18n::tr!("ui-options-section-extras")),
            _ => None,
        }
    }

    /// The page's tabs (one: no tab strip).
    pub fn tabs(self) -> Vec<&'static str> {
        pages::tabs(self)
    }

    /// A page by its demo name, or by its number in the window before the
    /// pages were reorganized (the docs' `PROTOTYPE_DEMO_OPTIONS=0:tonemap`).
    pub fn parse(text: &str) -> Option<(Page, usize)> {
        if let Some(page) = Page::ALL.iter().find(|p| p.key() == text) {
            return Some((*page, 0));
        }
        Some(match text.parse::<usize>().ok()? {
            0 => (Page::Graphics, 0),
            1 => (Page::Layout, 0),
            2 => (Page::Appearance, 0),
            3 => (Page::Chat, 0),
            4 => (Page::Controls, 0),
            5 => (Page::Appearance, 1),
            6 => (Page::Network, 0),
            7 => (Page::Sound, 0),
            8 => (Page::Shortcuts, 0),
            10 => (Page::Camera, 0),
            11 => (Page::Commands, 0),
            12 => (Page::Privacy, 0),
            13 => (Page::Prototype, 0),
            14 => (Page::Accessibility, 0),
            15 => (Page::Protections, 0),
            _ => return None,
        })
    }
}

/// `PROTOTYPE_DEMO_OPTIONS=<page>[/<tab>][:<what>]`: the page by its name
/// (`graphics`, `chat`…) or its number before the reorganisation, then a
/// search (`search=<text>`), the modified settings (`modified`), the reset
/// of the page and its undo (`reset`), or one of the rows the docs' former
/// filters scrolled to (`tonemap`, `keywords`, `voice`…).
pub fn demo(st: &mut OptionsUi, value: &str) {
    use prototype_i18n::tr;
    let (place, what) = value.split_once(':').unwrap_or((value, ""));
    let (name, tab) = place.split_once('/').unwrap_or((place, ""));
    let Some((page, first)) = Page::parse(name) else {
        return;
    };
    st.open(page, tab.parse().unwrap_or(first));
    let row = |page, tab, label| Some((page, tab, label));
    st.reveal = match what {
        "tonemap" => row(Page::Graphics, 4, tr!("ui-options-tone-mapping")),
        "occlusion" => row(Page::Graphics, 1, tr!("ui-options-screen-indirect-light")),
        "bloom" => row(Page::Graphics, 1, tr!("ui-options-bloom")),
        "name-tags" => row(Page::Avatars, 0, tr!("ui-options-show-own-tag")),
        "group-titles" => row(Page::Avatars, 1, tr!("ui-options-group-titles")),
        "plates" => row(Page::Avatars, 0, tr!("ui-options-plate-click")),
        "attention" => row(Page::Chat, 1, tr!("ui-options-flash-on-message")),
        "keywords" => row(Page::Chat, 2, tr!("ui-options-keywords-on")),
        "spelling" => row(Page::Chat, 6, tr!("ui-options-spelling-on")),
        "auto-replace" => row(Page::Chat, 6, tr!("ui-options-auto-replace-on")),
        "gifs" => row(Page::Chat, 0, tr!("ui-options-gifs-show")),
        "script-errors" => row(Page::Chat, 0, tr!("ui-options-script-errors-in-chat")),
        "crosshair" => row(Page::Camera, 2, tr!("ui-options-crosshair-image")),
        "joystick-zoom" => row(Page::Controls, 1, tr!("joystick-zoom-mode")),
        "recent" | "recent-period" => row(Page::Inventory, 0, tr!("ui-inventory-recent-period")),
        "auto-accept" => row(Page::Inventory, 0, tr!("received-auto-accept")),
        "animations" | "animations-sync" => row(Page::Prototype, 2, tr!("ui-options-prototype-animation-clock")),
        "remove-prototype" => row(Page::System, 0, tr!("ui-options-remove-prototype")),
        _ => None,
    };
    let audio = match what {
        "voice" => Some(AudioSection::Voice),
        "processing" => Some(AudioSection::Processing),
        "indicators" => Some(AudioSection::Indicators),
        _ => None,
    };
    if what == "auto-accept" {
        st.demo_open_cogs = true;
    }
    if let Some(section) = audio {
        st.open_audio(section);
    } else if what == "reset" {
        st.demo_reset = true;
    } else if what == "reset-ask" {
        st.confirm_reset = Some(ResetAsk::Page(page));
    } else if what == "reset-all-ask" {
        st.confirm_reset = Some(ResetAsk::All);
    } else if what == "modified" || what == "modified-ask" {
        st.modified_view = true;
        if what == "modified-ask" {
            st.confirm_reset = Some(ResetAsk::Modified);
        }
    } else if what == "cogs" {
        st.demo_open_cogs = true;
    } else if what == "system-fonts" || what == "system-fonts-scroll" {
        st.demo_font_list = Some(what == "system-fonts-scroll");
    } else if let Some(query) = what.strip_prefix("search=") {
        st.query = query.to_owned();
    } else if page == Page::Shortcuts && !what.is_empty() && what != "altgr" && st.reveal.is_none() {
        // the shortcut page's former filter: the window's search now
        st.query = what.to_owned();
    }
}

/// A reset waiting for its undo to go away before it is announced.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Done {
    /// The settings of a page (the app says so in the chat).
    Page(Page),
    /// The settings that differed from their defaults.
    Modified,
    /// Every setting (the app also forgets the windows' layout).
    All,
}

/// A reset asked by a button, waiting for its "Reset" in a dialog. `Undo`
/// still follows it: the dialog is for a slip of the mouse, the undo for a
/// change of mind.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ResetAsk {
    Page(Page),
    Modified,
    All,
}

/// The settings before a reset, while "Undo" shows.
struct Undo {
    before: Box<Settings>,
    text: String,
    shown: f64,
    done: Done,
}

/// How long "Undo" stays after a reset.
const UNDO_SECONDS: f64 = 8.0;

#[derive(Default)]
pub struct OptionsUi {
    pub at_login: bool,
    saved_graphics: saved_graphics::Editor,
    pub current_parcel: Option<crate::world::parcel_visibility::Entry>,
    pub selected_parcel: Option<crate::world::parcel_visibility::Entry>,
    pub rlva_tools: super::rlva::Tools,
    pub rlva_restrictions: Vec<(uuid::Uuid, String)>,
    pub rlva_camera_preset_locked: bool,
    pub rlva_camera_fov_locked: bool,
    /// RLVa `@sendim`: the Do Not Disturb response is an IM, its field greyed.
    pub rlva_sendim_locked: bool,
    pub joystick: crate::joystick::Snapshot,
    pub flycam_active: bool,
    pub anti_spam_blocks: Vec<(crate::world::anti_spam::Queue, uuid::Uuid, String)>,
    pub anti_spam_version: u64,
    /// The page shown and its tab.
    pub page: Page,
    pub tab: usize,
    /// The search of the sidebar, and the view of the modified settings.
    pub query: String,
    was_open: bool,
    /// Prototype: the settings as the window opened, which Cancel puts back.
    opened_with: Option<Box<Settings>>,
    focus_search: bool,
    pub modified_view: bool,
    /// Measured each frame (`Cx`): each row's control size and available
    /// width, the number of rows behind each cog and whether its row is off,
    /// the cogs open, and the sidebar counts.
    control_sizes: HashMap<egui::Id, (f32, Vec2)>,
    subs: HashMap<egui::Id, (usize, bool)>,
    open_subs: HashSet<egui::Id>,
    counts: Counts,
    /// When `counts` and `subs` were last taken and the search they were
    /// for, and whether an input or a change since asks for them again
    /// (`count_due`).
    counted: Option<(f64, String)>,
    count_stale: bool,
    /// "Go to" (search, crumbs, links): the page, its tab, the row to light.
    go_to: Option<(Page, usize, Option<egui::Id>)>,
    /// The cog to open with a row taken from the search.
    open_parent: Option<egui::Id>,
    /// The row lit after "Go to" (since when), and the scroll to it pending.
    flash: Option<(egui::Id, f64)>,
    flash_scroll: bool,
    /// A row to light by its label once its page is shown (demo, the other
    /// windows' "Preferences…").
    pub reveal: Option<(Page, usize, &'static str)>,
    /// The defaults the rows compare with.
    defaults: Option<Box<Settings>>,
    undo: Option<Undo>,
    /// Reset the page shown at the next frame (`PROTOTYPE_DEMO_OPTIONS=…:reset`).
    pub demo_reset: bool,
    /// Open every cog once counted (`PROTOTYPE_DEMO_OPTIONS=…:cogs`, captures).
    pub demo_open_cogs: bool,
    /// Keep the system font list open, scrolling it when true
    /// (`PROTOTYPE_DEMO_OPTIONS=appearance:system-fonts`, `…:system-fonts-scroll`).
    pub demo_font_list: Option<bool>,
    interface_scale: scale::ScaleSlider,
    text_scale: scale::ScaleSlider,
    /// The account's content ceiling and the rating it wants, while logged
    /// in (set by the app).
    pub maturity: Option<prototype_net::search::AgentAccess>,
    /// The account settings the grid keeps (Privacy), while logged in (set
    /// by the app).
    pub account: Option<privacy::Account>,
    /// The session's "Show my favorite landmarks at login", while logged in
    /// with an account the login screen remembers (set by the app).
    pub login_favorites: Option<bool>,
    /// The account's "Show Friends Only", while logged in (set by the app).
    pub friends_only: Option<bool>,
    /// The account's "Keep Show Friends Only after teleporting", while
    /// logged in (set by the app).
    pub friends_only_keep: Option<bool>,
    system_fonts: Option<Vec<SystemFont>>,
    font_filter: String,
    /// Disk cache usage (bytes) and whether a clear is running; set by the app.
    pub cache_usage: u64,
    pub cache_clearing: bool,
    /// Detected dedicated/shared heaps (MB) and integrated GPU; set by the app.
    pub vram: (Option<u64>, Option<u64>, bool),
    /// The GPUs that can draw the viewer, the one in use, and why the
    /// dedicated one may not be drawing; set by the app as soon as the
    /// renderer is created, so the login screen has them too (#894). The
    /// choice applies at the next start.
    pub gpus: Vec<String>,
    pub gpu_in_use: String,
    pub gpu_notice: Option<prototype_render::adapter_choice::Notice>,
    /// The sky in use is tone mapped (not classic); set by the app each
    /// frame: the tone mapper choice is greyed otherwise.
    pub sky_tone_mapped: bool,
    /// Audio devices (set by the app) and microphone level (0..1).
    pub output_devices: Vec<String>,
    pub input_devices: Vec<String>,
    /// The device lists are being read again (set by the app).
    pub devices_listing: bool,
    pub mic_level: f32,
    /// Media plugins folder found (empty: none) and plugins running.
    pub media_plugins: String,
    pub media_running: usize,
    /// Prototype's media plugins pack (set by the app), and its "Download" or
    /// "Retry" button (cleared by the app).
    pub media_pack: crate::media::pack::PackState,
    pub media_pack_download: bool,
    /// The account's media filter list while logged in (set by the app when
    /// it changes); the changes made here, applied by the app; the site
    /// being typed.
    pub media_filter: Option<Vec<crate::media::filter::Rule>>,
    pub media_filter_edits: Vec<crate::media::filter::Edit>,
    media_filter_typed: String,
    /// The site being typed in Preferences › Protections › Trusted sites.
    trusted_site_typed: String,
    /// The site being typed in Preferences › Chat › Picture sites.
    picture_site_typed: String,
    /// Preferences › Chat › Spelling: the auto-replace list shown, the
    /// keyword being typed, the file dialogs.
    spelling_page: pages::spelling::SpellingPage,
    /// Preferences › Camera › Mouselook: the crosshair's picture being
    /// chosen in the file dialog.
    crosshair_dialog: Option<std::sync::mpsc::Receiver<std::path::PathBuf>>,
    /// Shortcut or gesture waiting for its next input, captured by the app.
    pub capture: Option<Capture>,
    joystick_capture: Option<pages::joystick::JoystickCapture>,
    capture_modifier: Option<winit::keyboard::KeyCode>,
    clear: Option<(crate::keybinds::Action, usize)>,
    /// Sound assets being typed (Legacy name -> text).
    ui_sound_text: HashMap<&'static str, String>,
    /// The keyword alerts' sound asset being typed, and the asset it was
    /// started from.
    keyword_sound_text: Option<(uuid::Uuid, String)>,
    /// Folder of the conversation transcripts of the account logged in
    /// (set by the app; none on the login screen).
    pub chat_log_dir: Option<std::path::PathBuf>,
    /// What the last erasing of the transcripts did (set by the app).
    pub chat_log_status: String,
    /// The transcripts are in a folder the user chose (set by the app).
    pub chat_log_custom: bool,
    /// "Erase this account's history?" confirmation open.
    confirm_erase_logs: bool,
    /// "Reset …?" confirmation open, then the reset it said yes to, done by
    /// the next pass of the window's body.
    confirm_reset: Option<ResetAsk>,
    reset_confirmed: Option<ResetAsk>,
    /// The import of a Legacy Viewer's chat history (set by the app).
    pub chat_import: ChatImportUi,
    /// Files done and files to do of the import under way (set by the app).
    pub chat_log_importing: Option<(usize, usize)>,
    pub restart_voice: bool,
    /// A settings backup read, waiting for the confirmation of its restore
    /// (set by the app), and what the last backup and restore did.
    pub restore: Option<backup::Pending>,
    pub backup_status: String,
    pub restore_status: String,
}

impl OptionsUi {
    pub fn joystick_capturing(&self) -> bool {
        self.joystick_capture.is_some()
    }

    pub fn cancel_capture(&mut self) {
        self.capture = None;
        self.capture_modifier = None;
        self.joystick_capture = None;
    }

    pub fn cancel_joystick_capture(&mut self) {
        self.joystick_capture = None;
    }

    pub fn prepare_scale_input(&mut self, input: &egui::RawInput, pixels_per_point: f32, numeric_captured: bool) {
        self.interface_scale.prepare_input(input, pixels_per_point, numeric_captured);
        self.text_scale.prepare_input(input, pixels_per_point, numeric_captured);
    }

    /// A HUD modifier on its own is captured on release, leaving time to
    /// press a second key for combinations such as Ctrl+B.
    pub fn capture_key(&mut self, code: winit::keyboard::KeyCode, pressed: bool, repeat: bool) -> Option<Option<crate::keybinds::Input>> {
        use crate::keybinds::{Action, Input};
        use winit::keyboard::KeyCode;
        let capture = self.capture?;
        if repeat {
            return None;
        }
        if pressed && code == KeyCode::Escape {
            return Some(None);
        }
        if Input::is_modifier(code) {
            if matches!(capture, Capture::Binding(Action::HudDrag, _)) {
                if pressed {
                    self.capture_modifier = Some(code);
                } else if self.capture_modifier == Some(code) {
                    return Some(Some(Input::key(code)));
                }
            }
            return None;
        }
        pressed.then(|| Some(Input::key(code)))
    }

    pub fn take_capture(&mut self) -> Option<Capture> {
        self.capture_modifier = None;
        self.capture.take()
    }

    /// Opens a page on one of its tabs.
    pub fn open(&mut self, page: Page, tab: usize) {
        self.page = page;
        self.tab = tab;
        self.query.clear();
        self.modified_view = false;
        self.cancel_capture();
    }

    /// Opens the Chat page on the YouTube player's settings (a player's
    /// cog, #848).
    pub fn open_videos(&mut self) {
        self.open(Page::Chat, 0);
        self.reveal = Some((Page::Chat, 0, prototype_i18n::tr!("ui-options-video-volume")));
    }

    /// Opens Preferences › Camera › Behavior on the camera's limits (the
    /// login card of `camera_limits_notice.rs`).
    pub fn open_camera_limits(&mut self) {
        self.open(Page::Camera, 1);
        self.reveal = Some((Page::Camera, 1, prototype_i18n::tr!("ui-options-camera-unrestricted-camera")));
    }

    /// Opens Preferences › Chat on the GIF picker's settings (#932).
    pub fn open_gifs(&mut self) {
        self.open(Page::Chat, 0);
        self.reveal = Some((Page::Chat, 0, prototype_i18n::tr!("ui-options-gifs-show")));
    }

    /// Opens the Sound and voice page on one of its parts.
    pub fn open_audio(&mut self, section: AudioSection) {
        self.open(Page::Sound, 2);
        self.reveal = Some((
            Page::Sound,
            2,
            match section {
                AudioSection::Voice => prototype_i18n::tr!("ui-options-enable-voice"),
                AudioSection::Processing => prototype_i18n::tr!("voice-automatic-gain"),
                AudioSection::Indicators => prototype_i18n::tr!("voice-show-visualizers"),
            },
        ));
    }

    /// What a reset changes besides the settings: the app applies them
    /// again (the fonts, the sound, the camera's view, the interface's
    /// scale), and the shadows follow their quality.
    fn after_reset(&mut self, page: Page, s: &mut Settings, r: &mut OptionsResult) {
        s.shadows = s.shadow_quality > 0;
        r.changed = true;
        r.font_changed |= matches!(page, Page::Appearance);
        r.audio_changed |= matches!(page, Page::Sound | Page::Media);
        r.reset_camera |= matches!(page, Page::Camera);
        r.ui_scale_changed |= matches!(page, Page::Layout);
        self.cancel_capture();
        self.keyword_sound_text = None;
        self.ui_sound_text.clear();
        self.trusted_site_typed.clear();
        self.picture_site_typed.clear();
    }

    /// Announces a reset whose undo went away.
    fn finish_undo(&mut self, r: &mut OptionsResult) {
        if let Some(undo) = self.undo.take() {
            match undo.done {
                Done::Page(page) => r.reset_section = Some(page),
                Done::Modified => {}
                Done::All => r.reset = true,
            }
        }
    }
}

#[derive(Default)]
pub struct OptionsResult {
    pub toggle_flycam: bool,
    pub reset_anti_spam: bool,
    pub unblock_anti_spam: Option<(crate::world::anti_spam::Queue, uuid::Uuid)>,
    /// Settings that need the general application path (rendering, network,
    /// crash recovery, theme). UI zoom has its own per-frame path below.
    pub changed: bool,
    /// Persist the current settings while the floater stays open.
    pub save: bool,
    pub ui_scale_changed: bool,
    pub font_changed: bool,
    pub clear_cache: bool,
    /// Devices, volumes or voice settings changed: apply to the audio engine.
    pub audio_changed: bool,
    /// List the audio devices again.
    pub refresh_devices: bool,
    /// Every setting went back to its default.
    pub reset: bool,
    /// The settings of one page went back to their defaults.
    pub reset_section: Option<Page>,
    /// The camera settings went back to their defaults: the view follows.
    pub reset_camera: bool,
    /// Show the folder of the conversation transcripts.
    pub open_chat_logs: bool,
    /// Choose another folder for the cache or the transcripts, or go back
    /// to the default one (the app asks for the folder and moves the files).
    pub folder: Option<(Folder, FolderChoice)>,
    /// Erase the conversation transcripts of the account (confirmed).
    pub erase_chat_logs: bool,
    /// The import window opened: look for the Legacy Viewers' folders.
    pub find_legacy_logs: bool,
    /// Import from these folders.
    pub import_legacy_logs: Option<Vec<std::path::PathBuf>>,
    /// The content rating chosen (SIM_ACCESS_*), to send to the server.
    pub maturity: Option<u8>,
    /// An account setting of Privacy changed, to send to the server.
    pub account: Option<crate::world::user_info::Change>,
    /// "Show my favorite landmarks at login" switched (Privacy).
    pub login_favorites: Option<bool>,
    /// "Show Friends Only" switched (Controls).
    pub friends_only: Option<bool>,
    /// "Keep Show Friends Only after teleporting" switched (Controls).
    pub friends_only_keep: Option<bool>,
    /// Open the Dev Toolbox (Network and cache › Logs and tools).
    pub open_dev_toolbox: bool,
    /// "Remove Prototype from this computer" (Language and updates): the app
    /// asks for the confirmation.
    pub uninstall: bool,
    /// Back up the settings to a file the user chooses.
    pub back_up: bool,
    /// Choose a backup to restore (then `restore_modal` confirms it).
    pub pick_backup: bool,
    /// A restore confirmed: the app puts back what is not the settings.
    pub restored: Option<backup::Restored>,
}

/// A folder the user can choose (Preferences › Network and cache, Chat).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Folder {
    Cache,
    ChatLogs,
    /// A Legacy Viewer's transcripts, to import.
    LegacyLogs,
}

/// The window that shows what will be imported.
#[derive(Default)]
pub struct ChatImportUi {
    pub open: bool,
    /// The folders found, ticked or not; None until the search answers.
    pub found: Option<Vec<(crate::world::chat_log::import::Found, bool)>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FolderChoice {
    /// Ask for a folder.
    Pick,
    /// Back to the default folder.
    Default,
}

/// Width of the sidebar, height of the footer.
const SIDEBAR_W: f32 = 224.0;
const FOOTER_H: f32 = 64.0;
/// The controls' widths (a slider's rail, a drop-down list).
const SLIDER_W: f32 = 150.0;
const COMBO_W: f32 = 190.0;

/// A path as shown in the preferences: elided to the width, whole on hover.
fn path_label(ui: &mut egui::Ui, p: &Palette, dir: &std::path::Path, width: f32) {
    // without the verbatim prefix Windows gives a canonical path
    let path = dir.display().to_string();
    let path = path.strip_prefix(r"\\?\").unwrap_or(&path).to_owned();
    ui.add(super::widgets::elided_label(
        ui,
        RichText::new(&path).size(11.5).color(p.muted),
        width,
    ))
    .on_hover_text(&path);
}

/// Why Prototype's media plugins pack is not there yet.
fn media_pack_error(failure: prototype_updater::Failure) -> &'static str {
    use prototype_updater::Failure;
    match failure {
        Failure::Network | Failure::Busy | Failure::Startup => prototype_i18n::tr!("media-plugins-error-network"),
        Failure::Integrity | Failure::Package => prototype_i18n::tr!("media-plugins-error-integrity"),
        Failure::Filesystem => prototype_i18n::tr!("media-plugins-error-filesystem"),
        Failure::Unsupported => prototype_i18n::tr!("media-plugins-error-unsupported"),
    }
}

/// Hides the stock handle (Prototype paints its own white knob over it).
fn hide_handle(ui: &mut egui::Ui) {
    let w = &mut ui.visuals_mut().widgets;
    for state in [&mut w.inactive, &mut w.hovered, &mut w.active] {
        state.bg_fill = egui::Color32::TRANSPARENT;
        state.fg_stroke = egui::Stroke::NONE;
    }
}

fn fraction<T: egui::emath::Numeric>(v: T, range: &std::ops::RangeInclusive<T>) -> f32 {
    let (a, b) = (range.start().to_f64(), range.end().to_f64());
    if (b - a).abs() < f64::EPSILON {
        0.0
    } else {
        ((v.to_f64() - a) / (b - a)) as f32
    }
}

fn slider<T: egui::emath::Numeric>(ui: &mut egui::Ui, v: &mut T, range: std::ops::RangeInclusive<T>, suffix: &str) -> bool {
    ui.spacing_mut().slider_width = SLIDER_W;
    let p = crate::theme::Theme::current_palette(ui.ctx());
    let resp = ui
        .scope(|ui| {
            hide_handle(ui);
            ui.add(widgets::restricted_number(
                egui::Slider::new(v, range.clone())
                    .suffix(suffix)
                    .custom_parser(prototype_i18n::parse_number)
                    .trailing_fill(false),
                widgets::TextPolicy::Decimal,
            ))
        })
        .inner;
    super::widgets::paint_slider_knob(ui, &p, &resp, fraction(*v, &range));
    resp.changed()
}

/// Slider of whole values (the Legacy Viewer's decimal_digits="0").
fn slider_whole(ui: &mut egui::Ui, v: &mut f32, range: std::ops::RangeInclusive<f32>, suffix: &str) -> bool {
    ui.spacing_mut().slider_width = SLIDER_W;
    let p = crate::theme::Theme::current_palette(ui.ctx());
    let resp = ui
        .scope(|ui| {
            hide_handle(ui);
            ui.add(widgets::restricted_number(
                egui::Slider::new(v, range.clone())
                    .suffix(suffix)
                    .custom_parser(prototype_i18n::parse_number)
                    .fixed_decimals(0)
                    .step_by(1.0)
                    .trailing_fill(false),
                widgets::TextPolicy::Decimal,
            ))
        })
        .inner;
    super::widgets::paint_slider_knob(ui, &p, &resp, fraction(*v, &range));
    resp.changed()
}

/// Stepped slider with a named level shown next to it.
fn stepped(ui: &mut egui::Ui, p: &Palette, v: &mut u8, labels: &[&str]) -> bool {
    // Dragon Viewer: a level is picked on a row of orbs
    let count = labels.len().max(2) as u8;
    let changed = orb_strip(ui, p, v, count, 20.0, &[]);
    let label = labels.get((*v).min(count - 1) as usize).copied().unwrap_or("");
    ui.label(RichText::new(label).size(12.0).color(p.violet_pale));
    changed
}

/// Dragon Viewer's orb slider: `count` orbs on a glowing rail, one per
/// level. A click picks a level: the orbs up to it shine, the chosen one
/// larger in a blue halo, the others dim. Arrow keys step it.
pub(super) fn orb_strip(ui: &mut egui::Ui, p: &Palette, value: &mut u8, count: u8, orb: f32, labels: &[&str]) -> bool {
    let count = count.max(2);
    let gap = (orb * 0.42).max(6.0);
    let label_h = if labels.is_empty() { 0.0 } else { 16.0 };
    let width = count as f32 * orb + (count - 1) as f32 * gap + 8.0;
    let (rect, mut resp) = ui.allocate_exact_size(vec2(width, orb * 1.25 + label_h), Sense::click_and_drag());
    let row = Rect::from_min_size(rect.min, vec2(rect.width(), orb * 1.25));
    let centers: Vec<egui::Pos2> = (0..count)
        .map(|i| pos2(row.left() + 4.0 + orb * 0.5 + i as f32 * (orb + gap), row.center().y))
        .collect();
    let old = *value;
    if let Some(at) = resp.interact_pointer_pos()
        && (resp.clicked() || resp.dragged())
    {
        let nearest = centers
            .iter()
            .enumerate()
            .min_by(|a, b| (a.1.x - at.x).abs().total_cmp(&(b.1.x - at.x).abs()))
            .map_or(0, |(i, _)| i as u8);
        *value = nearest;
        resp.request_focus();
    }
    if resp.has_focus() {
        ui.input(|i| {
            if i.key_pressed(egui::Key::ArrowRight) || i.key_pressed(egui::Key::ArrowUp) {
                *value = (*value + 1).min(count - 1);
            }
            if i.key_pressed(egui::Key::ArrowLeft) || i.key_pressed(egui::Key::ArrowDown) {
                *value = value.saturating_sub(1);
            }
        });
    }
    *value = (*value).min(count - 1);
    let chosen = *value as usize;
    let painter = ui.painter();
    if p.high_contrast {
        for (i, c) in centers.iter().enumerate() {
            let on = i <= chosen;
            painter.circle_filled(*c, orb * 0.4, if on { p.violet } else { p.field });
            painter.circle_stroke(*c, orb * 0.4, egui::Stroke::new(if i == chosen { 3.0 } else { 1.0 }, p.ink));
        }
    } else {
        // the rail between the first and the last orb, lit up to the choice
        let rail = Rect::from_min_max(pos2(centers[0].x, row.center().y - 3.0), pos2(centers[count as usize - 1].x, row.center().y + 3.0));
        super::widgets::dragon_rail(painter, rail, centers[chosen].x);
        let hot = resp.hover_pos().map(|at| {
            centers
                .iter()
                .enumerate()
                .min_by(|a, b| (a.1.x - at.x).abs().total_cmp(&(b.1.x - at.x).abs()))
                .map_or(0, |(i, _)| i)
        });
        for (i, c) in centers.iter().enumerate() {
            let mut painter = painter.clone();
            if i > chosen {
                painter.multiply_opacity(0.55);
            }
            let radius = if i == chosen { orb * 0.56 } else { orb * 0.5 };
            if i == chosen {
                super::dragon_art::glow(&painter, *c, orb * 0.95, egui::Color32::from_rgb(69, 216, 255), 1.0);
                painter.circle_stroke(*c, radius + 2.0, egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 247, 161)));
            } else if hot == Some(i) {
                super::dragon_art::glow(&painter, *c, orb * 0.75, egui::Color32::from_rgb(255, 139, 32), 0.8);
            }
            super::dragon_art::orb(&painter, *c, radius, (i % 7) as u8 + 1);
        }
    }
    if !labels.is_empty() {
        for (i, c) in centers.iter().enumerate() {
            if let Some(label) = labels.get(i) {
                let ink = if i == chosen { p.ink } else { p.muted_dim };
                let galley = painter.layout_no_wrap((*label).to_owned(), egui::FontId::proportional(11.0), ink);
                super::widgets::paint_ink_centered(painter, galley, pos2(c.x, rect.bottom() - 7.0), ink);
            }
        }
    }
    if resp.has_focus() {
        painter.rect_stroke(rect, 6.0, egui::Stroke::new(1.0, p.ink.gamma_multiply(0.5)), egui::StrokeKind::Inside);
    }
    if *value != old {
        resp.mark_changed();
    }
    let label = labels.get(chosen).copied().unwrap_or("");
    resp.widget_info(|| egui::WidgetInfo::slider(ui.is_enabled(), f64::from(chosen as u8), label));
    resp.changed()
}

/// Color swatch opening the compact picker (opaque); `id` tells the
/// swatches of a page apart.
fn swatch(ui: &mut egui::Ui, p: &Palette, id: impl egui::AsIdSalt, v: &mut crate::settings::Rgba) -> bool {
    ui.spacing_mut().interact_size = Vec2::new(38.0, 18.0);
    super::widgets::color_button(ui, p, id, v, false).changed()
}

fn toggle(ui: &mut egui::Ui, p: &Palette, v: &mut bool) -> bool {
    super::widgets::switch(ui, p, v).changed()
}

/// A drop-down list of `(value, label)`, its id the row's.
fn combo<T: PartialEq + Copy>(ui: &mut egui::Ui, value: &mut T, choices: &[(T, &str)]) -> bool {
    let shown = choices.iter().find(|(v, _)| v == value).map_or("", |(_, l)| *l);
    let mut changed = false;
    egui::ComboBox::from_id_salt(ui.scope_id().with("combo"))
        .width(COMBO_W)
        .selected_text(shown)
        .truncate()
        .show_ui(ui, |ui| {
            for (v, label) in choices {
                changed |= ui.selectable_value(value, *v, *label).changed();
            }
        });
    changed
}

/// `combo` whose choices with a reason cannot be picked: greyed, the
/// reason shown on hover.
fn combo_blocked<T: PartialEq + Copy>(ui: &mut egui::Ui, value: &mut T, choices: &[(T, &str, Option<&str>)]) -> bool {
    let shown = choices.iter().find(|(v, _, _)| v == value).map_or("", |(_, l, _)| *l);
    let mut changed = false;
    egui::ComboBox::from_id_salt(ui.scope_id().with("combo"))
        .width(COMBO_W)
        .selected_text(shown)
        .truncate()
        .show_ui(ui, |ui| {
            for (v, label, blocked) in choices {
                let r = ui
                    .add_enabled_ui(blocked.is_none(), |ui| ui.selectable_value(value, *v, *label))
                    .inner;
                match blocked {
                    Some(why) => {
                        r.on_disabled_hover_text(*why);
                    }
                    None => changed |= r.changed(),
                }
            }
        });
    changed
}

/// A choice among a few options side by side: `widgets::segmented` over a
/// setting of any type; the options marked are lit in Prototype's accent.
fn segmented<T: PartialEq + Copy>(ui: &mut egui::Ui, p: &Palette, value: &mut T, choices: &[(T, &str, bool)]) -> bool {
    let options: Vec<(&str, bool)> = choices.iter().map(|(_, l, accent)| (*l, *accent)).collect();
    let mut i = choices.iter().position(|(v, _, _)| v == value).unwrap_or(usize::MAX);
    let changed = super::widgets::segmented(ui, p, &mut i, &options);
    if changed && let Some((v, _, _)) = choices.get(i) {
        *value = *v;
    }
    changed
}

/// The width a `segmented` choice takes.
fn segmented_width<T>(ui: &egui::Ui, choices: &[(T, &str, bool)]) -> f32 {
    let options: Vec<(&str, bool)> = choices.iter().map(|(_, l, a)| (*l, *a)).collect();
    super::widgets::segmented_size(ui, &options).x
}

fn format_bytes(b: u64) -> String {
    let mb = b as f64 / (1024.0 * 1024.0);
    if mb >= 1024.0 {
        prototype_i18n::tr!("memory-gb", value = prototype_i18n::number(mb / 1024.0, 1))
    } else {
        prototype_i18n::tr!("memory-mb", value = prototype_i18n::number(mb, 0))
    }
}

/// The content ratings an account may choose (LLFloaterPreference: the
/// list without "Adult" for an account that is not adult; no choice at all
/// below Moderate), most open first.
pub fn maturity_choices(access: prototype_net::search::AgentAccess) -> Vec<(u8, &'static str)> {
    use prototype_net::search::{SIM_ACCESS_ADULT, SIM_ACCESS_MATURE, SIM_ACCESS_PG};
    let all = [
        (SIM_ACCESS_ADULT, prototype_i18n::tr!("search-maturity-gma")),
        (SIM_ACCESS_MATURE, prototype_i18n::tr!("search-maturity-gm")),
        (SIM_ACCESS_PG, prototype_i18n::tr!("ui-options-content-general")),
    ];
    if access.max < SIM_ACCESS_MATURE {
        return Vec::new();
    }
    all.into_iter().filter(|(m, _)| *m <= access.max).collect()
}

/// "I want to access content rated" (maturity_desired_combobox): a list,
/// or the rating as text when the account has no choice.
fn maturity_choice(ui: &mut egui::Ui, p: &Palette, access: prototype_net::search::AgentAccess) -> Option<u8> {
    let choices = maturity_choices(access);
    let label = |m: u8| match m {
        prototype_net::search::SIM_ACCESS_ADULT => prototype_i18n::tr!("search-maturity-gma"),
        prototype_net::search::SIM_ACCESS_MATURE => prototype_i18n::tr!("search-maturity-gm"),
        _ => prototype_i18n::tr!("ui-options-content-general"),
    };
    if choices.is_empty() {
        ui.horizontal(|ui| {
            super::widgets::maturity_badges_up_to(ui, p, access.preferred, 16.0);
            ui.label(label(access.preferred));
        });
        return None;
    }
    let mut picked = access.preferred;
    ui.horizontal(|ui| {
        egui::ComboBox::from_id_salt("ui-maturity")
            .selected_text(label(access.preferred))
            .show_ui(ui, |ui| {
                for (m, name) in &choices {
                    // the badges of the navigation bar, as the Legacy
                    // Viewer's rating icons, before each choice
                    ui.horizontal(|ui| {
                        ui.add_sized(Vec2::new(54.0, 16.0), |ui: &mut egui::Ui| {
                            ui.horizontal(|ui| super::widgets::maturity_badges_up_to(ui, p, *m, 14.0)).response
                        });
                        if ui.selectable_label(picked == *m, *name).clicked() {
                            picked = *m;
                        }
                    });
                }
            });
        super::widgets::maturity_badges_up_to(ui, p, access.preferred, 16.0);
    });
    (picked != access.preferred).then_some(picked)
}

/// The five graphics levels, side by side; none is lit once a setting of
/// the page no longer matches any of them ("Custom", like the Legacy
/// Viewer's slider after a manual change).
fn preset_choice(ui: &mut egui::Ui, p: &Palette, s: &mut Settings) -> bool {
    use crate::graphics_presets::Preset;
    let current = Preset::matching(s);
    let options: Vec<(&str, bool)> = Preset::ALL.iter().map(|l| (l.label(), true)).collect();
    let mut selected = current.and_then(|c| Preset::ALL.iter().position(|l| *l == c)).unwrap_or(usize::MAX);
    let changed = super::widgets::segmented(ui, p, &mut selected, &options);
    if current.is_none() && !changed {
        ui.label(RichText::new(preset_label(s)).size(12.0).color(p.violet_pale));
    }
    match Preset::ALL.get(selected) {
        Some(level) if changed => {
            level.apply(s);
            true
        }
        _ => false,
    }
}

/// The line under the level row when no level matches: the Mac's own
/// default on macOS, "Custom" otherwise.
fn preset_label(s: &Settings) -> &'static str {
    if cfg!(target_os = "macos") && crate::graphics_presets::mac_mid_high::matches(s) {
        prototype_i18n::tr!("ui-options-preset-mac-mid-high")
    } else {
        prototype_i18n::tr!("ui-options-preset-custom")
    }
}

/// Device combo: "" = system default. Returns (a device was chosen, the
/// list is unfolded).
/// `listing`: the devices are being listed again (off the UI thread); the
/// list shows the last ones and says so, and the current choice is not
/// called missing meanwhile.
pub(super) fn device_combo(ui: &mut egui::Ui, id: &str, value: &mut String, devices: &[String], width: f32, listing: bool) -> (bool, bool) {
    choice_combo(
        ui,
        id,
        value,
        devices,
        width,
        prototype_i18n::tr!("ui-options-system-default"),
        listing,
    )
}

/// `device_combo` whose empty choice reads `default_label`.
pub(super) fn named_combo(
    ui: &mut egui::Ui,
    id: &str,
    value: &mut String,
    devices: &[String],
    width: f32,
    default_label: &str,
) -> (bool, bool) {
    choice_combo(ui, id, value, devices, width, default_label, false)
}

fn choice_combo(
    ui: &mut egui::Ui,
    id: &str,
    value: &mut String,
    devices: &[String],
    width: f32,
    default_label: &str,
    listing: bool,
) -> (bool, bool) {
    let mut changed = false;
    let shown = if value.is_empty() {
        default_label.to_owned()
    } else {
        value.clone()
    };
    let list = egui::ComboBox::from_id_salt(id)
        .width(width)
        .selected_text(shown)
        .truncate()
        .show_ui(ui, |ui| {
            changed |= ui.selectable_value(value, String::new(), default_label).changed();
            for d in devices {
                changed |= ui.selectable_value(value, d.clone(), d).changed();
            }
            if listing {
                ui.label(RichText::new(prototype_i18n::tr!("ui-options-listing-devices")).size(12.0).weak());
            } else if !value.is_empty() && !devices.contains(value) {
                ui.label(
                    RichText::new(prototype_i18n::tr!("status-value-missing", value = value.to_string()))
                        .size(12.0)
                        .weak(),
                );
            }
        });
    (changed, list.inner.is_some())
}

/// One binding of a shortcut: its keys as keycaps, a click to capture new
/// ones, a right click to clear it; a dot when another action uses it.
fn binding_cell(
    ui: &mut egui::Ui,
    p: &Palette,
    kb: &crate::keybinds::KeyBindings,
    st: &mut OptionsUi,
    a: crate::keybinds::Action,
    slot: usize,
) {
    let capturing = st.capture == Some(Capture::Binding(a, slot));
    let b = kb.get(a)[slot].clone();
    let fill = if capturing { p.violet } else { p.raised };
    // a bound key is drawn as keycaps over an empty button of their width
    let caps = match &b {
        Some(b) if !capturing => keycap::caps(ui, b, fill),
        _ => Vec::new(),
    };
    let (text, col) = if capturing {
        (prototype_i18n::tr!("common-press-a-key"), p.on_accent)
    } else if b.is_some() {
        ("", p.ink)
    } else {
        ("—", p.muted_dim)
    };
    let caps_width = keycap::row_width(&caps);
    let resp = ui.add(
        egui::Button::new(RichText::new(text).size(12.0).color(col))
            .fill(fill)
            .corner_radius(ui.visuals().widgets.inactive.corner_radius)
            .min_size(Vec2::new((caps_width + 12.0).max(132.0), 22.0)),
    );
    let caps_left = egui::pos2(resp.rect.center().x - caps_width / 2.0, resp.rect.center().y);
    keycap::paint_row(ui, p, &caps, caps_left, fill);
    if resp.clicked() {
        st.cancel_capture();
        if !capturing {
            st.capture = Some(Capture::Binding(a, slot));
        }
    }
    if resp.secondary_clicked() && b.is_some() {
        st.clear = Some((a, slot));
    }
    let others: Vec<&str> = b
        .as_ref()
        .map(|b| kb.users(b).into_iter().filter(|x| *x != a).map(crate::keybinds::label).collect())
        .unwrap_or_default();
    let tip = if capturing {
        prototype_i18n::tr!("ui-options-key-combination-ctrl-shift-alt-or-mouse-button-wheel").to_owned()
    } else if others.is_empty() {
        prototype_i18n::tr!("ui-options-click-change-right-click-clear").to_owned()
    } else {
        prototype_i18n::tr!(
            "ui-options-click-change-right-click-clear-also-used-by",
            value0 = others.join(", ").to_string()
        )
    };
    // conflict marker
    if !capturing && !others.is_empty() {
        ui.painter()
            .circle_filled(resp.rect.right_top() + Vec2::new(-4.0, 4.0), 3.0, p.amber);
    }
    resp.on_hover_text(tip);
}

fn uuid_text(id: uuid::Uuid) -> String {
    if id.is_nil() { String::new() } else { id.to_string() }
}

/// A typed sound asset: a UUID, or empty for no sound.
fn parse_uuid_text(text: &str) -> Option<uuid::Uuid> {
    let t = text.trim();
    if t.is_empty() {
        Some(uuid::Uuid::nil())
    } else {
        uuid::Uuid::parse_str(t).ok()
    }
}

/// Runs the rows of `page` (all its tabs, or the one given) in `mode`.
fn run_page(cx: &mut Cx, page: Page, tab: Option<usize>) {
    let tabs = page.tabs().len().max(1);
    for t in 0..tabs {
        if tab.is_some_and(|only| only != t) {
            continue;
        }
        cx.enter(page, t);
        pages::draw(cx, page, t);
    }
}

/// What Preferences says of each setting, in the interface's language: its
/// path (`camera.fov`, with no space), the row's label and its hint. For the
/// Debug Settings window, whose list is the settings themselves.
pub fn describe_settings() -> Vec<(String, String, String)> {
    let ctx = egui::Context::default();
    let p = crate::theme::Theme::default().palette();
    let mut s = Settings::default();
    let d = Settings::default();
    let mut st = OptionsUi::default();
    let mut r = OptionsResult::default();
    let mut out = Vec::new();
    run_ui(&ctx, egui::RawInput::default(), |ui| {
        let mut cx = Cx::new(ui, p, &mut s, &d, &mut st, &mut r, Mode::Collect, "");
        for page in Page::ALL {
            run_page(&mut cx, page, None);
        }
        out = std::mem::take(&mut cx.described)
            .into_iter()
            .map(|(path, label, hint)| (path.replace(' ', ""), label, hint))
            .collect();
    });
    out
}

/// The pages counted (modified rows, search hits), nothing drawn.
fn count(ui: &mut egui::Ui, p: &Palette, s: &mut Settings, d: &Settings, st: &mut OptionsUi, r: &mut OptionsResult) {
    let query = st.query.clone();
    let mut cx = Cx::new(ui, *p, s, d, st, r, Mode::Count, &query);
    for page in Page::ALL {
        run_page(&mut cx, page, None);
    }
    let counts = std::mem::take(&mut cx.counts);
    st.counts = counts;
}

/// How long the counts may wait for a change nothing announced (the app's
/// state behind a row: devices, account, grid).
const COUNT_REFRESH: f64 = 0.5;

/// The user did something this frame: a click, a key, typed text. What it
/// changes (here or in another window) is counted at the next frame.
fn interaction(ui: &egui::Ui) -> bool {
    ui.input(|i| {
        i.events.iter().any(|e| {
            matches!(
                e,
                egui::Event::PointerButton { .. }
                    | egui::Event::Key {
                        pressed: true,
                        repeat: false,
                        ..
                    }
                    | egui::Event::Text(_)
                    | egui::Event::Paste(_)
            )
        })
    })
}

/// Whether to count every page again this frame. Running all their rows
/// costs as much as drawing a page, so it waits for something that can
/// change the counts: an input (this frame or the last), a change of the
/// window, another search, or `COUNT_REFRESH` for the rest.
fn count_due(st: &OptionsUi, now: f64, input: bool) -> bool {
    input
        || st.count_stale
        || st.reveal.is_some()
        || st.demo_open_cogs
        || st
            .counted
            .as_ref()
            .is_none_or(|(at, query)| now - at >= COUNT_REFRESH || now < *at || *query != st.query)
}

/// One frame of `body` outside the window (tests, `describe_settings`), its
/// textures dropped as a renderer would take them.
pub(crate) fn run_ui(ctx: &egui::Context, raw: egui::RawInput, body: impl FnMut(&mut egui::Ui)) {
    let mut output = ctx.run_ui(raw, body);
    output.textures_delta.clear();
}

/// The reset of a page, outside the window (tests of the settings).
#[cfg(test)]
pub(crate) fn reset_page_settings(page: Page, s: &mut Settings) {
    let ctx = egui::Context::default();
    let p = crate::theme::Theme::default().palette();
    let d = Settings::default();
    let mut st = OptionsUi::default();
    let mut r = OptionsResult::default();
    run_ui(&ctx, egui::RawInput::default(), |ui| {
        reset_rows(ui, &p, s, &d, &mut st, &mut r, Some(page));
        st.after_reset(page, s, &mut r);
    });
}

/// Puts the rows of `page` (every page: `None`) back to their defaults.
fn reset_rows(
    ui: &mut egui::Ui,
    p: &Palette,
    s: &mut Settings,
    d: &Settings,
    st: &mut OptionsUi,
    r: &mut OptionsResult,
    page: Option<Page>,
) {
    let mut cx = Cx::new(ui, *p, s, d, st, r, Mode::Reset, "");
    for each in Page::ALL {
        if page.is_none_or(|only| only == each) {
            run_page(&mut cx, each, None);
        }
    }
}

pub fn show(ctx: &egui::Context, p: &Palette, icons: &Icons, s: &mut Settings, st: &mut OptionsUi, open: &mut bool) -> OptionsResult {
    let _ = icons;
    if let Some(capture) = &mut st.joystick_capture {
        capture.hide();
    }
    st.focus_search = *open && (st.focus_search || !st.was_open);
    if *open && !st.was_open && st.opened_with.is_none() {
        st.opened_with = Some(Box::new(s.clone()));
    }
    let mut r = OptionsResult::default();
    let d = st.defaults.take().unwrap_or_else(|| Box::new(Settings::default()));
    let now = ctx.input(|i| i.time);
    if st.undo.as_ref().is_some_and(|u| now - u.shown > UNDO_SECONDS) {
        st.finish_undo(&mut r);
    }
    let screen = ctx.content_rect();
    // At login the window opens wide enough for the full sidebar (with its
    // section titles) and untruncated footer buttons: under 840 points the
    // page switches to its compact layout.
    let default_size = if st.at_login { vec2(880.0, 590.0) } else { vec2(1120.0, 720.0) };
    let min_size = vec2(550.0, 365.0);
    let size = default_size.min(screen.size() - vec2(40.0, 80.0)).max(min_size);
    let centered = screen.center().x - size.x / 2.0;
    let left = if st.at_login && screen.width() >= 1370.0 {
        let beside_card = screen.left() + screen.width() * 0.335;
        beside_card.min(screen.right() - size.x - 14.0)
    } else {
        centered
    };
    // the title row's search edits a copy, given back after the frame
    let mut searched = false;
    let mut query = st.query.clone();
    let mut focus = st.focus_search;
    let mut floater = Floater::new(
        WINDOW,
        prototype_i18n::tr!("common-preferences"),
        // at login, the mockup's place: beside the card, a quarter down
        pos2(
            left.max(screen.left() + 10.0),
            screen.top() + if st.at_login { (screen.height() * 0.255).min(screen.height() - size.y - 30.0).max(79.0) } else { 60.0 },
        ),
        size,
    )
    .help(prototype_i18n::tr!("ui-options-changes-apply-immediately-and-are-saved-on-closing"))
    .search(
        &mut query,
        prototype_i18n::tr!("ui-options-search-settings"),
        &mut focus,
        &mut searched,
    )
    .flush_body()
    .scene_blur(!st.at_login)
    .steady_size();
    floater.min_size = min_size;
    let mut close = false;
    floater.show(ctx, p, open, |ui| {
        body(ui, p, s, &d, st, &mut r, &mut close);
    });
    st.focus_search = focus;
    if searched {
        st.query = query;
        st.modified_view = false;
        st.cancel_capture();
    }
    if close {
        *open = false;
    }
    if !*open {
        st.opened_with = None;
    }
    st.was_open = *open && !super::widgets::floater_minimized(ctx, WINDOW);
    if !st.was_open || st.joystick_capture.as_ref().is_some_and(|capture| !capture.visible()) {
        st.cancel_joystick_capture();
    }
    if !*open {
        // the window closed: a reset waiting for its undo is done
        st.finish_undo(&mut r);
    }
    erase_logs_modal(ctx, p, st, &mut r, *open);
    reset_modal(ctx, p, st, *open);
    chat_import_modal(ctx, p, st, &mut r, *open);
    backup::restore_modal(ctx, p, s, st, &mut r, *open);
    saved_graphics::confirm(ctx, p, s, st, &mut r, *open);
    st.defaults = Some(d);
    r
}

fn erase_logs_modal(ctx: &egui::Context, p: &Palette, st: &mut OptionsUi, r: &mut OptionsResult, open: bool) {
    if !(st.confirm_erase_logs && open) {
        return;
    }
    let modal = crate::ui::widgets::Modal::new(egui::Id::unique("options_erase_chat_logs")).show(ctx, |ui| {
        ui.set_width(360.0);
        ui.label(
            RichText::new(prototype_i18n::tr!("ui-options-chat-logs-erase-title"))
                .size(15.0)
                .strong()
                .color(p.ink),
        );
        ui.add_space(6.0);
        ui.label(
            RichText::new(prototype_i18n::tr!("ui-options-chat-logs-erase-text"))
                .size(12.5)
                .color(p.muted),
        );
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            use super::widgets::{ActionKind, action_button};
            let size = Vec2::new(0.0, 26.0);
            if action_button(
                ui,
                p,
                "trash-filled",
                prototype_i18n::tr!("ui-options-chat-logs-erase-confirm"),
                ActionKind::Danger,
                size,
            )
            .clicked()
            {
                r.erase_chat_logs = true;
                st.confirm_erase_logs = false;
            }
            if action_button(ui, p, "x", prototype_i18n::tr!("common-cancel"), ActionKind::Flat, size).clicked() {
                st.confirm_erase_logs = false;
            }
        });
    });
    if modal.should_close() {
        st.confirm_erase_logs = false;
    }
}

/// What an import will read, before it starts: the folders found for the
/// account, or the one chosen. The Legacy Viewer's files are only read.
fn chat_import_modal(ctx: &egui::Context, p: &Palette, st: &mut OptionsUi, r: &mut OptionsResult, open: bool) {
    use super::widgets::{ActionKind, action_button, action_button_if, checkbox};
    use prototype_i18n::tr;
    if !(st.chat_import.open && open) {
        return;
    }
    let modal = crate::ui::widgets::Modal::new(egui::Id::unique("options_import_chat_logs")).show(ctx, |ui| {
        ui.set_width(460.0);
        ui.label(
            RichText::new(tr!("ui-options-chat-logs-import-title"))
                .size(15.0)
                .strong()
                .color(p.ink),
        );
        ui.add_space(6.0);
        ui.label(RichText::new(tr!("ui-options-chat-logs-import-text")).size(12.5).color(p.muted));
        ui.add_space(8.0);
        match &mut st.chat_import.found {
            None => {
                ui.label(
                    RichText::new(tr!("ui-options-chat-logs-import-searching"))
                        .size(12.5)
                        .color(p.muted),
                );
            }
            Some(found) if found.is_empty() => {
                ui.label(RichText::new(tr!("ui-options-chat-logs-import-none")).size(12.5).color(p.muted));
            }
            Some(found) => {
                for (folder, ticked) in found.iter_mut() {
                    let usable = folder.files > 0 && !folder.same;
                    *ticked &= usable;
                    ui.add_enabled_ui(usable, |ui| {
                        checkbox(
                            ui,
                            p,
                            ticked,
                            RichText::new(folder.dir.display().to_string()).size(12.5).color(p.ink),
                        );
                    });
                    let note = if folder.same {
                        tr!("ui-options-chat-logs-import-same").to_owned()
                    } else if folder.files == 0 {
                        tr!("ui-options-chat-logs-import-empty").to_owned()
                    } else if folder.bytes < 1024 * 1024 {
                        // format_bytes would say 0 MB
                        tr!("ui-options-chat-logs-import-files", count = folder.files).to_owned()
                    } else {
                        format!(
                            "{} · {}",
                            tr!("ui-options-chat-logs-import-files", count = folder.files),
                            format_bytes(folder.bytes)
                        )
                    };
                    ui.indent("note", |ui| {
                        ui.label(RichText::new(note).size(11.5).color(p.muted));
                    });
                }
            }
        }
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            let size = Vec2::new(0.0, 26.0);
            let folders: Vec<std::path::PathBuf> = st
                .chat_import
                .found
                .iter()
                .flatten()
                .filter(|(_, ticked)| *ticked)
                .map(|(folder, _)| folder.dir.clone())
                .collect();
            let ready = !folders.is_empty() && st.chat_log_importing.is_none();
            if action_button_if(
                ui,
                p,
                "file-import",
                tr!("ui-options-chat-logs-import-confirm"),
                ActionKind::Primary,
                size,
                ready,
            )
            .clicked()
            {
                r.import_legacy_logs = Some(folders);
            }
            if action_button(
                ui,
                p,
                "folder-open-filled",
                tr!("ui-options-chat-logs-import-choose"),
                ActionKind::Flat,
                size,
            )
            .clicked()
            {
                r.folder = Some((Folder::LegacyLogs, FolderChoice::Pick));
            }
            if action_button(ui, p, "x", prototype_i18n::tr!("common-cancel"), ActionKind::Flat, size).clicked() {
                st.chat_import.open = false;
            }
        });
    });
    if modal.should_close() {
        st.chat_import.open = false;
    }
}

/// The window's body: the sidebar, the page (or the search, or the
/// modified settings), the footer, the undo of a reset.
fn body(ui: &mut egui::Ui, p: &Palette, s: &mut Settings, d: &Settings, st: &mut OptionsUi, r: &mut OptionsResult, close: &mut bool) {
    // under the title bar (the body is flush with the window's edges)
    let full = ui.available_rect_before_wrap();
    let now = ui.input(|i| i.time);
    // the rows behind a cog shown by "Go to"
    if let Some(parent) = st.open_parent.take() {
        st.open_subs.insert(parent);
    }
    if let Some((page, tab, row)) = st.go_to.take() {
        st.open(page, tab);
        st.flash = row.map(|id| (id, now));
        st.flash_scroll = row.is_some();
    }
    if std::mem::take(&mut st.demo_reset) {
        reset_page(ui, p, s, d, st, r, st.page, now);
    }
    match st.reset_confirmed.take() {
        Some(ResetAsk::Page(page)) => reset_page(ui, p, s, d, st, r, page, now),
        Some(ResetAsk::Modified) => reset_modified(ui, p, s, d, st, r, now),
        Some(ResetAsk::All) => reset_all(s, st, r, now),
        None => {}
    }
    // a row asked by its label (demo, another window): its page first,
    // then `Cx` opens its cog and lights it
    if let Some((page, tab, _)) = st.reveal
        && (st.page != page || st.tab != tab || st.modified_view || !st.query.is_empty())
    {
        let reveal = st.reveal.take();
        st.open(page, tab);
        st.reveal = reveal;
    }
    let input = interaction(ui);
    if count_due(st, now, input) {
        count(ui, p, s, d, st, r);
        st.counted = Some((now, st.query.clone()));
    }
    st.demo_open_cogs = false;

    let compact = full.width() < 840.0;
    let side_w = if compact { compact_sidebar(full.width()) } else { SIDEBAR_W };
    let radius = f32::from(ui.visuals().window_corner_radius.nw);
    finish::glass_wash(ui.painter(), p, full.shrink2(vec2(2.0, 0.0)), radius);
    let footer = Rect::from_min_max(
        pos2(full.left() + 2.0, full.bottom() - FOOTER_H),
        // clear of the gold frame's bottom ornament
        full.right_bottom() - vec2(2.0, 12.0),
    );
    let side = Rect::from_min_max(
        pos2(full.left() + 11.0, full.top() + 5.0),
        pos2(full.left() + 11.0 + side_w, footer.top() - 5.0),
    );
    let main = Rect::from_min_max(pos2(side.right() + 6.0, full.top()), pos2(full.right() - 11.0, footer.top()));

    // Dragon Viewer: the skin's ornaments, faint, behind the page
    if !p.high_contrast {
        let painter = ui.painter();
        let dragon = Rect::from_min_size(pos2(main.right() - main.width() * 0.78, main.bottom() - 215.0), vec2(main.width() * 0.8, 100.0));
        super::dragon_art::paint_fill(painter, "gold-dragon", dragon, egui::Color32::from_white_alpha(70));
        if let Some(t) = super::dragon_art::texture(ui.ctx(), "cloud-scroll") {
            let [w, h] = t.size();
            let size = vec2(192.0, 192.0 * h as f32 / w.max(1) as f32);
            let at = Rect::from_min_size(pos2(main.left() - 9.0, main.bottom() - size.y - 1.0), size);
            painter.image(t.id(), at, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), egui::Color32::from_white_alpha(87));
        }
        let title_flare = Rect::from_center_size(pos2(full.center().x, full.top() + 1.0), vec2(full.width() * 0.7, 26.0));
        super::dragon_art::paint_fill(painter, "gold-flare", title_flare, egui::Color32::from_white_alpha(200));
    }
    sidebar(ui, p, s, d, st, r, side, now);
    let searching = !st.query.trim().is_empty();
    let content_rect = page_area(ui, p, s, d, st, r, main, searching, compact);
    footer_bar(ui, p, s, d, st, r, footer, searching, close, now);
    undo_bar(ui, p, s, st, r, content_rect);
    st.count_stale = input || r.changed;
}

fn compact_sidebar(width: f32) -> f32 {
    (width * 0.25).clamp(125.0, 177.0)
}

#[allow(clippy::too_many_arguments)]
fn sidebar(
    ui: &mut egui::Ui,
    p: &Palette,
    s: &mut Settings,
    d: &Settings,
    st: &mut OptionsUi,
    r: &mut OptionsResult,
    side: Rect,
    now: f64,
) {
    let _ = (s, d, r, now);
    let compact = side.width() < SIDEBAR_W;
    ui.painter().vline(
        side.right(),
        side.y_range(),
        egui::Stroke::new(1.0, p.violet_pale.gamma_multiply(0.34)),
    );
    let mut ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(side.shrink2(vec2(4.0, 0.0)))
            .layout(egui::Layout::top_down(egui::Align::Min)),
    );
    ui.add_space(3.0);
    let searching = !st.query.trim().is_empty();
    let phases_h = if p.high_contrast { 0.0 } else { 22.0 };
    let list_h = (side.bottom() - ui.cursor().top() - 4.0 - phases_h).max(0.0);
    egui::ScrollArea::vertical()
        .id_salt("preferences-pages")
        .auto_shrink([false, false])
        .max_height(list_h)
        .min_scrolled_height(list_h)
        .show(&mut ui, |ui| {
            ui.spacing_mut().item_spacing.y = 5.0;
            // Prototype, then the modified settings, above the sections
            let on = !searching && !st.modified_view && st.page == Page::Prototype;
            let dim = searching && st.counts.hits(Page::Prototype) == 0;
            let entry = ui
                .scope(|ui| {
                    if dim {
                        ui.multiply_opacity(0.4);
                    }
                    let chip = nav_chip(st, Page::Prototype, searching);
                    let room = chip_room(ui, chip);
                    let entry = navigation(ui, p, Page::Prototype.icon(), Page::Prototype.name(), on, room);
                    paint_chip(ui, p, entry.rect, chip, searching);
                    entry
                })
                .inner;
            if entry.clicked() {
                st.open(Page::Prototype, 0);
            }
            let total = st.counts.total_modified();
            let on = !searching && st.modified_view;
            let chip = (total > 0).then_some(total);
            let entry = navigation(
                ui,
                p,
                "pencil-filled",
                prototype_i18n::tr!("ui-options-modified-settings"),
                on,
                chip_room(ui, chip),
            );
            paint_chip(ui, p, entry.rect, chip, false);
            if entry.clicked() {
                st.query.clear();
                st.modified_view = true;
                st.cancel_capture();
            }
            for page in Page::ALL.into_iter().skip(1) {
                if !compact && let Some(title) = page.section() {
                    ui.add_space(10.0);
                    let font = egui::FontId::new(11.0, super::fonts::semibold(ui.ctx()));
                    let galley = ui.painter().layout_no_wrap(title.to_uppercase(), font, p.violet_light);
                    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 20.0), Sense::hover());
                    super::widgets::paint_row_text(ui.painter(), galley, rect.left() + 8.0, rect.center().y, p.violet_light);
                }
                let on = !searching && !st.modified_view && st.page == page;
                let dim = searching && st.counts.hits(page) == 0;
                let entry = ui
                    .scope(|ui| {
                        if dim {
                            ui.multiply_opacity(0.4);
                        }
                        let chip = nav_chip(st, page, searching);
                        let marked = page.experimental();
                        let room = chip_room(ui, chip) + if marked { NAV_MARK + 8.0 } else { 0.0 };
                        let entry = navigation(ui, p, page.icon(), page.name(), on, room);
                        paint_chip(ui, p, entry.rect, chip, searching);
                        if marked {
                            paint_nav_mark(ui, p, entry.rect, chip, page);
                        }
                        entry
                    })
                    .inner;
                if entry.clicked() {
                    st.open(page, 0);
                }
            }
            ui.add_space(8.0);
        });
    if phases_h > 0.0 {
        finish::moon_phases(ui.painter(), p, side.center_bottom() - vec2(0.0, phases_h * 0.5));
    }
}

fn navigation(ui: &mut egui::Ui, p: &Palette, icon: &str, label: &str, selected: bool, room: f32) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(vec2((ui.available_width() - 4.0).max(0.0), 28.0), Sense::click());
    let ink = super::widgets::glass_pill(ui.painter(), p, rect, selected, response.hovered() || response.has_focus());
    widgets::paint_glyph(
        ui.painter(),
        icon,
        rect.left_center() + vec2(15.0, 0.0),
        15.0,
        if selected { ink } else { p.amber },
    );
    let galley = egui::WidgetText::from(label).into_galley(
        ui,
        Some(egui::TextWrapMode::Truncate),
        (rect.width() - 31.0 - room).max(1.0),
        egui::FontId::proportional(13.0),
    );
    widgets::paint_row_text(ui.painter(), galley, rect.left() + 28.0, rect.center().y, ink);
    response.widget_info(|| egui::WidgetInfo::selected(egui::Role::Button, true, selected, label));
    response.on_hover_cursor(egui::CursorIcon::PointingHand).on_hover_text(label)
}

/// The number at the right of a page's entry: its search hits while
/// searching, its modified settings otherwise.
fn nav_chip(st: &OptionsUi, page: Page, searching: bool) -> Option<usize> {
    let n = if searching {
        st.counts.hits(page)
    } else {
        st.counts.modified(page)
    };
    (n > 0).then_some(n)
}

/// The size of the experimental mark of a sidebar entry.
const NAV_MARK: f32 = 14.0;

/// The experimental mark at the right end of a page's sidebar entry, left
/// of its count.
fn paint_nav_mark(ui: &egui::Ui, p: &Palette, entry: Rect, chip: Option<usize>, page: Page) {
    let chip_w = chip.map_or(0.0, |n| chip_width(&chip_galley(ui, n, egui::Color32::PLACEHOLDER)) + 6.0);
    let center = pos2(entry.right() - 8.0 - chip_w - NAV_MARK / 2.0, entry.center().y);
    super::widgets::experimental_mark(
        ui,
        p,
        Rect::from_center_size(center, vec2(NAV_MARK, NAV_MARK)),
        egui::Id::unique(("preferences-experimental-page", page)),
    );
}

fn chip_galley(ui: &egui::Ui, n: usize, ink: egui::Color32) -> std::sync::Arc<egui::Galley> {
    let font = egui::FontId::new(10.5, super::fonts::semibold(ui.ctx()));
    ui.painter().layout_no_wrap(n.to_string(), font, ink)
}

fn chip_width(galley: &egui::Galley) -> f32 {
    (galley.size().x + 10.0).max(18.0)
}

/// What a sidebar entry's label leaves at its right for `paint_chip`: the
/// pill, its margin and a gap.
fn chip_room(ui: &egui::Ui, n: Option<usize>) -> f32 {
    n.map_or(0.0, |n| chip_width(&chip_galley(ui, n, egui::Color32::PLACEHOLDER)) + 12.0)
}

/// A count in a pill at the right end of a sidebar entry: violet for the
/// modified settings, teal for the search's hits.
fn paint_chip(ui: &egui::Ui, p: &Palette, entry: Rect, n: Option<usize>, hit: bool) {
    let Some(n) = n else { return };
    let (ink, face) = if hit {
        (p.bar, p.teal)
    } else {
        (p.violet_pale, p.violet.gamma_multiply(0.24))
    };
    let galley = chip_galley(ui, n, ink);
    let w = chip_width(&galley);
    let pill = Rect::from_center_size(pos2(entry.right() - 8.0 - w / 2.0, entry.center().y), vec2(w, 17.0));
    ui.painter().rect_filled(pill, ui.visuals().window_corner_radius, face);
    super::widgets::paint_ink_centered(ui.painter(), galley, pill.center(), ink);
}

/// Dragon Viewer's jewel button: the skin's gold (or red) face and a bold
/// outlined label.
pub(crate) fn jewel_button(ui: &mut egui::Ui, p: &Palette, label: &str, size: Vec2, red: bool) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let hot = resp.hovered() || resp.has_focus();
    if p.high_contrast {
        let ink = super::widgets::glass_pill(ui.painter(), p, rect, !red, hot);
        let galley = ui.painter().layout_no_wrap(label.to_owned(), egui::FontId::proportional(15.0), ink);
        super::widgets::paint_ink_centered(ui.painter(), galley, rect.center(), ink);
    } else {
        super::dragon_art::jewel(ui.painter(), rect, red, hot, resp.is_pointer_button_down_on());
        let (ink, edge) = if red {
            (egui::Color32::WHITE, egui::Color32::from_rgb(90, 6, 0))
        } else {
            (egui::Color32::from_rgb(255, 255, 250), egui::Color32::from_rgb(120, 50, 0))
        };
        super::dragon_art::outlined_text(ui.painter(), rect.center(), label, (size.y * 0.5).min(22.0), ink, edge);
    }
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, ui.is_enabled(), label));
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

fn pill_button(ui: &mut egui::Ui, p: &Palette, icon: Option<&str>, label: &str, size: Vec2, main: bool) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let ink = super::widgets::glass_pill(ui.painter(), p, rect, main, resp.hovered() || resp.has_focus());
    let lead = if icon.is_some() { 20.0 } else { 0.0 };
    let galley = egui::WidgetText::from(label).into_galley(
        ui,
        Some(egui::TextWrapMode::Truncate),
        (rect.width() - 12.0 - lead).max(1.0),
        egui::FontId::proportional(13.0),
    );
    if let Some(icon) = icon {
        if !main && !p.high_contrast {
            ui.painter().rect_stroke(
                rect,
                rect.height() * 0.5,
                egui::Stroke::new(1.0, p.amber.gamma_multiply(0.76)),
                egui::StrokeKind::Inside,
            );
        }
        let left = rect.center().x - (galley.size().x + lead) * 0.5;
        let center = pos2(left + 7.0, rect.center().y);
        if main && !p.high_contrast {
            super::prototype_bg::radial_glow(ui.painter(), center, 14.0, p.amber, 0.64);
            widgets::paint_glyph(ui.painter(), icon, center + vec2(0.0, 1.0), 15.0, p.indigo);
        }
        let gold = if main { p.amber.lerp_to_gamma(p.ink, 0.55) } else { p.amber };
        widgets::paint_glyph(ui.painter(), icon, center, 14.0, if p.high_contrast { ink } else { gold });
    }
    if p.high_contrast {
        super::widgets::paint_ink_centered(ui.painter(), galley, rect.center() + vec2(lead * 0.5, 0.0), ink);
    } else {
        // Dragon Viewer: the OK / Cancel label style on every footer button
        let _ = galley;
        super::dragon_art::outlined_text(ui.painter(), rect.center() + vec2(lead * 0.5, 0.0), label, 14.0, egui::Color32::WHITE, egui::Color32::from_rgb(40, 16, 4));
    }
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, ui.is_enabled(), label));
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

fn pill_tabs(ui: &mut egui::Ui, p: &Palette, tab: &mut usize, labels: &[&str], compact: bool) -> bool {
    let mut changed = false;
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = vec2(5.0, 5.0);
        let max_width = ui.available_width();
        for (i, label) in labels.iter().enumerate() {
            let on = *tab == i;
            let font = egui::FontId::proportional(13.0);
            let pad = if compact { 18.0 } else { 25.0 };
            let galley =
                egui::WidgetText::from(*label).into_galley(ui, Some(egui::TextWrapMode::Truncate), (max_width - pad).max(1.0), font);
            let size = vec2(galley.size().x + pad, 27.0);
            let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
            let ink = super::widgets::glass_pill(ui.painter(), p, rect, on, resp.hovered() || resp.has_focus());
            super::widgets::paint_ink_centered(ui.painter(), galley, rect.center(), ink);
            resp.widget_info(|| egui::WidgetInfo::selected(egui::Role::Button, ui.is_enabled(), on, *label));
            if resp.on_hover_cursor(egui::CursorIcon::PointingHand).on_hover_text(*label).clicked() && !on {
                *tab = i;
                changed = true;
            }
        }
    });
    changed
}

/// The head of the page, its tabs and its rows (or the search's, or the
/// modified ones). Returns the rect of the rows.
#[allow(clippy::too_many_arguments)]
fn page_area(
    ui: &mut egui::Ui,
    p: &Palette,
    s: &mut Settings,
    d: &Settings,
    st: &mut OptionsUi,
    r: &mut OptionsResult,
    main: Rect,
    searching: bool,
    compact: bool,
) -> Rect {
    let mut ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(main.shrink2(vec2(if compact { 8.0 } else { 24.0 }, 0.0)))
            .layout(egui::Layout::top_down(egui::Align::Min)),
    );
    let page = st.page;
    let tabs = page.tabs();
    st.tab = st.tab.min(tabs.len().saturating_sub(1));
    let modified_view = !searching && st.modified_view;
    let (title, about) = if searching {
        let hits = st.counts.hits.iter().map(|(_, n)| n).sum::<usize>();
        let pages = st.counts.hits.len();
        (
            prototype_i18n::tr!("ui-options-search-title").to_owned(),
            if hits == 0 {
                String::new()
            } else {
                prototype_i18n::tr!(
                    "ui-options-search-found",
                    count = hits,
                    pages = pages,
                    query = st.query.trim().to_owned()
                )
            },
        )
    } else if modified_view {
        let n = st.counts.total_modified();
        (
            prototype_i18n::tr!("ui-options-modified-settings").to_owned(),
            if n == 0 {
                prototype_i18n::tr!("ui-options-modified-none").to_owned()
            } else {
                prototype_i18n::tr!("ui-options-modified-count", count = n)
            },
        )
    } else {
        (page.name().to_owned(), page.about().to_owned())
    };

    // the head: title, its line, and the page's own buttons at the right
    ui.add_space(if compact { 5.0 } else { 14.0 });
    let head_w = ui.available_width();
    ui.horizontal_top(|ui| {
        ui.set_width(head_w);
        let buttons_w = if !searching && !modified_view && page == Page::Prototype {
            if compact { 76.0 } else { 380.0 }
        } else if modified_view && st.counts.total_modified() > 0 {
            if compact { 38.0 } else { 200.0 }
        } else {
            0.0
        };
        ui.vertical(|ui| {
            ui.set_width((head_w - buttons_w - 12.0).max(120.0));
            let font = egui::FontId::new(if compact { 17.0 } else { 24.0 }, super::fonts::semibold(ui.ctx()));
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = 14.0;
                let title = ui.label(RichText::new(&title).font(font).color(p.ink));
                if compact && !about.is_empty() && !searching && !modified_view {
                    title.on_hover_text(&about);
                } else if !about.is_empty() {
                    ui.add(egui::Label::new(RichText::new(&about).size(14.0).color(p.muted)).wrap());
                }
            });
        });
        if buttons_w > 0.0 {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                ui.spacing_mut().item_spacing.x = 6.0;
                use super::widgets::{ActionKind, action_button};
                let size = Vec2::new(0.0, 28.0);
                let action = |ui: &mut egui::Ui, icon, label, kind| {
                    action_button(ui, p, icon, if compact { "" } else { label }, kind, size).on_hover_text(label)
                };
                if modified_view {
                    if action(
                        ui,
                        "restore",
                        prototype_i18n::tr!("ui-options-modified-reset-all"),
                        ActionKind::Flat,
                    )
                    .clicked()
                    {
                        st.confirm_reset = Some(ResetAsk::Modified);
                    }
                } else {
                    let legacy = action(
                        ui,
                        "history",
                        prototype_i18n::tr!("ui-options-prototype-all-firestorm"),
                        ActionKind::Flat,
                    );
                    let prototype = action(
                        ui,
                        "sparkles-filled",
                        prototype_i18n::tr!("ui-options-prototype-all-prototype"),
                        ActionKind::Primary,
                    );
                    if prototype.clicked() {
                        pages::prototype::set_all(s, pages::prototype::Look::Prototype);
                        r.changed = true;
                    }
                    if legacy.clicked() {
                        pages::prototype::set_all(s, pages::prototype::Look::Legacy);
                        r.changed = true;
                    }
                }
            });
        }
    });
    ui.add_space(if compact { 4.0 } else { 10.0 });
    if !searching && !modified_view && tabs.len() > 1 {
        if pill_tabs(&mut ui, p, &mut st.tab, &tabs, compact) {
            st.cancel_capture();
            st.flash = None;
        }
        ui.add_space(6.0);
    } else {
        let (line, _) = ui.allocate_exact_size(vec2(ui.available_width(), 1.0), Sense::hover());
        ui.painter()
            .hline(line.x_range(), line.center().y, super::widgets::divider_stroke(p));
    }

    // The scrollable rows sit on a lightly shaded glass inset.
    let top = ui.cursor().top();
    let card = Rect::from_min_max(pos2(main.left(), top), pos2(main.right(), main.bottom() - 4.0));
    let radius = ui.visuals().window_corner_radius;
    ui.painter().rect_filled(card, radius, p.navy.gamma_multiply(0.29));
    ui.painter().rect_stroke(
        card,
        radius,
        egui::Stroke::new(1.0, p.indigo_light.gamma_multiply(0.32)),
        egui::StrokeKind::Inside,
    );
    ui.add_space(if compact { 4.0 } else { 10.0 });
    let top = ui.cursor().top();
    let rows_h = (main.bottom() - 14.0 - top).max(0.0);
    let rows = Rect::from_min_max(pos2(main.left(), top), main.right_bottom());
    let mode = if searching {
        Mode::Search
    } else if modified_view {
        Mode::Modified
    } else {
        Mode::Show
    };
    let scroll_id = match mode {
        Mode::Search => egui::Id::unique("preferences-search"),
        Mode::Modified => egui::Id::unique("preferences-modified"),
        _ => egui::Id::unique(("preferences-page", page, st.tab)),
    };
    let query = st.query.clone();
    let tab = st.tab;
    // Reserve the full scrollbar strip before entering the scroll area.
    // Its animated appearance otherwise changes the cells' width, invalidates
    // their control measurements and can alternate short/tall rows forever.
    let rows_width = (super::widgets::scrolled_width(&ui) - 6.0).max(0.0);
    egui::ScrollArea::vertical()
        .id_salt(scroll_id)
        .auto_shrink([false, false])
        .max_height(rows_h)
        .min_scrolled_height(rows_h)
        .show(&mut ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            ui.set_width(rows_width);
            ui.add_space(4.0);
            let mut cx = Cx::new(ui, *p, s, d, st, r, mode, &query);
            match mode {
                Mode::Show => run_page(&mut cx, page, Some(tab)),
                _ => {
                    for each in Page::ALL {
                        run_page(&mut cx, each, None);
                    }
                }
            }
            let changed = cx.changed;
            drop(cx);
            r.changed |= changed;
            let empty = match mode {
                Mode::Search => st
                    .counts
                    .hits
                    .is_empty()
                    .then(|| prototype_i18n::tr!("ui-options-search-none", query = query.trim().to_owned())),
                Mode::Modified => (st.counts.total_modified() == 0).then(|| prototype_i18n::tr!("ui-options-modified-empty").to_owned()),
                _ => None,
            };
            if let Some(text) = empty {
                ui.add_space(60.0);
                ui.vertical_centered(|ui| {
                    ui.add(egui::Label::new(RichText::new(text).size(13.0).color(p.muted)).wrap());
                });
            }
            ui.add_space(20.0);
        });
    rows
}

#[allow(clippy::too_many_arguments)]
fn footer_bar(
    ui: &mut egui::Ui,
    p: &Palette,
    s: &mut Settings,
    d: &Settings,
    st: &mut OptionsUi,
    r: &mut OptionsResult,
    footer: Rect,
    searching: bool,
    close: &mut bool,
    now: f64,
) {
    ui.painter().hline(
        footer.shrink2(vec2(9.0, 0.0)).x_range(),
        footer.top(),
        egui::Stroke::new(1.0, p.amber.gamma_multiply(0.37)),
    );
    let mut ui = ui.new_child(
        egui::UiBuilder::new()
            // inset past the frame's corner ornaments
            .max_rect(footer.shrink2(vec2(34.0, 0.0)))
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );
    ui.spacing_mut().item_spacing.x = 8.0;
    let reset_width = |ui: &egui::Ui, label: &str| {
        let text = ui
            .painter()
            .layout_no_wrap(label.to_owned(), egui::FontId::proportional(13.0), p.ink);
        (text.size().x + 20.0).min((footer.width() - 320.0).max(100.0) * 0.45)
    };
    let label = prototype_i18n::tr!("ui-options-reset-everything-2");
    let width = reset_width(&ui, label);
    let reset_all =
        pill_button(&mut ui, p, None, label, vec2(width, 31.0), false).on_hover_text(prototype_i18n::tr!("ui-options-reset-all-settings"));
    if reset_all.clicked() {
        st.confirm_reset = Some(ResetAsk::All);
    }
    if !searching && !st.modified_view {
        let label = prototype_i18n::tr!("ui-options-reset-page", page = st.page.name());
        let width = reset_width(&ui, &label);
        let b = pill_button(&mut ui, p, None, &label, vec2(width, 31.0), false)
            .on_hover_text(prototype_i18n::tr!("ui-options-reset-section-help", section = st.page.name()));
        if b.clicked() {
            st.confirm_reset = Some(ResetAsk::Page(st.page));
        }
    }
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        // Changes preview live. Apply commits a new Cancel checkpoint.
        // Dragon Viewer: the gold OK and the red Cancel jewels at the right
        let cancel = prototype_i18n::tr!("common-cancel");
        if jewel_button(ui, p, cancel, vec2(132.0, 40.0), true).on_hover_text(cancel).clicked() {
            if let Some(before) = st.opened_with.take() {
                *s = *before;
                r.changed = true;
            }
            *close = true;
        }
        let ok_label = prototype_i18n::tr!("common-ok");
        if jewel_button(ui, p, ok_label, vec2(132.0, 40.0), false).on_hover_text(ok_label).clicked() {
            *close = true;
        }
        let size = vec2(88.0, 31.0);
        let apply = prototype_i18n::tr!("common-apply");
        if pill_button(ui, p, Some("check"), apply, size, false).on_hover_text(apply).clicked() {
            st.finish_undo(r);
            st.opened_with = Some(Box::new(s.clone()));
            r.save = true;
        }
        ui.add_space(6.0);
        // the hint gives way to the buttons when they leave it no room
        let hint = ui.painter().layout_no_wrap(
            prototype_i18n::tr!("ui-options-changes-apply-immediately").to_owned(),
            egui::FontId::proportional(11.5),
            p.muted,
        );
        if hint.size().x <= ui.available_width() {
            ui.label(hint);
        }
    });
}

/// "Reset …?": the reset waits for its "Reset" here, and is done by the
/// next pass of the window's body, which has the rows to run.
fn reset_modal(ctx: &egui::Context, p: &Palette, st: &mut OptionsUi, open: bool) {
    let Some(ask) = st.confirm_reset.filter(|_| open) else {
        st.confirm_reset = None;
        return;
    };
    let (title, text): (String, String) = match ask {
        ResetAsk::Page(page) => (
            prototype_i18n::tr!("ui-options-reset-section-title", section = page.name()),
            prototype_i18n::tr!("ui-options-reset-section-text").to_string(),
        ),
        ResetAsk::Modified => (
            prototype_i18n::tr!("ui-options-modified-reset-title").to_string(),
            prototype_i18n::tr!("ui-options-modified-reset-text").to_string(),
        ),
        ResetAsk::All => (
            prototype_i18n::tr!("ui-options-reset-everything").to_string(),
            prototype_i18n::tr!("ui-options-reset-everything-text").to_string(),
        ),
    };
    let modal = crate::ui::widgets::Modal::new(egui::Id::unique("options_reset_confirm")).show(ctx, |ui| {
        ui.set_width(360.0);
        ui.label(RichText::new(title).size(15.0).strong().color(p.ink));
        ui.add_space(6.0);
        ui.label(RichText::new(text).size(12.5).color(p.muted));
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            use super::widgets::{ActionKind, action_button};
            let size = Vec2::new(0.0, 26.0);
            if action_button(
                ui,
                p,
                "restore",
                prototype_i18n::tr!("ui-options-reset-confirm"),
                ActionKind::Danger,
                size,
            )
            .clicked()
            {
                st.reset_confirmed = Some(ask);
                st.confirm_reset = None;
            }
            if action_button(ui, p, "x", prototype_i18n::tr!("common-cancel"), ActionKind::Flat, size).clicked() {
                st.confirm_reset = None;
            }
        });
    });
    if modal.should_close() {
        st.confirm_reset = None;
    }
    if st.reset_confirmed.is_some() {
        ctx.request_repaint();
    }
}

/// Every setting back to its default but the account's, with an undo.
fn reset_all(s: &mut Settings, st: &mut OptionsUi, r: &mut OptionsResult, now: f64) {
    let before = Box::new(s.clone());
    s.reset_keeping_account();
    for page in Page::ALL {
        st.after_reset(page, s, r);
    }
    st.finish_undo(r);
    st.undo = Some(Undo {
        before,
        text: prototype_i18n::tr!("ui-options-all-were-reset").to_owned(),
        shown: now,
        done: Done::All,
    });
}

/// The settings that differ from their defaults back to them, with an undo.
fn reset_modified(ui: &mut egui::Ui, p: &Palette, s: &mut Settings, d: &Settings, st: &mut OptionsUi, r: &mut OptionsResult, now: f64) {
    let before = Box::new(s.clone());
    reset_rows(ui, p, s, d, st, r, None);
    for page in Page::ALL {
        st.after_reset(page, s, r);
    }
    st.finish_undo(r);
    st.undo = Some(Undo {
        before,
        text: prototype_i18n::tr!("ui-options-modified-were-reset").to_owned(),
        shown: now,
        done: Done::Modified,
    });
}

/// A page back to its defaults, with an undo.
#[allow(clippy::too_many_arguments)]
fn reset_page(
    ui: &mut egui::Ui,
    p: &Palette,
    s: &mut Settings,
    d: &Settings,
    st: &mut OptionsUi,
    r: &mut OptionsResult,
    page: Page,
    now: f64,
) {
    let before = Box::new(s.clone());
    reset_rows(ui, p, s, d, st, r, Some(page));
    st.after_reset(page, s, r);
    st.finish_undo(r);
    st.undo = Some(Undo {
        before,
        text: prototype_i18n::tr!("ui-options-page-was-reset", page = page.name()),
        shown: now,
        done: Done::Page(page),
    });
}

/// "Graphics: default settings · Undo", over the bottom of the rows.
fn undo_bar(ui: &mut egui::Ui, p: &Palette, s: &mut Settings, st: &mut OptionsUi, r: &mut OptionsResult, area: Rect) {
    let Some(undo) = &st.undo else { return };
    let text = undo.text.clone();
    let galley = ui.painter().layout_no_wrap(text.clone(), egui::FontId::proportional(12.5), p.ink);
    let w = (galley.size().x + 130.0).min(area.width() - 40.0);
    let bar = Rect::from_center_size(pos2(area.center().x, area.bottom() - 30.0), vec2(w, 36.0));
    ui.painter().rect_filled(bar, ui.visuals().window_corner_radius, p.raised);
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(bar.shrink2(Vec2::new(14.0, 0.0)))
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );
    child.label(RichText::new(text).size(12.5).color(p.ink));
    child.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        let label = RichText::new(prototype_i18n::tr!("money-tracker-undo"))
            .size(12.5)
            .strong()
            .color(p.violet_pale);
        if ui
            .add(egui::Label::new(label).sense(Sense::click()))
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .clicked()
            && let Some(undo) = st.undo.take()
        {
            *s = *undo.before;
            for page in Page::ALL {
                st.after_reset(page, s, r);
            }
        }
    });
    ui.ctx().request_repaint_after(std::time::Duration::from_millis(500));
}

#[cfg(test)]
mod tests;
