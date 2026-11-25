//! The small questions: a protected file's password and the password a document is saved with.

use craft::ui::dialog::{DialogId, DialogResult, DialogStack, StackDialog, dialog_id};
use craft::ui::patterns::form_dialogs::{PasswordDialog, PasswordPurpose, Secret};
use craft::ui::runtime::env::Label;
use serde_json::{Value, json};

use super::host::{DialogHooks, Settle, open_with};
use crate::WordApp;

/// Opens `password` or `encryptPassword` by name; `None` for any other.
/// `password` reads `name`, `then` and `params`.
pub(super) fn open(app: &mut WordApp, name: &str, params: &Value, parent: Option<DialogId>) -> Option<DialogId> {
    let text = |key: &str| params.get(key).and_then(Value::as_str).unwrap_or("");
    match name {
        "password" => {
            let ask = Ask::Open {
                name: text("name").to_owned(),
                then: text("then").to_owned(),
                params: params.get("params").cloned().unwrap_or(Value::Null),
            };
            Some(open_prompt(app, parent, ask, None))
        }
        "encryptPassword" => Some(open_prompt(app, parent, Ask::Encrypt, None)),
        _ => None,
    }
}

/// What a password prompt is for.
#[derive(Clone)]
enum Ask {
    /// The password of file `name`; `then` runs again with `params` and the password.
    Open { name: String, then: String, params: Value },
    /// The password the document is saved with, or none to remove it.
    Encrypt,
}

/// What a finished password prompt hands back: the typed password and what it was for.
pub(super) struct PasswordReply {
    secret: Secret,
    ask: Ask,
}

/// A password prompt on the dialog host. Its fields never include the typed secret.
pub(super) struct PasswordPrompt {
    ask: Ask,
    error: Option<String>,
}

impl PasswordPrompt {
    fn key_name(&self) -> &'static str {
        match self.ask {
            Ask::Open { .. } => "password",
            Ask::Encrypt => "encryptPassword",
        }
    }
}

impl StackDialog for PasswordPrompt {
    type Result = PasswordReply;

    fn id(&self) -> DialogId {
        dialog_id(self.key_name())
    }

    fn key(&self) -> Option<&str> {
        Some(self.key_name())
    }

    fn show(&mut self, ui: &mut egui::Ui, _: &DialogStack) -> Option<DialogResult<PasswordReply>> {
        let purpose = match &self.ask {
            Ask::Open { name, .. } => PasswordPurpose::open(name.clone()),
            Ask::Encrypt => PasswordPurpose::SetOrClear,
        };
        let mut dialog = PasswordDialog::new(purpose);
        if let Some(error) = &self.error {
            dialog = dialog.error(Label::from(error.clone()));
        }
        let out = dialog.show(ui);
        match (out.answer, out.closed) {
            (Some(answer), _) => Some(DialogResult::Ok(PasswordReply { secret: answer.secret, ask: self.ask.clone() })),
            (None, Some(_)) => Some(DialogResult::Cancel),
            (None, None) => None,
        }
    }

    fn json_fields(&self) -> Option<Value> {
        Some(match &self.ask {
            Ask::Open { name, then, params } => json!({"name": name, "then": then, "params": params, "error": self.error}),
            Ask::Encrypt => json!({"error": self.error}),
        })
    }
}

fn open_prompt(app: &mut WordApp, parent: Option<DialogId>, ask: Ask, error: Option<String>) -> DialogId {
    open_with(app, parent, PasswordPrompt { ask, error }, DialogHooks { prepare: None, settle: settle_password })
}

/// Asks for the password of `name`; once given, `then` runs again with `params` and the password.
pub(crate) fn open_password(app: &mut WordApp, name: &str, then: &str, params: Value) -> DialogId {
    open_prompt(app, None, Ask::Open { name: name.to_owned(), then: then.to_owned(), params }, None)
}

fn settle_password(app: &mut WordApp, _ctx: &egui::Context, id: DialogId) -> Settle {
    let Some(result) = app.dialog_host.take_by_id::<PasswordPrompt>(id) else {
        return Settle::Keep;
    };
    let DialogResult::Ok(PasswordReply { secret, ask }) = result else {
        return Settle::Done;
    };
    match ask {
        Ask::Open { name, then, params } => {
            let mut p = params.clone();
            if let Some(o) = p.as_object_mut() {
                o.insert("password".into(), json!(secret.expose()));
            }
            // `execute`, not `run`: Save Changes was asked when the open began. The command that
            // asked runs again, so a protected template still starts a new document.
            match app.execute(&then, p) {
                Ok(_) => app.ui.backstage = false,
                Err(e) => {
                    let error = if paper_engine::io::wrong_password(&e) {
                        tl!("That password isn't right. Check it (passwords are case-sensitive) and try again.").to_string()
                    } else {
                        e
                    };
                    open_prompt(app, None, Ask::Open { name, then, params }, Some(error));
                }
            }
        }
        Ask::Encrypt => {
            let cleared = secret.expose().is_empty();
            let password = if cleared { Value::Null } else { json!(secret.expose()) };
            match app.run("file.encrypt", json!({"password": password})) {
                Ok(_) => app.status(if cleared {
                    tl!("The password is removed. Save the document to keep the change.")
                } else {
                    tl!("The document will be saved with a password. Save it to protect it.")
                }),
                Err(e) => {
                    open_prompt(app, None, Ask::Encrypt, Some(e));
                }
            }
        }
    }
    Settle::Done
}
