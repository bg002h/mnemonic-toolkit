//! `~/.mnemonic/mt.conf` — a small, hand-rolled INI store for machine-measured
//! tuning that cannot be known at compile time.
//!
//! **Why a file at all.** The search's optimal thread count is a property of the
//! machine, not of the code: measured on an i7-13700K (8 P-cores + 8 E-cores, 24
//! logical) the best count was **20** — neither the logical count (24, which ran
//! 25% SLOWER) nor the physical count (16). It is not derivable from any rule,
//! so it must be measured; and measuring on every invocation would tax every
//! short search to benefit the rare long one. So: measure once, record, reuse.
//!
//! **Why hand-rolled.** Adding `toml`/`dirs` would pull new crates into a
//! vendored, reproducible build for ~60 lines of parsing. INI with `[section]`
//! headers covers exactly what is needed and nothing more.
//!
//! **Failure is never fatal.** Every operation here degrades to "no config":
//! an unreadable, unwritable, corrupt or absent file must never stop a wallet
//! recovery. A tuning knob that can refuse to run is worse than no knob.

use std::collections::BTreeMap;
use std::path::PathBuf;

/// `~/.mnemonic` — the directory, resolved per-platform.
///
/// Unix uses `$HOME`; Windows uses `$USERPROFILE` (and falls back to
/// `$HOMEDRIVE$HOMEPATH`, which is what cmd.exe sets). Returns `None` rather
/// than guessing when none is set, so a daemon-like environment silently skips
/// the config instead of writing somewhere surprising.
pub fn config_dir() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")
        .filter(|h| !h.is_empty())
        .or_else(|| std::env::var_os("USERPROFILE").filter(|h| !h.is_empty()))
        .or_else(|| {
            let drive = std::env::var_os("HOMEDRIVE")?;
            let path = std::env::var_os("HOMEPATH")?;
            if drive.is_empty() || path.is_empty() {
                return None;
            }
            let mut s = drive;
            s.push(&path);
            Some(s)
        })?;
    Some(PathBuf::from(home).join(".mnemonic"))
}

/// `~/.mnemonic/mt.conf`.
pub fn config_path() -> Option<PathBuf> {
    Some(config_dir()?.join("mt.conf"))
}

/// A parsed INI document: section → (key → value), both order-stable.
pub type Ini = BTreeMap<String, BTreeMap<String, String>>;

/// Parse the INI subset used here: `[section]` headers, `key = value` pairs,
/// `#`/`;` comments, blank lines. Unknown syntax is SKIPPED rather than
/// rejected — a file a future version wrote must not brick an older binary.
pub fn parse_ini(text: &str) -> Ini {
    let mut out: Ini = BTreeMap::new();
    let mut section = String::new();
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        if let Some(rest) = line.strip_prefix('[') {
            if let Some(name) = rest.strip_suffix(']') {
                section = name.trim().to_string();
            }
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            out.entry(section.clone())
                .or_default()
                .insert(k.trim().to_string(), v.trim().to_string());
        }
    }
    out
}

/// Render an [`Ini`] back to text, with a header explaining what wrote it.
pub fn render_ini(doc: &Ini) -> String {
    let mut s = String::from(
        "# mnemonic-toolkit configuration\n\
         #\n\
         # Machine-measured tuning. Safe to delete: the tool re-measures and\n\
         # rewrites it. Safe to edit: values here win over measurement.\n\
         # Re-measure explicitly with `--recalibrate-threads`.\n",
    );
    for (section, kv) in doc {
        if section.is_empty() {
            continue;
        }
        s.push_str(&format!("\n[{section}]\n"));
        for (k, v) in kv {
            s.push_str(&format!("{k} = {v}\n"));
        }
    }
    s
}

/// Read and parse the config, or an empty document on any failure.
pub fn load() -> Ini {
    let Some(p) = config_path() else {
        return Ini::new();
    };
    match std::fs::read_to_string(&p) {
        Ok(t) => parse_ini(&t),
        Err(_) => Ini::new(),
    }
}

/// Merge `values` into `[section]` and write the file back.
///
/// Returns the path written, or `None` if anything failed — the caller reports
/// that as a note, never as an error.
pub fn store(section: &str, values: &[(&str, String)]) -> Option<PathBuf> {
    let dir = config_dir()?;
    let path = config_path()?;
    let mut doc = load();
    let entry = doc.entry(section.to_string()).or_default();
    for (k, v) in values {
        entry.insert((*k).to_string(), v.clone());
    }
    std::fs::create_dir_all(&dir).ok()?;
    std::fs::write(&path, render_ini(&doc)).ok()?;
    Some(path)
}

/// Read a `usize` from `[section] key`, if present and sane.
pub fn get_usize(doc: &Ini, section: &str, key: &str) -> Option<usize> {
    doc.get(section)?.get(key)?.parse::<usize>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_sections_and_values() {
        let mut doc = Ini::new();
        doc.entry("search".into())
            .or_default()
            .insert("threads".into(), "20".into());
        let text = render_ini(&doc);
        let back = parse_ini(&text);
        assert_eq!(get_usize(&back, "search", "threads"), Some(20));
    }

    #[test]
    fn unknown_syntax_is_skipped_not_rejected() {
        // A file a FUTURE version wrote must not brick an older binary: it reads
        // what it understands and ignores the rest.
        let text = "\
# comment
; also a comment
[search]
threads = 12
future_key = {nested = thing}
[unknown-section]
whatever = 1

[search]
extra = 7
";
        let doc = parse_ini(text);
        assert_eq!(get_usize(&doc, "search", "threads"), Some(12));
        assert_eq!(get_usize(&doc, "search", "extra"), Some(7));
        assert!(doc.contains_key("unknown-section"));
    }

    #[test]
    fn garbage_yields_no_value_rather_than_a_panic() {
        for text in ["", "[", "]", "=", "threads = ", "threads = -4", "\0\0\0"] {
            let doc = parse_ini(text);
            assert_eq!(
                get_usize(&doc, "search", "threads"),
                None,
                "garbage {text:?} must not yield a thread count"
            );
        }
    }

    #[test]
    fn a_value_outside_a_section_is_not_attributed_to_one() {
        let doc = parse_ini("threads = 99\n[search]\nthreads = 4\n");
        assert_eq!(get_usize(&doc, "search", "threads"), Some(4));
    }
}
