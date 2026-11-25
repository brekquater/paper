//! The `PaperEditor` class exported to JavaScript.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use base64::Engine as _;
use craft::host::{Files, Host};
use craft::platform::appearance::SystemReader;
use craft::platform::browser_files::BrowserFiles;
use craft::platform::clipboard::{Clipboard, ClipboardError, EventClipboard, Flavor, TEXT};
use craft::ui::InstallSpec;
use craft::ui::shell::embed;
use craft::window::system::NoWindows;
use paper_engine::Session;
use paper_ui_egui::{Services, WordApp};
use serde_json::{Value, json};
use wasm_bindgen::prelude::*;

mod events;
mod files;

use events::{EVENTS, Listeners, Seen};
use files::HostedSaves;

/// The largest document `load` takes, refused before it is copied (as the host bridge does).
const MAX_LOAD_BYTES: usize = 512 << 20;

/// The longest author name `setAuthor` takes (as the web app's `?author=` does).
const MAX_AUTHOR_CHARS: usize = 200;

/// One editor mounted in a host-owned canvas.
#[wasm_bindgen]
pub struct PaperEditor {
    runner: eframe::WebRunner,
    ctx: egui::Context,
    listeners: Rc<Listeners>,
}

#[wasm_bindgen]
impl PaperEditor {
    /// Mount an editor in `canvas`, with a blank document. Resolves once it is running.
    ///
    /// `ui` is `"ribbon"` (the default: the full interface with the ribbon, rulers, panes and
    /// status bar) or `"pages"` (the pages alone, for a host with its own toolbar).
    pub async fn create(canvas: web_sys::HtmlCanvasElement, ui: Option<String>) -> Result<PaperEditor, JsValue> {
        let pages_only = match ui.as_deref().unwrap_or("ribbon") {
            "ribbon" => false,
            "pages" => true,
            other => return Err(err(format!("`ui` is \"ribbon\" or \"pages\", not {other:?}"))),
        };
        eframe::WebLogger::init(log::LevelFilter::Info).ok();
        let runner = eframe::WebRunner::new();
        let listeners = Rc::new(Listeners::default());
        let started: Rc<RefCell<Option<egui::Context>>> = Rc::default();
        let slot = started.clone();
        let shell_listeners = listeners.clone();
        runner
            .start(
                canvas,
                eframe::WebOptions::default(),
                Box::new(move |cc| {
                    *slot.borrow_mut() = Some(cc.egui_ctx.clone());
                    let (app, saves) = new_app(&cc.egui_ctx, pages_only);
                    let seen = Seen::of(&app.session);
                    Ok(Box::new(Shell { app, saves, listeners: shell_listeners, seen }))
                }),
            )
            .await?;
        let ctx = started.borrow_mut().take().ok_or_else(|| err("the editor did not start"))?;
        Ok(PaperEditor { runner, ctx, listeners })
    }

    /// Listen for `event`: `change`, `selectionchange`, `dirty` or `save` (see `events.rs` for the
    /// details each one carries).
    pub fn on(&self, event: &str, listener: js_sys::Function) -> Result<(), JsValue> {
        if !EVENTS.contains(&event) {
            return Err(err(format!("unknown event {event:?}; one of {}", EVENTS.join(", "))));
        }
        self.listeners.add(event, listener);
        Ok(())
    }

    /// Stop `listener` hearing `event`.
    pub fn off(&self, event: &str, listener: &js_sys::Function) {
        self.listeners.remove(event, listener);
    }

    /// The name new comments and tracked changes are signed with. Display metadata only: it does
    /// not authenticate anyone.
    #[wasm_bindgen(js_name = setAuthor)]
    pub fn set_author(&self, name: &str) -> Result<(), JsValue> {
        let name: String = name.trim().chars().take(MAX_AUTHOR_CHARS).collect();
        self.run("file.setAuthor", json!({"name": name})).map(|_| ())
    }

    /// Run an engine or UI command by id with JSON `params` ("" for none); its JSON result.
    pub fn execute(&self, command: &str, params: &str) -> Result<String, JsValue> {
        let params: Value =
            if params.trim().is_empty() { json!({}) } else { serde_json::from_str(params).map_err(|e| err(format!("params: {e}")))? };
        self.run(command, params).map(|v| v.to_string())
    }

    /// Open a document from its bytes in place of the current one, whose unsaved changes are
    /// discarded; `name` gives the format by extension (`report.docx`). Its JSON result.
    pub fn load(&self, name: &str, bytes: &[u8]) -> Result<String, JsValue> {
        if bytes.len() > MAX_LOAD_BYTES {
            return Err(err(format!("the document is {} bytes, over the {} MiB limit", bytes.len(), MAX_LOAD_BYTES >> 20)));
        }
        let data = base64::engine::general_purpose::STANDARD.encode(bytes);
        self.with_app(|app| {
            let previous = app.session.document_id();
            let opened = app.execute("file.open", json!({"path": name, "data": data}))?;
            // Opening adds a document unless it replaced an unused blank one: close the old one.
            if app.session.document_id() != previous && app.session.is_open(previous) {
                app.execute("file.closeDocument", json!({"id": previous, "force": true}))?;
            }
            Ok::<_, String>(opened)
        })?
        .map(|v| v.to_string())
        .map_err(err)
    }

    /// The document as a file of `format` (`docx`, `html`, `pdf`…). The document stays as it is:
    /// an export is a copy, not a save.
    pub fn export(&self, format: &str) -> Result<Vec<u8>, JsValue> {
        let name = format!("document.{}", format.trim_start_matches('.'));
        self.with_app(|app| {
            let opts = paper_engine::cmd::file::export_options(&app.session, &json!({}));
            paper_engine::io::save_bytes_with(&name, &app.session.doc, None, &opts)
        })?
        .map_err(err)
    }

    /// Stop the editor: its listeners go, its frame loop ends and its app is dropped. Calling it
    /// again does nothing.
    pub fn destroy(&self) {
        self.runner.destroy();
        self.listeners.clear();
    }
}

impl PaperEditor {
    fn run(&self, command: &str, params: Value) -> Result<Value, JsValue> {
        self.with_app(|app| app.execute(command, params))?.map_err(err)
    }

    fn with_app<R>(&self, f: impl FnOnce(&mut WordApp) -> R) -> Result<R, JsValue> {
        let mut shell = self.runner.app_mut::<Shell>().ok_or_else(|| err("the editor was destroyed, or stopped after an error"))?;
        let r = f(&mut shell.app);
        drop(shell);
        self.ctx.request_repaint();
        Ok(r)
    }
}

impl Drop for PaperEditor {
    /// Freeing the JS object stops the editor too.
    fn drop(&mut self) {
        self.runner.destroy();
    }
}

/// A JavaScript `Error` with `e` as its message.
fn err(e: impl std::fmt::Display) -> JsValue {
    js_sys::Error::new(&e.to_string()).into()
}

/// The editor app, set up as the web app sets it up, minus what a component leaves to its host:
/// no stored preferences (settings stay in memory), no sample or start screen, and
/// saves handed to the host (the returned queue) instead of downloaded.
fn new_app(ctx: &egui::Context, pages_only: bool) -> (WordApp, Arc<HostedSaves>) {
    let frame_ctx = ctx.clone();
    let copy_ctx = ctx.clone();
    let clipboard = EventClipboard::new(move || frame_ctx.cumulative_frame_nr(), move |flavors| copy_out(&copy_ctx, flavors)).with_wake(waker(ctx));
    let region = paper_ui_egui::i18n::system_region();
    let lang = paper_ui_egui::i18n::Lang::from_pref(&paper_ui_egui::prefs::Prefs::default().language);
    // ponytail: the runtime keeps upstream's app id; it changes with the rebrand.
    let spec = InstallSpec::new(craft::host::target::ShellInfo::current(), Arc::new(NoWindows))
        .app_id("paper")
        .system(SystemReader::start(waker(ctx)))
        .clipboard(Clipboard::new(clipboard));
    if let Err(e) = embed::install(ctx, spec, &paper_ui_egui::ui_fonts(lang)) {
        log::error!("paper-web: the UI runtime did not install: {e:?}");
    }
    let saves = Arc::new(HostedSaves::new(Arc::new(BrowserFiles::new(waker(ctx)))));
    let files: Arc<dyn Files> = saves.clone();
    let host = Arc::new(Host::builder(paper_ui_egui::APP).i18n(paper_ui_egui::i18n::catalogs()).files(files).lang(lang.craft_lang()).build());
    let mut session = Session::new(paper_engine::blank_document(&region));
    session.system_region = region;
    let mut app = WordApp::new(session, Services::default()).with_host(host);
    // The pages alone: Full Screen View hides the title bar, ribbon, status bar, panes and rulers.
    if pages_only && let Err(e) = app.execute("view.fullScreenView", json!({"value": true})) {
        log::warn!("paper-web: no pages-only view: {e}");
    }
    (app, saves)
}

/// A wake that asks `ctx` for a frame; callable from any task.
fn waker(ctx: &egui::Context) -> impl Fn() + Send + Sync + Clone + 'static {
    let ctx = ctx.clone();
    move || ctx.request_repaint()
}

/// The clipboard's copy path: eframe's own text copy is the browser's only write.
fn copy_out(ctx: &egui::Context, flavors: &[Flavor]) -> Result<usize, ClipboardError> {
    let (index, text) = flavors.iter().enumerate().find(|(_, f)| f.mime == TEXT).ok_or(ClipboardError::NoFormat)?;
    ctx.copy_text(String::from_utf8_lossy(&text.data).into_owned());
    Ok(index)
}

/// The eframe app around the editor.
struct Shell {
    app: WordApp,
    saves: Arc<HostedSaves>,
    listeners: Rc<Listeners>,
    /// What the host last heard, to raise events for what each frame changed.
    seen: Seen,
}

impl Shell {
    /// After a frame: raise the events for what it changed and hand over what it saved.
    fn report(&mut self) {
        let now = Seen::of(&self.app.session);
        if now != self.seen {
            self.seen.report(&now, &self.listeners);
            self.seen = now;
        }
        for saved in self.saves.take() {
            events::report_save(saved, &self.listeners);
        }
    }
}

impl eframe::App for Shell {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.app.logic(ctx);
    }

    fn raw_input_hook(&mut self, ctx: &egui::Context, _raw: &mut egui::RawInput) {
        embed::before_input(ctx, |out| self.app.frame_input(out));
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.app.ui(ui);
        self.report();
    }

    /// Lets `PaperEditor` reach the app through `WebRunner::app_mut`.
    fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        Some(self)
    }
}
