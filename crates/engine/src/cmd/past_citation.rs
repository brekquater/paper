//! Citations › Move Past Citation.

use paper_doc::para::OBJ;
use paper_doc::{InlineObject, Pos};
use serde_json::Value;

use super::sel_result;
use crate::{CmdError, CmdResult, CommandSpec, Session};

pub fn specs() -> Vec<CommandSpec> {
    vec![CommandSpec::new("caret.pastCitation", "Move Past Citation", "Citations › Citations", past_citation).pure()]
}

/// Is this field code a citation or a bibliography (Zotero, Mendeley, EndNote, Word)?
pub fn is_citation_code(instr: &str) -> bool {
    let code = instr.trim_start();
    let word = code.split_whitespace().next().unwrap_or("").to_ascii_uppercase();
    if word == "CITATION" || word == "BIBLIOGRAPHY" {
        return true;
    }
    if word != "ADDIN" {
        return false;
    }
    let rest = code.get(5..).unwrap_or("").trim_start().to_ascii_uppercase();
    ["ZOTERO_ITEM", "ZOTERO_BIBL", "CSL_CITATION", "CSL_BIBLIOGRAPHY", "MENDELEY", "EN.CITE", "EN.REFLIST"].iter().any(|k| rest.starts_with(k))
}

/// Move the caret to just after the citation it's in or before (a run of adjacent citations
/// counts as one); with no such citation in the paragraph, after the next one in the document.
fn past_citation(s: &mut Session, _: &Value) -> CmdResult {
    let caret = s.sel.ordered().1;
    let story = caret.story;
    let obj = OBJ.len_utf8();
    // Every citation as (first position, position just after it).
    let mut spans: Vec<(Pos, Pos)> = s
        .doc
        .field_ranges(story)
        .into_iter()
        .filter(|r| is_citation_code(&r.instr))
        .map(|r| (r.start, Pos::new(story, r.end.path.clone(), r.end.off + obj)))
        .collect();
    for path in s.doc.para_paths(story) {
        let Some(para) = s.doc.para(story, &path) else { continue };
        for off in para.object_offsets() {
            if let Some(InlineObject::Field { instr, .. }) = para.object_at(off)
                && is_citation_code(instr)
            {
                spans.push((Pos::new(story, path.clone(), off), Pos::new(story, path.clone(), off + obj)));
            }
        }
    }
    spans.sort();
    // The citation the caret is in or right before, else the next one.
    let Some(mut i) = spans.iter().position(|(_, end)| *end > caret) else {
        return Err(CmdError::Failed("there's no citation after the caret".into()));
    };
    let mut after = spans.get(i).map(|x| x.1.clone()).unwrap_or(caret);
    // Swallow citations that follow directly ("(A)(B)" or "(A) (B)").
    while let Some((start, end)) = spans.get(i + 1) {
        let gap = if start.path == after.path { s.doc.para(story, &after.path).and_then(|p| p.text.get(after.off..start.off)) } else { None };
        if !gap.is_some_and(|g| g.chars().all(|c| c.is_whitespace() || c == ';' || c == ',')) {
            break;
        }
        after = end.clone();
        i += 1;
    }
    s.sel = crate::Selection::caret(s.doc.clamp(&after));
    s.goal_x = None;
    sel_result(s)
}

#[cfg(test)]
mod tests {
    use paper_doc::props::CharProps;
    use paper_doc::{Document, InlineObject, Path, Pos, StoryRef};
    use serde_json::json;

    use crate::Session;

    fn cite(d: &mut Document, block: usize, off: usize, text: &str) {
        let c = CharProps::default();
        let p = d.para_mut(StoryRef::Body, &Path::top(block)).unwrap();
        p.insert_object(off, InlineObject::FieldStart { instr: "ADDIN ZOTERO_ITEM CSL_CITATION {}".into(), locked: false, dirty: false }, &c)
            .unwrap();
        let n = p.insert_text(off + 3, text, &c).unwrap();
        p.insert_object(off + 3 + n, InlineObject::FieldEnd, &c).unwrap();
    }

    /// "Cats purr (Smith 2020). Dogs bark (Jones 2021; Lee 2019).\nReferences\nSmith J. Cats.\nJones K. Dogs."
    /// with Zotero citations and a Zotero bibliography over the last two paragraphs.
    fn doc() -> Document {
        let mut d = Document::from_text("Cats purr . Dogs bark .\nReferences\nSmith J. Cats.\nJones K. Dogs.");
        cite(&mut d, 0, 22, "(Jones 2021; Lee 2019)");
        cite(&mut d, 0, 10, "(Smith 2020)");
        let c = CharProps::default();
        d.para_mut(StoryRef::Body, &Path::top(2))
            .unwrap()
            .insert_object(0, InlineObject::FieldStart { instr: "ADDIN ZOTERO_BIBL {} CSL_BIBLIOGRAPHY".into(), locked: false, dirty: false }, &c)
            .unwrap();
        let p3 = d.para_mut(StoryRef::Body, &Path::top(3)).unwrap();
        let end = p3.len();
        p3.insert_object(end, InlineObject::FieldEnd, &c).unwrap();
        d
    }

    #[test]
    fn move_past_citation() {
        let mut s = Session::new(doc());
        // Before the first citation → just after it.
        s.sel = crate::Selection::caret(Pos::body(0, 5));
        s.run("caret.pastCitation", &json!({})).unwrap();
        let first_end = s.sel.focus.clone();
        let p = s.doc.para_at(&first_end).unwrap();
        assert_eq!(p.text.get(first_end.off..), Some(". Dogs bark \u{FFFC}(Jones 2021; Lee 2019)\u{FFFC}."));
        // Inside the second → after it.
        let inside = p.text.find("Lee").unwrap();
        s.sel = crate::Selection::caret(Pos::body(0, inside));
        s.run("caret.pastCitation", &json!({})).unwrap();
        let p = s.doc.para_at(&s.sel.focus).unwrap();
        assert_eq!(p.text.get(s.sel.focus.off..), Some("."));
        // After the last citation in the body text: on to the end of the bibliography.
        s.run("caret.pastCitation", &json!({})).unwrap();
        assert_eq!(s.sel.focus.path, Path::top(3));
        assert!(s.run("caret.pastCitation", &json!({})).is_err());
    }

    #[test]
    fn adjacent_citations_are_skipped_together() {
        let mut d = Document::from_text("A  b.");
        cite(&mut d, 0, 2, "(Lee)");
        cite(&mut d, 0, 1, "(Kim)");
        let mut s = Session::new(d);
        s.sel = crate::Selection::caret(Pos::body(0, 1));
        s.run("caret.pastCitation", &json!({})).unwrap();
        let p = s.doc.para_at(&s.sel.focus).unwrap();
        assert_eq!(p.text.get(s.sel.focus.off..), Some(" b."));
    }
}
