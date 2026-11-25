//! Embed the craft-i18n interface catalogs and the craft-fonts faces. Nothing is read at run time.
//!
//! - `CRAFT_I18N_DIR`, a craft-i18n checkout (storytold/craft-i18n, at the SHA in
//!   `craft-i18n.rev`): `craft_i18n.rs` holds `CRAFT_I18N`, one (locale, shared catalog, Paper
//!   catalog) entry per locale that has a `wordcraft.tsv`. Unset, or not a checkout (a warning),
//!   it is empty and the interface is English only; with `CRAFT_I18N_REQUIRED=1` (releases) that
//!   is an error instead. Recipe: craft-i18n's `docs/integration.md`. The catalogs are embedded with
//!   the app's name as Paper (see [`paper_catalog`]).

use std::fmt::Write as _;

fn main() {
    let out = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap_or_default());
    let (table, catalogs) = craft_i18n();
    let out = out.join("craft_i18n.rs");
    if let Err(e) = std::fs::write(&out, table) {
        println!("cargo::error=writing {}: {e}", out.display());
    }
    embed_fonts(catalogs);
}

/// Write `craft_fonts.rs`: the craft-fonts faces (from `CRAFT_FONTS_DIR`) with the CJK interface faces
/// cut to the characters of the embedded catalogs. A web build has a size budget, so it embeds only
/// the Japanese interface face.
fn embed_fonts(catalogs: Vec<std::path::PathBuf>) {
    println!("cargo::rerun-if-changed=build.rs");
    let wasm = craft_fonts_build::target_is_wasm();
    // The web build embeds only the cut Japanese interface face, which needs a craft-fonts checkout whose
    // manifest marks interface faces (`role` `ui`); with an older one it embeds none and falls back to the
    // Latin faces, so `CRAFT_FONTS_REQUIRED` binds the native build only.
    let embed = craft_fonts_build::Embed::new().subset_to(catalogs).filter(move |face| !wasm || face.has_script("Jpan"));
    let embed = if wasm { embed.required(false) } else { embed };
    let embedded = embed.write("craft_fonts.rs");
    if let Err(e) = embedded {
        println!("cargo::error={e}");
    }
}

/// The catalog this app reads in each locale: `locales/<locale>/wordcraft.tsv`.
const APP_CATALOG: &str = "wordcraft.tsv";

/// `CRAFT_I18N` from `CRAFT_I18N_DIR`: the `Embedded` list the craft-i18n crate reads, and the
/// catalog files it names (`wordcraft.tsv` and `common.tsv`).
fn craft_i18n() -> (String, Vec<std::path::PathBuf>) {
    println!("cargo::rerun-if-env-changed=CRAFT_I18N_DIR");
    println!("cargo::rerun-if-env-changed=CRAFT_I18N_REQUIRED");
    let required = std::env::var("CRAFT_I18N_REQUIRED").is_ok_and(|v| v.trim() == "1");
    let entries = match std::env::var_os("CRAFT_I18N_DIR").filter(|d| !d.is_empty()) {
        // Not canonicalized: on Windows that gives `\\?\C:\…`, which `include_str!` can't read.
        Some(dir) => match locales(&std::path::PathBuf::from(dir).join("locales")) {
            Ok(entries) => entries,
            Err(e) => {
                let level = if required { "error" } else { "warning" };
                println!("cargo::{level}=CRAFT_I18N_DIR: {e}; the interface is English only");
                Vec::new()
            }
        },
        None => {
            if required {
                println!("cargo::error=CRAFT_I18N_REQUIRED=1 but CRAFT_I18N_DIR is unset: a release must embed the translations");
            }
            Vec::new()
        }
    };
    let mut o = String::from("/// (locale, shared catalog, Paper's catalog), one entry per locale; `\"\"` for a missing file.\n");
    o.push_str("pub static CRAFT_I18N: &craft::i18n::Embedded = &[\n");
    let mut catalogs = Vec::new();
    for (code, shared, app) in entries {
        let had_shared = shared.is_some();
        let (shared, Some(app)) = (shared.and_then(|p| paper_catalog(&code, &p)), paper_catalog(&code, &app)) else {
            println!("cargo::error=writing the {code} catalogs to OUT_DIR failed");
            continue;
        };
        if had_shared && shared.is_none() {
            println!("cargo::error=writing the {code} shared catalog to OUT_DIR failed");
        }
        catalogs.extend(shared.iter().cloned());
        catalogs.push(app.clone());
        // `{:?}` escapes the path (Windows backslashes) as a Rust string literal.
        let shared = shared.map_or_else(|| "\"\"".to_string(), |p| format!("include_str!({p:?})"));
        let _ = writeln!(o, "    ({code:?}, {shared}, include_str!({app:?})),");
    }
    o.push_str("];\n");
    (o, catalogs)
}

/// A locale's code, its `common.tsv` if any, and its app catalog.
type Locale = (String, Option<std::path::PathBuf>, std::path::PathBuf);

/// Every locale under `dir` with this app's catalog, by code: (code, its `common.tsv` if any, its
/// app catalog). An error when `dir` isn't a craft-i18n `locales/` directory.
fn locales(dir: &std::path::Path) -> Result<Vec<Locale>, String> {
    // Build scripts run in the crate's directory and `include_str!` resolves against `OUT_DIR`, so
    // a relative path would mean two different places.
    if !dir.is_absolute() {
        return Err(format!("{} is not an absolute path", dir.display()));
    }
    let list = std::fs::read_dir(dir).map_err(|e| format!("{} is not a craft-i18n checkout ({e})", dir.display()))?;
    println!("cargo::rerun-if-changed={}", dir.display());
    let mut out = Vec::new();
    for entry in list.flatten() {
        let path = entry.path();
        let Some(code) = path.file_name().and_then(std::ffi::OsStr::to_str).map(str::to_string) else { continue };
        let app = path.join(APP_CATALOG);
        if !app.is_file() {
            continue;
        }
        println!("cargo::rerun-if-changed={}", app.display());
        let shared = Some(path.join("common.tsv")).filter(|p| p.is_file());
        if let Some(p) = &shared {
            println!("cargo::rerun-if-changed={}", p.display());
        }
        out.push((code, shared, app));
    }
    if out.is_empty() {
        return Err(format!("{} has no locale with a {APP_CATALOG}", dir.display()));
    }
    out.sort();
    Ok(out)
}

/// `catalog` (of locale `code`) copied to `OUT_DIR` with Paper, the engine's name, as Paper, in the
/// English text and the translations alike, so the renamed interface strings keep their translations.
/// `None` when the copy can't be written.
fn paper_catalog(code: &str, catalog: &std::path::Path) -> Option<std::path::PathBuf> {
    let text = std::fs::read_to_string(catalog).ok()?;
    let dir = std::path::PathBuf::from(std::env::var_os("OUT_DIR")?).join("i18n").join(code);
    std::fs::create_dir_all(&dir).ok()?;
    let out = dir.join(catalog.file_name()?);
    std::fs::write(&out, text.replace("WordCraft", "Paper")).ok()?;
    Some(out)
}
