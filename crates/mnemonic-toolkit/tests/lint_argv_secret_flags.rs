//! argv-leakage closure — a LEADING gate over every secret-bearing argv route.
//!
//! v0.36.2 rebuild (was: a hand-frozen `CANONICAL_FLAG_ROWS` of 28 rows that
//! froze at v0.13.0 and silently omitted ~16 post-v0.13.0 secret-argv routes —
//! see FOLLOWUP `lint-argv-secret-flags-canonical-table-rebuild-from-clap` +
//! `design/IMPLEMENTATION_PLAN_v0_36_2_argv_audit_closure.md`).
//!
//! Two orthogonal guarantees (R0 M3):
//!  - **Completeness closure** (`*_set_equals_gui_schema`): the declared route
//!    set must SET-EQUAL the live surface enumerated from `mnemonic gui-schema`
//!    (+ `secret_taxonomy`). A NEW secret-argv flag/`--from`/`--slot` consumer
//!    makes the live set grow → set-equality fails until acknowledged; a REMOVED
//!    one makes it shrink → also fails (preserves the original removal-detection
//!    intent without a hand-frozen count).
//!  - **Evidence anchor** (`every_route_has_nonargv_channel_evidence`): each
//!    declared route's source file must contain a non-argv-channel anchor —
//!    proving the `*-stdin` / `=-` / `@env:` / refusal route is actually WIRED
//!    in source, not merely named in clap.
//!
//! Three secret-argv axes (R0/R1/R2):
//!  1. flag-NAME axis — gui-schema flags with `secret==true && kind!="boolean"`
//!     (the `secret && boolean` flags are the `*-stdin` toggles = EVIDENCE, not
//!     routes). `flag_is_secret` EXCLUDES `--from`/`--slot` (their names are
//!     generic), so axes 2/3 cannot come from the flag bit.
//!  2. `--from` axis — subcommands whose clap surface declares a `--from` flag
//!     (per-subcommand; the `=-` route is value-uniform — covers every node the
//!     subcommand accepts — so no per-node enumeration is needed, R0 I1).
//!  3. `--slot` axis — subcommands declaring `--slot`.
//!
//! Boundary (v0.36.2 end-of-cycle M1): axis-1 completeness is TRANSITIVE on
//! `secrets::flag_is_secret` (gui-schema's `secret` bit = `flag_is_secret(name)`).
//! A future secret VALUE flag added with a name absent from `flag_is_secret` AND
//! not routed via `--from`/`--slot` would escape this closure — but it would also
//! escape the runtime advisory / zeroize / GUI-mask (a far more visible defect),
//! so `flag_is_secret` is the authoritative source this gate is anchored to, by
//! design. There is no separate gate over `flag_is_secret` completeness itself.
//!
//! `source_file` is EXPLICIT per route (R2 M-1): gui-schema FLATTENS nested
//! subcommands (`xpub-search-path-of-xpub`, `seedqr-decode`) and some stdin
//! routes live in SHARED modules (`src/repair.rs`), so it is NOT derivable from
//! the subcommand name.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::process::Command;

use mnemonic_toolkit::secret_taxonomy::{SECRET_NODE_TYPES_ARGV, SECRET_SLOT_SUBKEYS};

/// One secret-argv route + the evidence anchor proving its non-argv channel.
struct Route {
    /// gui-schema (flattened) subcommand name.
    subcommand: &'static str,
    /// `--<flag>` (flag-NAME axis) or `"--from"` / `"--slot"` (axes 2/3).
    flag: &'static str,
    /// File (relative to crate root) that CONTAINS the anchor. NOT derived from
    /// `subcommand` — see module doc (flattened names + shared modules).
    source_file: &'static str,
    /// OR-list: any substring present in `source_file` proves the route's
    /// non-argv channel (`*-stdin` / `=-` / `@env:` / refusal) is wired.
    evidence: &'static [&'static str],
}

// ── Axis 1: flag-NAME routes (25 = 9 pre-v0.13.0 + 16 backfilled v0.36.2) ──
const FLAG_ROUTES: &[Route] = &[
    // -- pre-v0.13.0 (9) --
    Route { subcommand: "bundle", flag: "--passphrase", source_file: "src/cmd/bundle.rs", evidence: &["passphrase_stdin", "passphrase-stdin"] },
    Route { subcommand: "verify-bundle", flag: "--passphrase", source_file: "src/cmd/verify_bundle.rs", evidence: &["passphrase_stdin", "passphrase-stdin"] },
    Route { subcommand: "convert", flag: "--passphrase", source_file: "src/cmd/convert.rs", evidence: &["passphrase_stdin", "passphrase-stdin"] },
    Route { subcommand: "convert", flag: "--bip38-passphrase", source_file: "src/cmd/convert.rs", evidence: &["bip38_passphrase_stdin", "bip38-passphrase-stdin"] },
    Route { subcommand: "derive-child", flag: "--passphrase", source_file: "src/cmd/derive_child.rs", evidence: &["passphrase_stdin", "passphrase-stdin"] },
    Route { subcommand: "slip39-split", flag: "--passphrase", source_file: "src/cmd/slip39.rs", evidence: &["passphrase_stdin", "passphrase-stdin"] },
    Route { subcommand: "slip39-combine", flag: "--passphrase", source_file: "src/cmd/slip39.rs", evidence: &["passphrase_stdin", "passphrase-stdin"] },
    Route { subcommand: "slip39-combine", flag: "--share", source_file: "src/cmd/slip39.rs", evidence: &["--share -", "secret_in_argv_warning"] },
    Route { subcommand: "seed-xor-combine", flag: "--share", source_file: "src/cmd/seed_xor.rs", evidence: &["--share phrase=", "secret_in_argv_warning"] },
    // -- v0.36.2 backfill (16) --
    Route { subcommand: "nostr", flag: "--secret", source_file: "src/cmd/nostr.rs", evidence: &["secret_stdin", "secret-stdin"] },
    Route { subcommand: "silent-payment", flag: "--secret", source_file: "src/cmd/silent_payment.rs", evidence: &["secret_stdin", "secret-stdin"] },
    Route { subcommand: "silent-payment", flag: "--passphrase", source_file: "src/cmd/silent_payment.rs", evidence: &["passphrase_stdin", "passphrase-stdin"] },
    Route { subcommand: "electrum-decrypt", flag: "--decrypt-password", source_file: "src/cmd/electrum_decrypt.rs", evidence: &["decrypt_password_stdin", "decrypt-password-stdin"] },
    Route { subcommand: "import-wallet", flag: "--decrypt-password", source_file: "src/cmd/import_wallet.rs", evidence: &["decrypt_password_stdin", "decrypt-password-stdin"] },
    // import-wallet/verify-bundle --ms1: @env:-only (no *-stdin/-) — R0 I3.
    Route { subcommand: "import-wallet", flag: "--ms1", source_file: "src/cmd/import_wallet.rs", evidence: &["@env:", "resolve_env_sentinels", "resolve_env_var_sentinel", "needs_env_sentinel_resolution"] },
    Route { subcommand: "verify-bundle", flag: "--ms1", source_file: "src/cmd/verify_bundle.rs", evidence: &["@env:", "resolve_env_sentinels", "resolve_env_var_sentinel"] },
    // inspect/repair --ms1: `-` sentinel handled in the SHARED src/repair.rs — R0 I-2.
    Route { subcommand: "inspect", flag: "--ms1", source_file: "src/repair.rs", evidence: &["value == \"-\"", "resolve_groups", "expand_dashes"] },
    Route { subcommand: "repair", flag: "--ms1", source_file: "src/repair.rs", evidence: &["value == \"-\"", "resolve_groups", "expand_dashes"] },
    Route { subcommand: "seedqr-decode", flag: "--digits", source_file: "src/cmd/seedqr.rs", evidence: &["read_stdin_to_string", "== \"-\"", "--digits -"] },
    // xpub-search ×3: --ms1 anchor in the SHARED seed_intake.rs; --passphrase per-mode file.
    Route { subcommand: "xpub-search-path-of-xpub", flag: "--ms1", source_file: "src/cmd/xpub_search/seed_intake.rs", evidence: &["--ms1-stdin", "ms1_stdin", "secret_in_argv_warning"] },
    Route { subcommand: "xpub-search-account-of-descriptor", flag: "--ms1", source_file: "src/cmd/xpub_search/seed_intake.rs", evidence: &["--ms1-stdin", "ms1_stdin", "secret_in_argv_warning"] },
    Route { subcommand: "xpub-search-passphrase-of-xpub", flag: "--ms1", source_file: "src/cmd/xpub_search/seed_intake.rs", evidence: &["--ms1-stdin", "ms1_stdin", "secret_in_argv_warning"] },
    Route { subcommand: "xpub-search-path-of-xpub", flag: "--passphrase", source_file: "src/cmd/xpub_search/path_of_xpub.rs", evidence: &["passphrase-stdin", "passphrase_stdin", "secret_in_argv_warning"] },
    Route { subcommand: "xpub-search-account-of-descriptor", flag: "--passphrase", source_file: "src/cmd/xpub_search/account_of_descriptor.rs", evidence: &["passphrase-stdin", "passphrase_stdin", "secret_in_argv_warning"] },
    Route { subcommand: "xpub-search-passphrase-of-xpub", flag: "--passphrase", source_file: "src/cmd/xpub_search/passphrase_of_xpub.rs", evidence: &["passphrase-stdin", "passphrase_stdin", "secret_in_argv_warning"] },
    // -- v0.38.0 (1): mnemonic addresses --
    Route { subcommand: "addresses", flag: "--passphrase", source_file: "src/cmd/addresses.rs", evidence: &["passphrase-stdin", "passphrase_stdin", "secret_in_argv_warning"] },
];

// ── Axis 2: `--from` routes (`=-` value-uniform per subcommand) ──
const FROM_ROUTES: &[Route] = &[
    Route { subcommand: "addresses", flag: "--from", source_file: "src/cmd/addresses.rs", evidence: &["=-", "value == \"-\""] },
    Route { subcommand: "convert", flag: "--from", source_file: "src/cmd/convert.rs", evidence: &["=-", "value == \"-\""] },
    Route { subcommand: "derive-child", flag: "--from", source_file: "src/cmd/derive_child.rs", evidence: &["=-", "value == \"-\""] },
    Route { subcommand: "final-word", flag: "--from", source_file: "src/cmd/final_word.rs", evidence: &["=-", "value == \"-\""] },
    Route { subcommand: "seed-xor-split", flag: "--from", source_file: "src/cmd/seed_xor.rs", evidence: &["=-", "value == \"-\""] },
    Route { subcommand: "slip39-split", flag: "--from", source_file: "src/cmd/slip39.rs", evidence: &["=-", "value == \"-\""] },
    // seedqr-decode/-encode are flattened → src/cmd/seedqr.rs (no seedqr-decode.rs) — R2 M-1.
    Route { subcommand: "seedqr-decode", flag: "--from", source_file: "src/cmd/seedqr.rs", evidence: &["=-", "== \"-\""] },
    Route { subcommand: "seedqr-encode", flag: "--from", source_file: "src/cmd/seedqr.rs", evidence: &["=-", "== \"-\""] },
];

// ── Axis 3: `--slot` routes (4; slot-stdin / @env: / refusal) ──
const SLOT_ROUTES: &[Route] = &[
    Route { subcommand: "bundle", flag: "--slot", source_file: "src/cmd/bundle.rs", evidence: &["slot_stdin", "slot-stdin", "apply_slot_stdin"] },
    Route { subcommand: "verify-bundle", flag: "--slot", source_file: "src/cmd/verify_bundle.rs", evidence: &["slot_stdin", "slot-stdin", "apply_slot_stdin"] },
    // import-wallet --slot @N.phrase= is @env:-only (R1 I-A).
    Route { subcommand: "import-wallet", flag: "--slot", source_file: "src/cmd/import_wallet.rs", evidence: &["@env:", "resolve_env_var_sentinel", "resolve_env_sentinels"] },
    // export-wallet REFUSES secret subkeys — the refusal IS the anchor (R1 M-A: runtime token).
    Route { subcommand: "export-wallet", flag: "--slot", source_file: "src/cmd/export_wallet.rs", evidence: &["validate_watch_only", "watch-only by"] },
];

fn crate_root() -> &'static Path {
    Path::new(".")
}

/// `serde_json::Value` of `mnemonic gui-schema`.
fn gui_schema() -> serde_json::Value {
    let bin = std::env::var("MNEMONIC_BIN").unwrap_or_else(|_| {
        env!("CARGO_BIN_EXE_mnemonic").to_string()
    });
    let out = Command::new(&bin)
        .arg("gui-schema")
        .output()
        .unwrap_or_else(|e| panic!("failed to run `{bin} gui-schema`: {e}"));
    assert!(out.status.success(), "gui-schema must exit 0; got {:?}", out.status);
    serde_json::from_slice(&out.stdout).expect("gui-schema stdout must be JSON")
}

/// (subcommand, flag) for every gui-schema flag with `secret==true && kind!="boolean"`.
fn live_secret_flag_routes(schema: &serde_json::Value) -> BTreeSet<(String, String)> {
    let mut set = BTreeSet::new();
    for sub in schema["subcommands"].as_array().expect("subcommands array") {
        let name = sub["name"].as_str().expect("subcommand name");
        for f in sub["flags"].as_array().into_iter().flatten() {
            let is_secret = f.get("secret").and_then(|v| v.as_bool()).unwrap_or(false);
            let kind = f["kind"].as_str().unwrap_or("");
            if is_secret && kind != "boolean" {
                set.insert((name.to_string(), f["name"].as_str().unwrap().to_string()));
            }
        }
    }
    set
}

/// Subcommands whose gui-schema flags include a flag literally named `flag_name`.
fn live_subcommands_with_flag(schema: &serde_json::Value, flag_name: &str) -> BTreeSet<String> {
    let mut set = BTreeSet::new();
    for sub in schema["subcommands"].as_array().expect("subcommands array") {
        let name = sub["name"].as_str().expect("subcommand name");
        let has = sub["flags"]
            .as_array()
            .into_iter()
            .flatten()
            .any(|f| f["name"].as_str() == Some(flag_name));
        if has {
            set.insert(name.to_string());
        }
    }
    set
}

// ── Completeness closures (set-equality vs the live surface) ──

#[test]
fn flag_axis_set_equals_gui_schema() {
    let live = live_secret_flag_routes(&gui_schema());
    let declared: BTreeSet<(String, String)> = FLAG_ROUTES
        .iter()
        .map(|r| (r.subcommand.to_string(), r.flag.to_string()))
        .collect();
    let missing: Vec<_> = live.difference(&declared).collect(); // live but undeclared
    let stale: Vec<_> = declared.difference(&live).collect(); // declared but gone from clap
    assert!(
        missing.is_empty() && stale.is_empty(),
        "secret-argv flag-route drift (axis 1):\n  NEW secret-argv flags in gui-schema, \
         not declared in FLAG_ROUTES (add a Route + wire its stdin route!): {missing:?}\n  \
         declared routes no longer in gui-schema (remove from FLAG_ROUTES): {stale:?}",
    );
}

#[test]
fn from_axis_set_equals_gui_schema() {
    let live = live_subcommands_with_flag(&gui_schema(), "--from");
    let declared: BTreeSet<String> = FROM_ROUTES.iter().map(|r| r.subcommand.to_string()).collect();
    assert_eq!(
        live, declared,
        "`--from` subcommand drift (axis 2): live gui-schema --from-bearing set != declared \
         FROM_ROUTES. The `=-` non-argv route is value-uniform per subcommand; add/remove a \
         Route (with its `=-` evidence) to match. SECRET_NODE_TYPES_ARGV={SECRET_NODE_TYPES_ARGV:?}",
    );
}

#[test]
fn slot_axis_set_equals_gui_schema() {
    let live = live_subcommands_with_flag(&gui_schema(), "--slot");
    let declared: BTreeSet<String> = SLOT_ROUTES.iter().map(|r| r.subcommand.to_string()).collect();
    assert_eq!(
        live, declared,
        "`--slot` subcommand drift (axis 3): live gui-schema --slot-bearing set != declared \
         SLOT_ROUTES. Each needs a non-argv anchor (slot-stdin / @env: / secret-subkey refusal). \
         SECRET_SLOT_SUBKEYS={SECRET_SLOT_SUBKEYS:?}",
    );
}

// ── Evidence anchor (non-argv channel is WIRED in source) ──

#[test]
fn every_route_has_nonargv_channel_evidence() {
    let mut missing: Vec<String> = Vec::new();
    for route in FLAG_ROUTES.iter().chain(FROM_ROUTES).chain(SLOT_ROUTES) {
        let path = crate_root().join(route.source_file);
        let source = fs::read_to_string(&path).unwrap_or_else(|e| {
            panic!(
                "failed to read evidence source {} for ({} {}): {e}",
                path.display(),
                route.subcommand,
                route.flag,
            )
        });
        if !route.evidence.iter().any(|needle| source.contains(needle)) {
            missing.push(format!(
                "  - {} {} ({}): no non-argv-channel anchor; expected one of {:?}",
                route.subcommand, route.flag, route.source_file, route.evidence,
            ));
        }
    }
    assert!(
        missing.is_empty(),
        "argv-leakage lint: {} secret-argv route(s) with NO wired non-argv channel \
         (*-stdin / =- / @env: / refusal) — a route with none is a REAL argv-leak:\n{}",
        missing.len(),
        missing.join("\n"),
    );
}
