//! v0.4 `--slot @N.<subkey>=<value>` value-parser + validator.
//!
//! Locked by Phase 2 SPIKE-2 (`design/agent-reports/spike-toolkit-v0_4-pre-phaseA.md`).
//! Phase C consumes `parse_slot_input` as the clap value-parser and `validate_slot_set`
//! as the post-parse / pre-binding gate.
#![allow(dead_code)] // parse_slot_input wired as a clap value-parser via bundle.rs / verify_bundle.rs.

use std::collections::BTreeMap;

use crate::error::ToolkitError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SlotSubkey {
    Phrase,
    Entropy,
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
    fn from_token(tok: &str) -> Option<Self> {
        Some(match tok {
            "phrase" => Self::Phrase,
            "entropy" => Self::Entropy,
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
            Self::Entropy => "entropy",
            Self::Xpub => "xpub",
            Self::MasterXpub => "master_xpub",
            Self::Fingerprint => "fingerprint",
            Self::Path => "path",
            Self::Wif => "wif",
            Self::Xprv => "xprv",
        }
    }
    pub fn is_secret_bearing(self) -> bool {
        matches!(self, Self::Phrase | Self::Entropy | Self::Xprv | Self::Wif)
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
            "unknown slot subkey {:?}; expected one of: phrase, entropy, xpub, fingerprint, path, wif, xprv",
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
                message: format!(
                    "slot indices must be contiguous starting at @0; missing @{i}"
                ),
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

        if has_secret && has_watch {
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
fn is_legal_set(set: &[SlotSubkey]) -> bool {
    use SlotSubkey::*;
    matches!(
        set,
        [Phrase]
            | [Entropy]
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
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn slot(index: u8, subkey: SlotSubkey, value: &str) -> SlotInput {
        SlotInput {
            index,
            subkey,
            value: value.to_string(),
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
                assert!(message.contains(
                    "slot indices must be contiguous starting at @0; missing @1"
                ));
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
}
