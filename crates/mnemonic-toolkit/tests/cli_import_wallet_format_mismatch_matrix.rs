//! Cross-format mismatch matrix — Option B narrow set (v0.28.7 / Slug 3).
//!
//! Closes FOLLOWUP `wallet-import-format-mismatch-matrix-completion` for
//! the 3 narrow arms (BSMS / BitcoinCore / ColdcardMultisig). The other 4
//! arms (Coldcard / Sparrow / Specter / Electrum) have additional residual
//! gaps discovered during P0 recon — tracked at NEW FOLLOWUP
//! `wallet-import-format-mismatch-matrix-completion-discovered-gaps`.

use assert_cmd::Command;

const FIXTURE_BASE: &str = "tests/fixtures/wallet_import";

fn assert_format_mismatch(user_format: &str, fixture: &str, detected_format: &str) {
    let path = std::path::PathBuf::from(FIXTURE_BASE).join(fixture);
    let out = Command::cargo_bin("mnemonic").unwrap()
        .args(["import-wallet", "--format", user_format,
               "--blob", path.to_str().unwrap(), "--json"])
        .output().expect("mnemonic spawn");
    assert_ne!(out.status.code(), Some(0),
        "expected non-zero exit for {user_format} vs {detected_format}");
    let stderr = String::from_utf8_lossy(&out.stderr);
    // The Display impl emits: "import-wallet: --format <user> supplied but blob looks like <detected>"
    // (see error.rs ImportWalletFormatMismatch Display arm). The kind() string is "ImportWalletFormatMismatch"
    // and appears in JSON envelope context. Accept either surface.
    assert!(
        stderr.contains("blob looks like") || stderr.contains("ImportWalletFormatMismatch"),
        "expected format-mismatch stderr for {user_format} vs {detected_format}, got: {stderr}"
    );
}

// FIXTURES (verified to exist at HEAD `885f522`):
// - coldcard           → coldcard-singlesig-bip84-mainnet.json
// - coldcard-multisig  → coldcard-ms-2of3-p2wsh-with-xfp.txt (NOTE: .txt)
// - electrum           → electrum-standard-bip84-mainnet.json
// - jade               → jade-multisig-2of3-p2wsh.json (only valid Jade fixture; singlesig-refused)
// - sparrow            → sparrow-singlesig-p2wpkh.json
// - specter            → specter-singlesig-p2wpkh.json

// BSMS arm — refuses 7 other formats (BitcoinCore already covered; add 6 new).
#[test] fn bsms_refuses_coldcard()           { assert_format_mismatch("bsms", "coldcard-singlesig-bip84-mainnet.json", "coldcard"); }
#[test] fn bsms_refuses_coldcard_multisig()  { assert_format_mismatch("bsms", "coldcard-ms-2of3-p2wsh-with-xfp.txt", "coldcard-multisig"); }
#[test] fn bsms_refuses_electrum()           { assert_format_mismatch("bsms", "electrum-standard-bip84-mainnet.json", "electrum"); }
#[test] fn bsms_refuses_jade()               { assert_format_mismatch("bsms", "jade-multisig-2of3-p2wsh.json", "jade"); }
#[test] fn bsms_refuses_sparrow()            { assert_format_mismatch("bsms", "sparrow-singlesig-p2wpkh.json", "sparrow"); }
#[test] fn bsms_refuses_specter()            { assert_format_mismatch("bsms", "specter-singlesig-p2wpkh.json", "specter"); }

// BitcoinCore arm — symmetric (6 new refusals).
#[test] fn bitcoin_core_refuses_coldcard()           { assert_format_mismatch("bitcoin-core", "coldcard-singlesig-bip84-mainnet.json", "coldcard"); }
#[test] fn bitcoin_core_refuses_coldcard_multisig()  { assert_format_mismatch("bitcoin-core", "coldcard-ms-2of3-p2wsh-with-xfp.txt", "coldcard-multisig"); }
#[test] fn bitcoin_core_refuses_electrum()           { assert_format_mismatch("bitcoin-core", "electrum-standard-bip84-mainnet.json", "electrum"); }
#[test] fn bitcoin_core_refuses_jade()               { assert_format_mismatch("bitcoin-core", "jade-multisig-2of3-p2wsh.json", "jade"); }
#[test] fn bitcoin_core_refuses_sparrow()            { assert_format_mismatch("bitcoin-core", "sparrow-singlesig-p2wpkh.json", "sparrow"); }
#[test] fn bitcoin_core_refuses_specter()            { assert_format_mismatch("bitcoin-core", "specter-singlesig-p2wpkh.json", "specter"); }

// ColdcardMultisig arm — 5 new refusals.
#[test] fn coldcard_multisig_refuses_coldcard()  { assert_format_mismatch("coldcard-multisig", "coldcard-singlesig-bip84-mainnet.json", "coldcard"); }
#[test] fn coldcard_multisig_refuses_electrum()  { assert_format_mismatch("coldcard-multisig", "electrum-standard-bip84-mainnet.json", "electrum"); }
#[test] fn coldcard_multisig_refuses_jade()      { assert_format_mismatch("coldcard-multisig", "jade-multisig-2of3-p2wsh.json", "jade"); }
#[test] fn coldcard_multisig_refuses_sparrow()   { assert_format_mismatch("coldcard-multisig", "sparrow-singlesig-p2wpkh.json", "sparrow"); }
#[test] fn coldcard_multisig_refuses_specter()   { assert_format_mismatch("coldcard-multisig", "specter-singlesig-p2wpkh.json", "specter"); }

// ── v0.34.4: matrix completion — the 10 residual off-diagonal arms ─────────
// Closes `wallet-import-format-mismatch-matrix-completion-discovered-gaps`.
// The 8×7 = 56-cell off-diagonal matrix is now complete (the 4 arms below
// join bsms/bitcoin-core/coldcard-multisig/jade, which were already 7/7).

// Coldcard arm — 2 new refusals.
#[test] fn coldcard_refuses_electrum()  { assert_format_mismatch("coldcard", "electrum-standard-bip84-mainnet.json", "electrum"); }
#[test] fn coldcard_refuses_jade()      { assert_format_mismatch("coldcard", "jade-multisig-2of3-p2wsh.json", "jade"); }

// Electrum arm — 1 new refusal.
#[test] fn electrum_refuses_jade()      { assert_format_mismatch("electrum", "jade-multisig-2of3-p2wsh.json", "jade"); }

// Sparrow arm — 4 new refusals.
#[test] fn sparrow_refuses_coldcard()   { assert_format_mismatch("sparrow", "coldcard-singlesig-bip84-mainnet.json", "coldcard"); }
#[test] fn sparrow_refuses_electrum()   { assert_format_mismatch("sparrow", "electrum-standard-bip84-mainnet.json", "electrum"); }
#[test] fn sparrow_refuses_jade()       { assert_format_mismatch("sparrow", "jade-multisig-2of3-p2wsh.json", "jade"); }
#[test] fn sparrow_refuses_specter()    { assert_format_mismatch("sparrow", "specter-singlesig-p2wpkh.json", "specter"); }

// Specter arm — 3 new refusals.
#[test] fn specter_refuses_coldcard()   { assert_format_mismatch("specter", "coldcard-singlesig-bip84-mainnet.json", "coldcard"); }
#[test] fn specter_refuses_electrum()   { assert_format_mismatch("specter", "electrum-standard-bip84-mainnet.json", "electrum"); }
#[test] fn specter_refuses_jade()       { assert_format_mismatch("specter", "jade-multisig-2of3-p2wsh.json", "jade"); }
