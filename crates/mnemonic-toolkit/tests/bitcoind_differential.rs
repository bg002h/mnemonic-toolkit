//! Stress Cycle E (toolkit leg) — Bitcoin Core END-TO-END differential.
//!
//! Bitcoin Core is an INDEPENDENT C++ implementation of address derivation.
//! md-codec's own `bitcoind_differential` cross-checks the CODEC-internal
//! `Descriptor::derive_address` against Core. THIS test cross-checks the
//! TOOLKIT's END-TO-END user pipeline: `bundle --descriptor` (engrave,
//! watch-only) → `restore --md1` (reconstruct) → assert the toolkit's
//! reported first addresses against Core's `deriveaddresses` on the
//! reconstructed descriptor (and the original). That exercises the toolkit's
//! OWN derivation (`derive_address.rs` / `address_render.rs` — the v0.49.1
//! route-AROUND md-codec for taproot) plus the reconstruction path, with an
//! external C++ oracle. STRESS-A's O3 is a same-ecosystem rust-miniscript
//! oracle (the toolkit delegates to the same patched fork); Bitcoin Core is
//! the only oracle outside that ecosystem. The headline shape
//! `tr(NUMS,sortedmulti_a)` is derivable ONLY by the toolkit's patched fork
//! (`95fdd1c` has `Terminal::SortedMultiA`; md-codec's crates.io 13.0.0 does
//! not), so md-codec's oracle cannot cover it — only this one can.
//!
//! **Wiring contract — CONNECT-ONLY (the test NEVER spawns bitcoind).**
//! CI (or the local recipe) owns the lifecycle: it starts an offline
//! `-chain=main` node and exports four env vars the test reads —
//! `MNEMONIC_BIN` (path to the built `mnemonic` binary; falls back to the
//! cargo-built test binary), `BITCOINCLI_BIN` (path to the pinned
//! `bitcoin-cli`), `BITCOIND_DATADIR` (so `bitcoin-cli` finds the `.cookie`),
//! and `BITCOIND_RPCPORT`. The test shells `$BITCOINCLI_BIN -chain=main
//! -datadir=$BITCOIND_DATADIR -rpcport=$BITCOIND_RPCPORT <rpc> …` (cookie
//! auth, no credentials).
//!
//! - The three bitcoind vars UNSET → skip (the standard `#[ignore]` default).
//! - SET but `bitcoin-cli getblockchaininfo` fails / `chain != "main"` →
//!   `panic!` (broken provisioning fails RED, never green-by-skip).
//!
//! `#[ignore]`-by-default; run with
//! `cargo test -p mnemonic-toolkit --test bitcoind_differential
//! -- --ignored --nocapture` after exporting the vars.
//!
//! Pinned oracle: Bitcoin Core v27.0
//! (sha256 `2a6974c5486f528793c79d42694b5987401e4a43c97f62b1383abf35bcee44a8`).
//! Network: offline `-chain=main` (mainnet) — regtest rejects mainnet xpubs,
//! and the toolkit's TLV→xpub path always renders mainnet `xpub…`.

use miniscript::{descriptor::DescriptorPublicKey, DefiniteDescriptorKey, Descriptor};
use serde_json::Value;
use std::process::Command;
use std::str::FromStr;

/// How many addresses to derive per (shape, chain): indices 0..=N.
const N: u32 = 4;

/// BIP-341 NUMS H-point (x-only), the unspendable taproot internal key the
/// toolkit substitutes for `tr(...)` multisig (mirrors `parse_descriptor.rs`).
const NUMS_HEX: &str = "50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0";

/// Anti-vacuity golden — shape-1 (`wpkh`) chain-0 idx-0, derived INDEPENDENTLY
/// (see `derive_receive`) from `wpkh(K0/<0;1>/*)`, captured 2026-06-16. The
/// test asserts both the independent derivation AND `restore`'s reported
/// address equal this BEFORE any Core compare, so a silently-wrong bitcoind
/// connection can never make the test vacuously pass.
const WPKH_CHAIN0_IDX0_GOLDEN: &str = "bc1qqew6k2qzwadxjdzr8qw5dwupjyccj49z7yey9r";

// Origin-annotated MAINNET xpubs — the STRESS-A frozen key pool
// (`prop_backup_restore_roundtrip.rs`). Origin `/48h/0h/0h/2h` is OPAQUE
// metadata (the toolkit does no purpose/depth validation; derivation uses
// xpub + `/0/i`, and Core's `deriveaddresses` ignores origins) — so it is
// used verbatim for every shape, single-sig and taproot included.
const K0: &str = "[11111111/48h/0h/0h/2h]xpub661MyMwAqRbcEZVB4dScxMAdx6d4nFc9nvyvH3v4gJL378CSRZiYmhRoP7mBy6gSPSCYk6SzXPTf3ND1cZAceL7SfJ1Z3GC8vBgp2epUt13";
const K1: &str = "[22222222/48h/0h/0h/2h]xpub661MyMwAqRbcFtXgS5sYJABqqG9YLmC4Q1Rdap9gSE8NqtwybGhePY2gZ29ESFjqJoCu1Rupje8YtGqsefD265TMg7usUDFdp6W1EGMcet8";
const K2: &str = "[33333333/48h/0h/0h/2h]xpub661MyMwAqRbcFW31YEwpkMuc5THy2PSt5bDMsktWQcFF8syAmRUapSCGu8ED9W6oDMSgv6Zz8idoc4a6mr8BDzTJY47LJhkJ8UB7WEGuduB";

struct Shape {
    label: &'static str,
    descriptor: String,
}

/// The 9-shape positive corpus — each ∈ (toolkit-derivable ∩ Core-v27-sane ∩
/// restore-reconstructable). Every shape's bundle→restore round-trip was
/// proven locally (2026-06-16) before this test was written. Excluded
/// (restore refuses): @-in-both, depth-≥2 taproot, nested sortedmulti_a,
/// sortedmulti-in-combinator — those are covered by `cli_restore_multisig.rs`.
fn corpus() -> Vec<Shape> {
    let m = "/<0;1>/*";
    vec![
        Shape {
            label: "wpkh",
            descriptor: format!("wpkh({K0}{m})"),
        },
        Shape {
            label: "pkh",
            descriptor: format!("pkh({K0}{m})"),
        },
        Shape {
            label: "wsh-sortedmulti-2of3",
            descriptor: format!("wsh(sortedmulti(2,{K0}{m},{K1}{m},{K2}{m}))"),
        },
        Shape {
            label: "wsh-multi-2of3",
            descriptor: format!("wsh(multi(2,{K0}{m},{K1}{m},{K2}{m}))"),
        },
        Shape {
            label: "sh-wsh-sortedmulti-2of3",
            descriptor: format!("sh(wsh(sortedmulti(2,{K0}{m},{K1}{m},{K2}{m})))"),
        },
        Shape {
            label: "wsh-timelocked",
            descriptor: format!("wsh(and_v(v:pk({K0}{m}),older(144)))"),
        },
        Shape {
            label: "wsh-thresh-2of3",
            descriptor: format!("wsh(thresh(2,pk({K0}{m}),s:pk({K1}{m}),s:pk({K2}{m})))"),
        },
        Shape {
            label: "tr-nums-multi_a-2of3",
            descriptor: format!("tr({NUMS_HEX},multi_a(2,{K0}{m},{K1}{m},{K2}{m}))"),
        },
        Shape {
            // The toolkit-UNIQUE surface: md-codec's crates.io miniscript
            // 13.0.0 cannot render sortedmulti_a; the toolkit's 95fdd1c fork
            // can, so only this oracle puts it in front of Core.
            label: "tr-nums-sortedmulti_a-2of3",
            descriptor: format!("tr({NUMS_HEX},sortedmulti_a(2,{K0}{m},{K1}{m},{K2}{m}))"),
        },
    ]
}

// ─── toolkit CLI pipeline ───────────────────────────────────────────────

/// The `mnemonic` binary: `MNEMONIC_BIN` wins, else the cargo-built test bin.
fn mnemonic_bin() -> String {
    std::env::var("MNEMONIC_BIN").unwrap_or_else(|_| env!("CARGO_BIN_EXE_mnemonic").to_string())
}

/// `bundle --descriptor <concrete> … --json` (watch-only, no slots) → md1.
fn bundle_md1(desc: &str) -> Vec<String> {
    let out = Command::new(mnemonic_bin())
        .args([
            "bundle",
            "--descriptor",
            desc,
            "--network",
            "mainnet",
            "--json",
            "--no-engraving-card",
        ])
        .output()
        .expect("spawn mnemonic bundle");
    assert!(
        out.status.success(),
        "bundle failed for {desc}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v: Value = serde_json::from_slice(&out.stdout).expect("bundle --json stdout");
    v["md1"]
        .as_array()
        .expect("md1 array")
        .iter()
        .map(|x| x.as_str().unwrap().to_string())
        .collect()
}

/// `restore --md1 … --count {N+1} --json` → (reconstructed descriptor,
/// reported receive `first_addresses`).
fn restore(md1: &[String]) -> (String, Vec<String>) {
    let mut args = vec![
        "restore".to_string(),
        "--network".into(),
        "mainnet".into(),
        "--count".into(),
        (N + 1).to_string(),
        "--json".into(),
    ];
    for c in md1 {
        args.push("--md1".into());
        args.push(c.clone());
    }
    let out = Command::new(mnemonic_bin())
        .args(&args)
        .output()
        .expect("spawn mnemonic restore");
    assert!(
        out.status.success(),
        "restore failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v: Value = serde_json::from_slice(&out.stdout).expect("restore --json stdout");
    let w = &v["wallets"][0];
    let desc = w["descriptor"].as_str().expect("descriptor").to_string();
    let addrs = w["first_addresses"]
        .as_array()
        .expect("first_addresses")
        .iter()
        .map(|x| x.as_str().unwrap().to_string())
        .collect();
    (desc, addrs)
}

// ─── independent rust-miniscript derivation (anti-vacuity second oracle) ──

/// Independent receive-address derivation via rust-miniscript — a SEPARATE
/// code path from `restore`'s internal derivation (it does not read the
/// unit-under-test's output). Replicated verbatim from STRESS-A's
/// `derive_receive` (`prop_backup_restore_roundtrip.rs`); uses the
/// non-deprecated `derive_at_index` (clippy `-D warnings`-clean).
fn derive_receive(desc: &str, count: u32) -> Vec<String> {
    let d = Descriptor::<DescriptorPublicKey>::from_str(desc).unwrap();
    let receive = if d.is_multipath() {
        d.clone().into_single_descriptors().unwrap().remove(0)
    } else {
        d.clone()
    };
    (0..count)
        .map(|i| {
            let def: Descriptor<DefiniteDescriptorKey> = if receive.has_wildcard() {
                receive.derive_at_index(i).unwrap()
            } else {
                Descriptor::<DefiniteDescriptorKey>::try_from(receive.clone()).unwrap()
            };
            def.address(bitcoin::Network::Bitcoin).unwrap().to_string()
        })
        .collect()
}

/// Split a `/<0;1>/*` multipath descriptor to its per-chain single descriptor
/// STRING (Core rejects multipath). `chain` 0 = receive, 1 = change. The
/// returned string carries miniscript's BIP-380 `#checksum`.
fn single_chain_desc(desc: &str, chain: usize) -> String {
    let d = Descriptor::<DescriptorPublicKey>::from_str(desc).unwrap();
    let mut singles = d.into_single_descriptors().unwrap();
    assert!(
        singles.len() > chain,
        "descriptor has no chain {chain}: {desc}"
    );
    singles.remove(chain).to_string()
}

// ─── bitcoind connection (connect-only cookie client) ───────────────────

struct Wiring {
    cli_bin: String,
    datadir: String,
    rpcport: String,
}

/// Read the three bitcoind wiring vars: NONE set → None (skip); ALL set →
/// Some; partially set → panic (ambiguous broken provision).
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

/// Shell `$BITCOINCLI_BIN -chain=main -datadir=… -rpcport=… <args>` (cookie
/// auth) → parsed JSON. `panic!`s on process failure or RPC error.
fn bitcoin_cli(w: &Wiring, args: &[&str]) -> Value {
    let out = Command::new(&w.cli_bin)
        .arg("-chain=main")
        .arg(format!("-datadir={}", w.datadir))
        .arg(format!("-rpcport={}", w.rpcport))
        .args(args)
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn bitcoin-cli ({}): {e}", w.cli_bin));
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

/// `deriveaddresses <chain_desc> "[0,N]"` → the N+1 addresses.
fn core_addresses(w: &Wiring, chain_desc: &str) -> Vec<String> {
    let range = format!("[0,{N}]");
    let arr = bitcoin_cli(w, &["deriveaddresses", chain_desc, &range]);
    let addrs: Vec<String> = arr
        .as_array()
        .unwrap_or_else(|| panic!("deriveaddresses not an array: {arr:?}"))
        .iter()
        .map(|v| v.as_str().expect("address string").to_string())
        .collect();
    assert_eq!(
        addrs.len(),
        (N as usize) + 1,
        "expected {} addresses, got {}",
        N + 1,
        addrs.len()
    );
    addrs
}

// ─── the differential ────────────────────────────────────────────────────

/// End-to-end: bundle→restore→reported addresses vs Bitcoin Core
/// `deriveaddresses`, for the 9-shape positive corpus. `#[ignore]`-by-default
/// (needs the wiring vars + a running offline `-chain=main` node).
#[test]
#[ignore = "requires a pre-running offline -chain=main bitcoind (wiring env vars)"]
fn bitcoind_end_to_end_differential() {
    let Some(w) = read_wiring() else {
        eprintln!(
            "skipping: bitcoind env not set (BITCOINCLI_BIN/BITCOIND_DATADIR/BITCOIND_RPCPORT)"
        );
        return;
    };

    // Fail-LOUD if set-but-silent.
    let info = bitcoin_cli(&w, &["getblockchaininfo"]);
    assert_eq!(
        info.get("chain").and_then(|c| c.as_str()),
        Some("main"),
        "bitcoind must be on -chain=main (got {info:?})"
    );

    // Anti-vacuity: the INDEPENDENT miniscript derivation of shape-1 chain-0
    // idx-0 must equal the pinned golden BEFORE any pipeline/Core call (a
    // key-pool or derivation drift fails here, not via a re-captured snapshot).
    let wpkh_desc = format!("wpkh({K0}/<0;1>/*)");
    assert_eq!(
        derive_receive(&wpkh_desc, 1)[0],
        WPKH_CHAIN0_IDX0_GOLDEN,
        "independent miniscript derivation drifted from the pinned golden"
    );

    let mut total_checks = 0usize;
    let mut golden_asserted = false;

    for shape in corpus() {
        let md1 = bundle_md1(&shape.descriptor);
        let (recon_desc, reported) = restore(&md1);
        assert_eq!(
            reported.len(),
            (N as usize) + 1,
            "[{}] restore reported {} addresses, expected {}",
            shape.label,
            reported.len(),
            N + 1
        );

        // Golden: shape-1 chain-0 idx-0 == the pinned independent golden,
        // BEFORE the Core compare.
        if shape.label == "wpkh" {
            assert_eq!(
                reported[0], WPKH_CHAIN0_IDX0_GOLDEN,
                "anti-vacuity golden: wpkh restore address drifted"
            );
            golden_asserted = true;
        }

        let recon_c0 = single_chain_desc(&recon_desc, 0);

        // [M-4] Checksum round-trip: Core's checksum for the reconstructed
        // chain-0 descriptor must equal miniscript's (both BIP-380).
        let md_csum = recon_c0
            .rsplit_once('#')
            .unwrap_or_else(|| panic!("[{}] recon desc has no #csum: {recon_c0}", shape.label))
            .1;
        let dinfo = bitcoin_cli(&w, &["getdescriptorinfo", &recon_c0]);
        let core_csum = dinfo
            .get("checksum")
            .and_then(|c| c.as_str())
            .unwrap_or_else(|| panic!("getdescriptorinfo had no checksum: {dinfo:?}"));
        assert_eq!(
            core_csum, md_csum,
            "CHECKSUM DRIFT [{}]: core={core_csum} toolkit={md_csum} desc={recon_c0}",
            shape.label
        );

        // Primary FUNDS-CRITICAL: Core's receive addresses for the
        // reconstructed descriptor == the toolkit's reported addresses.
        let core_recon = core_addresses(&w, &recon_c0);
        for i in 0..=(N as usize) {
            assert_eq!(
                reported[i], core_recon[i],
                "ADDRESS DIVERGENCE (FUNDS-CRITICAL) [{}] reconstructed chain0 idx{i}: \
                 toolkit={} bitcoind={} desc={recon_c0}",
                shape.label, reported[i], core_recon[i]
            );
            total_checks += 1;
        }

        // Cross-check vs the ORIGINAL descriptor: catches a restore that
        // reconstructs a DIFFERENT-but-Core-valid descriptor.
        let orig_c0 = single_chain_desc(&shape.descriptor, 0);
        let core_orig = core_addresses(&w, &orig_c0);
        for i in 0..=(N as usize) {
            assert_eq!(
                reported[i], core_orig[i],
                "RECONSTRUCTION MISMATCH [{}] original chain0 idx{i}: \
                 toolkit={} bitcoind(original)={}",
                shape.label, reported[i], core_orig[i]
            );
        }

        // [Risk 9] Chain-1 (change) descriptor-equivalence: the toolkit
        // surfaces only the receive branch, so prove restore reconstructed a
        // Core-identical descriptor on the CHANGE branch too (Core on
        // reconstructed chain-1 == Core on original chain-1).
        let core_recon_c1 = core_addresses(&w, &single_chain_desc(&recon_desc, 1));
        let core_orig_c1 = core_addresses(&w, &single_chain_desc(&shape.descriptor, 1));
        assert_eq!(
            core_recon_c1, core_orig_c1,
            "CHANGE-BRANCH MISMATCH [{}]: reconstructed chain-1 addresses differ from the original",
            shape.label
        );
    }

    assert!(
        golden_asserted,
        "anti-vacuity golden never asserted — the wpkh shape is missing from the corpus"
    );
    eprintln!(
        "toolkit bitcoind end-to-end differential PASS: {} shapes, \
         {total_checks} receive-address checks (+ original cross-check + \
         change-branch equivalence + checksum round-trip per shape), all \
         byte-identical vs bitcoind v27.0",
        corpus().len()
    );
}
