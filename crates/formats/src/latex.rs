//! LaTeX (`.tex`) import and export via `craft-latex`.

use paper_doc::{Document, SectionProps};

pub use craft_latex::{escape_text, latex_to_linear, linear_to_latex};

pub fn import(bytes: &[u8]) -> Document {
    paper_docx::from_shared_model(craft_latex::import(bytes)).unwrap_or_default()
}

pub fn import_on(bytes: &[u8], page: &SectionProps) -> Document {
    let mut d = import(bytes);
    d.last_section = page.clone();
    d
}

pub fn parse(text: &str) -> craft_flow::Flow {
    craft_latex::parse(text)
}

/// The first line `craft_latex::export` writes names its original app; Paper's export names Paper.
const UPSTREAM_HEADER: &str = "% Written by WordCraft (https://getartcraft.com/apps/wordcraft).\n";

pub fn export(doc: &Document) -> String {
    match paper_docx::to_shared_model(doc) {
        Ok(shared) => {
            let tex = craft_latex::export(&shared);
            match tex.strip_prefix(UPSTREAM_HEADER) {
                Some(rest) => format!("% Written by Paper.\n{rest}"),
                None => tex,
            }
        }
        Err(_) => String::new(),
    }
}
