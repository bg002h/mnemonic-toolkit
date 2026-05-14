//! `mnemonic gui-schema` subcommand — emit SPEC §7 GUI-overlay schema JSON.
//!
//! Companion to the `mnemonic-gui` v0.2 Phase C.2 contract
//! (`bg002h/mnemonic-gui` `FOLLOWUPS.md` entry `mnemonic-gui-schema-mirror`).
//!
//! Walks the clap `Command` tree via `clap::CommandFactory` and serializes a
//! machine-readable schema of every existing subcommand's flag surface to
//! stdout as JSON. The GUI consumes this schema to render forms; on
//! `version != 1` the GUI refuses to launch.
//!
//! ## SPEC §7 contract
//!
//! ```json
//! {
//!   "version": 1,
//!   "cli": "mnemonic",
//!   "subcommands": [
//!     {
//!       "name": "bundle",
//!       "flags":       [ { "name", "required", "kind", "choices": [..] | null } ],
//!       "positionals": [ { "name", "required", "repeating" } ]
//!     }
//!   ]
//! }
//! ```
//!
//! ## kind mapping
//!
//! | Rust type / clap annotation                          | kind         | choices            |
//! |------------------------------------------------------|--------------|--------------------|
//! | `bool` (`ArgAction::SetTrue`)                        | `boolean`    | null               |
//! | numeric `value_parser` (i64/u32/u64/u8/...)          | `number`     | null               |
//! | enum w/ `value_enum` or `PossibleValuesParser`       | `dropdown`   | array of variants  |
//! | `PathBuf` / `Path`                                   | `path`       | null               |
//! | everything else (`String`, custom value_parsers, …)  | `text`       | null               |
//!
//! The mapping is intentionally lossy for complex GUI variants
//! (NodeValueComposite / TaggedOrIndexed / Range / Timestamp) per the
//! SPEC §7 contract — those collapse to `"text"` upstream and the GUI
//! re-parses client-side.
//!
//! Self-reference is suppressed: the `gui-schema` subcommand itself is
//! filtered out of its own output.

use crate::error::ToolkitError;
use clap::{Args, Command};
use serde::Serialize;
use std::io::Write;

#[derive(Args, Debug)]
pub struct GuiSchemaArgs {}

#[derive(Serialize, Debug)]
struct Schema {
    version: u32,
    cli: String,
    subcommands: Vec<Subcommand>,
}

#[derive(Serialize, Debug)]
struct Subcommand {
    name: String,
    flags: Vec<Flag>,
    positionals: Vec<Positional>,
}

#[derive(Serialize, Debug)]
struct Flag {
    name: String,
    required: bool,
    kind: String,
    choices: Option<Vec<String>>,
}

#[derive(Serialize, Debug)]
struct Positional {
    name: String,
    required: bool,
    repeating: bool,
}

/// Build the SPEC §7 schema from a clap `Command` tree.
///
/// Walks `cmd.get_subcommands()` and, for each subcommand, partitions its
/// arguments into named flags and positionals. The `gui-schema` subcommand
/// is filtered out (self-reference suppression).
///
/// Nested-subcommand flattening (v0.13.0 P2.1): when a subcommand `S` is
/// itself a `#[command(subcommand)]` parent (i.e. its own
/// `get_subcommands()` returns non-empty entries after filtering the
/// auto-generated `help`), its nested sub-subcommands are emitted as
/// hyphenated entries (`S-sub_sub`) IN PLACE OF `S`. This repairs the
/// pre-existing v0.12.0 seed-xor empty-flags rendering (where the
/// per-sub-sub flag tables were invisible to `mnemonic-gui`) and
/// generalizes to v0.13.0 slip39 + any future nested-subcommand parent.
/// Schema `version` stays at 1 — the change is additive at the name set.
fn build_schema(cmd: &Command) -> Schema {
    let mut subs: Vec<Subcommand> = Vec::new();
    for s in cmd
        .get_subcommands()
        .filter(|s| s.get_name() != "gui-schema" && s.get_name() != "help")
    {
        let nested: Vec<&Command> = s
            .get_subcommands()
            .filter(|ss| ss.get_name() != "help")
            .collect();
        if nested.is_empty() {
            subs.push(build_subcommand(s));
        } else {
            for ss in nested {
                let flat = build_subcommand(ss);
                subs.push(Subcommand {
                    name: format!("{}-{}", s.get_name(), ss.get_name()),
                    flags: flat.flags,
                    positionals: flat.positionals,
                });
            }
        }
    }

    // Deterministic ordering by subcommand name (stable across clap versions).
    subs.sort_by(|a, b| a.name.cmp(&b.name));

    Schema {
        version: 1,
        cli: "mnemonic".to_string(),
        subcommands: subs,
    }
}

fn build_subcommand(sub: &Command) -> Subcommand {
    let mut flags: Vec<Flag> = Vec::new();
    let mut positionals: Vec<Positional> = Vec::new();

    for arg in sub.get_arguments() {
        if arg.is_positional() {
            positionals.push(Positional {
                name: arg.get_id().to_string(),
                required: arg.is_required_set(),
                repeating: matches!(
                    arg.get_action(),
                    clap::ArgAction::Append | clap::ArgAction::Count
                ) || arg.get_num_args().is_some_and(|n| n.max_values() > 1),
            });
        } else {
            // Skip the auto-generated --help flag; it's not user surface.
            if arg.get_id().as_str() == "help" {
                continue;
            }
            let name = arg
                .get_long()
                .map(|l| format!("--{l}"))
                .unwrap_or_else(|| arg.get_id().to_string());
            let (kind, choices) = classify_kind(arg);
            flags.push(Flag {
                name,
                required: arg.is_required_set(),
                kind,
                choices,
            });
        }
    }

    // Deterministic ordering: flags by long name, positionals by declaration order.
    flags.sort_by(|a, b| a.name.cmp(&b.name));

    Subcommand {
        name: sub.get_name().to_string(),
        flags,
        positionals,
    }
}

/// Map a clap `Arg` to the SPEC §7 `kind` enum.
///
/// Order matters:
/// 1. boolean (clap `SetTrue` / `SetFalse`) wins before value-parser inspection
///    because flag args have a hidden bool value_parser.
/// 2. `PossibleValuesParser` (or any value_parser exposing `possible_values()`)
///    → dropdown with the enumerated choices.
/// 3. numeric `ValueParser::type_id()` match → `number`.
/// 4. `PathBuf` parser → `path`.
/// 5. fallthrough → `text`.
fn classify_kind(arg: &clap::Arg) -> (String, Option<Vec<String>>) {
    use std::any::TypeId;

    // (1) boolean flag — clap encodes these as ArgAction::SetTrue / SetFalse.
    if matches!(
        arg.get_action(),
        clap::ArgAction::SetTrue | clap::ArgAction::SetFalse
    ) {
        return ("boolean".to_string(), None);
    }

    // (2) dropdown via PossibleValuesParser (used by `#[arg(value_enum)]` and
    // by hand-built PossibleValuesParser arms). `possible_values()` returns
    // `Some(_)` iff the parser is enumeration-bounded.
    let parser = arg.get_value_parser();
    if let Some(pvs) = parser.possible_values() {
        let choices: Vec<String> = pvs.map(|v| v.get_name().to_string()).collect();
        if !choices.is_empty() {
            return ("dropdown".to_string(), Some(choices));
        }
    }

    // (3) numeric: `ValueParser::type_id()` returns an `AnyValueId` that
    // implements `PartialEq<std::any::TypeId>`, so we can match against
    // the std numeric primitives directly.
    let tid = parser.type_id();
    let is_numeric = tid == TypeId::of::<u8>()
        || tid == TypeId::of::<u16>()
        || tid == TypeId::of::<u32>()
        || tid == TypeId::of::<u64>()
        || tid == TypeId::of::<u128>()
        || tid == TypeId::of::<i8>()
        || tid == TypeId::of::<i16>()
        || tid == TypeId::of::<i32>()
        || tid == TypeId::of::<i64>()
        || tid == TypeId::of::<i128>()
        || tid == TypeId::of::<usize>()
        || tid == TypeId::of::<isize>()
        || tid == TypeId::of::<f32>()
        || tid == TypeId::of::<f64>();
    if is_numeric {
        return ("number".to_string(), None);
    }

    // (4) path-like — `PathBuf` is one of the four built-in ValueParserInner
    // variants. We match on type_id rather than the Debug string for stability.
    if tid == TypeId::of::<std::path::PathBuf>() {
        return ("path".to_string(), None);
    }

    // (5) fallthrough — String / custom value_parsers (FromInput, ToInput,
    // SlotInput, XpubPrefix, ...) / complex GUI variants. The GUI re-parses
    // these client-side per the SPEC §7 lossy-mapping contract.
    ("text".to_string(), None)
}

/// Emit the SPEC §7 schema for the supplied clap `Command` tree to `stdout`
/// as a single JSON line (no trailing newline, matching `--json` envelope
/// conventions elsewhere in the toolkit).
pub fn run<W: Write>(
    _args: &GuiSchemaArgs,
    root: &Command,
    stdout: &mut W,
) -> Result<(), ToolkitError> {
    let schema = build_schema(root);
    // Schema is a closed type tree with no untrusted input; serialization is
    // infallible in practice. Match the `.ok()` pattern used by `bundle --json`
    // / `verify-bundle --json` / `convert --json`.
    serde_json::to_writer(&mut *stdout, &schema).ok();
    writeln!(stdout).ok();
    Ok(())
}
