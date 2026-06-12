//! v0.4 `--slot @N.<subkey>=<value>` value-parser + validator.
//!
//! Locked by Phase 2 SPIKE-2 (`design/agent-reports/spike-toolkit-v0_4-pre-phaseA.md`).
//! Phase C consumes `parse_slot_input` as the clap value-parser and `validate_slot_set`
//! as the post-parse / pre-binding gate.
//!
//! v0.9.0 Cycle A Phase 1 — adds `apply_slot_stdin` (single-stdin consume
//! step for `@N.<secret>=-` sentinels) per SPEC §1 item 1.
#![allow(dead_code)] // parse_slot_input wired as a clap value-parser via bundle.rs / verify_bundle.rs.

use std::collections::BTreeMap;
use std::io::Read;

use crate::error::ToolkitError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SlotSubkey {
    Phrase,
    /// v0.31.3 — SeedQR digit-string (48 or 96 ASCII digits encoding a
    /// BIP-39 phrase per the SeedSigner SeedQR spec). Secret-bearing;
    /// decoded inline via `seedqr::decode` at slot-emit time, then
    /// dispatched through the same materialization path as `Phrase`.
    /// Position-critical: declared BEFORE `Entropy` so derived `Ord`
    /// slots Seedqr at position 1, making `[Seedqr, Path]` and
    /// `[Seedqr, Fingerprint, Path]` ascending-sorted (parallel to the
    /// existing `[Phrase, Path]` / `[Phrase, Fingerprint, Path]`
    /// v0.19.0 §6.6.b exception).
    Seedqr,
    Entropy,
    /// v0.41.0 — raw `ms1` codex32 secret string (BIP-93). Secret-bearing;
    /// decoded inline via `ms_codec::decode` at slot-emit time, then routed
    /// through the existing entropy materialization path (with wire-language
    /// authority for the `mnem` payload form). Declared AFTER `Entropy` so
    /// derived `Ord` slots Ms1 at position 3, making `[Ms1, Path]` and
    /// `[Ms1, Fingerprint, Path]` ascending-sorted (parallel to the existing
    /// `[Phrase, Path]` / `[Seedqr, Path]` v0.19.0 §6.6.b exceptions).
    Ms1,
    Xpub,
    /// SPEC_export_wallet_v0_8.md §2 + §5.1 — depth-0 master xpub for slot
    /// `@N`. Watch-only-class (BIP-32 base58, no secret material). Only
    /// consumed by formats that publish a master xpub at the top level
    /// (currently only Coldcard generic JSON §5.1). Optional in every
    /// invocation. Derived `Ord` slots this AFTER `Xpub` so sorted
    /// legal-sets read `[Xpub, MasterXpub, ...]`.
    MasterXpub,
    Fingerprint,
    Path,
    Wif,
    Xprv,
}

impl SlotSubkey {
    pub fn from_token(tok: &str) -> Option<Self> {
        Some(match tok {
            "phrase" => Self::Phrase,
            "seedqr" => Self::Seedqr,
            "entropy" => Self::Entropy,
            "ms1" => Self::Ms1,
            "xpub" => Self::Xpub,
            "master_xpub" => Self::MasterXpub,
            "fingerprint" => Self::Fingerprint,
            "path" => Self::Path,
            "wif" => Self::Wif,
            "xprv" => Self::Xprv,
            _ => return None,
        })
    }
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Phrase => "phrase",
            Self::Seedqr => "seedqr",
            Self::Entropy => "entropy",
            Self::Ms1 => "ms1",
            Self::Xpub => "xpub",
            Self::MasterXpub => "master_xpub",
            Self::Fingerprint => "fingerprint",
            Self::Path => "path",
            Self::Wif => "wif",
            Self::Xprv => "xprv",
        }
    }
    pub fn is_secret_bearing(self) -> bool {
        matches!(
            self,
            Self::Phrase | Self::Seedqr | Self::Entropy | Self::Ms1 | Self::Xprv | Self::Wif
        )
    }
    pub fn is_watch_only(self) -> bool {
        matches!(
            self,
            Self::Xpub | Self::MasterXpub | Self::Fingerprint | Self::Path
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotInput {
    pub index: u8,
    pub subkey: SlotSubkey,
    pub value: String,
}

impl SlotInput {
    /// SPEC v0.9.0 §1 item 1 — is this slot a `@N.<secret>=-` stdin
    /// sentinel? Returns true iff the subkey is secret-bearing AND the
    /// value is the literal `-`. Watch-only subkeys (`xpub`/`fingerprint`/
    /// `path`/`master_xpub`) NEVER consume stdin even if their value is
    /// `-` — those values are public, no argv-leakage protection needed.
    pub fn is_stdin_sentinel(&self) -> bool {
        self.subkey.is_secret_bearing() && self.value == "-"
    }
}

/// Clap value-parser error. Wraps a String so clap can format it under
/// `error: invalid value '{arg}' for '--slot <slot>': {msg}`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError(pub String);

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for ParseError {}

/// Parse `@<u8>.<subkey>=<value>` into a typed `SlotInput`.
///
/// Empty value (`@0.phrase=`) is REJECTED at the parser per SPIKE-2 lock.
pub fn parse_slot_input(s: &str) -> Result<SlotInput, ParseError> {
    let rest = s.strip_prefix('@').ok_or_else(|| {
        ParseError(format!(
            "slot input must start with '@N.<subkey>=<value>'; got {:?}",
            s
        ))
    })?;
    let dot = rest.find('.').ok_or_else(|| {
        ParseError(format!(
            "slot input missing '.<subkey>=' after '@N'; got {:?}",
            s
        ))
    })?;
    let (idx_str, after_dot) = rest.split_at(dot);
    if idx_str.is_empty() {
        return Err(ParseError(format!(
            "slot input missing index after '@'; got {:?}",
            s
        )));
    }
    let index: u8 = idx_str.parse().map_err(|_| {
        ParseError(format!(
            "slot input index must be a u8 (0..=255); got {:?}",
            idx_str
        ))
    })?;
    let after_dot = &after_dot[1..]; // skip '.'
    let eq = after_dot.find('=').ok_or_else(|| {
        ParseError(format!(
            "slot input missing '=' between subkey and value; got {:?}",
            s
        ))
    })?;
    let (subkey_tok, after_eq) = after_dot.split_at(eq);
    let value = &after_eq[1..]; // skip '='
    if subkey_tok.is_empty() {
        return Err(ParseError(format!(
            "slot input missing subkey between '.' and '='; got {:?}",
            s
        )));
    }
    let subkey = SlotSubkey::from_token(subkey_tok).ok_or_else(|| {
        ParseError(format!(
            "unknown slot subkey {:?}; expected one of: phrase, seedqr, entropy, ms1, xpub, master_xpub, fingerprint, path, wif, xprv",
            subkey_tok
        ))
    })?;
    if value.is_empty() {
        return Err(ParseError(format!(
            "slot input value is empty for subkey {:?}; supply a non-empty value",
            subkey.as_str()
        )));
    }
    Ok(SlotInput {
        index,
        subkey,
        value: value.to_string(),
    })
}

/// SPEC v0.9.0 §1 item 1 — consume stdin once for any `@N.<secret>=-`
/// slot in `slots`, substituting the stdin-read value in place.
///
/// Invariants enforced (single-stdin-per-invocation):
/// - At most ONE slot may carry `@N.<secret>=-` (refuse with
///   `ToolkitError::BadInput` otherwise — exit 1).
/// - Caller separately enforces that `--passphrase-stdin` (and any other
///   stdin-consuming flag) is NOT set when a stdin-slot is present
///   (callers do this at run() entry before calling
///   `apply_slot_stdin`).
///
/// Trailing `\r?\n` is stripped per the `convert.rs::read_stdin_passphrase`
/// precedent — preserves leading/trailing whitespace and internal NULL
/// bytes (the BIP-38 V3 NULL-byte passphrase gap motivates this).
pub fn apply_slot_stdin<R: Read + ?Sized>(
    slots: &mut [SlotInput],
    stdin: &mut R,
) -> Result<(), ToolkitError> {
    let stdin_idxs: Vec<usize> = slots
        .iter()
        .enumerate()
        .filter_map(|(i, s)| s.is_stdin_sentinel().then_some(i))
        .collect();
    match stdin_idxs.len() {
        0 => Ok(()),
        1 => {
            let mut buf = String::new();
            stdin
                .read_to_string(&mut buf)
                .map_err(|e| ToolkitError::BadInput(format!("stdin read: {e}")))?;
            if buf.ends_with('\n') {
                buf.pop();
                if buf.ends_with('\r') {
                    buf.pop();
                }
            }
            slots[stdin_idxs[0]].value = buf;
            Ok(())
        }
        _ => Err(ToolkitError::BadInput(
            "at most one --slot @N.<secret>=- per invocation (single stdin per invocation)".into(),
        )),
    }
}

/// Validate the per-slot subkey set per SPEC §6.6.b validity matrix +
/// contiguity (§6.6 row 8). Returns the SPEC §6.6 row-N error verbatim on
/// the first violation.
///
/// Allowed subkey shapes per slot:
/// - `{phrase}`                                — secret BIP-39
/// - `{entropy}`                               — secret entropy
/// - `{xprv}`                                  — secret xpriv
/// - `{wif}`                                   — secret WIF (degenerate single-key)
/// - `{xpub}` / `{xpub,fingerprint}` / `{xpub,path}` / `{xpub,fingerprint,path}`
///                                              — watch-only with origin metadata
///
/// Any other subkey set or any slot mixing secret-bearing + watch-only subkeys
/// → exit 2 + SPEC §6.6 row 4 stderr text.
/// Slot indices must be contiguous starting at @0 → exit 2 + SPEC row 8.
pub fn validate_slot_set(slots: &[SlotInput]) -> Result<(), ToolkitError> {
    let mut by_index: BTreeMap<u8, Vec<&SlotInput>> = BTreeMap::new();
    for s in slots {
        by_index.entry(s.index).or_default().push(s);
    }
    if by_index.is_empty() {
        return Ok(());
    }

    // §6.6 row 8: contiguity from @0 with no gaps.
    let max_idx = *by_index.keys().last().unwrap();
    for i in 0..=max_idx {
        if !by_index.contains_key(&i) {
            return Err(ToolkitError::SlotInputViolation {
                kind: "gap",
                message: format!("slot indices must be contiguous starting at @0; missing @{i}"),
            });
        }
    }

    // §6.6.b per-slot subkey-set validity.
    for (idx, slot_inputs) in &by_index {
        let mut subkeys: Vec<SlotSubkey> = slot_inputs.iter().map(|s| s.subkey).collect();
        subkeys.sort();
        subkeys.dedup();
        // Repeated identical subkey for the same slot is illegal — SPEC says
        // "each subkey appears at most once per slot" implicitly via the set semantics.
        if subkeys.len() != slot_inputs.len() {
            return Err(ToolkitError::SlotInputViolation {
                kind: "duplicate-subkey",
                message: format!(
                    "slot @{idx} has duplicate subkey assignments; each subkey may appear at most once per slot"
                ),
            });
        }

        let has_secret = subkeys.iter().any(|s| s.is_secret_bearing());
        let has_watch = subkeys.iter().any(|s| s.is_watch_only());

        // v0.19.0 SPEC §6.6.b exception: `{phrase, path}` and
        // `{phrase, fingerprint, path}` are legal in non-canonical
        // descriptor mode (carries explicit per-`@N` origin path with
        // optional fingerprint attestation). Exempt these specific subkey
        // sets from the secret+watch-only conflict refusal; the
        // canonical-mode rejection is enforced post-parse in
        // `cmd::bundle::bundle_run_unified_descriptor` once
        // `canonical_origin(&tree)` is known.
        let exempted_v0_19_0 = matches!(
            subkeys.as_slice(),
            [SlotSubkey::Phrase, SlotSubkey::Path]
                | [
                    SlotSubkey::Phrase,
                    SlotSubkey::Fingerprint,
                    SlotSubkey::Path
                ]
                | [SlotSubkey::Seedqr, SlotSubkey::Path]
                | [
                    SlotSubkey::Seedqr,
                    SlotSubkey::Fingerprint,
                    SlotSubkey::Path
                ]
                | [SlotSubkey::Ms1, SlotSubkey::Path]
                | [SlotSubkey::Ms1, SlotSubkey::Fingerprint, SlotSubkey::Path]
        );

        if has_secret && has_watch && !exempted_v0_19_0 {
            return Err(ToolkitError::SlotInputViolation {
                kind: "conflict",
                message: format!(
                    "slot @{idx} has both secret-bearing input and watch-only input; pick one per slot."
                ),
            });
        }

        if !is_legal_set(&subkeys) {
            return Err(ToolkitError::SlotInputViolation {
                kind: "invalid-set",
                message: format!(
                    "slot @{idx} subkey set {:?} is not in the SPEC §6.6.b validity matrix",
                    subkeys.iter().map(|s| s.as_str()).collect::<Vec<_>>()
                ),
            });
        }
    }

    Ok(())
}

/// Caller pre-sorts; SlotSubkey's derived Ord is Phrase < Entropy < Xpub <
/// MasterXpub < Fingerprint < Path < Wif < Xprv, so only the canonical-order
/// arms below are reachable. `MasterXpub` (SPEC_export_wallet_v0_8.md §2) is
/// an optional add-on to any watch-only set that already includes `Xpub`.
/// v0.19.0 (SPEC §6.6.b extension): `[Phrase, Path]` and
/// `[Phrase, Fingerprint, Path]` are added for non-canonical descriptor
/// mode (explicit per-`@N` origin path overrides the §4.12.b default).
/// Canonical-mode rejection of these pairs is enforced post-parse in
/// `cmd::bundle::bundle_run_unified_descriptor` after the descriptor's
/// canonicity verdict is known.
fn is_legal_set(set: &[SlotSubkey]) -> bool {
    use SlotSubkey::*;
    matches!(
        set,
        [Phrase]
            | [Seedqr]
            | [Entropy]
            | [Ms1]
            | [Xpub]
            | [Wif]
            | [Xprv]
            | [Xpub, MasterXpub]
            | [Xpub, Fingerprint]
            | [Xpub, MasterXpub, Fingerprint]
            | [Xpub, Path]
            | [Xpub, MasterXpub, Path]
            | [Xpub, Fingerprint, Path]
            | [Xpub, MasterXpub, Fingerprint, Path]
            | [Phrase, Path]
            | [Phrase, Fingerprint, Path]
            | [Seedqr, Path]
            | [Seedqr, Fingerprint, Path]
            | [Ms1, Fingerprint, Path]
            | [Ms1, Path]
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use mnemonic_toolkit::secret_taxonomy::SECRET_SLOT_SUBKEYS;

    fn slot(index: u8, subkey: SlotSubkey, value: &str) -> SlotInput {
        SlotInput {
            index,
            subkey,
            value: value.to_string(),
        }
    }

    // ---- secret_taxonomy parity ----

    /// Declare the complete list of `SlotSubkey` variants exactly once.
    /// See the equivalent macro doc in
    /// `cmd::convert::secret_taxonomy_parity_tests` for the full
    /// rationale: the macro produces BOTH
    /// `ALL_SLOT_SUBKEY_VARIANTS` and a `_exhaustiveness_check` whose
    /// match is non-exhaustive iff a new enum variant is added without
    /// extending the macro's input list — so the variant list and the
    /// exhaustiveness check share a single source of truth.
    macro_rules! declare_slot_subkey_variants {
        ( $( $variant:ident ),* $(,)? ) => {
            const ALL_SLOT_SUBKEY_VARIANTS: &[SlotSubkey] =
                &[ $( SlotSubkey::$variant ),* ];

            #[allow(dead_code)]
            fn _exhaustiveness_check(s: SlotSubkey) {
                match s {
                    $( SlotSubkey::$variant )|* => (),
                }
            }
        };
    }

    declare_slot_subkey_variants!(
        Phrase,
        Seedqr,
        Entropy,
        Ms1,
        Xpub,
        MasterXpub,
        Fingerprint,
        Path,
        Wif,
        Xprv,
    );

    #[test]
    fn secret_taxonomy_parity_with_is_secret_bearing() {
        for &v in ALL_SLOT_SUBKEY_VARIANTS {
            let predicate = v.is_secret_bearing();
            let in_taxonomy = SECRET_SLOT_SUBKEYS.contains(&v.as_str());
            assert_eq!(
                predicate,
                in_taxonomy,
                "drift: SlotSubkey::{:?}.is_secret_bearing()={} but \
                 secret_taxonomy::SECRET_SLOT_SUBKEYS.contains({:?})={}. \
                 If you added a SlotSubkey variant, the macro expansion \
                 above means `ALL_SLOT_SUBKEY_VARIANTS` already includes \
                 it — so this assertion is firing because the variant's \
                 secret-class status disagrees between \
                 `is_secret_bearing()` (`slot_input.rs`) and \
                 `secret_taxonomy::SECRET_SLOT_SUBKEYS` \
                 (`src/secret_taxonomy.rs`). Bring them into agreement.",
                v,
                predicate,
                v.as_str(),
                in_taxonomy,
            );
        }
    }

    #[test]
    fn secret_taxonomy_entries_round_trip_via_from_token() {
        for &token in SECRET_SLOT_SUBKEYS {
            let parsed = SlotSubkey::from_token(token).unwrap_or_else(|| {
                panic!(
                    "secret_taxonomy::SECRET_SLOT_SUBKEYS entry {:?} does \
                     not parse as a SlotSubkey via from_token — drift",
                    token
                )
            });
            assert_eq!(parsed.as_str(), token);
            assert!(
                parsed.is_secret_bearing(),
                "secret_taxonomy::SECRET_SLOT_SUBKEYS contains {:?} but \
                 SlotSubkey::{:?}.is_secret_bearing()=false — drift",
                token,
                parsed,
            );
        }
    }

    // ---- parse_slot_input ----

    #[test]
    fn parse_happy_phrase() {
        let s = "@0.phrase=word word word";
        assert_eq!(
            parse_slot_input(s).unwrap(),
            slot(0, SlotSubkey::Phrase, "word word word")
        );
    }
    #[test]
    fn parse_happy_entropy() {
        assert_eq!(
            parse_slot_input("@1.entropy=0102").unwrap(),
            slot(1, SlotSubkey::Entropy, "0102")
        );
    }
    /// v0.31.3 — `@N.seedqr=<digit-string>` parses as a typed
    /// `SlotSubkey::Seedqr`. The decode itself is deferred to slot-emit
    /// time inside `cmd/bundle.rs` (et al.); the parser is value-shape-agnostic.
    #[test]
    fn parse_happy_seedqr() {
        let digits = "000100020003000400050006000700080009001000110012";
        assert_eq!(
            parse_slot_input(&format!("@0.seedqr={digits}")).unwrap(),
            slot(0, SlotSubkey::Seedqr, digits)
        );
    }
    /// v0.31.3 — `@N.seedqr=-` triggers the existing stdin-sentinel
    /// pathway (`apply_slot_stdin`) because `Seedqr.is_secret_bearing()`
    /// returns true.
    #[test]
    fn parse_seedqr_stdin_sentinel() {
        let parsed = parse_slot_input("@0.seedqr=-").unwrap();
        assert_eq!(parsed.subkey, SlotSubkey::Seedqr);
        assert!(
            parsed.is_stdin_sentinel(),
            "@0.seedqr=- must be a stdin sentinel; got is_stdin_sentinel()=false"
        );
    }
    #[test]
    fn parse_happy_xpub() {
        assert_eq!(
            parse_slot_input("@2.xpub=xpub-stub").unwrap(),
            slot(2, SlotSubkey::Xpub, "xpub-stub")
        );
    }
    #[test]
    fn parse_happy_master_xpub() {
        // SPEC_export_wallet_v0_8.md §2 + §5.1 — depth-0 master xpub slot subkey.
        assert_eq!(
            parse_slot_input("@0.master_xpub=xpub6CUGRUo").unwrap(),
            slot(0, SlotSubkey::MasterXpub, "xpub6CUGRUo")
        );
    }
    #[test]
    fn parse_happy_fingerprint() {
        assert_eq!(
            parse_slot_input("@0.fingerprint=deadbeef").unwrap(),
            slot(0, SlotSubkey::Fingerprint, "deadbeef")
        );
    }
    #[test]
    fn parse_happy_path() {
        assert_eq!(
            parse_slot_input("@0.path=48'/0'/0'/2'").unwrap(),
            slot(0, SlotSubkey::Path, "48'/0'/0'/2'")
        );
    }
    #[test]
    fn parse_happy_wif() {
        assert_eq!(
            parse_slot_input("@0.wif=KwDi").unwrap(),
            slot(0, SlotSubkey::Wif, "KwDi")
        );
    }
    #[test]
    fn parse_happy_xprv() {
        assert_eq!(
            parse_slot_input("@0.xprv=xprv-stub").unwrap(),
            slot(0, SlotSubkey::Xprv, "xprv-stub")
        );
    }

    // ---- v0.41.0: SlotSubkey::Ms1 ----

    #[test]
    fn parse_happy_ms1() {
        assert_eq!(
            parse_slot_input("@0.ms1=ms1abc").unwrap(),
            slot(0, SlotSubkey::Ms1, "ms1abc")
        );
    }
    #[test]
    fn ms1_is_secret_bearing_and_stdin_sentinel() {
        assert!(SlotSubkey::Ms1.is_secret_bearing());
        let p = parse_slot_input("@0.ms1=-").unwrap();
        assert!(p.is_stdin_sentinel(), "@0.ms1=- must be a stdin sentinel");
    }
    #[test]
    fn ms1_token_round_trips() {
        assert_eq!(SlotSubkey::from_token("ms1"), Some(SlotSubkey::Ms1));
        assert_eq!(SlotSubkey::Ms1.as_str(), "ms1");
    }
    #[test]
    fn unknown_subkey_error_lists_ms1() {
        let e = parse_slot_input("@0.bogus=x").unwrap_err();
        assert!(e.0.contains("ms1"), "expected-tokens list must include ms1");
    }
    #[test]
    fn parse_index_max_u8() {
        assert_eq!(
            parse_slot_input("@255.xpub=v").unwrap(),
            slot(255, SlotSubkey::Xpub, "v")
        );
    }
    #[test]
    fn parse_no_at_prefix_rejected() {
        let e = parse_slot_input("0.phrase=v").unwrap_err();
        assert!(e.0.contains("must start with '@N.<subkey>=<value>'"));
    }
    #[test]
    fn parse_missing_index_rejected() {
        let e = parse_slot_input("@.phrase=v").unwrap_err();
        assert!(e.0.contains("missing index after '@'"));
    }
    #[test]
    fn parse_non_numeric_index_rejected() {
        let e = parse_slot_input("@xx.phrase=v").unwrap_err();
        assert!(e.0.contains("must be a u8"));
    }
    #[test]
    fn parse_index_overflow_rejected() {
        let e = parse_slot_input("@256.xpub=v").unwrap_err();
        assert!(e.0.contains("must be a u8"));
    }
    #[test]
    fn parse_missing_dot_rejected() {
        let e = parse_slot_input("@0phrase=v").unwrap_err();
        assert!(e.0.contains("missing '.<subkey>=' after '@N'"));
    }
    #[test]
    fn parse_missing_equals_rejected() {
        let e = parse_slot_input("@0.phrase").unwrap_err();
        assert!(e.0.contains("missing '=' between subkey and value"));
    }
    #[test]
    fn parse_unknown_subkey_rejected() {
        let e = parse_slot_input("@0.unknown=v").unwrap_err();
        assert!(e.0.contains("unknown slot subkey \"unknown\""));
    }
    #[test]
    fn parse_empty_subkey_rejected() {
        let e = parse_slot_input("@0.=v").unwrap_err();
        assert!(e.0.contains("missing subkey between '.' and '='"));
    }
    #[test]
    fn parse_empty_value_rejected() {
        // SPIKE-2 lock: empty value rejected at parser.
        let e = parse_slot_input("@0.phrase=").unwrap_err();
        assert!(e.0.contains("value is empty for subkey \"phrase\""));
    }

    // ---- validate_slot_set ----

    #[test]
    fn validate_empty_passes() {
        validate_slot_set(&[]).unwrap();
    }

    #[test]
    fn validate_single_phrase_passes() {
        validate_slot_set(&[slot(0, SlotSubkey::Phrase, "x")]).unwrap();
    }

    /// v0.31.3 — `[Seedqr]` alone is legal (parallels `[Phrase]`; decode
    /// happens at slot-emit time so structural validation is content-blind).
    #[test]
    fn validate_single_seedqr_passes() {
        validate_slot_set(&[slot(0, SlotSubkey::Seedqr, "x")]).unwrap();
    }

    #[test]
    fn validate_single_entropy_passes() {
        validate_slot_set(&[slot(0, SlotSubkey::Entropy, "x")]).unwrap();
    }

    #[test]
    fn validate_single_xprv_passes() {
        validate_slot_set(&[slot(0, SlotSubkey::Xprv, "x")]).unwrap();
    }

    #[test]
    fn validate_single_wif_passes() {
        validate_slot_set(&[slot(0, SlotSubkey::Wif, "x")]).unwrap();
    }

    #[test]
    fn validate_xpub_only_passes() {
        validate_slot_set(&[slot(0, SlotSubkey::Xpub, "x")]).unwrap();
    }

    #[test]
    fn validate_xpub_fingerprint_path_passes() {
        validate_slot_set(&[
            slot(0, SlotSubkey::Xpub, "x"),
            slot(0, SlotSubkey::Fingerprint, "deadbeef"),
            slot(0, SlotSubkey::Path, "48'/0'/0'/2'"),
        ])
        .unwrap();
    }

    #[test]
    fn validate_xpub_master_xpub_passes() {
        // SPEC_export_wallet_v0_8.md §2 — `[Xpub, MasterXpub]` is a legal set.
        validate_slot_set(&[
            slot(0, SlotSubkey::Xpub, "x"),
            slot(0, SlotSubkey::MasterXpub, "x_root"),
        ])
        .unwrap();
    }

    #[test]
    fn validate_xpub_master_xpub_fingerprint_path_passes() {
        // SPEC_export_wallet_v0_8.md §2 — full Coldcard-input set is legal.
        validate_slot_set(&[
            slot(0, SlotSubkey::Xpub, "x"),
            slot(0, SlotSubkey::MasterXpub, "x_root"),
            slot(0, SlotSubkey::Fingerprint, "deadbeef"),
            slot(0, SlotSubkey::Path, "84'/0'/0'"),
        ])
        .unwrap();
    }

    #[test]
    fn validate_master_xpub_alone_rejected_invalid_set() {
        // MasterXpub without Xpub is invalid: it carries no derivation context.
        let e = validate_slot_set(&[slot(0, SlotSubkey::MasterXpub, "x_root")]).unwrap_err();
        match e {
            ToolkitError::SlotInputViolation { kind, .. } => assert_eq!(kind, "invalid-set"),
            other => panic!("unexpected variant {other:?}"),
        }
    }

    #[test]
    fn validate_hybrid_n2_passes() {
        // Slot 0 phrase, slot 1 watch-only — legitimate hybrid mode.
        validate_slot_set(&[
            slot(0, SlotSubkey::Phrase, "x"),
            slot(1, SlotSubkey::Xpub, "y"),
            slot(1, SlotSubkey::Fingerprint, "deadbeef"),
            slot(1, SlotSubkey::Path, "48'/0'/0'/2'"),
        ])
        .unwrap();
    }

    #[test]
    fn validate_secret_plus_watch_in_same_slot_row4() {
        let e = validate_slot_set(&[
            slot(0, SlotSubkey::Phrase, "x"),
            slot(0, SlotSubkey::Xpub, "y"),
        ])
        .unwrap_err();
        match e {
            ToolkitError::SlotInputViolation { message, .. } => {
                assert!(message.contains(
                    "slot @0 has both secret-bearing input and watch-only input; pick one per slot."
                ));
            }
            other => panic!("unexpected variant {other:?}"),
        }
    }

    #[test]
    fn validate_phrase_plus_entropy_in_same_slot_invalid_set() {
        // Two secret-bearing subkeys in one slot is not in the validity matrix.
        let e = validate_slot_set(&[
            slot(0, SlotSubkey::Phrase, "x"),
            slot(0, SlotSubkey::Entropy, "y"),
        ])
        .unwrap_err();
        match e {
            ToolkitError::SlotInputViolation { kind, .. } => assert_eq!(kind, "invalid-set"),
            other => panic!("unexpected variant {other:?}"),
        }
    }

    #[test]
    fn validate_gap_at_index_1_row8() {
        let e = validate_slot_set(&[
            slot(0, SlotSubkey::Phrase, "x"),
            slot(2, SlotSubkey::Xpub, "y"),
        ])
        .unwrap_err();
        match e {
            ToolkitError::SlotInputViolation { kind, message } => {
                assert_eq!(kind, "gap");
                assert!(
                    message.contains("slot indices must be contiguous starting at @0; missing @1")
                );
            }
            other => panic!("unexpected variant {other:?}"),
        }
    }

    #[test]
    fn validate_duplicate_subkey_within_slot_rejected() {
        let e = validate_slot_set(&[
            slot(0, SlotSubkey::Phrase, "x"),
            slot(0, SlotSubkey::Phrase, "y"),
        ])
        .unwrap_err();
        match e {
            ToolkitError::SlotInputViolation { kind, .. } => assert_eq!(kind, "duplicate-subkey"),
            other => panic!("unexpected variant {other:?}"),
        }
    }

    // ---- v0.19.0 SPEC §6.6.b extension: [Phrase, Path] + [Phrase, Fingerprint, Path] ----

    #[test]
    fn validate_phrase_plus_path_passes_v0_19_0() {
        // SPEC §6.6.b v0.19.0 — non-canonical-descriptor explicit-origin form.
        // Slot grammar accepts the pair; canonical-mode rejection is enforced
        // post-parse in cmd::bundle (this layer is structural).
        validate_slot_set(&[
            slot(0, SlotSubkey::Phrase, "word word word"),
            slot(0, SlotSubkey::Path, "48'/0'/0'/2'"),
        ])
        .unwrap();
    }

    #[test]
    fn validate_phrase_plus_fingerprint_plus_path_passes_v0_19_0() {
        // SPEC §6.6.b v0.19.0 — phrase + per-`@N` origin attestation
        // (fingerprint + path).
        validate_slot_set(&[
            slot(0, SlotSubkey::Phrase, "word word word"),
            slot(0, SlotSubkey::Fingerprint, "deadbeef"),
            slot(0, SlotSubkey::Path, "48'/0'/0'/2'"),
        ])
        .unwrap();
    }

    /// v0.31.3 — `[Seedqr, Path]` parallels `[Phrase, Path]` per the v0.19.0
    /// SPEC §6.6.b exemption (non-canonical-descriptor explicit-origin
    /// form). Decode happens at slot-emit time; the structural validator
    /// is content-blind.
    #[test]
    fn validate_seedqr_plus_path_passes_v0_19_0() {
        validate_slot_set(&[
            slot(0, SlotSubkey::Seedqr, "<digit-string>"),
            slot(0, SlotSubkey::Path, "48'/0'/0'/2'"),
        ])
        .unwrap();
    }

    /// v0.31.3 — `[Seedqr, Fingerprint, Path]` parallels
    /// `[Phrase, Fingerprint, Path]`.
    #[test]
    fn validate_seedqr_plus_fingerprint_plus_path_passes_v0_19_0() {
        validate_slot_set(&[
            slot(0, SlotSubkey::Seedqr, "<digit-string>"),
            slot(0, SlotSubkey::Fingerprint, "deadbeef"),
            slot(0, SlotSubkey::Path, "48'/0'/0'/2'"),
        ])
        .unwrap();
    }

    /// v0.31.3 — `[Seedqr, Xpub]` is REFUSED (secret+watch-only conflict;
    /// Seedqr is secret-bearing per `is_secret_bearing`).
    #[test]
    fn validate_seedqr_plus_xpub_still_conflict() {
        let e = validate_slot_set(&[
            slot(0, SlotSubkey::Seedqr, "x"),
            slot(0, SlotSubkey::Xpub, "xpub-stub"),
        ])
        .unwrap_err();
        if let ToolkitError::SlotInputViolation { kind, .. } = e {
            assert_eq!(kind, "conflict");
        } else {
            panic!("expected SlotInputViolation");
        }
    }

    #[test]
    fn validate_phrase_plus_fingerprint_without_path_still_conflict() {
        // Narrowing of the v0.19.0 exemption: [Phrase, Fingerprint] (no Path)
        // is NOT in the exempted set; the conflict refusal still fires.
        // Distinguishes "phrase with origin attestation" (Path required) from
        // "phrase with bare fingerprint annotation" (ambiguous; rejected).
        let e = validate_slot_set(&[
            slot(0, SlotSubkey::Phrase, "x"),
            slot(0, SlotSubkey::Fingerprint, "deadbeef"),
        ])
        .unwrap_err();
        match e {
            ToolkitError::SlotInputViolation { kind, message } => {
                assert_eq!(kind, "conflict");
                assert!(message.contains(
                    "slot @0 has both secret-bearing input and watch-only input; pick one per slot."
                ));
            }
            other => panic!("unexpected variant {other:?}"),
        }
    }

    #[test]
    fn validate_phrase_plus_xpub_still_conflict_v0_19_0() {
        // The v0.19.0 exemption is narrow: only Path + optionally Fingerprint
        // pair with Phrase. Xpub-class peers still trigger the conflict.
        let e = validate_slot_set(&[
            slot(0, SlotSubkey::Phrase, "x"),
            slot(0, SlotSubkey::Xpub, "y"),
        ])
        .unwrap_err();
        match e {
            ToolkitError::SlotInputViolation { kind, .. } => assert_eq!(kind, "conflict"),
            other => panic!("unexpected variant {other:?}"),
        }
    }

    // ---- v0.41.0: Ms1 legal-sets (full parity with phrase) ----

    #[test]
    fn validate_single_ms1_passes() {
        validate_slot_set(&[slot(0, SlotSubkey::Ms1, "x")]).unwrap();
    }
    #[test]
    fn validate_ms1_plus_path_passes() {
        validate_slot_set(&[
            slot(0, SlotSubkey::Ms1, "x"),
            slot(0, SlotSubkey::Path, "48'/0'/0'/2'"),
        ])
        .unwrap();
    }
    #[test]
    fn validate_ms1_plus_fingerprint_plus_path_passes() {
        validate_slot_set(&[
            slot(0, SlotSubkey::Ms1, "x"),
            slot(0, SlotSubkey::Fingerprint, "deadbeef"),
            slot(0, SlotSubkey::Path, "48'/0'/0'/2'"),
        ])
        .unwrap();
    }
    #[test]
    fn validate_ms1_plus_xpub_conflict() {
        let e = validate_slot_set(&[
            slot(0, SlotSubkey::Ms1, "x"),
            slot(0, SlotSubkey::Xpub, "y"),
        ])
        .unwrap_err();
        assert!(
            matches!(e, ToolkitError::SlotInputViolation { kind, .. } if kind == "conflict"),
            "expected SlotInputViolation conflict; got {e:?}"
        );
    }

    #[test]
    fn validate_entropy_plus_path_still_rejected_v0_19_0() {
        // The v0.19.0 exemption applies to Phrase only, not Entropy. Users
        // wanting entropy + custom path must convert entropy → phrase first.
        let e = validate_slot_set(&[
            slot(0, SlotSubkey::Entropy, "0102030405060708090a0b0c0d0e0f10"),
            slot(0, SlotSubkey::Path, "48'/0'/0'/2'"),
        ])
        .unwrap_err();
        match e {
            ToolkitError::SlotInputViolation { kind, .. } => assert_eq!(kind, "conflict"),
            other => panic!("unexpected variant {other:?}"),
        }
    }
}
