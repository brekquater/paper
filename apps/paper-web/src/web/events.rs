//! Events an editor sends its host page, and the per-frame check that raises them.
//!
//! | Event | Detail |
//! |---|---|
//! | `change` | `{document, revision}`: the document's content or layout revision changed, or another document was opened |
//! | `selectionchange` | `{document, anchor, focus}`: positions as `docs/control-protocol.md` describes them |
//! | `dirty` | `{dirty}`: unsaved changes appeared or went away |
//! | `save` | `{name, saveAs, bytes}`: a file the editor saved (Save, Save As, exports), for the host to store |
//!
//! Listeners run after the frame that raised them (on the microtask queue), never inside it, so a
//! listener may call the editor.

use std::cell::RefCell;

use paper_engine::{Selection, Session};
use serde_json::{Value, json};
use wasm_bindgen::JsValue;

use super::files::Saved;

/// The events `on` accepts.
pub const EVENTS: [&str; 4] = ["change", "selectionchange", "dirty", "save"];

/// One editor's listeners, by event.
#[derive(Default)]
pub struct Listeners(RefCell<Vec<(String, js_sys::Function)>>);

impl Listeners {
    pub fn add(&self, event: &str, listener: js_sys::Function) {
        self.0.borrow_mut().push((event.to_string(), listener));
    }

    /// Remove `listener` from `event`, as `removeEventListener` does.
    pub fn remove(&self, event: &str, listener: &js_sys::Function) {
        self.0.borrow_mut().retain(|(e, f)| !(e == event && js_sys::Object::is(f, listener)));
    }

    pub fn clear(&self) {
        self.0.borrow_mut().clear();
    }

    /// Call `event`'s listeners with `detail` once the current frame has finished.
    pub fn emit(&self, event: &'static str, detail: JsValue) {
        let targets: Vec<js_sys::Function> = self.0.borrow().iter().filter(|(e, _)| e == event).map(|(_, f)| f.clone()).collect();
        if targets.is_empty() {
            return;
        }
        wasm_bindgen_futures::spawn_local(async move {
            for f in targets {
                if let Err(e) = f.call1(&JsValue::NULL, &detail) {
                    log::warn!("paper-web: a `{event}` listener threw: {e:?}");
                }
            }
        });
    }
}

/// What the host last heard about the editor.
#[derive(PartialEq)]
pub struct Seen {
    document: u64,
    revision: u64,
    selection: Selection,
    dirty: bool,
}

impl Seen {
    pub fn of(s: &Session) -> Self {
        Seen { document: s.document_id(), revision: s.rev(), selection: s.sel.clone(), dirty: s.dirty }
    }

    /// Raise the events for what changed between `self` (before the frame) and `now`.
    pub fn report(&self, now: &Seen, listeners: &Listeners) {
        if (now.document, now.revision) != (self.document, self.revision) {
            listeners.emit("change", detail(json!({"document": now.document, "revision": now.revision})));
        }
        if now.document != self.document || now.selection != self.selection {
            let sel = serde_json::to_value(&now.selection).unwrap_or(Value::Null);
            listeners.emit("selectionchange", detail(json!({"document": now.document, "anchor": sel.get("anchor"), "focus": sel.get("focus")})));
        }
        if now.dirty != self.dirty {
            listeners.emit("dirty", detail(json!({"dirty": now.dirty})));
        }
    }
}

/// Hand a saved file to the host page.
pub fn report_save(saved: Saved, listeners: &Listeners) {
    let obj = js_sys::Object::new();
    // `Reflect::set` on a fresh plain object cannot fail.
    let _ = js_sys::Reflect::set(&obj, &"name".into(), &saved.name.into());
    let _ = js_sys::Reflect::set(&obj, &"saveAs".into(), &saved.save_as.into());
    let _ = js_sys::Reflect::set(&obj, &"bytes".into(), &js_sys::Uint8Array::from(saved.bytes.as_slice()).into());
    listeners.emit("save", obj.into());
}

/// A JSON value as a JavaScript object.
fn detail(v: Value) -> JsValue {
    js_sys::JSON::parse(&v.to_string()).unwrap_or(JsValue::NULL)
}
