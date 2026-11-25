use super::*;
use egui_kittest::kittest::{By, Queryable};

/// The title bar's search field.
const SEARCH_NAME: &str = "Search commands and help";

/// Whether the command is enabled, by the state the command board resolves.
fn enabled(ctx: &egui::Context, id: &str) -> bool {
    craft::ui::runtime::commands::resolve(ctx, id).enabled
}

/// The event of a pressed chord, as the keyboard would send it.
fn chord_event(chord: &craft::keymap::Chord) -> Option<egui::Event> {
    let key = egui::Key::from_name(&chord.key.to_string())?;
    let mac = cfg!(target_os = "macos");
    let m = chord.mods;
    let modifiers = egui::Modifiers { alt: m.alt, shift: m.shift, ctrl: m.ctrl || (m.cmd && !mac), mac_cmd: m.cmd && mac, command: m.cmd };
    Some(egui::Event::Key { key, physical_key: None, pressed: true, repeat: false, modifiers })
}

fn shows_tabs(h: &egui_kittest::Harness<'_, WordApp>) -> bool {
    h.query_by_label("Insert").is_some()
}

/// The text the title bar's search field holds, or `None` while no field shows.
fn search_text(h: &egui_kittest::Harness<'_, WordApp>) -> Option<String> {
    h.query(By::new().role(egui::accesskit::Role::TextInput).label(SEARCH_NAME)).map(|n| n.value().map(|v| v.to_string()).unwrap_or_default())
}

/// Focus hides the title bar and ribbon; the screen bar's button drops the ribbon over the
/// page without leaving Focus, and Escape leaves Focus (and Full Screen View).
#[test]
fn focus_hides_the_chrome_drops_the_ribbon_and_esc_leaves() {
    let mut a = app();
    a.run("text.insert", json!({"text": "Hello"})).unwrap();
    let mut h = harness(a);
    h.run_steps(4);
    assert!(shows_tabs(&h), "the ribbon's tabs show in the usual window");
    assert!(h.query_by_label(SEARCH_NAME).is_some(), "with the title bar's search");
    h.state_mut().run("view.focus", json!({})).unwrap();
    h.run_steps(3);
    assert!(h.state().session.view.focus_mode, "view.focus turns Focus on");
    assert!(!shows_tabs(&h), "Focus hides the ribbon");
    assert!(h.query_by_label(SEARCH_NAME).is_none(), "and the title bar's search");
    // The screen bar's button.
    h.state_mut().run("ui.focusRibbon", json!({})).unwrap();
    h.run_steps(3);
    assert!(shows_tabs(&h), "the ribbon drops down");
    assert!(h.state().session.view.focus_mode, "still in Focus");
    // A click on the dropped ribbon works it and leaves it down; one on the page puts it away.
    h.get_by_label("Insert").click();
    h.run_steps(3);
    assert!(h.state().ribbon_dropped && h.state().ui.tab == "Insert", "a click on the ribbon keeps it down");
    // The dropped ribbon lies over the page but is chrome: its click hands the keyboard back
    // to the page, as a ribbon click does in the usual window.
    assert!(h.state().canvas.focused(&h.ctx), "a click on the dropped ribbon gives the page the keyboard");
    h.event(egui::Event::Text("x".into()));
    h.run_steps(2);
    assert!(h.state().session.doc.plain_text(paper_doc::StoryRef::Body).contains("Hellox"), "typing after it reaches the page");
    assert!(h.state().ribbon_dropped, "typing leaves the dropped ribbon down");
    let page = crate::canvas::page_to_screen(h.state_mut(), 0, 300.0, 600.0).unwrap();
    h.event(egui::Event::PointerMoved(page));
    h.event(egui::Event::PointerButton { pos: page, button: egui::PointerButton::Primary, pressed: true, modifiers: Default::default() });
    h.event(egui::Event::PointerButton { pos: page, button: egui::PointerButton::Primary, pressed: false, modifiers: Default::default() });
    h.run_steps(3);
    assert!(!h.state().ribbon_dropped && h.state().session.view.focus_mode, "a click on the page puts it away, still in Focus");
    h.state_mut().run("ui.focusRibbon", json!({})).unwrap();
    h.run_steps(3);
    h.key_press(egui::Key::Escape);
    h.run_steps(3);
    assert!(!h.state().session.view.focus_mode && !h.state().ribbon_dropped, "Escape leaves Focus");
    h.state_mut().run("view.fullScreenView", json!({})).unwrap();
    h.run_steps(3);
    assert!(!shows_tabs(&h), "Full Screen View shows the pages alone");
    assert!(h.query_by_label(SEARCH_NAME).is_some(), "with Search at the top");
    h.key_press(egui::Key::Escape);
    h.run_steps(3);
    assert!(!h.state().session.view.full_screen_view, "Escape leaves Full Screen View");
    // Full Screen (F11) is the whole window, ribbon and all.
    h.state_mut().run("view.fullScreen", json!({})).unwrap();
    h.run_steps(3);
    assert!(shows_tabs(&h), "Full Screen keeps the ribbon");
}

/// Focus's dropped ribbon is a command, `ui.focusRibbon`, which the screen bar's button and a
/// click on the page run; outside Focus it does nothing and never fails.
#[test]
fn focus_ribbon_is_a_command() {
    let mut a = app();
    let off = json!({"focusRibbon": false});
    assert_eq!(a.run("ui.focusRibbon", json!({})).unwrap(), off, "outside Focus there is no ribbon to drop");
    assert_eq!(a.run("ui.focusRibbon", json!({"value": true})).unwrap(), off);
    assert!(!a.ribbon_dropped);
    a.run("view.focus", json!({})).unwrap();
    assert_eq!(a.run("ui.focusRibbon", json!({})).unwrap(), json!({"focusRibbon": true}), "no value toggles it");
    assert!(a.ribbon_dropped && a.session.view.focus_mode);
    assert_eq!(a.run("ui.focusRibbon", json!({})).unwrap(), off);
    assert_eq!(a.run("ui.focusRibbon", json!({"value": true})).unwrap()["focusRibbon"], true);
    assert_eq!(a.run("ui.focusRibbon", json!({"value": true})).unwrap()["focusRibbon"], true, "true keeps it down");
    for junk in [json!({"value": "yes"}), json!({"value": 1}), json!({"value": null}), json!({"value": [true]})] {
        assert!(a.run("ui.focusRibbon", junk).is_err());
    }
    assert!(a.ribbon_dropped, "a refused value leaves it as it was");
    assert_eq!(a.run("ui.focusRibbon", json!({"value": false})).unwrap(), off);
    assert!(a.session.view.focus_mode, "putting the ribbon away stays in Focus");
}

/// One Escape path. In Focus with text selected on the page, the first Escape only
/// collapses the selection, as in a normal window, and the second leaves Focus. A handler that
/// takes Escape for Focus before the page sees it would leave Focus with the selection kept.
#[test]
fn focus_esc_collapses_the_selection_first_then_leaves() {
    let mut h = canvas_harness();
    h.state_mut().run("view.focus", json!({})).unwrap();
    h.run_steps(3);
    h.state_mut().run("select.all", json!({})).unwrap();
    h.run_steps(2);
    assert!(h.state().canvas.focused(&h.ctx) && h.state().session.view.focus_mode);
    assert!(!h.state().session.sel.is_collapsed(), "text is selected");
    h.key_press(egui::Key::Escape);
    h.run_steps(3);
    assert!(h.state().session.sel.is_collapsed(), "the first Escape collapses the selection");
    assert!(h.state().session.view.focus_mode, "and keeps Focus");
    h.key_press(egui::Key::Escape);
    h.run_steps(3);
    assert!(!h.state().session.view.focus_mode, "the second Escape leaves Focus");
}

/// Started with nothing to open, the app shows File › Home (the Start screen) as Word
/// does. No document is open behind it, so Esc (or Back) shows an empty window, with
/// no page, the ribbon greyed out but for Help, and nothing to save on close. With the setting
/// off it starts in the editor, and `WordApp::new` alone (tests, ui_shot, windows opened for a
/// document) never shows it.
#[test]
fn start_screen_shows_file_home_and_esc_leaves_no_document() {
    let mut a = app();
    assert!(!a.ui.backstage, "`new` starts in the editor");
    assert!(a.has_document());
    a.ui.backstage_page = "print".into();
    assert!(a.start_screen(), "on by default");
    assert!(a.ui.backstage && a.ui.backstage_page == "home");
    assert!(!a.has_document(), "no document behind the Start screen");
    let mut h = harness(a);
    h.run_steps(4);
    assert!(h.query_by_label("Blank document").is_some(), "Home lists Blank document and the templates");
    // The Recent heading and the Recent side-menu entry both carry the label.
    assert!(h.query_all_by_label("Recent").next().is_some(), "and the Recent list");
    h.key_press(egui::Key::Escape);
    h.run_steps(4);
    let ctx = h.ctx.clone();
    let a = h.state_mut();
    assert!(!a.ui.backstage, "Esc leaves the Start screen");
    assert!(!a.has_document(), "for an empty window, not a blank document");
    assert_eq!(a.window_title(), "Paper", "the window is titled with the app alone");
    // The ribbon's commands are off, but Help, the File tab and the settings work.
    for id in ["format.bold", "text.insert", "file.save", "file.saveAs", "file.print", "edit.paste", "insert.table"] {
        assert!(!enabled(&ctx, id), "{id} is off with no document");
    }
    for id in ["file.new", "file.open", "ui.backstage", "ui.tab", "file.options", "ui.theme", "ui.statusCount"] {
        assert!(enabled(&ctx, id), "{id} works with no document");
    }
    assert!(!crate::chrome::qat_enabled(&ctx, a, "file.save"));
    // Only the listed interface commands work: the ones that act on the document don't.
    for id in ["ui.contextMenu", "ui.render", "ui.linkTextBox", "ui.unlistedCommand"] {
        assert!(!works_without_document(id), "{id} needs a document");
    }
    let refused = a.run("ui.contextMenu", json!({})).unwrap_err();
    assert!(refused.contains("No document is open"), "refused before it runs: {refused}");
    assert!(works_without_document("font.add"), "a font file dropped on the empty window is added");
    // File › Options › Proofing › AutoCorrect Options edits a user setting, not the document.
    a.run("tools.autocorrect", json!({})).unwrap();
    assert!(a.dialog_host.open_dialogs().iter().any(|d| d.key.as_deref() == Some("autoCorrect")));
    crate::dialogs::close_all(a, &ctx);
    // Neither keys nor commands reach the hidden placeholder.
    assert!(a.run("text.insert", json!({"text": "Hello"})).is_err());
    assert!(a.execute("format.bold", json!({})).is_err());
    h.event(egui::Event::Text("typed".into()));
    h.key_press(egui::Key::Enter);
    h.run_steps(2);
    let a = h.state_mut();
    assert_eq!(a.session.doc.plain_text(paper_doc::StoryRef::Body).trim(), "");
    assert!(!a.session.dirty && !a.has_document());
    assert!(a.close_requested(), "closing asks nothing");
    assert!(a.dialog_host.open_dialogs().is_empty());

    let mut b = app();
    b.run("ui.startScreen", json!({"value": false})).unwrap();
    assert!(!b.start_screen(), "off: today's behaviour");
    assert!(!b.ui.backstage && b.has_document());
}

/// The empty window's File tab goes back to the Backstage with only Home, New, Open
/// and Options; picking Blank document there opens one, ready for typing.
#[test]
fn blank_document_opens_a_document_from_the_start_screen() {
    let mut a = app();
    a.start_screen();
    a.run("ui.backstage", json!({"value": false})).unwrap();
    a.run("ui.backstage", json!({"value": true, "page": "info"})).unwrap();
    let first = a.session.document_id();
    let mut h = harness(a);
    h.run_steps(4);
    assert_eq!(h.state().ui.backstage_page, "home", "Info needs a document");
    h.get_by_label("Blank document").click();
    h.run_steps(4);
    let ctx = h.ctx.clone();
    let a = h.state_mut();
    assert!(a.has_document() && !a.ui.backstage, "Blank document opens one");
    assert_ne!(a.session.document_id(), first);
    assert!(enabled(&ctx, "format.bold"));
    a.run("text.insert", json!({"text": "Hello"})).unwrap();
    assert_eq!(a.session.doc.plain_text(paper_doc::StoryRef::Body).trim(), "Hello");
    let title = a.window_title();
    assert!(title.starts_with(&a.title_stem()), "{title}");
}

/// A template or a file opened from the Start screen (or handed over later) is the
/// document; a failed open leaves the window empty.
#[test]
fn templates_and_files_open_from_the_start_screen() {
    let mut a = app();
    a.start_screen();
    a.run("file.new", json!({"template": "letter"})).unwrap();
    assert!(a.has_document());

    let dir = scratch("start-screen-open");
    let path = docx_on_disk(&dir, "Signed copy");
    let mut b = app();
    b.start_screen();
    assert!(b.run("file.open", json!({"path": dir.join("missing.docx").to_string_lossy()})).is_err());
    assert!(!b.has_document(), "a failed open opens nothing");
    b.run("file.open", json!({"path": path.to_string_lossy()})).unwrap();
    assert!(b.has_document());
    assert!(body_text(&b).contains("Signed copy"));
    let _ = std::fs::remove_dir_all(dir);
}

/// Alt+Q opens Search with its box focused, as in Word: the Q isn't typed into the
/// document, and what is typed next goes into the search box.
#[test]
fn alt_q_opens_search_and_typing_goes_there() {
    let mut h = harness(app());
    h.run_steps(6);
    let key = |key, modifiers| egui::Event::Key { key, physical_key: None, pressed: true, repeat: false, modifiers };
    h.event(egui::Event::Text("Kilns".into()));
    h.step();
    // Some platforms send the key's text with an Alt shortcut; it isn't typed.
    // The chord the keymap gives Search on this platform; macOS has none (Option+Q types a character).
    // Off macOS the chord is the interface's Search Commands (`app/command_board.rs`), which the engine's own
    // `tools.search` binding gives way to: it opens Search in the title bar's box where the window shows one.
    let search = craft::ui::runtime::commands::CommandRef::with_params("ui.dialog", json!({"name": "commands"}));
    let Some(chord) = craft::ui::runtime::commands::chord(&h.ctx, &search) else {
        #[cfg(not(target_os = "macos"))]
        panic!("Search has a chord off macOS");
        #[cfg(target_os = "macos")]
        return;
    };
    let Some(open) = chord_event(&chord) else {
        panic!("the chord {chord} names a key egui knows");
    };
    // Both in one frame's input, as the platform sends them: kittest's `event` runs a frame for each queued event.
    h.input_mut().events.extend([open.clone(), egui::Event::Text("q".into())]);
    h.step();
    // The chord fires the command in that pass; the app runs it before the next pass (`raw_input_hook`).
    h.step();
    // The title bar shows the Search box, so Search opens there, in place.
    assert!(h.state().dialog_host.open_dialogs().is_empty(), "no Search Commands dialog over the page");
    assert!(crate::chrome::search::is_open(h.state()), "Search is open");
    h.step();
    assert_eq!(search_text(&h).as_deref(), Some(""), "the field is empty");
    assert!(h.query_by_label_contains("Word Count").is_some(), "an empty box suggests commands, as Word's does");
    h.event(egui::Event::Text("bold".into()));
    h.step();
    assert_eq!(search_text(&h).as_deref(), Some("bold"), "typing goes into the search box");
    assert_eq!(h.state().session.doc.plain_text(paper_doc::StoryRef::Body), "Kilns");
    h.event(key(egui::Key::Escape, egui::Modifiers::NONE));
    h.step();
    h.step();
    // The framework gives Escape to a field holding a draft first (`overlay/dismissal.rs`): the typed "bold" takes the
    // first one, and the second closes Search. The old app closed on one.
    h.event(key(egui::Key::Escape, egui::Modifiers::NONE));
    h.step();
    h.step();
    assert!(!crate::chrome::search::is_open(h.state()), "Escape closes Search");
    assert!(!h.state().session.view.full_screen_view);
    // Words that name no command: Enter finds them in the document, in the Navigation pane.
    h.event(open);
    h.step();
    h.step();
    // Typing sends each key's press before its text.
    h.event(key(egui::Key::K, egui::Modifiers::SHIFT));
    h.event(egui::Event::Text("Kilns".into()));
    h.step();
    h.event(key(egui::Key::Enter, egui::Modifiers::NONE));
    h.step();
    h.step();
    assert!(!crate::chrome::search::is_open(h.state()), "Enter ran the search");
    assert!(h.state().session.view.nav_pane, "the Navigation pane shows the results");
    assert_eq!(h.state().session.find.results.len(), 1);
}

/// Focus hides the Search box, so Search there is the dialog, even on the frame
/// Focus comes on; and Search left open when Focus comes on doesn't keep Escape from leaving it.
#[test]
fn focus_mode_search_never_opens_in_the_hidden_box() {
    let mut h = harness(app());
    h.run_steps(6);
    let key = |key, modifiers| egui::Event::Key { key, physical_key: None, pressed: true, repeat: false, modifiers };
    // Search, as Alt+Q opens it (Windows and Linux; macOS has no Alt+Q, Option+Q types a character).
    let search = |h: &mut egui_kittest::Harness<WordApp>| {
        let _ = h.state_mut().run("ui.dialog", json!({"name": "commands"}));
    };
    // The last frame drew the box; Focus comes on just before Search.
    h.state_mut().run("view.focus", json!({"value": true})).unwrap();
    search(&mut h);
    h.step();
    assert!(!crate::chrome::search::is_open(h.state()), "no Search in a box Focus hides");
    assert!(!h.state().dialog_host.open_dialogs().is_empty(), "Search opens as the dialog");
    let ctx = h.ctx.clone();
    crate::dialogs::close_all(h.state_mut(), &ctx);
    h.step();
    h.state_mut().run("view.focus", json!({"value": false})).unwrap();
    h.run_steps(3);
    // Search open in the title bar, then Focus comes on: Escape leaves Focus.
    search(&mut h);
    h.run_steps(2);
    assert!(crate::chrome::search::is_open(h.state()));
    h.state_mut().run("view.focus", json!({"value": true})).unwrap();
    h.run_steps(3);
    assert!(!crate::chrome::search::is_open(h.state()), "Search whose box is gone is dropped");
    h.event(key(egui::Key::Escape, egui::Modifiers::NONE));
    h.run_steps(2);
    assert!(!h.state().session.view.focus_mode, "Escape leaves Focus");
}

/// `tools.search` with a query lists matching commands for scripts and agents; every
/// word has to match, so "insert table" finds the command that inserts a table.
#[test]
fn search_command_matches_every_word() {
    let mut a = app();
    let r = a.run("tools.search", json!({"query": "insert table"})).unwrap();
    let ids: Vec<&str> = r["matches"].as_array().unwrap().iter().filter_map(|m| m["id"].as_str()).collect();
    assert!(ids.contains(&"insert.table"), "{ids:?}");
    assert!(r["matches"].as_array().unwrap().iter().all(|m| {
        let hay = format!("{} {} {}", m["label"], m["location"], m["id"]).to_lowercase();
        hay.contains("insert") && hay.contains("table")
    }));
    assert_eq!(a.run("tools.search", json!({"query": "   "})).unwrap()["matches"], json!([]));
    assert_eq!(a.run("tools.search", json!({"query": "zzqqxx"})).unwrap()["matches"], json!([]));
    assert!(a.dialog_host.open_dialogs().is_empty(), "a query never opens the box");
    a.run("tools.search", json!({})).unwrap();
    // The title bar's in-place field takes the request, where there is a box to show it; no dialog opens.
    assert!(crate::chrome::search::is_open(&a), "no query opens Search");
}

/// Taking documents off the list needs no open document — the Start screen shows
/// the recent list, and its buttons must work from there.
#[test]
fn recent_list_edits_work_without_a_document() {
    let mut a = app();
    a.ui.recent = vec!["/tmp/alpha.docx".into()];
    assert!(!a.session.dirty && a.session.path.is_none());
    assert!(works_without_document("ui.recentRemove") && works_without_document("ui.recentClear"));
    assert_eq!(a.run("ui.recentRemove", json!({"path": "/tmp/alpha.docx"})).unwrap()["removed"], 1);
    assert_eq!(a.run("ui.recentClear", json!({})).unwrap()["removed"], 0);
}
