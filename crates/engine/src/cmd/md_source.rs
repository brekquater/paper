//! A Markdown file's own text ([`FileText`]).
//!
//! The document can't hold everything a Markdown file says (front matter, comments, a code
//! block's language, the author's own spacing), so the text the file was opened or last saved
//! with is kept beside it: while that text still stands for the document, Save writes it back
//! unchanged. An edit makes the document something only writing it out again can say.

use std::cell::RefCell;
use std::sync::Arc;

use paper_doc::Document;

use crate::Session;

/// The largest file text kept, in bytes.
pub const MAX_SOURCE_BYTES: usize = 2 * 1024 * 1024;

/// The text a Markdown file was opened or last saved with.
#[derive(Clone, Default)]
pub struct FileText {
    saved: Option<String>,
    cached_exact: RefCell<Option<(u64, Option<String>)>>,
}

impl FileText {
    /// A file's bytes as its text, if they can be written back as they are: valid UTF-8,
    /// not empty, and no longer than [`MAX_SOURCE_BYTES`].
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let keep = !bytes.is_empty() && bytes.len() <= MAX_SOURCE_BYTES;
        let saved = keep.then(|| std::str::from_utf8(bytes).ok()).flatten().map(str::to_string);
        Self { saved, cached_exact: RefCell::new(None) }
    }

    /// The text just written to the file, which is the file's own text now.
    pub fn saved(text: Option<String>) -> Self {
        Self { saved: text, cached_exact: RefCell::new(None) }
    }

    /// No text is kept.
    pub fn is_empty(&self) -> bool {
        self.saved.is_none()
    }
}

/// The kept text, if it still stands for the document as it is.
fn exact_text(s: &Session) -> Option<String> {
    if let Some((cached_rev, ref res)) = *s.md_file.cached_exact.borrow()
        && cached_rev == s.rev()
    {
        return res.clone();
    }
    let res = s.md_file.saved.as_deref().filter(|text| parse(&s.doc, text).same_content(&s.doc)).map(str::to_string);
    *s.md_file.cached_exact.borrow_mut() = Some((s.rev(), res.clone()));
    res
}

/// What Save writes to the Markdown file `path`, in place of the document written out again: the
/// file's own text, while it stands for the document. Not for a Print Markup copy, which adds
/// the comments.
pub(crate) fn text_to_save(s: &Session, path: &std::path::Path, markup: bool) -> Option<String> {
    if markup || !is_markdown_file(path) {
        return None;
    }
    exact_text(s)
}

/// Whether a file's name says it is Markdown.
pub fn is_markdown_file(path: &std::path::Path) -> bool {
    path.extension().and_then(std::ffi::OsStr::to_str).is_some_and(|e| e.eq_ignore_ascii_case("md") || e.eq_ignore_ascii_case("markdown"))
}

/// `text` as a document, with what isn't in Markdown from `current`: its page, settings,
/// properties and language, as opening the file gave it.
fn parse(current: &Document, text: &str) -> Document {
    let (mut doc, _) = paper_formats::markdown::import_on_with_lines(text, &current.last_section);
    doc.ensure_nonempty();
    doc.settings = current.settings.clone();
    doc.core = current.core.clone();
    Arc::make_mut(&mut doc.styles).default_chr.lang = current.styles.default_chr.lang.clone();
    doc
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};
    use std::path::PathBuf;

    /// What the document can't hold, all of which a save without formatted edits keeps.
    const OWN: &str = "---\ntitle: Notes\n---\n\nTitle\n=====\n\n<!-- a comment -->\n\n* star bullet\n* another\n\n- [ ] a task\n- [x] done\n\n```bash\necho hi\n```\n\nSome *emphasis* and a [link](https://example.com \"its title\").\nA wrapped\nline.\n\n![a picture](pic.png)\n\nLast paragraph.\n";

    static TEST_DIR_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

    /// Generates a unique temporary directory for test file isolation.
    fn unique_temp_dir(tag: &str) -> PathBuf {
        let n = TEST_DIR_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_nanos());
        std::env::temp_dir().join(format!("paper-md-{tag}-{}-{n}-{nanos}", std::process::id()))
    }

    /// A scratch folder holding `name` with `text`, opened in a new session.
    fn open_file(dir: &str, name: &str, text: &[u8]) -> (Session, PathBuf) {
        let dir = unique_temp_dir(&format!("own-{dir}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(name);
        std::fs::write(&path, text).unwrap();
        let mut s = Session::new(Document::from_text(""));
        s.run("file.open", &json!({"path": path.to_string_lossy()})).unwrap();
        (s, path)
    }

    /// Saves the session document via `file.save`.
    fn save(s: &mut Session) -> Value {
        s.run("file.save", &json!({})).unwrap()
    }

    /// A scratch folder holding `name`, opened from the bytes `file.open` is handed (`data`, as a
    /// platform hands a file over). The path holds other bytes, so a save that left them alone
    /// would show.
    fn open_handed_over(dir: &str, name: &str, text: &[u8]) -> (Session, PathBuf) {
        let dir = unique_temp_dir(&format!("handed-{dir}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(name);
        std::fs::write(&path, b"not the opened text").unwrap();
        let mut s = Session::new(Document::from_text(""));
        let data = crate::cmd::insert::base64_encode(text);
        s.run("file.open", &json!({"path": path.to_string_lossy(), "data": data})).unwrap();
        (s, path)
    }

    /// Verifies a Markdown file opened from handed-over bytes keeps its own text and saves it back
    /// byte-for-byte, as one opened from its path does.
    #[test]
    fn a_file_opened_from_handed_over_bytes_keeps_its_own_text() {
        for (dir, text) in [("plain", OWN.to_string()), ("crlf", OWN.replace('\n', "\r\n"))] {
            let (mut s, path) = open_handed_over(dir, "own.md", text.as_bytes());
            assert!(!s.md_file.is_empty(), "{dir}: the opened text must be kept");
            assert_eq!(save(&mut s)["rewritten"], false, "{dir}");
            assert_eq!(std::fs::read(&path).unwrap(), text.as_bytes(), "{dir}");
            done(&path);
        }
    }

    /// Removes the temporary test folder created for `path`.
    fn done(path: &std::path::Path) {
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    /// Verifies unedited Markdown files are written back byte-for-byte upon save.
    #[test]
    fn a_file_saved_without_edits_is_written_back_byte_for_byte() {
        for (dir, text) in [("plain", OWN.to_string()), ("crlf", OWN.replace('\n', "\r\n")), ("bom", format!("\u{FEFF}{OWN}"))] {
            let (mut s, path) = open_file(dir, "own.md", text.as_bytes());
            assert_eq!(save(&mut s)["rewritten"], false, "{dir}");
            assert_eq!(std::fs::read_to_string(&path).unwrap(), text, "{dir}");
            done(&path);
        }
    }

    /// Verifies BOM-prefixed Markdown files retain original bytes when saved without edits.
    #[test]
    fn a_bom_prefixed_file_saved_without_edits_retains_its_original_bytes() {
        let bom = [b"\xEF\xBB\xBF".as_slice(), OWN.as_bytes()].concat();
        let (mut s, path) = open_file("bom-exact", "own.md", &bom);
        assert!(!s.md_file.is_empty(), "BOM file text must be kept");
        assert_eq!(save(&mut s)["rewritten"], false);
        let read = std::fs::read(&path).unwrap();
        assert_eq!(read, bom, "Saved bytes must match original BOM bytes byte-for-byte");
        done(&path);
    }

    /// Verifies formatted edits force re-exporting the document on save with a warning.
    #[test]
    fn an_edit_writes_the_document_out_again_once_with_a_warning() {
        let (mut s, path) = open_file("formatted", "own.md", OWN.as_bytes());
        s.run("text.insert", &json!({"text": "New "})).unwrap();
        assert_eq!(save(&mut s)["rewritten"], true);
        let written = std::fs::read_to_string(&path).unwrap();
        assert!(written.contains("New "), "{written}");
        assert!(!written.contains("<!-- a comment -->"), "written out again: {written}");
        s.run("text.insert", &json!({"text": "Again "})).unwrap();
        assert_eq!(save(&mut s)["rewritten"], false, "warned once: the file holds the document's own text now");
        done(&path);
    }

    /// Verifies files that cannot go back as they were are written out without rewriting warnings.
    #[test]
    fn files_that_cannot_go_back_as_they_were_are_written_out_without_a_warning() {
        let latin1 = b"Caf\xE9 au lait.\n".to_vec();
        for (dir, bytes) in [("latin1", latin1), ("empty", Vec::new())] {
            let (mut s, path) = open_file(dir, "own.md", &bytes);
            assert!(s.md_file.is_empty(), "{dir}");
            s.run("text.insert", &json!({"text": "New "})).unwrap();
            assert_eq!(save(&mut s)["rewritten"], false, "{dir}");
            done(&path);
        }
        // A Word document saved as Markdown is a copy written out as always.
        let mut s = Session::new(Document::from_text("Hello"));
        let dir = unique_temp_dir("own-docx");
        std::fs::create_dir_all(&dir).unwrap();
        s.path = Some(dir.join("a.docx"));
        let r = s.run("file.saveAs", &json!({"path": dir.join("a.md").to_string_lossy()})).unwrap();
        assert_eq!(r["rewritten"], false);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Verifies undoing back to original kept text restores original formatting.
    #[test]
    fn undoing_back_to_a_kept_text_finds_it_again() {
        let (mut s, path) = open_file("undo", "own.md", OWN.as_bytes());
        s.run("text.insert", &json!({"text": "New "})).unwrap();
        s.run("edit.undo", &json!({})).unwrap();
        assert_eq!(save(&mut s)["rewritten"], false);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), OWN);
        done(&path);
    }
}
