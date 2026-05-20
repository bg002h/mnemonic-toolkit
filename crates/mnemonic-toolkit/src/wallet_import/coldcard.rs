//! Coldcard single-sig `wallet.json` parser (Phase P3).
//!
//! Per `design/SPEC_wallet_import_v0_28_0.md` §11.3. Coldcard's generic-
//! wallet-export `wallet.json` is the singlesig surface; the multisig
//! text-format export lands in §11.4 (`coldcard_multisig.rs`, Phase P4).
//!
//! ## On-disk shape (multiple firmware variants — see SPEC §11.3.1 table)
//!
//! ```json
//! {
//!   "chain": "BTC" | "XTN",
//!   "xfp": "<8-char uppercase hex master fingerprint>",
//!   "account": 0,
//!   "xpub": "<top-level account xpub — legacy Mk1/Mk2 firmware>",
//!   "bip44": { "name": "p2pkh",      "deriv": "m/44'/<coin>'/<acc>'", "xfp": "<parent>", "xpub": "...", "first": "..." },
//!   "bip49": { "name": "p2wpkh-p2sh","deriv": "m/49'/<coin>'/<acc>'", "xfp": "<parent>", "xpub": "...", "_pub": "ypub...", "first": "..." },
//!   "bip84": { "name": "p2wpkh",     "deriv": "m/84'/<coin>'/<acc>'", "xfp": "<parent>", "xpub": "...", "_pub": "zpub...", "first": "..." },
//!   "bip86": { "name": "p2tr",       "deriv": "m/86'/<coin>'/<acc>'", "xfp": "<parent>", "xpub": "...", "first": "..." },
//!   "bip48_1": { ... multisig hint — IGNORED by single-sig parser ... },
//!   "bip48_2": { ... multisig hint — IGNORED by single-sig parser ... }
//! }
//! ```
//!
//! ## Sniff signature (SPEC §11.3, Q3-lock relaxed per R0 I8)
//!
//! Top-level JSON object containing ALL of:
//! - `chain` ∈ {"BTC", "XTN"}
//! - `xfp` (string)
//! - At-least-one-of: `xpub`, `bip44`, `bip49`, `bip84`, `bip86`, `bip48_1`,
//!   `bip48_2`
//!
//! The disjunction in the third clause absorbs Coldcard firmware variance
//! (different firmware versions emit different combinations of per-BIP
//! derivation blocks). See SPEC §11.3.1 for the firmware-variance table.
//!
//! ## Parse contract (SPEC §11.3 + §11.3.1)
//!
//! Phase P3A is **skeleton + sniff only**; `parse` returns `unimplemented!()`.
//! Phase P3B implements the real parse:
//! 1. Extract `chain` → network (BTC → mainnet, XTN → testnet).
//! 2. Extract `xfp` (string) → master fingerprint (`[u8; 4]`).
//! 3. Pick dominant BIP block per SPEC §11.3.1 dominance order:
//!    BIP-86 > BIP-84 > BIP-49 > BIP-44, falling back to top-level `xpub` +
//!    SLIP-132 prefix inference for legacy Mk1/Mk2 firmware.
//! 4. Build a descriptor body from the selected block's `deriv`, parent
//!    `xfp`, and `xpub`; route through the same `parse_descriptor` pipeline
//!    as BSMS / Bitcoin Core.
//! 5. `bip48_1` / `bip48_2` are silently IGNORED — they are multisig-context
//!    hints; the authoritative multisig surface is Phase P4
//!    (`coldcard_multisig.rs`).

use super::WalletFormatParser;
use crate::error::ToolkitError;
use serde_json::Value;
use std::io::Write;

#[allow(dead_code)] // P3A: only the type-level skeleton lands at this phase.
                    // Phase P3B's `parse` body wires the parser into the
                    // dispatch flow + uses the constructor; clippy false-
                    // positives on dead_code for an empty unit struct that
                    // is constructed only inside the trait impl during P3B.
pub(crate) struct ColdcardParser;

/// Sniff-time marker set for the third clause of the SPEC §11.3 sniff
/// predicate: presence of ANY of these top-level keys (alongside `chain` +
/// `xfp`) classifies the blob as Coldcard.
///
/// Listed in alphabetical order to match the SPEC §11.3.1 firmware-variance
/// table reading order; ordering is not load-bearing for the sniff (the
/// predicate is a logical OR), only for human-readable diff-stability.
const COLDCARD_PER_BIP_MARKERS: &[&str] = &[
    "bip44", "bip48_1", "bip48_2", "bip49", "bip84", "bip86", "xpub",
];

/// SPEC §11.3 — `chain` field domain. BTC → mainnet, XTN → testnet
/// (signet/regtest absent from Coldcard's schema; the export side maps
/// signet → "XTN" by convention but the import side accepts only the two
/// canonical values per SPEC).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // P3A: variant constructed in P3B parse-impl
pub(crate) enum ColdcardChain {
    Btc,
    Xtn,
}

/// SPEC §11.3 — dominant-BIP selection result. Phase P3B's parse impl picks
/// ONE variant per `ColdcardSourceMetadata` per the §11.3.1 dominance order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // P3A: variants constructed in P3B parse-impl
pub(crate) enum ColdcardBip {
    Bip44,
    Bip49,
    Bip84,
    Bip86,
}

/// SPEC §11.3 — Coldcard single-sig parser provenance.
///
/// Carried inside `ImportProvenance::Coldcard(...)` and surfaced via the
/// `--json` envelope's `source_metadata` field (Phase P3B/P3C wire-up).
#[derive(Debug, Clone)]
#[allow(dead_code)] // P3A: fields populated in P3B parse-impl; held here so
                    // the type is structurally complete + so cross-instance
                    // dispatch site at `mod.rs::ImportProvenance` can wire
                    // up at P0C without a follow-up forward-ref.
pub(crate) struct ColdcardSourceMetadata {
    /// SPEC §11.3 — `chain` field value (BTC / XTN).
    pub chain: ColdcardChain,
    /// SPEC §11.3 — master fingerprint extracted from top-level `xfp`.
    pub xfp: [u8; 4],
    /// SPEC §11.3.1 — which BIP block the parser selected via dominance order.
    pub bip_derivation: ColdcardBip,
    /// SPEC §11.3 — `account` field value (defaults to 0 if absent per
    /// Coldcard schema).
    pub raw_account: u32,
    /// SPEC §11.3 — per-entry fields that appeared in the source blob but
    /// are not preserved in the toolkit's parsed bundle (e.g. `first` address
    /// strings, `_pub` SLIP-132 alternates when the BIP-32 `xpub` carries
    /// the canonical key already). Empty unless P3B's parse populates it.
    pub dropped_fields: Vec<String>,
}

impl WalletFormatParser for ColdcardParser {
    /// SPEC §11.3 sniff predicate (Q3-lock relaxed per R0 I8).
    ///
    /// Returns `true` if the blob is a JSON object containing all of:
    /// - `chain` ∈ {"BTC", "XTN"}
    /// - `xfp` (any string value)
    /// - At least one of: `xpub`, `bip44`, `bip49`, `bip84`, `bip86`,
    ///   `bip48_1`, `bip48_2`
    ///
    /// The third clause is a disjunction — it absorbs firmware-variant
    /// shapes (Mk1/Mk2 emit only `xpub`; Mk3 adds `bip44`/`bip49`/`bip84`
    /// blocks; Mk4 adds `bip86`; Q adds `bip48_1`/`bip48_2`). See SPEC
    /// §11.3.1 firmware-variance table.
    fn sniff(blob: &[u8]) -> bool {
        let trimmed = trim_leading_ws(blob);
        if !trimmed.starts_with(b"{") {
            return false;
        }
        let value: Value = match serde_json::from_slice(blob) {
            Ok(v) => v,
            Err(_) => return false,
        };
        let obj = match value.as_object() {
            Some(o) => o,
            None => return false,
        };

        // Clause 1: `chain` ∈ {BTC, XTN}.
        let chain_ok = obj
            .get("chain")
            .and_then(|v| v.as_str())
            .map(|s| s == "BTC" || s == "XTN")
            .unwrap_or(false);
        if !chain_ok {
            return false;
        }

        // Clause 2: `xfp` is present as a string.
        if !obj.get("xfp").map(|v| v.is_string()).unwrap_or(false) {
            return false;
        }

        // Clause 3: at-least-one-of the per-BIP markers.
        COLDCARD_PER_BIP_MARKERS
            .iter()
            .any(|m| obj.contains_key(*m))
    }

    fn parse(_blob: &[u8], _stderr: &mut dyn Write) -> Result<Vec<super::ParsedImport>, ToolkitError> {
        // Phase P3A delivers the type-level skeleton + sniff only; the
        // parse body lands in Phase P3B.
        unimplemented!(
            "Phase P3B: ColdcardParser::parse not yet wired (skeleton-only at P3A per SPEC §11.3)"
        )
    }
}

/// Strip leading ASCII whitespace before the JSON parse for sniff
/// robustness. Mirrors `bitcoin_core::trim_leading_ws`.
fn trim_leading_ws(blob: &[u8]) -> &[u8] {
    let mut i = 0;
    while i < blob.len()
        && (blob[i] == b' ' || blob[i] == b'\t' || blob[i] == b'\n' || blob[i] == b'\r')
    {
        i += 1;
    }
    &blob[i..]
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------------
    // Sniff predicate — SPEC §11.3 clause coverage
    // ---------------------------------------------------------------------

    /// SPEC §11.3 happy path: BTC + xfp + bip84 → sniff TRUE.
    #[test]
    fn sniff_true_on_mk3_bip84_btc() {
        let blob = br#"{"chain":"BTC","xfp":"5436D724","account":0,"bip84":{"name":"p2wpkh","deriv":"m/84'/0'/0'","xfp":"127EA0E6","xpub":"xpub...","first":"bc1q..."}}"#;
        assert!(ColdcardParser::sniff(blob), "BTC + xfp + bip84 must sniff true");
    }

    /// SPEC §11.3 happy path: XTN + xfp + bip49 → sniff TRUE.
    #[test]
    fn sniff_true_on_mk3_bip49_xtn() {
        let blob = br#"{"chain":"XTN","xfp":"5436D724","account":0,"bip49":{"name":"p2wpkh-p2sh","deriv":"m/49'/1'/0'","xfp":"CF1D3830","xpub":"tpub...","first":"2N..."}}"#;
        assert!(ColdcardParser::sniff(blob), "XTN + xfp + bip49 must sniff true");
    }

    /// SPEC §11.3 happy path: BTC + xfp + bip44 → sniff TRUE.
    #[test]
    fn sniff_true_on_mk3_bip44_btc() {
        let blob = br#"{"chain":"BTC","xfp":"5436D724","account":0,"bip44":{"name":"p2pkh","deriv":"m/44'/0'/0'","xfp":"ABCDEF01","xpub":"xpub..."}}"#;
        assert!(ColdcardParser::sniff(blob), "BTC + xfp + bip44 must sniff true");
    }

    /// SPEC §11.3.1 Mk4-era variance: BTC + xfp + bip86 (taproot) → sniff TRUE.
    #[test]
    fn sniff_true_on_mk4_bip86_btc() {
        let blob = br#"{"chain":"BTC","xfp":"5436D724","account":0,"bip86":{"name":"p2tr","deriv":"m/86'/0'/0'","xfp":"ABCDEF01","xpub":"xpub..."}}"#;
        assert!(ColdcardParser::sniff(blob), "BTC + xfp + bip86 must sniff true");
    }

    /// SPEC §11.3.1 Q-era variance: BTC + xfp + bip48_2 (multisig hint, no
    /// singlesig BIP block) → sniff TRUE per Q3-lock disjunction. The
    /// parser is still classifies the blob as Coldcard at sniff time;
    /// Phase P3B's dominant-BIP selection chooses among `bip44/49/84/86`
    /// (or top-level `xpub`) — falling back to a parse error if none of
    /// those is present alongside `bip48_*`.
    #[test]
    fn sniff_true_on_q_bip48_only() {
        let blob = br#"{"chain":"BTC","xfp":"5436D724","account":0,"bip48_2":{"name":"p2wsh","deriv":"m/48'/0'/0'/2'","xpub":"xpub..."}}"#;
        assert!(
            ColdcardParser::sniff(blob),
            "Q-era bip48_* only must sniff true per SPEC §11.3 Q3-lock disjunction"
        );
    }

    /// SPEC §11.3.1 Mk1/Mk2 legacy: BTC + xfp + top-level `xpub` (no bip*
    /// blocks at all) → sniff TRUE per Q3-lock disjunction (third clause
    /// absorbs legacy firmware variance).
    #[test]
    fn sniff_true_on_mk1_legacy_top_level_xpub() {
        let blob = br#"{"chain":"BTC","xfp":"5436D724","account":0,"xpub":"xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX"}"#;
        assert!(
            ColdcardParser::sniff(blob),
            "Mk1/Mk2 legacy top-level xpub-only must sniff true per SPEC §11.3 Q3-lock disjunction"
        );
    }

    // ---------------------------------------------------------------------
    // Sniff predicate — refusal cases
    // ---------------------------------------------------------------------

    /// SPEC §11.3 clause 1 violation: `chain` value outside {BTC, XTN} →
    /// sniff FALSE. Defends against Specter/Sparrow blobs that happen to
    /// carry a `chain` field with `"main"` / `"test"` values.
    #[test]
    fn sniff_false_on_chain_value_main_specter_style() {
        let blob = br#"{"chain":"main","xfp":"5436D724","bip84":{"name":"p2wpkh","xpub":"xpub..."}}"#;
        assert!(
            !ColdcardParser::sniff(blob),
            "chain=`main` (Specter convention, not Coldcard) must sniff false"
        );
    }

    /// SPEC §11.3 clause 1 violation: `chain` absent → sniff FALSE.
    #[test]
    fn sniff_false_on_missing_chain() {
        let blob = br#"{"xfp":"5436D724","bip84":{"name":"p2wpkh","xpub":"xpub..."}}"#;
        assert!(!ColdcardParser::sniff(blob), "missing chain must sniff false");
    }

    /// SPEC §11.3 clause 2 violation: `xfp` absent → sniff FALSE.
    #[test]
    fn sniff_false_on_missing_xfp() {
        let blob = br#"{"chain":"BTC","bip84":{"name":"p2wpkh","xpub":"xpub..."}}"#;
        assert!(!ColdcardParser::sniff(blob), "missing xfp must sniff false");
    }

    /// SPEC §11.3 clause 2 violation: `xfp` present but not a string
    /// (e.g. an integer) → sniff FALSE.
    #[test]
    fn sniff_false_on_xfp_non_string() {
        let blob = br#"{"chain":"BTC","xfp":12345,"bip84":{"name":"p2wpkh","xpub":"xpub..."}}"#;
        assert!(!ColdcardParser::sniff(blob), "xfp non-string must sniff false");
    }

    /// SPEC §11.3 clause 3 violation: chain + xfp present but no per-BIP
    /// markers → sniff FALSE.
    #[test]
    fn sniff_false_on_chain_and_xfp_alone() {
        let blob = br#"{"chain":"BTC","xfp":"5436D724","account":0}"#;
        assert!(
            !ColdcardParser::sniff(blob),
            "chain + xfp without per-BIP markers must sniff false (clause 3)"
        );
    }

    /// Robustness: invalid JSON → sniff FALSE.
    #[test]
    fn sniff_false_on_invalid_json() {
        let blob = b"{not json}";
        assert!(!ColdcardParser::sniff(blob), "invalid JSON must sniff false");
    }

    /// Robustness: BSMS text blob → sniff FALSE (BSMS leads with `BSMS 1.0`,
    /// not `{`).
    #[test]
    fn sniff_false_on_bsms_text_blob() {
        let blob = b"BSMS 1.0\nwpkh(xpub...)\n";
        assert!(!ColdcardParser::sniff(blob), "BSMS text blob must sniff false");
    }

    /// Robustness: bare JSON array (Bitcoin Core's bare-array shape) →
    /// sniff FALSE.
    #[test]
    fn sniff_false_on_bare_array() {
        let blob = br#"[{"desc":"wpkh(xpub...)"}]"#;
        assert!(!ColdcardParser::sniff(blob), "bare array must sniff false");
    }

    /// Robustness: empty blob → sniff FALSE.
    #[test]
    fn sniff_false_on_empty() {
        assert!(!ColdcardParser::sniff(b""), "empty blob must sniff false");
    }

    /// Robustness: leading whitespace before `{` → sniff TRUE (per
    /// `trim_leading_ws` discipline mirroring `bitcoin_core.rs`).
    #[test]
    fn sniff_true_on_leading_whitespace() {
        let blob = b"  \n\t{\"chain\":\"BTC\",\"xfp\":\"5436D724\",\"bip84\":{\"xpub\":\"xpub...\"}}";
        assert!(
            ColdcardParser::sniff(blob),
            "leading whitespace must not block sniff TRUE"
        );
    }

    // ---------------------------------------------------------------------
    // Format-disambiguation — sniff must NOT claim other vendors' blobs
    // ---------------------------------------------------------------------

    /// Bitcoin Core's `listdescriptors` JSON has `descriptors` array, NO
    /// `chain` key (and NO `xfp`) → Coldcard sniff FALSE.
    #[test]
    fn sniff_false_on_bitcoin_core_listdescriptors() {
        let blob = br#"{"wallet_name":"x","descriptors":[{"desc":"wpkh(xpub...)#00000000"}]}"#;
        assert!(
            !ColdcardParser::sniff(blob),
            "Bitcoin Core listdescriptors blob must sniff false on Coldcard"
        );
    }

    /// Specter's wallet JSON carries `blockheight`, `devices`, `descriptor`,
    /// `label` keys but not the Coldcard's `xfp` + `chain ∈ {BTC, XTN}`
    /// combination (Specter uses lowercase `"main"`/`"test"` for chain).
    #[test]
    fn sniff_false_on_specter_blob() {
        let blob = br#"{"chain":"main","label":"daily","blockheight":700000,"devices":["unknown"],"descriptor":"wpkh(xpub...)"}"#;
        assert!(
            !ColdcardParser::sniff(blob),
            "Specter blob (chain=main) must sniff false on Coldcard"
        );
    }

    /// Electrum wallet JSON has `seed_version` + `wallet_type`, no `chain`
    /// — Coldcard sniff FALSE.
    #[test]
    fn sniff_false_on_electrum_blob() {
        let blob = br#"{"seed_version":42,"wallet_type":"standard","keystore":{"xpub":"xpub..."}}"#;
        assert!(
            !ColdcardParser::sniff(blob),
            "Electrum blob must sniff false on Coldcard"
        );
    }

    /// Jade's `multisig_file`-shape JSON wrapper does not carry a top-level
    /// `chain` field — Coldcard sniff FALSE.
    #[test]
    fn sniff_false_on_jade_multisig_file_blob() {
        let blob = br#"{"multisig_file":"Name: foo\nPolicy: 2-of-3\n..."}"#;
        assert!(
            !ColdcardParser::sniff(blob),
            "Jade multisig_file blob must sniff false on Coldcard"
        );
    }

    // ---------------------------------------------------------------------
    // P3A skeleton invariants
    // ---------------------------------------------------------------------

    /// P3A delivers skeleton-only; `parse` MUST panic. P3B replaces the
    /// `unimplemented!()` body with a real impl.
    #[test]
    #[should_panic(expected = "Phase P3B")]
    fn parse_skeleton_panics_at_p3a() {
        let blob = br#"{"chain":"BTC","xfp":"5436D724","bip84":{"name":"p2wpkh","xpub":"xpub..."}}"#;
        let mut stderr = Vec::new();
        let _ = ColdcardParser::parse(blob, &mut stderr);
    }

    /// Provenance type-level invariants — ensure ColdcardSourceMetadata
    /// fields are stable across P3A → P3B → P3C. Constructed inline (no
    /// Default impl) so any field-shape drift surfaces here at compile time.
    #[test]
    fn provenance_struct_is_constructible_p3a_shape_lock() {
        let _meta = ColdcardSourceMetadata {
            chain: ColdcardChain::Btc,
            xfp: [0x54, 0x36, 0xD7, 0x24],
            bip_derivation: ColdcardBip::Bip84,
            raw_account: 0,
            dropped_fields: Vec::new(),
        };
    }
}
