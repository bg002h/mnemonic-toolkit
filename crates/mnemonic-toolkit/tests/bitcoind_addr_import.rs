//! PLAN Phase 1b — the IMPORT gate for `--format bitcoin-core-addresses`.
//!
//! **Why an import test and not an emit test.** This format exists *precisely
//! because* the descriptor route emits fine and fails at import: Bitcoin Core
//! refuses a signatureless spend path at `importdescriptors` on every version
//! through v31.1, and unlike our side the rule is non-waivable there. The
//! plan's round-1 Critical was exactly this — every listed test stopped at
//! emission, so none of them would have noticed that the toolkit's own
//! shape-perfect JSON came back `success: false`. Accepting this format on
//! emission alone would repeat that defect. So this file asserts a live
//! `importdescriptors` returning **per-entry `success: true`**, and asserts
//! the DESCRIPTOR route for the same wallet still comes back
//! `success: false` — the negative control that makes the format's existence
//! provable rather than asserted.
//!
//! **Wiring contract — CONNECT-ONLY (the test NEVER spawns bitcoind).**
//! Identical to `tests/bitcoind_differential.rs`, deliberately: CI (or the
//! local recipe) owns the lifecycle: it starts an offline `-chain=main` node
//! and exports four env vars the test reads — `MNEMONIC_BIN` (path to the
//! built `mnemonic` binary; falls back to the cargo-built test binary),
//! `BITCOINCLI_BIN` (path to the pinned `bitcoin-cli`), `BITCOIND_DATADIR` (so
//! `bitcoin-cli` finds the `.cookie`), and `BITCOIND_RPCPORT`.
//!
//! - The three bitcoind vars UNSET → **`panic!`**. This test is
//!   `#[ignore]`-by-default, so running it is an explicit request for the
//!   import gate; a request that cannot reach Core must not report success.
//!   `#[ignore]` is the skip mechanism, not the env check.
//! - SET but `bitcoin-cli getblockchaininfo` fails / `chain != "main"` →
//!   `panic!` (broken provisioning fails RED, never green-by-skip).
//!
//! `#[ignore]`-by-default; run with
//! `cargo test -p mnemonic-toolkit --test bitcoind_addr_import
//! -- --ignored --nocapture` after exporting the vars.
//! `.github/workflows/bitcoind-differential.yml` does exactly that.
//!
//! Pinned oracle: Bitcoin Core v27.0
//! (sha256 `2a6974c5486f528793c79d42694b5987401e4a43c97f62b1383abf35bcee44a8`).
//!
//! **Network: offline `-chain=main` (mainnet), NOT regtest.** The plan's
//! Acceptance says "regtest `importdescriptors`"; that word is wrong for this
//! constellation and `bitcoind-differential.yml`'s header says why — regtest
//! rejects mainnet xpubs, and every address the toolkit derives from this
//! wallet is a mainnet `bc1…`. A regtest node could not be shown the journey's
//! addresses at all, so the address cross-check and the import gate would be
//! testing two different wallets.

use serde_json::Value;
use std::process::Command;

// ─── the wallet under test ──────────────────────────────────────────────

/// Addresses per chain. Five, because that is what the journey captured, so
/// the anti-vacuity golden below covers the whole emitted list rather than a
/// prefix of it.
const COUNT: usize = 5;

fn fixture(dir: &str, name: &str) -> String {
    let p = format!("tests/fixtures/{dir}/{name}");
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("fixture {p}: {e}"))
}

fn rcw_wsh_descriptor() -> String {
    fixture("export_wallet_allow", "rcw_wsh_descriptor.txt")
        .trim()
        .to_string()
}

/// The journey's own capture — see
/// `tests/fixtures/export_wallet_addresses/PROVENANCE.md`. The device and a
/// BIP-129 BSMS canary already agree with these.
fn journey(name: &str) -> Vec<String> {
    fixture("export_wallet_addresses", name)
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect()
}

// ─── toolkit CLI ────────────────────────────────────────────────────────

/// The `mnemonic` binary: `MNEMONIC_BIN` wins, else the cargo-built test bin.
fn mnemonic_bin() -> String {
    std::env::var("MNEMONIC_BIN").unwrap_or_else(|_| env!("CARGO_BIN_EXE_mnemonic").to_string())
}

/// Run `mnemonic export-wallet …` and return its stdout, panicking on refusal.
fn export(args: &[&str]) -> String {
    let out = Command::new(mnemonic_bin())
        .arg("export-wallet")
        .args(args)
        .output()
        .expect("spawn mnemonic export-wallet");
    assert!(
        out.status.success(),
        "mnemonic export-wallet {args:?} failed ({}): {}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).expect("utf-8 stdout")
}

// ─── bitcoind connection (connect-only cookie client) ───────────────────

struct Wiring {
    cli_bin: String,
    datadir: String,
    rpcport: String,
}

/// Read the three bitcoind wiring vars: NONE set → None; ALL set → Some;
/// partially set → panic (ambiguous broken provision).
fn read_wiring() -> Option<Wiring> {
    let cli_bin = std::env::var("BITCOINCLI_BIN").ok();
    let datadir = std::env::var("BITCOIND_DATADIR").ok();
    let rpcport = std::env::var("BITCOIND_RPCPORT").ok();
    match (cli_bin, datadir, rpcport) {
        (None, None, None) => None,
        (Some(cli_bin), Some(datadir), Some(rpcport)) => Some(Wiring {
            cli_bin,
            datadir,
            rpcport,
        }),
        (cli_bin, datadir, rpcport) => panic!(
            "bitcoind wiring partially set — all three of BITCOINCLI_BIN/\
             BITCOIND_DATADIR/BITCOIND_RPCPORT must be set together \
             (BITCOINCLI_BIN={cli_bin:?}, BITCOIND_DATADIR={datadir:?}, \
             BITCOIND_RPCPORT={rpcport:?})"
        ),
    }
}

/// `read_wiring`, but UNSET is a FAILURE rather than a silent pass — the same
/// contract `bitcoind_differential.rs` adopted on 2026-08-19 after measuring
/// four tests reporting `ok` in 0.00s without contacting bitcoind at all.
fn require_wiring() -> Wiring {
    read_wiring().unwrap_or_else(|| {
        panic!(
            "bitcoind wiring not set. This test is #[ignore]-by-default, so \
             running it is an explicit request for the import gate and cannot \
             pass without Core. Export BITCOINCLI_BIN, BITCOIND_DATADIR and \
             BITCOIND_RPCPORT against an offline -chain=main node (see \
             .github/workflows/bitcoind-differential.yml for the provisioning \
             CI uses)."
        )
    })
}

fn cli_raw(w: &Wiring, args: &[&str]) -> std::process::Output {
    Command::new(&w.cli_bin)
        .arg("-chain=main")
        .arg(format!("-datadir={}", w.datadir))
        .arg(format!("-rpcport={}", w.rpcport))
        .args(args)
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn bitcoin-cli ({}): {e}", w.cli_bin))
}

/// Shell `$BITCOINCLI_BIN -chain=main -datadir=… -rpcport=… <args>` (cookie
/// auth) → parsed JSON. `panic!`s on process failure or RPC error.
fn bitcoin_cli(w: &Wiring, args: &[&str]) -> Value {
    let out = cli_raw(w, args);
    if !out.status.success() {
        panic!(
            "bitcoin-cli {:?} failed (status {}): stderr={}",
            args,
            out.status,
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("bitcoin-cli {args:?} output not JSON ({e}): {stdout}"))
}

/// A fresh blank watch-only descriptor wallet, uniquely named so a re-run
/// against a persistent datadir does not collide with the previous one.
fn create_watch_wallet(w: &Wiring, tag: &str) -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let name = format!("p1b-{tag}-{nanos}");
    bitcoin_cli(
        w,
        &[
            "-named",
            "createwallet",
            &format!("wallet_name={name}"),
            "disable_private_keys=true",
            "blank=true",
            "descriptors=true",
        ],
    );
    name
}

fn unload_wallet(w: &Wiring, name: &str) {
    // Best-effort teardown; a failure here must not mask the assertion result.
    let _ = cli_raw(w, &["unloadwallet", name]);
}

/// `addr(<address>)#<csum>` → `("addr(<address>)", "<csum>")`.
fn split_checksum(desc: &str) -> (&str, &str) {
    desc.rsplit_once('#')
        .unwrap_or_else(|| panic!("entry has no BIP-380 checksum: {desc}"))
}

// ─── the gate ───────────────────────────────────────────────────────────

/// The Phase 1b acceptance criterion: a live `importdescriptors` of the
/// emitted artifact returns **per-entry `success: true`**, on a pinned Core,
/// for the real wallet's real addresses.
///
/// Five assertions, in an order chosen so a broken harness cannot fake any of
/// them:
///
/// 1. **anti-vacuity, before Core is contacted at all** — the emitted
///    addresses must equal the journey's captured list. A silently-wrong
///    binary or a drifted derivation fails here, not by importing the wrong
///    wallet successfully.
/// 2. **checksums come from Core, not from us** — every entry's checksum is
///    re-derived through `getdescriptorinfo` and compared, per the plan's
///    *"assert against `getdescriptorinfo` rather than trusting the
///    computation"*.
/// 3. **the import itself** — per-entry `success: true`, no `error` key, and
///    the count matches what we emitted.
/// 4. **it actually landed** — `listdescriptors` returns every one of them, so
///    a hypothetical `success: true` that stored nothing is caught.
/// 5. **the negative control** — the DESCRIPTOR route for the SAME wallet,
///    emitted by the same binary, is refused per-entry by the same node. That
///    is what makes "this format exists because the descriptor route fails at
///    import" a measurement rather than a claim.
#[test]
#[ignore = "requires a pre-running offline -chain=main bitcoind (wiring env vars)"]
fn addr_list_imports_into_bitcoin_core_and_the_descriptor_route_does_not() {
    let w = require_wiring();

    // Fail-LOUD if set-but-silent.
    let info = bitcoin_cli(&w, &["getblockchaininfo"]);
    assert_eq!(
        info.get("chain").and_then(|c| c.as_str()),
        Some("main"),
        "bitcoind must be on -chain=main (got {info:?})"
    );

    let descriptor = rcw_wsh_descriptor();
    let count = COUNT.to_string();
    let emitted = export(&[
        "--descriptor",
        &descriptor,
        "--format",
        "bitcoin-core-addresses",
        "--count",
        &count,
        "--allow",
        "sigless-branch",
    ]);
    let array: Value = serde_json::from_str(&emitted).expect("emitted artifact is JSON");
    let entries = array.as_array().expect("importdescriptors array").clone();
    assert_eq!(entries.len(), COUNT * 2, "receive + change");

    // ── 1. anti-vacuity, BEFORE any Core call ──────────────────────────
    let mut recv = Vec::new();
    let mut chg = Vec::new();
    for e in &entries {
        let desc = e["desc"].as_str().expect("desc string");
        let (body, _) = split_checksum(desc);
        let addr = body
            .strip_prefix("addr(")
            .and_then(|b| b.strip_suffix(')'))
            .unwrap_or_else(|| panic!("not an addr() descriptor: {desc}"))
            .to_string();
        if e["internal"].as_bool().expect("internal bool") {
            chg.push(addr);
        } else {
            recv.push(addr);
        }
    }
    assert_eq!(
        recv,
        journey("journey_wsh_receive.txt"),
        "emitted receive addresses drifted from the journey's capture — the \
         import below would be importing a different wallet"
    );
    assert_eq!(chg, journey("journey_wsh_change.txt"));

    // ── 2. Core's own checksum for every entry ─────────────────────────
    for e in &entries {
        let desc = e["desc"].as_str().unwrap();
        let (body, ours) = split_checksum(desc);
        let dinfo = bitcoin_cli(&w, &["getdescriptorinfo", body]);
        let theirs = dinfo
            .get("checksum")
            .and_then(|c| c.as_str())
            .unwrap_or_else(|| panic!("getdescriptorinfo had no checksum: {dinfo:?}"));
        assert_eq!(
            ours, theirs,
            "BIP-380 checksum disagreement on {body}: ours={ours} core={theirs}"
        );
        assert_eq!(
            dinfo.get("isrange").and_then(|v| v.as_bool()),
            Some(false),
            "an addr() entry must not be ranged: {dinfo:?}"
        );
    }

    // ── 3. the import ──────────────────────────────────────────────────
    let wallet = create_watch_wallet(&w, "addr");
    let rpcwallet = format!("-rpcwallet={wallet}");
    let result = bitcoin_cli(&w, &[&rpcwallet, "importdescriptors", emitted.trim()]);
    let rows = result.as_array().expect("importdescriptors result array");
    assert_eq!(
        rows.len(),
        entries.len(),
        "Core answered {} entries for {} sent",
        rows.len(),
        entries.len()
    );
    for (i, row) in rows.iter().enumerate() {
        assert_eq!(
            row.get("success").and_then(|s| s.as_bool()),
            Some(true),
            "entry {i} was NOT imported: {row}"
        );
        assert!(
            row.get("error").is_none(),
            "entry {i} imported with an error: {row}"
        );
    }

    // ── 4. it actually landed ──────────────────────────────────────────
    let listed = bitcoin_cli(&w, &[&rpcwallet, "listdescriptors"]);
    let stored: Vec<String> = listed["descriptors"]
        .as_array()
        .expect("listdescriptors.descriptors")
        .iter()
        .map(|d| d["desc"].as_str().expect("desc").to_string())
        .collect();
    assert_eq!(
        stored.len(),
        entries.len(),
        "wallet holds {} descriptors, imported {}",
        stored.len(),
        entries.len()
    );
    for e in &entries {
        let desc = e["desc"].as_str().unwrap();
        assert!(stored.contains(&desc.to_string()), "{desc} did not land");
    }
    unload_wallet(&w, &wallet);

    // ── 5. the negative control ────────────────────────────────────────
    // The SAME wallet through the DESCRIPTOR route, emitted by the SAME
    // binary, must be refused by the SAME node. Without this the import above
    // proves only that Core accepts addr() entries, not that it had to be
    // addr() entries.
    let descriptor_route = export(&[
        "--descriptor",
        &descriptor,
        "--format",
        "bitcoin-core",
        "--allow",
        "sigless-branch",
    ]);
    let wallet2 = create_watch_wallet(&w, "desc");
    let rpcwallet2 = format!("-rpcwallet={wallet2}");
    let refused = bitcoin_cli(
        &w,
        &[&rpcwallet2, "importdescriptors", descriptor_route.trim()],
    );
    let rows2 = refused.as_array().expect("array");
    assert!(!rows2.is_empty());
    for (i, row) in rows2.iter().enumerate() {
        assert_eq!(
            row.get("success").and_then(|s| s.as_bool()),
            Some(false),
            "the descriptor route was ACCEPTED at entry {i} — if Core has \
             started accepting signatureless spend paths, this whole format's \
             rationale needs re-deriving before the test is relaxed: {row}"
        );
        let msg = row["error"]["message"].as_str().unwrap_or_default();
        assert!(
            msg.contains("witnesses without signature exist"),
            "entry {i} refused for a DIFFERENT reason than the sigless branch: {row}"
        );
    }
    unload_wallet(&w, &wallet2);
}
