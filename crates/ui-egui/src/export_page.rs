//! File › Export: the formats a copy can be saved in.

use craft::ui::buttons::Button;
use craft::ui::contract::{Props as _, Show as _, Sizing, clicked};
use craft::ui::display::Text;
use craft::ui::primitives::HAlign;
use egui::Ui;

use crate::WordApp;

/// The export "format" of a Word 2003 XML Document: an `.xml` file saved with `format: "wordml2003"`.
pub(crate) const EXPORT_WORD_2003: &str = "wordml2003";

/// The "save a copy" formats as (label, format), also the built-in browser's types when exporting.
pub(crate) const EXPORT_FORMATS: &[(&str, &str)] = &[
    ("PDF document (*.pdf)", "pdf"),
    ("Word document (*.docx)", "docx"),
    ("Word XML Document (*.xml)", "xml"),
    ("Word 2003 XML Document (*.xml)", EXPORT_WORD_2003),
    ("OpenDocument Text (*.odt)", "odt"),
    ("OpenDocument Text Template (*.ott)", "ott"),
    ("Flat OpenDocument Text (*.fodt)", "fodt"),
    ("Rich Text Format (*.rtf)", "rtf"),
    ("Web page (*.html)", "html"),
    ("Markdown (*.md)", "md"),
    ("LaTeX (*.tex)", "tex"),
    ("Plain text (*.txt)", "txt"),
    ("Page image (*.png)", "png"),
];

/// The file extension an export format writes.
pub(crate) fn export_file_ext(format: &str) -> &str {
    if format == EXPORT_WORD_2003 { "xml" } else { format }
}

/// Is `ext` an extension of a document or export type (the Export types, plus the older `.doc`
/// and `.htm`)? Such an extension on a typed name gives way to the chosen type's; any other
/// (`notes.v2`) is part of the name.
pub(crate) fn is_document_ext(ext: &str) -> bool {
    ["doc", "htm"].iter().any(|e| e.eq_ignore_ascii_case(ext)) || EXPORT_FORMATS.iter().any(|&(_, f)| export_file_ext(f).eq_ignore_ascii_case(ext))
}

/// The name an export to `format` is saved under: the chosen type's extension replaces a known
/// document extension (`notes.docx` as plain text is `notes.txt`) and is added to any other name.
pub(crate) fn export_name(name: &str, format: &str) -> String {
    let want = export_file_ext(format);
    let path = std::path::Path::new(name);
    // A dotfile name (`.docx`) has no extension for `Path::extension`, which the engine's save also
    // reads, so it is a stem and gets one (`.docx.xml`); a bare extension would be refused.
    match path.extension().and_then(|e| e.to_str()) {
        Some(e) if e.eq_ignore_ascii_case(want) => name.to_string(),
        Some(e) if is_document_ext(e) => path.with_extension(want).to_string_lossy().into_owned(),
        // `a.` has an empty extension: its dot is dropped before the type's is added.
        _ => format!("{}.{want}", name.trim_end_matches('.')),
    }
}

/// The "save a copy" formats (File › Export).
pub fn export_formats(app: &mut WordApp, ui: &mut Ui) {
    // A refused or failed export, said here because Backstage hides the status bar (the Export page draws this).
    if !app.export_error.is_empty() {
        Text::new(app.export_error.clone()).show(ui);
    }
    // Each format is a button across the column, its name at the reading start, as production's 320 pt rows.
    for &(label, ext) in EXPORT_FORMATS {
        if clicked(&Button::new(tl!(label)).id_salt(ext).sizing(Sizing::FillWidth).align(HAlign::Start).show(ui)) {
            app.export_dialog(ext);
        }
    }
    // Linux and the BSDs: Paper's own browser when the system dialog doesn't appear.
    #[cfg(all(unix, not(target_os = "macos"), not(target_arch = "wasm32")))]
    {
        if clicked(&Button::new(tl!("Browse in Paper…")).id_salt("browse").sizing(Sizing::FillWidth).align(HAlign::Start).show(ui)) {
            let _ = app.run("ui.fileBrowser", json!({"purpose": "export"}));
        }
    }
}
