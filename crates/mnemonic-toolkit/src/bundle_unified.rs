//! v0.4 unified `bundle` dispatch helpers (pre-clap trap + mode detection +
//! pre-check ladder). Phase D wires these into `cmd/bundle.rs::run` once the
//! multi-source synthesis path lands.

#![allow(dead_code)] // Wired in Phase D.

use crate::error::ToolkitError;
use crate::slot_input::SlotInput;

/// v0.4 bundle-mode classification (impl plan Phase C.3). Auto-detected
/// from per-slot subkeys. Descriptor presence is orthogonal — `--descriptor`
/// does NOT add a variant; it is consumed by the synthesis path independently
/// of `BundleMode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BundleMode {
    /// N=1, slot @0 secret-bearing (phrase / entropy / xprv / wif).
    SingleSigFull,
    /// N=1, slot @0 watch-only ({xpub} or {xpub, fingerprint, ...}).
    SingleSigWatchOnly,
    /// N≥2, every slot secret-bearing.
    MultisigMultiSource,
    /// N≥2, every slot watch-only.
    MultisigWatchOnly,
    /// N≥2, mix of secret-bearing and watch-only slots.
    MultisigHybrid,
}

/// Auto-detect `BundleMode` from a validated slot vector. The caller MUST
/// have already run `validate_slot_set(slots)` so the per-slot subkey-set
/// is legal and slot indices are contiguous from @0.
///
/// Returns `Err(ToolkitError::SlotInputViolation)` only on degenerate input
/// (empty slots) — all other shape errors are caught by `validate_slot_set`.
pub fn detect_bundle_mode(slots: &[SlotInput]) -> Result<BundleMode, ToolkitError> {
    if slots.is_empty() {
        return Err(ToolkitError::SlotInputViolation {
            kind: "empty",
            message: "no --slot inputs supplied; bundle mode cannot be detected".to_string(),
        });
    }
    let mut by_index: std::collections::BTreeMap<u8, Vec<&SlotInput>> = Default::default();
    for s in slots {
        by_index.entry(s.index).or_default().push(s);
    }
    let n = by_index.len();
    let mut secret_slots = 0usize;
    let mut watch_slots = 0usize;
    for slot_inputs in by_index.values() {
        let any_secret = slot_inputs.iter().any(|s| s.subkey.is_secret_bearing());
        if any_secret {
            secret_slots += 1;
        } else {
            watch_slots += 1;
        }
    }
    Ok(match (n, secret_slots, watch_slots) {
        (1, 1, 0) => BundleMode::SingleSigFull,
        (1, 0, 1) => BundleMode::SingleSigWatchOnly,
        (_, s, 0) if s >= 2 => BundleMode::MultisigMultiSource,
        (_, 0, w) if w >= 2 => BundleMode::MultisigWatchOnly,
        _ => BundleMode::MultisigHybrid,
    })
}

/// SPEC §6.6 row 9 + 9.5: threshold range + presence checks.
/// Row 9: 1 ≤ T ≤ N. Row 9.5: multisig template requires --threshold.
pub fn pre_check_threshold(
    threshold: Option<u8>,
    n: usize,
    multisig_template: Option<&str>,
) -> Result<(), ToolkitError> {
    if let Some(t) = threshold {
        if (t as usize) > n || t == 0 {
            return Err(ToolkitError::SlotInputViolation {
                kind: "threshold-range",
                message: format!(
                    "threshold {t} out of range for N={n} cosigners (must be 1..={n})"
                ),
            });
        }
    } else if n >= 2 {
        if let Some(template) = multisig_template {
            return Err(ToolkitError::SlotInputViolation {
                kind: "missing-threshold",
                message: format!("--threshold required for multisig template '{template}'"),
            });
        }
    }
    Ok(())
}

/// SPEC §6.6 row 10 + 11: template/N compatibility.
/// Row 10: single-sig template + N>1 → reject. Row 11: multisig template + N=1 → reject.
pub fn pre_check_template_n(template: &str, is_multisig_template: bool, n: usize) -> Result<(), ToolkitError> {
    if !is_multisig_template && n > 1 {
        return Err(ToolkitError::SlotInputViolation {
            kind: "single-sig-multi-slot",
            message: format!(
                "single-sig template '{template}' incompatible with N={n} slots; use a multisig template or --descriptor"
            ),
        });
    }
    if is_multisig_template && n == 1 {
        return Err(ToolkitError::SlotInputViolation {
            kind: "multisig-single-slot",
            message: format!(
                "multisig template '{template}' requires N > 1; use a single-sig template for N=1"
            ),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::slot_input::SlotSubkey;

    fn s(idx: u8, sk: SlotSubkey, v: &str) -> SlotInput {
        SlotInput { index: idx, subkey: sk, value: v.to_string() }
    }

    // ---- detect_bundle_mode ----

    #[test]
    fn mode_single_sig_full_phrase() {
        assert_eq!(
            detect_bundle_mode(&[s(0, SlotSubkey::Phrase, "x")]).unwrap(),
            BundleMode::SingleSigFull
        );
    }

    #[test]
    fn mode_single_sig_full_entropy() {
        assert_eq!(
            detect_bundle_mode(&[s(0, SlotSubkey::Entropy, "x")]).unwrap(),
            BundleMode::SingleSigFull
        );
    }

    #[test]
    fn mode_single_sig_full_xprv() {
        assert_eq!(
            detect_bundle_mode(&[s(0, SlotSubkey::Xprv, "x")]).unwrap(),
            BundleMode::SingleSigFull
        );
    }

    #[test]
    fn mode_single_sig_full_wif() {
        assert_eq!(
            detect_bundle_mode(&[s(0, SlotSubkey::Wif, "x")]).unwrap(),
            BundleMode::SingleSigFull
        );
    }

    #[test]
    fn mode_single_sig_watch_only_xpub_alone() {
        assert_eq!(
            detect_bundle_mode(&[s(0, SlotSubkey::Xpub, "x")]).unwrap(),
            BundleMode::SingleSigWatchOnly
        );
    }

    #[test]
    fn mode_single_sig_watch_only_xpub_fp_path() {
        assert_eq!(
            detect_bundle_mode(&[
                s(0, SlotSubkey::Xpub, "x"),
                s(0, SlotSubkey::Fingerprint, "deadbeef"),
                s(0, SlotSubkey::Path, "p"),
            ])
            .unwrap(),
            BundleMode::SingleSigWatchOnly
        );
    }

    #[test]
    fn mode_multisig_multi_source_n3() {
        assert_eq!(
            detect_bundle_mode(&[
                s(0, SlotSubkey::Phrase, "a"),
                s(1, SlotSubkey::Phrase, "b"),
                s(2, SlotSubkey::Entropy, "c"),
            ])
            .unwrap(),
            BundleMode::MultisigMultiSource
        );
    }

    #[test]
    fn mode_multisig_watch_only_n3() {
        assert_eq!(
            detect_bundle_mode(&[
                s(0, SlotSubkey::Xpub, "a"),
                s(1, SlotSubkey::Xpub, "b"),
                s(2, SlotSubkey::Xpub, "c"),
            ])
            .unwrap(),
            BundleMode::MultisigWatchOnly
        );
    }

    #[test]
    fn mode_multisig_hybrid_phrase_plus_xpub() {
        assert_eq!(
            detect_bundle_mode(&[
                s(0, SlotSubkey::Phrase, "a"),
                s(1, SlotSubkey::Xpub, "b"),
            ])
            .unwrap(),
            BundleMode::MultisigHybrid
        );
    }

    #[test]
    fn mode_empty_rejected() {
        assert!(matches!(
            detect_bundle_mode(&[]),
            Err(ToolkitError::SlotInputViolation { kind: "empty", .. })
        ));
    }

    // ---- pre_check_threshold ----

    #[test]
    fn threshold_range_pass() {
        pre_check_threshold(Some(2), 3, Some("wsh-sortedmulti")).unwrap();
        pre_check_threshold(Some(1), 1, None).unwrap();
        pre_check_threshold(Some(3), 3, Some("wsh-multi")).unwrap();
    }

    #[test]
    fn threshold_zero_rejected() {
        let e = pre_check_threshold(Some(0), 3, Some("wsh-multi")).unwrap_err();
        match e {
            ToolkitError::SlotInputViolation { kind, message } => {
                assert_eq!(kind, "threshold-range");
                assert!(message.contains("threshold 0 out of range for N=3"));
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn threshold_above_n_rejected() {
        let e = pre_check_threshold(Some(4), 3, Some("wsh-multi")).unwrap_err();
        match e {
            ToolkitError::SlotInputViolation { message, .. } => {
                assert!(message.contains("threshold 4 out of range for N=3"));
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn threshold_required_for_multisig_template() {
        let e = pre_check_threshold(None, 3, Some("wsh-sortedmulti")).unwrap_err();
        match e {
            ToolkitError::SlotInputViolation { kind, message } => {
                assert_eq!(kind, "missing-threshold");
                assert!(message.contains("--threshold required for multisig template 'wsh-sortedmulti'"));
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn threshold_optional_for_n1() {
        pre_check_threshold(None, 1, None).unwrap();
    }

    // ---- pre_check_template_n ----

    #[test]
    fn template_single_sig_n1_passes() {
        pre_check_template_n("wpkh", false, 1).unwrap();
    }

    #[test]
    fn template_multisig_n3_passes() {
        pre_check_template_n("wsh-sortedmulti", true, 3).unwrap();
    }

    #[test]
    fn template_single_sig_n2_rejected() {
        let e = pre_check_template_n("wpkh", false, 2).unwrap_err();
        match e {
            ToolkitError::SlotInputViolation { kind, message } => {
                assert_eq!(kind, "single-sig-multi-slot");
                assert!(message.contains(
                    "single-sig template 'wpkh' incompatible with N=2 slots; use a multisig template or --descriptor"
                ));
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn template_multisig_n1_rejected() {
        let e = pre_check_template_n("wsh-sortedmulti", true, 1).unwrap_err();
        match e {
            ToolkitError::SlotInputViolation { kind, message } => {
                assert_eq!(kind, "multisig-single-slot");
                assert!(message.contains(
                    "multisig template 'wsh-sortedmulti' requires N > 1; use a single-sig template for N=1"
                ));
            }
            _ => panic!("wrong variant"),
        }
    }
}
