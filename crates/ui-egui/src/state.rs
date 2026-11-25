/// How Save in the Save Changes prompt went.
pub(crate) enum PromptSave {
    /// Written, keeping everything: the held-back command may run.
    Saved,
    /// Not saved (a format that doesn't keep everything); the command is cancelled.
    NotSaved,
    /// Web: the browser's save picker finishes later; its ticket.
    Later,
}

/// Whether saving to `name` keeps the whole document (the formats `file.save` treats as the
/// document's own file, including macro-enabled documents and templates); anything else is a
/// copy that leaves it unsaved, unless it is the file the document was opened from.
pub fn keeps_everything(name: &str) -> bool {
    let ext = std::path::Path::new(name).extension().map(|e| e.to_string_lossy().to_ascii_lowercase()).unwrap_or_default();
    matches!(ext.as_str(), "docx" | "docm" | "dotx" | "dotm" | "odt" | "ott" | "fodt" | "rtf" | "json" | "xml")
}
