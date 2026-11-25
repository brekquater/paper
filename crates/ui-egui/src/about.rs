//! Help › About: Paper's name and version, what it is, the licence notice the Paper code it is
//! built on requires, and the licences of the spelling and hyphenation data.

use craft::ui::contract::{Show as _, ShowContainer as _};
use craft::ui::dialog::{Dialog, DialogId, DialogResult, DialogStack, StackDialog, dialog_id};
use craft::ui::display::{Shade, Text};
use craft::ui::sections::{DialogRole, FooterButton};
use craft::ui::style::TypeRole;
use egui::Ui;

/// The dialog's key, as the framework's About had it, so scripts and tests that ask for it still find it.
const KEY: &str = "help.about";

/// The tallest the language data list grows before it scrolls, in points.
const LANGUAGE_DATA_HEIGHT: f32 = 240.0;

/// The About dialog.
pub(crate) struct AboutDialog;

impl StackDialog for AboutDialog {
    type Result = ();

    fn id(&self) -> DialogId {
        dialog_id(KEY)
    }

    fn key(&self) -> Option<&str> {
        Some(KEY)
    }

    fn show(&mut self, ui: &mut Ui, _stack: &DialogStack) -> Option<DialogResult<()>> {
        let build = option_env!("PAPER_BUILD_SHA").map(|s| s.get(..8).unwrap_or(s)).unwrap_or(tl!("development build"));
        let version = crate::i18n::fmt(tl!("Version {version} ({build})"), &[("version", env!("CARGO_PKG_VERSION")), ("build", build)]);
        let shown = Dialog::new(tl!("About Paper").to_string()).buttons([FooterButton::new(tl!("Close").to_string(), DialogRole::Accept)]).show(
            ui,
            |ui, _| {
                Text::new(tl!("Paper")).role(TypeRole::Heading).show(ui);
                Text::new(version.to_string()).show(ui);
                Text::new(tl!("A document editor you can embed in your web app.")).show(ui);
                // The copyright notice of the engine Paper is built on, which its licence requires; keep it.
                Text::new(tl!(
                    "Copyright (c) 2026 Brekquater. Includes WordCraft, Copyright (c) 2026 ArtCraft Team and the WordCraft contributors. Apache License 2.0."
                ))
                .shade(Shade::Faint)
                .show(ui);
                Text::new(tl!("Language data")).role(TypeRole::Heading).show(ui);
                egui::ScrollArea::vertical().max_height(LANGUAGE_DATA_HEIGHT).show(ui, crate::language_packs::language_data);
            },
        );
        shown.output.closed.map(|_| DialogResult::Ok(()))
    }
}
