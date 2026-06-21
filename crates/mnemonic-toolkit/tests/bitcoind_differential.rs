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

/// P2.5 divergent-shape anti-vacuity golden — chain-0 idx-0 of
/// `wsh(multi(2,K0/<0;1>/*,K1/<2;3>/*,K2/<4;5>/*))`, derived INDEPENDENTLY
/// (rust-miniscript `derive_receive`), captured 2026-06-19. Because @1/@2 take
/// their OWN multipath alt0 (children 2 and 4) instead of the baseline child 0,
/// this address is DISTINCT from the all-baseline `wsh-multi-2of3` address — the
/// discriminator that proves a baseline-clobber regression. Pinned + cross-checked
/// against the all-baseline counterpart by `divergent_differential_golden`.
const WSH_MULTI_DIVERGENT_CHAIN0_IDX0_GOLDEN: &str =
    "bc1qlfsk4typrllqv4sa0jxp0ps8mys6asup7kk2rjd5xr0mg34sfjzs6ed09p";

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
            // P2.5 — DIVERGENT per-cosigner use-site suffixes: @1 and @2 carry
            // their OWN multipath groups (`<2;3>`, `<4;5>`) ≠ @0's baseline
            // `<0;1>`. The bundle→restore→derive path must preserve each group;
            // the independent `derive_receive` oracle + Core both honor per-key
            // multipath natively. Anchored by `divergent_differential_golden`.
            label: "wsh-multi-2of3-divergent",
            descriptor: format!("wsh(multi(2,{K0}{m},{K1}/<2;3>/*,{K2}/<4;5>/*))"),
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
            // P2.5 (#26, opportunistic) — DIVERGENT per-cosigner use-site suffixes
            // on the TAPROOT multi_a leaf: @1/@2 carry their OWN multipath groups
            // (`<2;3>`, `<4;5>`) ≠ @0's baseline `<0;1>`. This is the second-engine
            // (bitcoind `deriveaddresses`) corroboration of the #26 multi_a
            // override leg; restore now reconstructs each `@N`'s OWN suffix. The
            // default-CI gate is the `derive_receive`/golden oracle in
            // `cli_restore_multisig_general.rs`; this row is `#[ignore]`/env-gated.
            label: "tr-nums-multi_a-2of3-divergent",
            descriptor: format!("tr({NUMS_HEX},multi_a(2,{K0}{m},{K1}/<2;3>/*,{K2}/<4;5>/*))"),
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
        // P2.5 divergent-shape golden: restore's reported @1/@2-divergent address
        // == the INDEPENDENT golden, anchoring per-key suffix fidelity BEFORE Core.
        if shape.label == "wsh-multi-2of3-divergent" {
            assert_eq!(
                reported[0], WSH_MULTI_DIVERGENT_CHAIN0_IDX0_GOLDEN,
                "anti-vacuity golden: divergent-suffix restore address drifted (baseline clobber?)"
            );
            assert!(
                recon_desc.contains("<2;3>/*") && recon_desc.contains("<4;5>/*"),
                "divergent shape must reconstruct @1/@2's OWN suffixes: {recon_desc}"
            );
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

/// P2.5 anti-vacuity (DEFAULT-CI, NOT `#[ignore]`): the divergent-shape golden
/// must (a) match the INDEPENDENT rust-miniscript derivation of the divergent
/// corpus descriptor AND (b) DIFFER from the all-baseline `wsh-multi-2of3`
/// counterpart — proving the golden anchors @1/@2's divergence (not codec
/// self-agreement) and would catch a baseline-clobber regression. Also doubles
/// as the generator: run with `--nocapture` to (re)capture the pinned value.
#[test]
fn divergent_differential_golden() {
    let m = "/<0;1>/*";
    let divergent = format!("wsh(multi(2,{K0}{m},{K1}/<2;3>/*,{K2}/<4;5>/*))");
    let baseline = format!("wsh(multi(2,{K0}{m},{K1}{m},{K2}{m}))");
    let divergent_addr = derive_receive(&divergent, 1).remove(0);
    let baseline_addr = derive_receive(&baseline, 1).remove(0);
    eprintln!("WSH_MULTI_DIVERGENT_CHAIN0_IDX0_GOLDEN = {divergent_addr}");
    eprintln!("(all-baseline wsh-multi-2of3 counterpart = {baseline_addr})");
    assert_ne!(
        divergent_addr, baseline_addr,
        "the divergent golden must ANCHOR divergence: @1/@2's own alt0 (children \
         2/4) must yield a DIFFERENT chain0/idx0 address than the baseline (child 0)"
    );
    assert_eq!(
        divergent_addr, WSH_MULTI_DIVERGENT_CHAIN0_IDX0_GOLDEN,
        "independent miniscript derivation drifted from the pinned divergent golden"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// #28 phase 2, P5 — TEMPLATE-COMPLETION corpus rows.
//
// The 9-shape corpus above exercises `bundle --descriptor` (concrete) → restore.
// This section adds the keyless-TEMPLATE completion pipeline against the SAME
// external C++ oracle:
//
//   bundle --md1-form=template (keyless md1 + per-cosigner mk1s)
//     → restore --md1 <template> --from <own seed> --account <acct>
//         --cosigner <other mk1s> --expect-wallet-id <id> --json
//     → assert the COMPLETED descriptor's addresses == Core `deriveaddresses`.
//
// Shapes: a CANONICAL `wsh-sortedmulti` (BIP-48) and a GENERAL `wsh(or_i(...))`
// (BIP-84). Both are built from controlled BIP-39 seeds (the template emit needs
// the operator's seed + per-cosigner mk1 origins, which concrete xpubs cannot
// supply). `#[ignore]`/env-gated with the SAME CONNECT-ONLY contract.
//
// ANTI-VACUITY: each row asserts the COMPLETED restore address == an INDEPENDENT
// rust-miniscript `derive_receive` of the ORIGINAL concrete descriptor BEFORE
// the Core compare — so a silently-wrong bitcoind can never make the row pass
// vacuously. That leg also runs UNCONDITIONALLY in a DEFAULT-CI test
// (`template_completion_anti_vacuity_leg`) so the completion↔independent-oracle
// equivalence is gated even without a node.
// ═══════════════════════════════════════════════════════════════════════════

use assert_cmd::Command as AssertCommand;
use bip39::Mnemonic;
use bitcoin::bip32::{DerivationPath, Xpriv, Xpub};
use bitcoin::secp256k1::Secp256k1;

const SEED_A: &str = "legal winner thank year wave sausage worth useful legal winner thank yellow";
const SEED_B: &str =
    "letter advice cage absurd amount doctor acoustic avoid letter advice cage above";

/// Derive a mainnet account xpub + master fingerprint at `path_str`.
fn xpub_at(phrase: &str, path_str: &str) -> (Xpub, String) {
    let secp = Secp256k1::new();
    let m = Mnemonic::parse_in(bip39::Language::English, phrase).unwrap();
    let seed = m.to_seed("");
    let master = Xpriv::new_master(bitcoin::NetworkKind::Main, &seed).unwrap();
    let fp = master.fingerprint(&secp);
    let path = DerivationPath::from_str(path_str).unwrap();
    let xpriv = master.derive_priv(&secp, &path).unwrap();
    let xpub = Xpub::from_priv(&secp, &xpriv);
    (xpub, fp.to_string().to_lowercase())
}

fn key_str(phrase: &str, path: &str) -> String {
    let (xpub, fp) = xpub_at(phrase, path);
    let origin = path.replace('\'', "h");
    format!("[{fp}/{origin}]{xpub}/<0;1>/*")
}

/// One template-completion corpus case.
struct TemplateCase {
    label: &'static str,
    /// The ORIGINAL concrete descriptor (the independent-golden + Core source).
    descriptor: String,
    /// `bundle` argv prefix (form is substituted: "template" → md1, "policy" → mk1).
    bundle_args: Vec<String>,
    /// Own account (slot @0 = SEED_A at this account).
    own_account: u32,
    /// Which slot indices are EXTERNAL cosigners (supplied as --cosigner).
    cosigner_slots: Vec<usize>,
}

fn template_corpus() -> Vec<TemplateCase> {
    // Canonical wsh-sortedmulti 2-of-2 {A@0, B@0} at BIP-48.
    let canon_keys = [
        key_str(SEED_A, "48'/0'/0'/2'"),
        key_str(SEED_B, "48'/0'/0'/2'"),
    ];
    let canon_desc = format!("wsh(sortedmulti(2,{},{}))", canon_keys[0], canon_keys[1]);
    let mut canon_bundle: Vec<String> = vec![
        "bundle".into(),
        "--network".into(),
        "mainnet".into(),
        "--template".into(),
        "wsh-sortedmulti".into(),
        "--threshold".into(),
        "2".into(),
        "--md1-form".into(),
        "FORM".into(),
        "--group-size".into(),
        "0".into(),
        "--no-engraving-card".into(),
    ];
    for (idx, seed) in [SEED_A, SEED_B].iter().enumerate() {
        let path = "48'/0'/0'/2'";
        let (xpub, fp) = xpub_at(seed, path);
        canon_bundle.push("--slot".into());
        canon_bundle.push(format!("@{idx}.xpub={xpub}"));
        canon_bundle.push("--slot".into());
        canon_bundle.push(format!("@{idx}.fingerprint={fp}"));
        canon_bundle.push("--slot".into());
        canon_bundle.push(format!("@{idx}.path={path}"));
    }

    // General wsh(or_i(pk(@0), and_v(v:pk(@1), pk(@2)))) 3-key at BIP-84.
    let gen_keys = [
        key_str(SEED_A, "84'/0'/0'"),
        key_str(SEED_B, "84'/0'/1'"),
        key_str(SEED_C, "84'/0'/2'"),
    ];
    let gen_desc = format!(
        "wsh(or_i(pk({}),and_v(v:pk({}),pk({}))))",
        gen_keys[0], gen_keys[1], gen_keys[2]
    );
    let gen_bundle: Vec<String> = vec![
        "bundle".into(),
        "--network".into(),
        "mainnet".into(),
        "--md1-form".into(),
        "FORM".into(),
        "--group-size".into(),
        "0".into(),
        "--no-engraving-card".into(),
        "--descriptor".into(),
        gen_desc.clone(),
    ];

    vec![
        TemplateCase {
            label: "template-wsh-sortedmulti-2of2",
            descriptor: canon_desc,
            bundle_args: canon_bundle,
            own_account: 0,
            cosigner_slots: vec![1],
        },
        TemplateCase {
            label: "template-general-or_i-bip84",
            descriptor: gen_desc,
            bundle_args: gen_bundle,
            own_account: 0,
            cosigner_slots: vec![1, 2],
        },
    ]
}

const SEED_C: &str = "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong";

/// Run the (assert_cmd) `mnemonic` bundle for `case` with a given `--md1-form`.
fn run_template_bundle(case: &TemplateCase, form: &str) -> std::process::Output {
    let mut args = case.bundle_args.clone();
    for a in args.iter_mut() {
        if a == "FORM" {
            *a = form.to_string();
        }
    }
    AssertCommand::cargo_bin("mnemonic")
        .unwrap()
        .args(&args)
        .output()
        .expect("spawn mnemonic bundle")
}

fn section_lines(stdout: &str, header: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut in_sec = false;
    for line in stdout.lines() {
        if line.starts_with(header) {
            in_sec = true;
            continue;
        }
        if in_sec {
            if line.trim().is_empty() {
                in_sec = false;
                continue;
            }
            out.push(line.trim().to_string());
        }
    }
    out
}

fn mk1_groups(stdout: &str) -> Vec<Vec<String>> {
    let mut groups: Vec<Vec<String>> = Vec::new();
    let mut cur: Option<Vec<String>> = None;
    for line in stdout.lines() {
        if line.starts_with("# mk1") {
            if let Some(g) = cur.take() {
                if !g.is_empty() {
                    groups.push(g);
                }
            }
            cur = Some(Vec::new());
            continue;
        }
        if let Some(g) = cur.as_mut() {
            let t = line.trim();
            if t.starts_with("mk1") {
                g.push(t.to_string());
            }
        }
    }
    if let Some(g) = cur.take() {
        if !g.is_empty() {
            groups.push(g);
        }
    }
    groups
}

/// Emit the template md1, the per-cosigner mk1s, and the recorded WalletPolicyId.
fn emit_template(case: &TemplateCase) -> (Vec<String>, Vec<Vec<String>>, String) {
    let t = run_template_bundle(case, "template");
    assert!(
        t.status.success(),
        "template bundle failed for {}: {}",
        case.label,
        String::from_utf8_lossy(&t.stderr)
    );
    let t_stdout = String::from_utf8_lossy(&t.stdout).to_string();
    let md1 = section_lines(&t_stdout, "# md1");
    let id = String::from_utf8_lossy(&t.stderr)
        .lines()
        .find(|l| l.contains("wallet-id (hex)"))
        .and_then(|l| l.split(':').next_back())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| panic!("no wallet-id (hex) for {}", case.label));

    let p = run_template_bundle(case, "policy");
    assert!(
        p.status.success(),
        "policy bundle failed for {}",
        case.label
    );
    let cosigners = mk1_groups(&String::from_utf8_lossy(&p.stdout));
    (md1, cosigners, id)
}

/// Complete the keyless template via id-search → (reconstructed descriptor,
/// reported receive addresses). Asserts the restore succeeded.
fn complete_template(case: &TemplateCase, count: u32) -> (String, Vec<String>) {
    let (md1, cosigners, id) = emit_template(case);
    let mut args = vec!["restore".to_string(), "--network".into(), "mainnet".into()];
    for c in &md1 {
        args.push("--md1".into());
        args.push(c.clone());
    }
    args.extend([
        "--from".into(),
        format!("phrase={SEED_A}"),
        "--account".into(),
        case.own_account.to_string(),
        "--expect-wallet-id".into(),
        id,
        "--count".into(),
        count.to_string(),
        "--json".into(),
    ]);
    for &slot in &case.cosigner_slots {
        for c in &cosigners[slot] {
            args.push("--cosigner".into());
            args.push(c.clone());
        }
    }
    let out = AssertCommand::cargo_bin("mnemonic")
        .unwrap()
        .args(&args)
        .output()
        .expect("spawn mnemonic restore");
    assert!(
        out.status.success(),
        "template completion failed for {}: {}",
        case.label,
        String::from_utf8_lossy(&out.stderr)
    );
    let v: Value = serde_json::from_slice(&out.stdout).expect("restore --json");
    let w = &v["wallets"][0];
    let recon = w["descriptor"].as_str().expect("descriptor").to_string();
    let addrs = w["first_addresses"]
        .as_array()
        .expect("first_addresses")
        .iter()
        .map(|x| x.as_str().unwrap().to_string())
        .collect();
    (recon, addrs)
}

/// DEFAULT-CI anti-vacuity leg (NOT `#[ignore]`): the COMPLETED restore address
/// for each template case == the INDEPENDENT rust-miniscript derivation of the
/// ORIGINAL descriptor. This gates the completion↔independent-oracle equivalence
/// without needing a node — and is the same assertion the bitcoind row makes
/// BEFORE the Core compare (so the gated row can never pass vacuously).
#[test]
fn template_completion_anti_vacuity_leg() {
    for case in template_corpus() {
        let independent = derive_receive(&case.descriptor, N + 1);
        let (recon, reported) = complete_template(&case, N + 1);
        assert_eq!(
            reported, independent,
            "[{}] COMPLETED restore addresses must equal the INDEPENDENT \
             rust-miniscript derivation of the original descriptor\n  recon: {recon}",
            case.label
        );
        // The reconstructed descriptor must itself derive identically (the
        // toolkit's reported addrs come from it; cross-check via rust-miniscript).
        let recon_independent = derive_receive(&recon, N + 1);
        assert_eq!(
            recon_independent, independent,
            "[{}] the reconstructed descriptor must derive the SAME addresses as the original",
            case.label
        );
    }
}

/// End-to-end: keyless TEMPLATE → restore COMPLETION → Bitcoin Core
/// `deriveaddresses` on the completed descriptor. `#[ignore]`/env-gated (same
/// CONNECT-ONLY contract as `bitcoind_end_to_end_differential`).
#[test]
#[ignore = "requires a pre-running offline -chain=main bitcoind (wiring env vars)"]
fn bitcoind_template_completion_differential() {
    let Some(w) = read_wiring() else {
        eprintln!(
            "skipping: bitcoind env not set (BITCOINCLI_BIN/BITCOIND_DATADIR/BITCOIND_RPCPORT)"
        );
        return;
    };
    let info = bitcoin_cli(&w, &["getblockchaininfo"]);
    assert_eq!(
        info.get("chain").and_then(|c| c.as_str()),
        Some("main"),
        "bitcoind must be on -chain=main (got {info:?})"
    );

    let mut total_checks = 0usize;
    for case in template_corpus() {
        // ANTI-VACUITY: the COMPLETED restore addrs == the INDEPENDENT
        // rust-miniscript derivation of the original — asserted BEFORE Core.
        let independent = derive_receive(&case.descriptor, N + 1);
        let (recon_desc, reported) = complete_template(&case, N + 1);
        assert_eq!(
            reported, independent,
            "[{}] anti-vacuity: completed addrs must equal the independent derivation BEFORE Core",
            case.label
        );

        // Core on the COMPLETED (reconstructed) chain-0 descriptor.
        let recon_c0 = single_chain_desc(&recon_desc, 0);
        let core_recon = core_addresses(&w, &recon_c0);
        for i in 0..=(N as usize) {
            assert_eq!(
                reported[i], core_recon[i],
                "TEMPLATE-COMPLETION ADDRESS DIVERGENCE (FUNDS-CRITICAL) [{}] idx{i}: \
                 toolkit={} bitcoind={} desc={recon_c0}",
                case.label, reported[i], core_recon[i]
            );
            total_checks += 1;
        }

        // Cross-check vs Core on the ORIGINAL descriptor (catches a completion
        // that reconstructs a DIFFERENT-but-Core-valid wallet).
        let orig_c0 = single_chain_desc(&case.descriptor, 0);
        let core_orig = core_addresses(&w, &orig_c0);
        for i in 0..=(N as usize) {
            assert_eq!(
                reported[i], core_orig[i],
                "TEMPLATE-COMPLETION RECONSTRUCTION MISMATCH [{}] idx{i}: \
                 toolkit={} bitcoind(original)={}",
                case.label, reported[i], core_orig[i]
            );
        }
    }
    eprintln!(
        "toolkit bitcoind TEMPLATE-COMPLETION differential PASS: {} shapes, \
         {total_checks} receive-address checks (+ original cross-check per shape), \
         all byte-identical vs bitcoind v27.0",
        template_corpus().len()
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// H12 (cycle-1) — descriptor-mode taproot multisig defaults the BIP-48 origin
// to script-type 3' (P2TR), not 2' (P2WSH).
//
// The bug is funds-critical AND only visible through a SEED-derived cosigner:
// when a `tr(NUMS, multi_a/sortedmulti_a)` descriptor's cosigner origin is
// elided, the toolkit re-derives each cosigner's ACCOUNT xpub AT the inferred
// origin path. Pre-H12 that path ended in `2'`, so every cosigner xpub — and
// thus every receive/change address — landed in the wrong (P2WSH) subtree,
// un-cosignable by Sparrow/Coldcard/Jade (which re-derive at `3'`).
//
// This row bundles an ORIGIN-ELIDED taproot multisig from two controlled seeds
// (account 0), reads back the per-cosigner xpubs the toolkit derived, and
// builds the concrete descriptor it implies. The oracle asserts that descriptor
// derives the SAME addresses as the INDEPENDENT `48'/0'/0'/3'` derivation, and
// DIFFERS from the `48'/0'/0'/2'` (pre-H12 wrong) subtree — via BOTH
// rust-miniscript (DEFAULT-CI anti-vacuity) AND Core `deriveaddresses`
// (env-gated heavy leg). Mainnet (the harness's offline `-chain=main` contract;
// the toolkit renders mainnet `xpub…`).

/// Build the toolkit's bundled taproot-multisig descriptor by reading back the
/// per-cosigner xpubs from `bundle --json` for an ORIGIN-ELIDED descriptor
/// (seed slots, account 0, mainnet). Returns `(concrete_descriptor, origins)`.
fn h12_bundle_concrete(at_template: &str) -> (String, Vec<String>) {
    let out = AssertCommand::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--network",
            "mainnet",
            "--account",
            "0",
            "--descriptor",
            at_template,
            "--slot",
            &format!("@0.phrase={SEED_A}"),
            "--slot",
            &format!("@1.phrase={SEED_B}"),
            "--no-engraving-card",
            "--json",
        ])
        .output()
        .expect("spawn mnemonic bundle");
    assert!(
        out.status.success(),
        "h12 bundle failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v: Value = serde_json::from_slice(&out.stdout).expect("bundle --json");
    let cosigners = v["multisig"]["cosigners"].as_array().expect("cosigners");
    let xpubs: Vec<String> = cosigners
        .iter()
        .map(|c| c["xpub"].as_str().expect("xpub").to_string())
        .collect();
    let origins: Vec<String> = cosigners
        .iter()
        .map(|c| c["origin_path"].as_str().expect("origin_path").to_string())
        .collect();
    assert_eq!(xpubs.len(), 2, "2-cosigner taproot multisig");
    // `multi_a` (unsorted) preserves cosigner order; `sortedmulti_a` is order-
    // independent for derivation, so positional order here is fine for both.
    let inner = if at_template.contains("sortedmulti_a") {
        "sortedmulti_a"
    } else {
        "multi_a"
    };
    let desc = format!(
        "tr({NUMS_HEX},{inner}(2,{}/<0;1>/*,{}/<0;1>/*))",
        xpubs[0], xpubs[1]
    );
    (desc, origins)
}

/// Concrete `tr(NUMS, <inner>(2, …))` from the two seeds derived at `st_path`
/// (the INDEPENDENT origin-subtree oracle).
fn h12_independent_concrete(inner: &str, st_path: &str) -> String {
    let (xa, _) = xpub_at(SEED_A, st_path);
    let (xb, _) = xpub_at(SEED_B, st_path);
    format!("tr({NUMS_HEX},{inner}(2,{xa}/<0;1>/*,{xb}/<0;1>/*))")
}

/// DEFAULT-CI anti-vacuity leg (NOT `#[ignore]`): the toolkit's bundled taproot
/// cosigner xpubs (origin defaulted) must derive the SAME addresses as the
/// INDEPENDENT `3'` subtree, and DIFFER from the `2'` subtree. This is the
/// rust-miniscript half of the H12 oracle; it gates the fix without a node.
#[test]
fn h12_taproot_default_origin_anti_vacuity_leg() {
    for (template, inner) in [
        ("tr(NUMS,multi_a(2,@0/<0;1>/*,@1/<0;1>/*))", "multi_a"),
        (
            "tr(NUMS,sortedmulti_a(2,@0/<0;1>/*,@1/<0;1>/*))",
            "sortedmulti_a",
        ),
    ] {
        let (bundled_desc, origins) = h12_bundle_concrete(template);

        // (a) The emitted origin is the 3' (P2TR) subtree.
        for op in &origins {
            assert_eq!(
                op, "m/48'/0'/0'/3'",
                "[{inner}] taproot cosigner origin must default to 3'; got {op}"
            );
        }

        let bundled_addrs = derive_receive(&bundled_desc, N + 1);
        let addrs_3 = derive_receive(&h12_independent_concrete(inner, "48'/0'/0'/3'"), N + 1);
        let addrs_2 = derive_receive(&h12_independent_concrete(inner, "48'/0'/0'/2'"), N + 1);

        // (b) MATCH the 3' subtree.
        assert_eq!(
            bundled_addrs, addrs_3,
            "[{inner}] bundled taproot addresses must equal the INDEPENDENT 3' derivation"
        );
        // (c) DIFFER from the 2' subtree (the pre-H12 wrong subtree).
        assert_ne!(
            bundled_addrs, addrs_2,
            "[{inner}] bundled taproot addresses must DIFFER from the 2' (wrong) subtree — \
             a 2'/3' subtree confusion would make this vacuous"
        );
        // Sanity: 2' and 3' subtrees are genuinely distinct (anti-vacuity of (c)).
        assert_ne!(
            addrs_3, addrs_2,
            "[{inner}] the 2' and 3' subtree derivations must differ (oracle sanity)"
        );
    }
}

/// Env-gated heavy leg: Core `deriveaddresses` corroborates the H12 oracle — the
/// toolkit's bundled taproot descriptor matches Core on the 3' subtree and
/// DIFFERS from Core on the 2' subtree. Same CONNECT-ONLY contract.
#[test]
#[ignore = "requires a pre-running offline -chain=main bitcoind (wiring env vars)"]
fn bitcoind_h12_taproot_default_origin_differential() {
    let Some(w) = read_wiring() else {
        eprintln!(
            "skipping: bitcoind env not set (BITCOINCLI_BIN/BITCOIND_DATADIR/BITCOIND_RPCPORT)"
        );
        return;
    };
    let info = bitcoin_cli(&w, &["getblockchaininfo"]);
    assert_eq!(
        info.get("chain").and_then(|c| c.as_str()),
        Some("main"),
        "bitcoind must be on -chain=main (got {info:?})"
    );

    let mut total_checks = 0usize;
    for (template, inner) in [
        ("tr(NUMS,multi_a(2,@0/<0;1>/*,@1/<0;1>/*))", "multi_a"),
        (
            "tr(NUMS,sortedmulti_a(2,@0/<0;1>/*,@1/<0;1>/*))",
            "sortedmulti_a",
        ),
    ] {
        let (bundled_desc, origins) = h12_bundle_concrete(template);
        for op in &origins {
            assert_eq!(op, "m/48'/0'/0'/3'", "[{inner}] origin must be 3'");
        }

        let bundled_addrs = derive_receive(&bundled_desc, N + 1);
        let desc3 = h12_independent_concrete(inner, "48'/0'/0'/3'");
        let desc2 = h12_independent_concrete(inner, "48'/0'/0'/2'");

        // Anti-vacuity BEFORE Core: bundled == independent 3'.
        assert_eq!(
            bundled_addrs,
            derive_receive(&desc3, N + 1),
            "[{inner}] anti-vacuity: bundled addrs must equal independent 3' BEFORE Core"
        );

        // Core on the BUNDLED descriptor's chain-0 == toolkit's reported addrs.
        let core_bundled = core_addresses(&w, &single_chain_desc(&bundled_desc, 0));
        // Core on the independent 3' and 2' subtrees.
        let core_3 = core_addresses(&w, &single_chain_desc(&desc3, 0));
        let core_2 = core_addresses(&w, &single_chain_desc(&desc2, 0));

        for i in 0..=(N as usize) {
            assert_eq!(
                bundled_addrs[i], core_bundled[i],
                "H12 ADDRESS DIVERGENCE (FUNDS-CRITICAL) [{inner}] idx{i}: \
                 toolkit={} bitcoind={}",
                bundled_addrs[i], core_bundled[i]
            );
            assert_eq!(
                core_bundled[i], core_3[i],
                "[{inner}] idx{i}: bundled must match Core on the 3' subtree"
            );
            assert_ne!(
                core_bundled[i], core_2[i],
                "[{inner}] idx{i}: bundled (3') must DIFFER from Core on the 2' subtree"
            );
            total_checks += 1;
        }
    }
    eprintln!(
        "toolkit bitcoind H12 differential PASS: {total_checks} taproot 3'-vs-2' \
         address checks, byte-identical to Core on 3' and diverging from 2'"
    );
}

// H1 (cycle-1) — verify-bundle MUST report `result: mismatch` (exit≠0) for a
// supplied md1 that reconstructs a DIFFERENT wallet than the engraved bundle —
// even when the cosigner pubkey SET is identical (the legacy sorted-multiset
// gate false-GREENed these). Discriminator cases: wrong threshold
// (`sortedmulti(1)` vs `(2)`), sorted-vs-unsorted (`multi` vs `sortedmulti`),
// script-type wrapper (`sh-wsh` vs `wsh`), and the C-PLAN-1 multipath-divergence
// (`<0;1>` vs `<2;3>` change-chains → DIFFERENT watched-address set, same
// `.tree`). The verdict is verify-bundle exit-code-behavioral (no Core derive
// needed for the mismatch verdict). The `<0;1>`-vs-`<2;3>` "different addresses"
// premise is already anchored by `derive_receive` over divergent multipath
// groups elsewhere in this harness (the corpus shapes + `divergent_differential_
// golden`). DEFAULT-CI (NOT `#[ignore]`) — needs no bitcoind.

/// Generate a watch-only multisig bundle's `(ms1, mk1_flat, md1)` from the SAME
/// two seeds (SEED_A/SEED_B) for a chosen `--template` + `--threshold`. The
/// cosigner pubkey SET is identical across all template/threshold choices.
fn h1_bundle_cards(template: &str, threshold: &str) -> (Vec<String>, Vec<String>, Vec<String>) {
    let out = AssertCommand::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--network",
            "mainnet",
            "--template",
            template,
            "--threshold",
            threshold,
            "--slot",
            &format!("@0.phrase={SEED_A}"),
            "--slot",
            &format!("@1.phrase={SEED_B}"),
            "--json",
        ])
        .output()
        .expect("spawn mnemonic bundle");
    assert!(
        out.status.success(),
        "h1 bundle failed ({template}/{threshold}): {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v: Value = serde_json::from_slice(&out.stdout).expect("bundle --json");
    let ms1 = v["ms1"]
        .as_array()
        .expect("ms1 array")
        .iter()
        .map(|x| x.as_str().unwrap().to_string())
        .collect();
    let mut mk1 = Vec::new();
    for inner in v["mk1"].as_array().expect("mk1 array") {
        for chunk in inner.as_array().expect("inner mk1 array") {
            mk1.push(chunk.as_str().unwrap().to_string());
        }
    }
    let md1 = v["md1"]
        .as_array()
        .expect("md1 array")
        .iter()
        .map(|x| x.as_str().unwrap().to_string())
        .collect();
    (ms1, mk1, md1)
}

/// Run `verify-bundle` for the GENUINE 2-of-3… (here 2-of-2) `wsh-sortedmulti`
/// expected (derived from SEED_A/SEED_B) but supply a (possibly DIVERGENT) md1.
/// Returns whether verify-bundle reported success (`result: ok`, exit 0).
fn h1_verify_supplied_md1(
    genuine_ms1: &[String],
    genuine_mk1: &[String],
    supplied_md1: &[String],
) -> bool {
    let mut args: Vec<String> = vec![
        "verify-bundle".into(),
        "--network".into(),
        "mainnet".into(),
        "--template".into(),
        "wsh-sortedmulti".into(),
        "--threshold".into(),
        "2".into(),
        "--slot".into(),
        format!("@0.phrase={SEED_A}"),
        "--slot".into(),
        format!("@1.phrase={SEED_B}"),
    ];
    for s in genuine_ms1 {
        args.push("--ms1".into());
        args.push(s.clone());
    }
    for s in genuine_mk1 {
        args.push("--mk1".into());
        args.push(s.clone());
    }
    for s in supplied_md1 {
        args.push("--md1".into());
        args.push(s.clone());
    }
    let out = AssertCommand::cargo_bin("mnemonic")
        .unwrap()
        .args(&args)
        .output()
        .expect("spawn mnemonic verify-bundle");
    out.status.success()
}

/// Replace the `<0;1>` change-chain multipath in every md1-bearing engrave with
/// `<2;3>` by re-routing through `bundle --descriptor` on the read-back cosigner
/// xpubs. (We reuse the toolkit's own bundle pipeline so the resulting md1 is a
/// valid, decodable card with the SAME `.tree`/pubkeys but a DIVERGENT
/// `use_site_path` — the C-PLAN-1 case.)
fn h1_divergent_multipath_md1() -> Vec<String> {
    // Read back the genuine cosigner xpubs from a wsh-sortedmulti(2) bundle.
    let out = AssertCommand::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--network",
            "mainnet",
            "--template",
            "wsh-sortedmulti",
            "--threshold",
            "2",
            "--slot",
            &format!("@0.phrase={SEED_A}"),
            "--slot",
            &format!("@1.phrase={SEED_B}"),
            "--json",
        ])
        .output()
        .expect("spawn mnemonic bundle");
    assert!(out.status.success());
    let v: Value = serde_json::from_slice(&out.stdout).expect("bundle --json");
    let cosigners = v["multisig"]["cosigners"].as_array().expect("cosigners");
    let xpubs: Vec<String> = cosigners
        .iter()
        .map(|c| c["xpub"].as_str().expect("xpub").to_string())
        .collect();
    let origins: Vec<String> = cosigners
        .iter()
        .map(|c| c["origin_path"].as_str().expect("origin_path").to_string())
        .collect();
    // Build a concrete descriptor with the SAME pubkeys/threshold/script-type
    // but `<2;3>` change-chains, with EXPLICIT origins (so re-bundle derives the
    // identical cosigner xpubs → identical pubkeys/.tree, only use_site_path
    // differs).
    let origin_strip = |op: &str| op.trim_start_matches("m/").to_string();
    let desc = format!(
        "wsh(sortedmulti(2,[{fp0}/{o0}]{x0}/<2;3>/*,[{fp1}/{o1}]{x1}/<2;3>/*))",
        fp0 = cosigners[0]["master_fingerprint"].as_str().unwrap(),
        o0 = origin_strip(&origins[0]),
        x0 = xpubs[0],
        fp1 = cosigners[1]["master_fingerprint"].as_str().unwrap(),
        o1 = origin_strip(&origins[1]),
        x1 = xpubs[1],
    );
    bundle_md1(&desc)
}

/// DEFAULT-CI H1 discriminator: a divergent-policy supplied md1 (wrong-k /
/// sorted-vs-unsorted / script-type / multipath) must make verify-bundle FAIL
/// (`result: mismatch`, exit≠0); the genuine md1 must PASS (`result: ok`).
#[test]
fn h1_verify_bundle_rejects_divergent_policy_md1() {
    // The GENUINE engraved bundle: 2-of-2 wsh-sortedmulti.
    let (ms1, mk1, genuine_md1) = h1_bundle_cards("wsh-sortedmulti", "2");

    // CLEAN-NEGATIVE: genuine md1 → PASS (no over-rejection).
    assert!(
        h1_verify_supplied_md1(&ms1, &mk1, &genuine_md1),
        "genuine matching md1 must verify OK (no over-rejection)"
    );

    // 1. wrong threshold: 1-of-2 anyone-spends (same pubkey set).
    let (_, _, k1_md1) = h1_bundle_cards("wsh-sortedmulti", "1");
    assert!(
        !h1_verify_supplied_md1(&ms1, &mk1, &k1_md1),
        "wrong-threshold (1-of-2 vs 2-of-2) md1 must FAIL verify-bundle"
    );

    // 2. sorted-vs-unsorted: wsh(multi(2,…)) (different Tag).
    let (_, _, unsorted_md1) = h1_bundle_cards("wsh-multi", "2");
    assert!(
        !h1_verify_supplied_md1(&ms1, &mk1, &unsorted_md1),
        "sorted-vs-unsorted md1 must FAIL verify-bundle"
    );

    // 3. script-type wrapper: sh(wsh(sortedmulti(2,…))) (P2SH-nested).
    let (_, _, shwsh_md1) = h1_bundle_cards("sh-wsh-sortedmulti", "2");
    assert!(
        !h1_verify_supplied_md1(&ms1, &mk1, &shwsh_md1),
        "wsh-vs-sh(wsh(...)) md1 must FAIL verify-bundle"
    );

    // 4. C-PLAN-1 multipath divergence: <0;1> vs <2;3> (same .tree, same
    //    pubkeys, DIFFERENT watched-address set).
    let divergent_md1 = h1_divergent_multipath_md1();
    assert!(
        !h1_verify_supplied_md1(&ms1, &mk1, &divergent_md1),
        "use_site_path-divergent (<0;1> vs <2;3>) md1 must FAIL verify-bundle — \
         a .tree-only gate would false-GREEN this different-address wallet"
    );
}
