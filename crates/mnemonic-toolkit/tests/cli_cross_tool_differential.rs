//! Stress Cycle D — cross-tool md1 DIFFERENTIAL harness (toolkit vs md-cli).
//!
//! Two tools turn a descriptor STRING into an md1 card via near-identical
//! hand-written walkers (`mnemonic`'s `parse_descriptor` and md-cli's
//! `parse_template`), but they are NOT cross-checked, so they can silently
//! disagree — engraving DIFFERENT cards for the SAME wallet (an interop
//! hazard: a card made by `mnemonic` won't match one made by `md` for the
//! same descriptor; a third party reconstructing may get a different
//! `wallet_policy_id`).
//!
//! This test runs BOTH binaries on a curated descriptor corpus and compares
//! their `wallet_policy_id` + `wallet_descriptor_template_id` (decoded from
//! each tool's md1 via `md inspect --json`). Each corpus entry declares its
//! EXPECTED verdict; the test fails if the ACTUAL verdict differs — a
//! known-Diverge starting to Match means the canonicity fix landed (update
//! the table); a known-Match diverging is a regression.
//!
//! It pins the ONE known divergence: the toolkit keeps `Tag::Check(PkK/PkH)`
//! in wsh/sh (gated on `tap_context` at parse_descriptor.rs:601-602, the
//! deliberate test `walk_check_kept_in_non_tap_context`:2551), whereas
//! descriptor-mnemonic SPEC v0.30 §5.1 mandates BARE `PkK`/`PkH` regardless
//! of context. md-cli is conformant; the toolkit is the deviant. See FOLLOWUP
//! `toolkit-check-pkk-non-tap-non-canonical` (both repos). Both md1 forms
//! DECODE to the same descriptor — this is a WIRE-canonicity divergence, an
//! interop hazard, not funds-loss.
//!
//! Design + 2 R0 rounds (GREEN 0C/0I):
//!   design/BRAINSTORM_stress_cycle_d_cross_tool_differential.md
//!   design/agent-reports/cycle-d-differential-r0-round{1,2}-review.md
//!
//! GATING: `#[ignore]` by default — it needs BOTH compiled binaries. Run with
//!   MNEMONIC_BIN=/path/to/mnemonic MD_BIN=/path/to/md \
//!     cargo test -p mnemonic-toolkit --test cli_cross_tool_differential \
//!       -- --ignored --nocapture
//! `MNEMONIC_BIN` defaults to the cargo-built `mnemonic` (CARGO_BIN_EXE_mnemonic)
//! when unset; `MD_BIN` must be provided (no in-workspace build) or the test
//! skips with a clear message.

use serde_json::Value;
use std::process::Command;

// ── FROZEN, DEPTH-MATCHED xpub literals [m5] ──────────────────────────────
// Derived (raw BIP-32) from the canonical abandon×11 about BIP-39 phrase,
// master fingerprint 73c5da0a. md-cli enforces BIP-32 depth per script
// context (depth-3 for wpkh/pkh/SingleSig, depth-4 for wsh/MultiSig), and
// `mnemonic convert` cannot derive an arbitrary depth-4 path — so these are
// shipped as frozen literals (deterministic). The depth-4 key reproduces the
// R0 evidence (`xpub6DkFAXWQ2dHxq…r6KFrf`, mfp 73c5da0a, m/48'/0'/0'/2').
const FP: &str = "73c5da0a";
// depth-3 (single-sig contexts)
const XPUB3_84: &str = "xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V"; // m/84'/0'/0'
const XPUB3_44: &str = "xpub6BosfCnifzxcFwrSzQiqu2DBVTshkCXacvNsWGYJVVhhawA7d4R5WSWGFNbi8Aw6ZRc1brxMyWMzG3DSSSSoekkudhUd9yLb6qx39T9nMdj"; // m/44'/0'/0'
// depth-4 (wsh / multisig contexts), two DISTINCT keys (different accounts)
const XPUB4_0: &str = "xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf"; // m/48'/0'/0'/2'
const XPUB4_1: &str = "xpub6DzhyrnFFYQ1HimDiM388xHnDiRPNdZJFBmmxge3Y1WWcHLtMJLfRuhRHqnQCPbTj3fGKTuKFLHzzwpJkp5Dtc3UtLKZKaVZe1yqMBXd6Vk"; // m/48'/0'/1'/2'

/// Four-arm verdict [I3] — a corpus entry is Match/Diverge ONLY when BOTH
/// tools exit 0 AND emit a parseable md1 whose ids `md inspect` can read.
/// Otherwise the entry is BothError / ToolError(which) — NEVER silently Match.
#[derive(Debug, PartialEq, Eq, Clone)]
enum Verdict {
    /// Both ids (policy + template) equal.
    Match,
    /// At least one id differs (a genuine walker divergence).
    Diverge,
    /// Both tools failed to produce a parseable, inspectable md1.
    BothError,
    /// Exactly one tool failed.
    ToolError(Tool),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Tool {
    Toolkit,
    MdCli,
}

/// The two ids `md inspect --json` reports for an md1 [m1].
#[derive(Debug, Clone, PartialEq, Eq)]
struct Ids {
    policy_id: String,
    template_id: String,
}

/// md-cli `--key` triple (one cosigner) — a BARE depth-N xpub (no `[fp/path]`
/// bracket; a bracket → base58 decode error). The fingerprint + path are
/// supplied separately. All keys in an entry share one common origin path
/// (md-cli's `--path` is a single shared value).
struct MdKey {
    placeholder: &'static str,
    xpub: &'static str,
}

/// One corpus entry: the toolkit concrete descriptor, the md-cli
/// template+keys+path that reconstruct the SAME wallet, and the expected
/// verdict.
struct Entry {
    label: &'static str,
    /// Toolkit input: ONE concrete descriptor with a mandatory `[fp/path]xpub`
    /// bracket per key.
    toolkit_descriptor: String,
    /// md-cli input: `@N`-form template.
    md_template: &'static str,
    /// md-cli per-key bare xpubs.
    md_keys: Vec<MdKey>,
    /// md-cli shared origin path (applied to every key via `--path`).
    md_path: &'static str,
    expect: Verdict,
}

/// Resolve `mnemonic` (env `MNEMONIC_BIN`, else the cargo-built binary).
fn mnemonic_bin() -> String {
    std::env::var("MNEMONIC_BIN").unwrap_or_else(|_| env!("CARGO_BIN_EXE_mnemonic").to_string())
}

/// Resolve `md` (env `MD_BIN`); `None` → skip (there is no in-workspace md).
fn md_bin() -> Option<String> {
    std::env::var("MD_BIN").ok()
}

/// Run a command, returning (exit-success, stdout). stderr is discarded
/// (the toolkit's secret-on-argv advisory goes to stderr; we capture stdout
/// only).
fn run(cmd: &str, args: &[String]) -> (bool, String) {
    let out = Command::new(cmd)
        .args(args)
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn {cmd}: {e}"));
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
    )
}

/// Feed an md1 (one or more chunks as SEPARATE positional args [m2]) to
/// `md inspect --json` and read `wallet_policy_id.hex` /
/// `wallet_descriptor_template_id.hex` [m1]. `None` on any failure.
fn inspect_ids(md: &str, chunks: &[String]) -> Option<Ids> {
    let mut args = vec!["inspect".to_string(), "--json".to_string()];
    args.extend(chunks.iter().cloned());
    let (ok, stdout) = run(md, &args);
    if !ok {
        return None;
    }
    let v: Value = serde_json::from_str(&stdout).ok()?;
    let policy_id = v
        .get("wallet_policy_id")?
        .get("hex")?
        .as_str()?
        .to_string();
    let template_id = v
        .get("wallet_descriptor_template_id")?
        .get("hex")?
        .as_str()?
        .to_string();
    Some(Ids {
        policy_id,
        template_id,
    })
}

/// Toolkit md1 ids: `bundle --descriptor … --network mainnet --json` →
/// stdout JSON `.md1` (a CHUNK ARRAY) → `md inspect`.
fn toolkit_ids(mnemonic: &str, md: &str, descriptor: &str) -> Option<Ids> {
    let args = vec![
        "bundle".to_string(),
        "--descriptor".to_string(),
        descriptor.to_string(),
        "--network".to_string(),
        "mainnet".to_string(),
        "--json".to_string(),
    ];
    let (ok, stdout) = run(mnemonic, &args);
    if !ok {
        return None;
    }
    let v: Value = serde_json::from_str(&stdout).ok()?;
    let chunks: Vec<String> = v
        .get("md1")?
        .as_array()?
        .iter()
        .map(|c| c.as_str().map(str::to_string))
        .collect::<Option<Vec<_>>>()?;
    if chunks.is_empty() {
        return None;
    }
    inspect_ids(md, &chunks)
}

/// md-cli md1 ids: `md encode <@N-template> --key … --fingerprint … --path …
/// --json` → stdout JSON `.phrase` (single string) OR `.chunks` (array) [m3]
/// → `md inspect`.
fn md_cli_ids(md: &str, entry: &Entry) -> Option<Ids> {
    let mut args = vec!["encode".to_string(), entry.md_template.to_string()];
    for k in &entry.md_keys {
        args.push("--key".to_string());
        args.push(format!("{}={}", k.placeholder, k.xpub));
    }
    for k in &entry.md_keys {
        args.push("--fingerprint".to_string());
        args.push(format!("{}={}", k.placeholder, FP));
    }
    args.push("--path".to_string());
    args.push(entry.md_path.to_string());
    args.push("--json".to_string());

    let (ok, stdout) = run(md, &args);
    if !ok {
        return None;
    }
    let v: Value = serde_json::from_str(&stdout).ok()?;
    // m3: `.phrase` single-string for the corpus; `.chunks` for large policies.
    let chunks: Vec<String> = if let Some(p) = v.get("phrase").and_then(|p| p.as_str()) {
        vec![p.to_string()]
    } else if let Some(arr) = v.get("chunks").and_then(|c| c.as_array()) {
        arr.iter()
            .map(|c| c.as_str().map(str::to_string))
            .collect::<Option<Vec<_>>>()?
    } else {
        return None;
    };
    inspect_ids(md, &chunks)
}

/// The oracle [I3][I4]: assign a verdict from each tool's ids. Match iff BOTH
/// ids equal; Diverge iff either differs; BothError/ToolError if a tool failed
/// to produce inspectable ids.
fn classify(tk: Option<Ids>, md: Option<Ids>) -> (Verdict, Option<Ids>, Option<Ids>) {
    let verdict = match (&tk, &md) {
        (Some(a), Some(b)) => {
            if a == b {
                Verdict::Match
            } else {
                Verdict::Diverge
            }
        }
        (None, None) => Verdict::BothError,
        (None, Some(_)) => Verdict::ToolError(Tool::Toolkit),
        (Some(_), None) => Verdict::ToolError(Tool::MdCli),
    };
    (verdict, tk, md)
}

fn key(ph: &'static str, xpub: &'static str) -> MdKey {
    MdKey {
        placeholder: ph,
        xpub,
    }
}

/// The curated corpus. MATCH controls (anti-vacuity) + DIVERGE pins (the known
/// Check(PkK)-in-non-tap finding). Multi-key entries give BOTH cosigners the
/// SAME origin `[73c5da0a/48'/0'/0'/2']` so the toolkit origin metadata
/// matches md-cli's single shared `--path m/48'/0'/0'/2'` [I2].
fn corpus() -> Vec<Entry> {
    let shared4 = format!("[{FP}/48'/0'/0'/2']");
    vec![
        // ── Expect::Match controls (no Check in non-tap; or tap-collapse) ──
        Entry {
            label: "wpkh",
            toolkit_descriptor: format!("wpkh([{FP}/84'/0'/0']{XPUB3_84}/<0;1>/*)"),
            md_template: "wpkh(@0/<0;1>/*)",
            md_keys: vec![key("@0", XPUB3_84)],
            md_path: "m/84'/0'/0'",
            expect: Verdict::Match,
        },
        Entry {
            label: "pkh",
            toolkit_descriptor: format!("pkh([{FP}/44'/0'/0']{XPUB3_44}/<0;1>/*)"),
            md_template: "pkh(@0/<0;1>/*)",
            md_keys: vec![key("@0", XPUB3_44)],
            md_path: "m/44'/0'/0'",
            expect: Verdict::Match,
        },
        Entry {
            label: "wsh-multi-2of2",
            toolkit_descriptor: format!(
                "wsh(multi(2,{shared4}{XPUB4_0}/<0;1>/*,{shared4}{XPUB4_1}/<0;1>/*))"
            ),
            md_template: "wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*))",
            md_keys: vec![key("@0", XPUB4_0), key("@1", XPUB4_1)],
            md_path: "m/48'/0'/0'/2'",
            expect: Verdict::Match,
        },
        Entry {
            // tap context: the toolkit collapses Check(PkK)→bare in tap leaves
            // too (parse_descriptor.rs:519/523 pass tap=true), so it MATCHES.
            label: "tr-pk-leaf",
            toolkit_descriptor: format!(
                "tr({shared4}{XPUB4_0}/<0;1>/*,pk({shared4}{XPUB4_1}/<0;1>/*))"
            ),
            md_template: "tr(@0/<0;1>/*,pk(@1/<0;1>/*))",
            md_keys: vec![key("@0", XPUB4_0), key("@1", XPUB4_1)],
            md_path: "m/48'/0'/0'/2'",
            expect: Verdict::Match,
        },
        // ── Expect::Diverge — the known finding ──────────────────────────
        // Each contains a Check(PkK|PkH) the toolkit KEEPS in non-tap
        // (parse_descriptor.rs:602 `if tap_context` gate + the deliberate test
        // `walk_check_kept_in_non_tap_context`:2551), while md-cli emits bare
        // per descriptor-mnemonic SPEC v0.30 §5.1. FOLLOWUP:
        // toolkit-check-pkk-non-tap-non-canonical. WIRE-canonicity, not
        // funds-loss (both md1s decode to the same descriptor).
        Entry {
            label: "wsh-pk",
            toolkit_descriptor: format!("wsh(pk({shared4}{XPUB4_0}/<0;1>/*))"),
            md_template: "wsh(pk(@0/<0;1>/*))",
            md_keys: vec![key("@0", XPUB4_0)],
            md_path: "m/48'/0'/0'/2'",
            expect: Verdict::Diverge,
        },
        Entry {
            label: "wsh-pkh",
            toolkit_descriptor: format!("wsh(pkh({shared4}{XPUB4_0}/<0;1>/*))"),
            md_template: "wsh(pkh(@0/<0;1>/*))",
            md_keys: vec![key("@0", XPUB4_0)],
            md_path: "m/48'/0'/0'/2'",
            expect: Verdict::Diverge,
        },
        Entry {
            label: "wsh-and_v",
            toolkit_descriptor: format!(
                "wsh(and_v(v:pk({shared4}{XPUB4_0}/<0;1>/*),pk({shared4}{XPUB4_1}/<0;1>/*)))"
            ),
            md_template: "wsh(and_v(v:pk(@0/<0;1>/*),pk(@1/<0;1>/*)))",
            md_keys: vec![key("@0", XPUB4_0), key("@1", XPUB4_1)],
            md_path: "m/48'/0'/0'/2'",
            expect: Verdict::Diverge,
        },
        Entry {
            label: "wsh-or_d",
            toolkit_descriptor: format!(
                "wsh(or_d(pk({shared4}{XPUB4_0}/<0;1>/*),pk({shared4}{XPUB4_1}/<0;1>/*)))"
            ),
            md_template: "wsh(or_d(pk(@0/<0;1>/*),pk(@1/<0;1>/*)))",
            md_keys: vec![key("@0", XPUB4_0), key("@1", XPUB4_1)],
            md_path: "m/48'/0'/0'/2'",
            expect: Verdict::Diverge,
        },
    ]
}

#[test]
#[ignore = "needs both compiled binaries; set MNEMONIC_BIN + MD_BIN (CI: cross-tool-differential.yml)"]
fn cross_tool_md1_differential() {
    let mnemonic = mnemonic_bin();
    let md = match md_bin() {
        Some(m) => m,
        None => {
            eprintln!(
                "SKIP: MD_BIN unset (no in-workspace md binary). \
                 Set MD_BIN=/path/to/md to run this differential."
            );
            return;
        }
    };

    let entries = corpus();

    // Anti-vacuity: the corpus MUST declare at least one Match and one Diverge
    // (else the test could pass while exercising only one verdict path).
    let n_match = entries
        .iter()
        .filter(|e| e.expect == Verdict::Match)
        .count();
    let n_diverge = entries
        .iter()
        .filter(|e| e.expect == Verdict::Diverge)
        .count();
    assert!(
        n_match >= 1 && n_diverge >= 1,
        "corpus must be non-vacuous: at least one Match and one Diverge \
         (got {n_match} Match, {n_diverge} Diverge)"
    );

    let mut failures = Vec::new();
    // Track which verdict arms the run actually EXERCISED (not just declared) —
    // proves the harness ran both tools through to comparable ids on at least
    // one Match and one Diverge entry.
    let mut saw_match = false;
    let mut saw_diverge = false;

    for e in &entries {
        let tk = toolkit_ids(&mnemonic, &md, &e.toolkit_descriptor);
        let md_ids = md_cli_ids(&md, e);
        let (actual, tk_ids, md_ids2) = classify(tk, md_ids);

        match &actual {
            Verdict::Match => saw_match = true,
            Verdict::Diverge => saw_diverge = true,
            _ => {}
        }

        if actual != e.expect {
            failures.push(format!(
                "[{}] EXPECTED {:?} but got {:?}\n      toolkit={:?}\n      md-cli ={:?}\n      descriptor: {}",
                e.label, e.expect, actual, tk_ids, md_ids2, e.toolkit_descriptor
            ));
        } else {
            eprintln!(
                "[{:16}] {:?} OK  toolkit={:?} md-cli={:?}",
                e.label, actual, tk_ids, md_ids2
            );
        }
    }

    assert!(
        failures.is_empty(),
        "cross-tool differential verdict mismatches:\n{}\n\nA known-Diverge \
         starting to Match means the canonicity fix landed → flip its expect \
         to Match (see FOLLOWUP toolkit-check-pkk-non-tap-non-canonical). A \
         known-Match diverging is a REGRESSION in either walker. Unexpected \
         BothError/ToolError is an invocation/corpus bug.",
        failures.join("\n")
    );

    // Non-vacuity, exercised: the run actually reached a real Match AND a real
    // Diverge verdict (both tools produced inspectable ids on each).
    assert!(
        saw_match,
        "harness vacuity: no entry actually produced a Match verdict"
    );
    assert!(
        saw_diverge,
        "harness vacuity: no entry actually produced a Diverge verdict"
    );
}
