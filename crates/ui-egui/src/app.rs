use super::services::Services;
use super::state::{PromptSave, keeps_everything};
use super::{
    DEFAULT_TAB, MAX_DICTIONARY, MAX_RECENT, SaveInFlight, Session, Status, Value, WordApp, applied_font, canvas, chrome, crop, dialogs, documents,
    file_dialogs, i18n, json, panes, paste_picture, push_recent_font, recent_fonts_from, ribbon, system_clipboard, table_pen, theme,
};
use craft::platform::clock::monotonic_ms;
use craft::ui::runtime::commands::CommandInput;
use craft::ui::runtime::env::{AltCodePages, ResolvedRoot, RootEnv, WheelSpeed};
use craft::ui::runtime::frame::{FrameInput, FrameOutput};
use craft::ui::runtime::windows::WindowHost;

pub(crate) mod command_board;
mod commands;
mod control;
mod files;
pub(crate) use files::Closing;
pub(crate) mod fonts;
mod frame;
mod input;
mod pictures;
mod prefs;

#[cfg(test)]
pub(crate) use commands::works_without_document;

/// Paper's own sizes over the size set. The display role is the File pages' titles (Info, Open, Print,
/// Options…), production's 26 pt semibold over its 15 pt section headings; the suite's display size (38 pt) is a
/// start screen's brand title, which Paper does not show.
static APP_SIZES: std::sync::LazyLock<std::sync::Arc<craft::theme::AppSizes>> = std::sync::LazyLock::new(|| {
    use craft::theme::SizeValue;
    let all = [("typography.display.size", 26.0), ("typography.display.line_height", 32.0)]
        .into_iter()
        .map(|(path, value)| (path.to_owned(), SizeValue::Number(value)))
        .collect();
    std::sync::Arc::new(craft::theme::AppSizes { all, ..Default::default() })
});

/// Paper's black-and-white theme (`themes/paper.json`), parsed once. `None` only if the embedded
/// file is invalid, which `paper_theme_parses` rules out; the library default then applies.
static PAPER_THEME: std::sync::LazyLock<Option<std::sync::Arc<craft::theme::ThemeDef>>> =
    std::sync::LazyLock::new(|| craft::theme::format::parse(include_bytes!("../themes/paper.json")).ok().map(|file| std::sync::Arc::new(file.theme)));

impl WordApp {
    /// Creates the app: adopts the system locale, then initializes the document session, UI state,
    /// services, canvas, dialogs, previews and font tracking.
    pub fn new(session: Session, services: Services) -> Self {
        // The app's own icons (ribbon, menus) sit beside the framework's catalog; registering twice is a no-op.
        crate::icons::register();
        // Numeral › System follows the system locale, found here rather than in layout.
        if let Some(tag) = sys_locale::get_locale() {
            paper_engine::set_system_locale(&tag);
        }
        // What the window shows starts as a new window's.
        let canvas = Default::default();
        let restrict = Default::default();
        let spelling_review = None;
        let status = None;
        let announcement = None;
        let previews = Default::default();
        let change_picture_target = None;
        let recipient_list_pending = false;
        let ribbon_dropped = false;
        let nav_search_focus = false;
        let escape_free = false;
        let status_counts = documents::NOT_COUNTED;
        let missing_fonts = Default::default();
        let export_error = String::new();
        let equation_prev_tab = None;
        let smart_art_item = None;
        let outline_prev_tab = None;
        let file_dialog = None;
        let info_editing = None;
        let shown_document = session.document_id();
        let backstage_state = Default::default();
        let chrome = Default::default();
        let dialog_host = craft::ui::dialog::DialogHost::new();
        let dialog_hooks = Default::default();
        let dialog_answers = Default::default();
        let edits = Default::default();
        let keytips_active = false;
        let options_state = Default::default();
        let shape_variation = None;
        let equation_symbol_set = Default::default();
        let more_colors = Default::default();
        WordApp {
            session,
            ui: super::prefs::Prefs::default(),
            services,
            host: std::sync::Arc::new(craft::host::Host::builder(super::APP).i18n(i18n::catalogs()).lang(craft::i18n::Lang::EN).build()),
            settings: None,
            settings_notices: Default::default(),
            command_catalog: None,
            command_catalog_mac: cfg!(target_os = "macos"),
            command_catalog_lang: "",
            key_overrides: Default::default(),
            command_states: Default::default(),
            fonts_frames: 0,
            clipboard_published: false,
            packs_rt: Default::default(),
            control: None,
            capture_token: 0,
            native_scale: 1.0,
            auto_scale_limit: f32::INFINITY,
            legacy_interface_size: None,
            applied_editing: None,
            frame_ms: 0.0,
            quit_requested: false,
            ctx: None,
            ime_origin: false,
            user_origin: false,
            saves_in_flight: Vec::new(),
            fallback_chooser: false,
            font_generation: paper_fonts::FontDb::global().generation(),
            no_document: None,
            parked: Default::default(),
            closing: None,
            canvas,
            restrict,
            spelling_review,
            status,
            announcement,
            previews,
            change_picture_target,
            recipient_list_pending,
            ribbon_dropped,
            nav_search_focus,
            escape_free,
            status_counts,
            missing_fonts,
            export_error,
            equation_prev_tab,
            smart_art_item,
            outline_prev_tab,
            file_dialog,
            info_editing,
            shown_document,
            backstage_state,
            chrome,
            dialog_host,
            dialog_hooks,
            dialog_answers,
            edits,
            keytips_active,
            options_state,
            shape_variation,
            equation_symbol_set,
            more_colors,
        }
    }

    /// The app with `host` as its services: files, jobs, settings sections and the language.
    pub fn with_host(mut self, host: std::sync::Arc<craft::host::Host>) -> Self {
        self.host = host;
        self
    }

    /// Whether saving is a download: the browser keeps the files, so there is no path to write to.
    pub(crate) fn browser_saves(&self) -> bool {
        self.host.files().browser_saves()
    }

    /// This frame's root values: the wheel speed, the Alt-code pages, the interface mode,
    /// motion, scale and direction. Built from the saved settings every frame, so a change
    /// applies at once.
    pub fn root_env(&self) -> RootEnv {
        let lang = i18n::Lang::from_pref(&self.ui.language);
        let direction = if lang.rtl() { craft::i18n::Direction::Rtl } else { craft::i18n::Direction::Ltr };
        let locale = sys_locale::get_locale();
        #[cfg(windows)]
        let (ansi, oem) = craft::codepage::windows_code_pages(locale.as_deref());
        #[cfg(not(windows))]
        let (ansi, oem) = (1252, paper_engine::cmd::text::oem_code_page(locale.as_deref()));
        let line = self.ui.scroll_speed.wheel_factor();
        #[cfg(not(target_arch = "wasm32"))]
        let point = self.ui.touchpad_scroll_speed.touchpad_factor();
        #[cfg(target_arch = "wasm32")]
        let point = self.ui.scroll_speed.wheel_factor();
        let mut env = RootEnv::new()
            .scale(self.ui.interface_size.scale(self.native_scale, self.auto_scale_limit))
            .direction(direction)
            .mode(self.ui.theme.mode())
            .wheel_speed(WheelSpeed::new(line, point))
            .alt_code_pages(AltCodePages::new(ansi, oem));
        env.motion = self.ui.animations.motion();
        env.sizes = std::sync::Arc::clone(&APP_SIZES);
        if let Some(theme) = PAPER_THEME.as_ref() {
            env = env.library_theme(std::sync::Arc::clone(theme));
        }
        env
    }

    /// What this frame's mode and motion choices resolve to: the system's answer replaces `System`.
    /// Before the first frame there is no context to ask, and the system reports nothing.
    pub(crate) fn resolved_root(&self) -> ResolvedRoot {
        let root = self.root_env();
        match &self.ctx {
            Some(ctx) => root.resolved(ctx),
            None => root.resolved_for(craft::theme::SystemAppearance::default()),
        }
    }

    /// Whether the interface draws in dark mode, `System` resolved.
    pub(crate) fn ui_is_dark(&self) -> bool {
        self.resolved_root().mode == craft::theme::Mode::Dark
    }

    /// Whether interface animations run now: the choice and the system's reduced-motion setting resolved.
    pub(crate) fn animations_on(&self) -> bool {
        self.resolved_root().motion != craft::behavior::MotionMode::Off
    }

    /// Whether the system allows animations (it does not ask for reduced motion, or reports nothing).
    pub(crate) fn system_animations(&self) -> bool {
        self.ctx.as_ref().is_none_or(|ctx| !craft::ui::runtime::env::system_appearance(ctx).appearance.reduced_motion.unwrap_or(false))
    }

    /// Before each frame's passes, from `embed::before_input`: runs the commands the runtime
    /// collected, binds the interface language, and returns the frame's input (root values,
    /// command states, host). It runs before `logic`, so what needs the frame's own flags
    /// (a late paste, the control channel) is in `logic`.
    pub fn frame_input(&mut self, out: FrameOutput) -> FrameInput {
        if let Some(ctx) = self.ctx.clone() {
            self.run_fired(&ctx, &out);
            self.canvas.take_focus_after_pass(&ctx);
            self.native_scale = theme::usable_scale(ctx.native_pixels_per_point().unwrap_or(1.0));
            if let Some(native) = ctx.native_pixels_per_point()
                && let Some(old) = self.legacy_interface_size.take()
            {
                self.ui.interface_size = theme::migrate_legacy_interface_size(old, native);
            }
            // Auto never draws larger than the smallest screen a window is on allows.
            let host = WindowHost::of(&ctx);
            self.auto_scale_limit =
                theme::auto_scale_limit_over(host.monitor_of(egui::ViewportId::ROOT).into_iter().map(|m| (m.size_px, m.scale)), self.native_scale);
        }
        self.fonts_frames = self.fonts_frames.saturating_add(1).min(2);
        let lang = i18n::Lang::from_pref(&self.ui.language);
        i18n::apply_language(lang);
        if self.host.lang() != lang.craft_lang() {
            self.host = std::sync::Arc::new(self.host.with_lang(lang.craft_lang()));
        }
        let commands: CommandInput = self.command_input();
        FrameInput::new(self.root_env()).commands(commands).host(self.host.clone())
    }

    /// Start-up with no document to open: show File › Home (Blank document, the templates
    /// and Recent), as Word starts, when the Start screen setting is on (the default). No document
    /// is open until the user picks Blank document or a template or opens a file: Esc or
    /// the back arrow shows an empty window with the ribbon greyed out, as Word does. Hosts call
    /// this after [`WordApp::apply_prefs`] when they have nothing to open; [`WordApp::new`] never
    /// does, so tests, `ui_shot` and windows opened for a document start in the editor. Returns
    /// whether the Start screen shows.
    pub fn start_screen(&mut self) -> bool {
        if self.ui.start_screen {
            self.ui.backstage = true;
            self.ui.backstage_page = "home".into();
            self.no_document = Some(self.session.document_id());
            self.session.release_untitled_name();
        }
        self.ui.start_screen
    }

    /// Whether a document is open: false from a Start screen launch until the user creates or
    /// opens one.
    pub fn has_document(&self) -> bool {
        self.no_document != Some(self.session.document_id())
    }

    /// Whether a modal dialog is open on the drawn window: the rest of the window takes no
    /// clicks, keys or shortcuts until it closes. Another window's dialog does not block
    /// this one.
    pub(crate) fn modal_open(&self) -> bool {
        self.dialog_host.shows_modal()
    }

    /// The drawn window's dialog host.
    pub fn dialog_host_mut(&mut self) -> &mut craft::ui::dialog::DialogHost {
        &mut self.dialog_host
    }

    /// Per-frame logic before layout: background work, the control channel, file results.
    pub fn logic(&mut self, ctx: &egui::Context) {
        let lang = i18n::Lang::from_pref(&self.ui.language);
        if self.ctx.is_none() {
            self.ctx = Some(ctx.clone());
        }
        crate::language_packs::tick(self);
        // PDFs and prints show a page reference or a REF to a missing bookmark in the
        // interface's language, as Word prints them in its own.
        paper_engine::set_bookmark_error(i18n::tr(lang, "Error! Bookmark not defined."));
        paper_engine::set_reference_error(i18n::tr(lang, "Error! Reference source not found."));
        // New documents are written in the interface's language when Paper can proof it
        // (a Polish interface: Polish spelling, grammar and hyphenation from the first word).
        // An English interface writes the region's English (English (UK) on a British system,
        // a Spanish one the region's Spanish (Spanish (Uruguay) on a Uruguayan system,
        // Spain's elsewhere).
        let editing = match lang.editing_lang() {
            Some(t) => Some(t.to_string()),
            None if lang.code() == "en" => self.session.region_english(),
            None if lang.code() == "es" => Some(self.session.region_spanish()),
            None => None,
        };
        if self.applied_editing.as_ref().is_none_or(|(l, e)| *l != lang || *e != editing) {
            self.session.set_editing_language(editing.as_deref());
            self.applied_editing = Some((lang, editing));
        }
        self.settle_saves();
        // The active window's own work; with native windows each window does it again as it
        // draws, for itself.
        self.window_work();
        self.drain_inbox();
        self.paste_arrived(ctx);
        // Focus hides the title bar's search box: a search left open there has no box to show it, and would keep
        // Escape from leaving Focus.
        if self.chrome.search && matches!(crate::chrome::Screen::of(&self.session.view), crate::chrome::Screen::Focus) {
            self.chrome.search = false;
        }
        self.pump_close_prompt();
        self.control_logic(ctx);
        self.watch_fonts(ctx);
        // A script, an agent or a later result may have made another document the active one.
        self.sync_document();
        self.report_dirty();
        self.save_prefs_step(ctx);
    }

    /// The window's own work: its file dialog. Runs in [`WordApp::logic`].
    pub(crate) fn window_work(&mut self) {
        self.clear_stale_change_picture();
        let _ = self.poll_file_dialog();
    }

    /// Lay out the window.
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        self.window_ui(ui);
    }

    /// Lay out the window.
    pub(crate) fn window_ui(&mut self, ui: &mut egui::Ui) {
        let t0 = monotonic_ms();
        let ctx = ui.ctx().clone();
        self.escape_free = self.dialog_host.is_empty()
            && !self.ui.backstage
            && !self.keytips_active
            && !craft::ui::runtime::overlay::any_open(&ctx, craft::ui::runtime::overlay::OpenFilter::Keys)
            && !crate::chrome::search::is_open(self);
        // A failed Recent open, or a failed Save As, is news only until File closes.
        if !self.ui.backstage {
            self.backstage_state.clear_errors();
            self.export_error.clear();
        }
        let regions = self.paint_window(ui);
        self.show_overlays(ui, &ctx, &regions);
        // Last in the pass: the Escape that nothing in the window took.
        crate::keys::window_cancel(self, &ctx);
        self.frame_ms = monotonic_ms() - t0;
        // Typing and formatting land here, after `logic` has reported.
        self.report_dirty();
        self.control_ui(&ctx);
    }
}

#[cfg(test)]
mod paper_theme_tests {
    #[test]
    fn paper_theme_parses() {
        let file = craft::theme::format::parse(include_bytes!("../themes/paper.json"));
        assert!(file.is_ok(), "themes/paper.json is invalid: {:?}", file.err());
        assert!(super::PAPER_THEME.is_some());
    }
}
