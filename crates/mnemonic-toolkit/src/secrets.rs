//! Authoritative classification of which `mnemonic` CLI flag names carry secret
//! material.
//!
//! Source-of-truth for the `secret: bool` field in the gui-schema v5 envelope
//! emitted by `cmd::gui_schema` (v0.24.x Tranche B.1, plan §2.B.1 + D32 + C3-fold).
//! The GUI consumes this predicate via the gui-schema JSON's per-flag
//! `secret` bit: its hand schema mirrors the bit per `(subcommand, flag)`
//! and the GUI-side drift gate (`schema_mirror_secret_drift`) asserts
//! set-equality of the `secret: true` pairs against this binary's live
//! `gui-schema` output. (v0.53.2 doc fix: an earlier claim here that the
//! GUI mirrors a "token-for-token parallel enumeration" was stale —
//! `mnemonic-gui/src/secrets.rs::SECRET_FLAG_NAMES` is a 3-token v0.1
//! legacy fallback, not the mirror.)
//!
//! ## Semantics
//!
//! "Secret" here means: a non-empty value supplied to this flag is sensitive
//! material that should trigger paste-warn / run-confirm modals and exit-time
//! zeroize sweeps. Compare against the related but distinct
//! `secret_taxonomy::SECRET_NODE_TYPES` / `SECRET_SLOT_SUBKEYS` arrays
//! (composite-node-form + slot-row-form secrets); this `flag_is_secret`
//! predicate covers FLAT flag-name-form secrets only.
//!
//! ## Membership rationale per entry
//!
//! - `--passphrase` / `--bip38-passphrase` — BIP-39 / BIP-38 passphrases
//!   (passed inline as argv value; value is sensitive).
//! - `--passphrase-stdin` / `--bip38-passphrase-stdin` — boolean toggles
//!   whose VALUE is just `true`/`false` (not sensitive itself), but the
//!   GUI's secret-class treatment treats the toggle as a sentinel for the
//!   paste-warn / run-confirm modal pathway since selecting it implies the
//!   user is about to stream secret material via stdin.
//! - `--ms1` — single-slot BIP-39 entropy chunk. Distinguished from `--mk1`
//!   (xpub) and `--md1` (descriptor), which are non-secret by design.
//!   `--ms1-stdin` — its stdin sentinel toggle (same rationale as
//!   `--passphrase-stdin` above; v0.53.1 audit-I3 fix — was a gap vs the
//!   established toggle convention).
//! - `--phrase` / `--phrase-stdin` — raw BIP-39 master phrase supplied
//!   inline to the three `xpub-search` modes, plus its stdin sentinel
//!   toggle (v0.53.1 audit-I3 fix; the runtime already treated the value
//!   as secret-equivalent via the `@env:` sentinel + stdin route).
//! - `--share` — SLIP-39 / seed-XOR share. Share material is secret-class
//!   under both schemes' security models (any share's compromise reduces
//!   the K threshold).
//!
//! Not included (deliberately):
//! - `--mk1` / `--md1` — xpub / descriptor (non-secret by design).
//! - `--from` / `--to` — composite-node selectors; secrecy is value-dependent
//!   (e.g., `--from phrase=...` is secret but `--from xpub=...` is not). The
//!   GUI applies node-type-level secret classification via
//!   `secret_taxonomy::SECRET_NODE_TYPES`, not flag-name-level.
//! - `--final-word` — not a flag name; final-word is a subcommand.

/// True iff a non-empty value supplied to the given long-flag name should be
/// treated as secret material (triggering paste-warn / run-confirm modals
/// and exit-time zeroize sweeps in `mnemonic-gui`).
///
/// Pass the flag's long-form name including the `--` prefix, e.g.
/// `flag_is_secret("--passphrase")`.
pub fn flag_is_secret(flag_name: &str) -> bool {
    matches!(
        flag_name,
        "--passphrase"
            | "--passphrase-stdin"
            | "--bip38-passphrase"
            | "--bip38-passphrase-stdin"
            | "--decrypt-password"
            | "--decrypt-password-stdin"
            | "--digits"
            | "--ms1"
            | "--ms1-stdin"
            | "--phrase"
            | "--phrase-stdin"
            | "--secret"
            | "--secret-stdin"
            | "--share"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_secret_flags_classify_as_secret() {
        for name in [
            "--passphrase",
            "--passphrase-stdin",
            "--bip38-passphrase",
            "--bip38-passphrase-stdin",
            "--decrypt-password",
            "--decrypt-password-stdin",
            "--digits",
            "--ms1",
            "--ms1-stdin",
            "--phrase",
            "--phrase-stdin",
            "--share",
        ] {
            assert!(flag_is_secret(name), "{name} must classify as secret");
        }
    }

    #[test]
    fn known_non_secret_flags_classify_as_non_secret() {
        // --mk1 = xpub (non-secret); --md1 = descriptor (non-secret).
        // --account / --template / --network / --from / --to / --no-auto-repair
        // are non-secret-bearing (or value-dependent, handled elsewhere).
        // --decrypt-password-file holds a filesystem PATH, not the secret
        // itself — the inline --decrypt-password / --decrypt-password-stdin
        // forms carry the secret value and ARE classified secret above.
        for name in [
            "--mk1",
            "--md1",
            "--account",
            "--template",
            "--network",
            "--from",
            "--to",
            "--no-auto-repair",
            "--threshold",
            "--language",
            "--decrypt-password-file",
            "--ciphertext",
        ] {
            assert!(
                !flag_is_secret(name),
                "{name} must NOT classify as secret"
            );
        }
    }

    #[test]
    fn empty_and_unknown_flags_classify_as_non_secret() {
        assert!(!flag_is_secret(""));
        assert!(!flag_is_secret("--this-flag-does-not-exist"));
        assert!(!flag_is_secret("passphrase")); // missing leading `--`
    }

    #[test]
    fn nostr_secret_flags_are_secret() {
        assert!(flag_is_secret("--secret"));
        assert!(flag_is_secret("--secret-stdin"));
        assert!(!flag_is_secret("--secret-file")); // a path, not the secret itself
        assert!(!flag_is_secret("--pubkey"));
    }
}
