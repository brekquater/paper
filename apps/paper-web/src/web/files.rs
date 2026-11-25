//! Saves go to the host page instead of a download. File › Save, Save As and the exports inside the
//! editor queue the file here; the editor hands each one to the page as a `save` event on its next
//! frame. Opening files and pictures still uses the browser's picker.
//!
//! The queue belongs to one editor, so two editors on a page never see each other's saves.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};

use craft::host::{FileError, Files, PickId, PickRequest, Picked};
use craft::platform::browser_files::BrowserFiles;

/// The name a save gets when the editor gives none.
const DEFAULT_NAME: &str = "Document.docx";

/// A file the editor saved, waiting to reach the page.
pub struct Saved {
    pub name: String,
    /// Whether the file becomes the document's file (Save As, or an export to a format that
    /// keeps everything); `false` for a save over the document's own file or a copy.
    pub save_as: bool,
    pub bytes: Vec<u8>,
}

/// The browser's files, with saves queued for the host page.
pub struct HostedSaves {
    inner: Arc<BrowserFiles>,
    /// Save answers by the id the inner files issued for them.
    answers: Mutex<HashMap<u64, Picked>>,
    queue: Mutex<Vec<Saved>>,
}

impl HostedSaves {
    pub fn new(inner: Arc<BrowserFiles>) -> Self {
        Self { inner, answers: Mutex::default(), queue: Mutex::default() }
    }

    /// The saves since the last call, oldest first.
    pub fn take(&self) -> Vec<Saved> {
        std::mem::take(&mut *lock(&self.queue))
    }
}

/// The lock even after a panic elsewhere poisoned it: the data is a plain queue.
fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

impl Files for HostedSaves {
    fn pick(&self, request: PickRequest) -> PickId {
        self.inner.pick(request)
    }

    fn take_answer(&self, id: PickId) -> Option<Picked> {
        lock(&self.answers).remove(&id.0).or_else(|| self.inner.take_answer(id))
    }

    fn forget(&self, id: PickId) {
        if lock(&self.answers).remove(&id.0).is_none() {
            self.inner.forget(id);
        }
    }

    fn pick_pending(&self) -> bool {
        self.inner.pick_pending()
    }

    fn read(&self, path: &Path) -> Result<Vec<u8>, FileError> {
        self.inner.read(path)
    }

    fn write(&self, path: &Path, bytes: &[u8]) -> Result<(), FileError> {
        self.inner.write(path, bytes)
    }

    fn last_folder(&self, purpose: &str) -> Option<PathBuf> {
        self.inner.last_folder(purpose)
    }

    /// Queue the file for the page and answer at once: the page stores it and reports its own
    /// failures.
    fn save(&self, request: PickRequest, bytes: Vec<u8>) -> PickId {
        let requested = request.file_name.clone().unwrap_or_else(|| DEFAULT_NAME.to_string());
        let name = Path::new(&requested).file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or(requested);
        lock(&self.queue).push(Saved { name: name.clone(), save_as: request.save_as, bytes });
        // Save As makes the queued file the document's file; anything else leaves it as it is.
        let answer = if request.save_as { Picked::Paths(vec![PathBuf::from(&name)]) } else { Picked::Downloaded(name) };
        // The inner files own the id space: a Folder pick is answered at once without a picker,
        // so it only yields an id, which is released and reused for this answer.
        let id = self.inner.pick(PickRequest::folder("hosted-save"));
        self.inner.forget(id);
        lock(&self.answers).insert(id.0, answer);
        id
    }

    fn can_pick(&self) -> bool {
        self.inner.can_pick()
    }

    fn browser_saves(&self) -> bool {
        true
    }
}
