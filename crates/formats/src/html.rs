//! HTML import and export via `craft-html`.

use paper_doc::{Document, SectionProps};

pub use craft_html::doc::{ImageLoader, PAGE_FONT, css_len_pt, decode, decode_as, decode_entity, esc, export_flow, parse, parse_with, unescape};

fn shared_section(page: &SectionProps) -> craft_wordmodel::SectionProps {
    let mut d = Document::new();
    d.last_section = page.clone();
    paper_docx::to_shared_model(&d).map(|s| s.last_section).unwrap_or_default()
}

pub fn import(html: &str) -> Document {
    paper_docx::from_shared_model(craft_html::doc::import(html)).unwrap_or_default()
}

pub fn import_page(html: &str, images: ImageLoader<'_>) -> Document {
    paper_docx::from_shared_model(craft_html::doc::import_page(html, images)).unwrap_or_default()
}

pub fn import_with(html: &str, images: ImageLoader<'_>) -> Document {
    paper_docx::from_shared_model(craft_html::doc::import_with(html, images)).unwrap_or_default()
}

pub fn import_on(html: &str, images: ImageLoader<'_>, page: &SectionProps) -> Document {
    paper_docx::from_shared_model(craft_html::doc::import_on(html, images, &shared_section(page))).unwrap_or_default()
}

/// The generator tag `craft_html::doc::export` writes names its original app; Paper's names Paper.
const UPSTREAM_GENERATOR: &str = "<meta name=\"generator\" content=\"WordCraft\">";

pub fn export(doc: &Document) -> String {
    match paper_docx::to_shared_model(doc) {
        Ok(shared) => craft_html::doc::export(&shared).replacen(UPSTREAM_GENERATOR, "<meta name=\"generator\" content=\"Paper\">", 1),
        Err(_) => String::new(),
    }
}

pub fn export_fragment(doc: &Document) -> String {
    match paper_docx::to_shared_model(doc) {
        Ok(shared) => craft_html::doc::export_fragment(&shared),
        Err(_) => String::new(),
    }
}
