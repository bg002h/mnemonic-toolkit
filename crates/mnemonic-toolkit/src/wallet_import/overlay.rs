//! v0.26.0 seed overlay (SPEC §8.3).
//!
//! Re-attaches user-supplied entropy (`--ms1 <STRING>`) or BIP-39 phrase
//! (`--slot @N.phrase=<STRING>`) to a watch-only `ParsedImport`, after
//! verifying that the supplied seed derives the blob-declared xpub at the
//! blob-declared origin path.
//!
//! Pipeline per cosigner index `i`:
//! 1. If `ms1[i]` is `Some` non-empty → decode via `ms_codec::decode` →
//!    `Payload::Entr(bytes)`.
//! 2. If a `--slot @i.phrase=<P>` overlay exists → parse BIP-39 mnemonic
//!    → `mnemonic.to_entropy()` bytes.
//! 3. Empty-string `ms1[i] == ""` (v0.25.1 watch-only sentinel) → leave
//!    cosigner watch-only + emit NOTICE.
//! 4. `None` → leave watch-only silently.
//!
//! For (1) + (2): derive the master xpriv from entropy → master seed →
//! `Xpriv::new_master(network_kind, seed)` → `derive_priv(secp, path)` →
//! `Xpub::from_priv`. Compare derived xpub vs `cosigner.xpub`. On match:
//! set `cosigner.entropy = Some(Zeroizing::new(entropy_bytes))`. On
//! mismatch: `ImportWalletSeedMismatch` (exit 4) with the canonical SPEC
//! §2.4 stderr template.
//!
//! Per SPEC §8.3 line "derivation uses `.path` (typed `DerivationPath`)
//! for the cryptographic operation; the error report uses `.path_raw`".
//!
//! Re-uses `synthesize::derive_xpub_at_path` shape (an existing
//! toolkit-side derivation helper) by inlining the master-xpriv +
//! `derive_priv` invocation here. We deliberately call
//! `master.derive_priv(secp, &path)` directly (path is already typed
//! `DerivationPath`) rather than going through `derive_xpub_at_path` —
//! the latter takes a string path-spec that we'd need to reformat from
//! the typed path. Both routes produce the same xpub.

use super::ParsedImport;
use crate::error::ToolkitError;
use crate::language::CliLanguage;
use bitcoin::bip32::Xpriv;
use bitcoin::bip32::Xpub;
use bitcoin::secp256k1::Secp256k1;
use std::io::Write;
use zeroize::Zeroizing;

/// SPEC §8.3 — apply seed overlay across ALL bundles in `parsed`. Each
/// bundle's cosigner vector is mutated in place. The `ms1_args` index is
/// the cosigner index (0-based), matching the SPEC §3.1 row-3
/// "(repeatable, positional cosigner-index) seed overlay" semantics.
///
/// `phrase_overlays` carries `(slot_index, bip39_phrase_string)` from
/// `--slot @N.phrase=<P>` invocations. Conflicts (both `--ms1[i]` and
/// `--slot @i.phrase=` for the same i) → exit 1 `BadInput`.
///
/// All bundles share the same overlay arguments (the user does NOT
/// disambiguate per-bundle for Bitcoin Core multi-entry blobs; cosigner
/// index `i` is the same across all bundles). This matches the SPEC
/// §8.3 line "Seed overlay happens AFTER `WalletFormatParser::parse`".
pub(crate) fn apply_seed_overlay(
    parsed: &mut [ParsedImport],
    ms1_args: &[Option<String>],
    phrase_overlays: &[(u8, String)],
    language: CliLanguage,
    stderr: &mut dyn Write,
) -> Result<(), ToolkitError> {
    // Build a (cosigner_index → entropy-source) map. Mutually exclusive
    // forms; conflict → BadInput.
    enum Source {
        Ms1(String),
        Phrase(String),
    }
    let mut by_index: Vec<Option<Source>> = Vec::new();

    // ms1_args: positional cosigner-index. `ms1_args[i] == Some(s)` means
    // user supplied `--ms1 s` for cosigner i (in repeat-flag order).
    for (i, v) in ms1_args.iter().enumerate() {
        while by_index.len() <= i {
            by_index.push(None);
        }
        if let Some(s) = v {
            by_index[i] = Some(Source::Ms1(s.clone()));
        }
    }

    // phrase overlays via `--slot @N.phrase=<phrase>`.
    for (idx, phrase) in phrase_overlays {
        let i = *idx as usize;
        while by_index.len() <= i {
            by_index.push(None);
        }
        if by_index[i].is_some() {
            return Err(ToolkitError::BadInput(format!(
                "import-wallet: cosigner {i} has both `--ms1` and `--slot @{i}.phrase=` \
                 supplied; use one or the other"
            )));
        }
        by_index[i] = Some(Source::Phrase(phrase.clone()));
    }

    let secp = Secp256k1::new();

    // Apply to every bundle. Same overlay map; per-bundle cosigner count
    // may differ (Core multi-entry blobs may have heterogeneous N — though
    // typical wallets have N uniform).
    for bundle in parsed.iter_mut() {
        for (i, src_opt) in by_index.iter().enumerate() {
            if i >= bundle.cosigners.len() {
                // Overlay for a cosigner index that doesn't exist in this
                // bundle → silent skip (per-bundle cosigner-set may be
                // smaller than the highest overlay index).
                continue;
            }
            let Some(src) = src_opt else { continue };

            // Decode entropy + handle empty-string watch-only sentinel.
            let entropy: Zeroizing<Vec<u8>> = match src {
                Source::Ms1(s) => {
                    if s.is_empty() {
                        // v0.25.1 empty-string sentinel — leave watch-only +
                        // emit NOTICE. No derivation, no comparison.
                        let _ = writeln!(
                            stderr,
                            "notice: import-wallet: cosigner {i} ms1 supplied \
                             as empty-string sentinel; treated as watch-only"
                        );
                        continue;
                    }
                    match ms_codec::decode(s) {
                        Ok((_tag, ms_codec::Payload::Entr(bytes))) => Zeroizing::new(bytes),
                        Ok(_) => {
                            return Err(ToolkitError::BadInput(format!(
                                "import-wallet: --ms1 for cosigner {i}: decoded payload is not entropy"
                            )));
                        }
                        Err(e) => {
                            return Err(ToolkitError::BadInput(format!(
                                "import-wallet: --ms1 for cosigner {i}: ms_codec decode failed: {e:?}"
                            )));
                        }
                    }
                }
                Source::Phrase(phrase) => {
                    let mnemonic =
                        bip39::Mnemonic::parse_in(language.into(), phrase).map_err(|e| {
                            ToolkitError::BadInput(format!(
                                "import-wallet: --slot @{i}.phrase=: BIP-39 parse error: {e}"
                            ))
                        })?;
                    Zeroizing::new(mnemonic.to_entropy())
                }
            };

            // Derive xpub at cosigner.path from entropy. Pipeline:
            //   entropy → mnemonic → seed (passphrase="") → master xpriv
            //   → derive_priv(path) → Xpub::from_priv.
            //
            // The toolkit's import-wallet surface does NOT take a passphrase
            // (per SPEC §2.1; passphrase is a sibling-cycle concern). If
            // the source wallet used a BIP-39 passphrase, the user must
            // recover the seed elsewhere — out of scope for v0.26.0.
            let mnemonic =
                bip39::Mnemonic::from_entropy_in(language.into(), &entropy).map_err(|e| {
                    ToolkitError::BadInput(format!(
                        "import-wallet: cosigner {i}: entropy → mnemonic: {e}"
                    ))
                })?;
            let seed = mnemonic.to_seed("");
            let master = Xpriv::new_master(bundle.network, &seed[..])
                .map_err(|e| ToolkitError::Bitcoin(crate::error::BitcoinErrorKind::Bip32(e)))?;
            let child = master
                .derive_priv(&secp, &bundle.cosigners[i].path)
                .map_err(|e| ToolkitError::Bitcoin(crate::error::BitcoinErrorKind::Bip32(e)))?;
            let derived_xpub: Xpub = Xpub::from_priv(&secp, &child);

            // SPEC §8.3 — byte-exact xpub equality at the derived path.
            if derived_xpub != bundle.cosigners[i].xpub {
                return Err(ToolkitError::ImportWalletSeedMismatch {
                    cosigner_index: i,
                    derived_xpub: derived_xpub.to_string(),
                    blob_xpub: bundle.cosigners[i].xpub.to_string(),
                    path: bundle.cosigners[i].path_raw.clone(),
                });
            }

            // Match — attach entropy to the cosigner.
            bundle.cosigners[i].entropy = Some(entropy);
        }
    }

    Ok(())
}
