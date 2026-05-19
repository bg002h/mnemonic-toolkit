//! v0.26.0 cross-cutting `@env:<VAR>` CLI value-source sentinel resolver.
//!
//! Resolves the SPEC_wallet_import_v0_26_0.md §3 value-source sentinel for the
//! 6 secret-flag surfaces enumerated in §3.1:
//!   1. `--passphrase`
//!   2. `--bip38-passphrase`
//!   3. `--ms1`
//!   4. `--share`
//!   5. `--slot @N.<subkey>=` — secret-bearing subkeys `phrase` / `entropy`
//!      / `wif` / `xprv` (`SlotSubkey` enum at `slot_input.rs:17-32`)
//!   6. `--from <node>=` — secret-bearing composite nodes `phrase` /
//!      `entropy` / `wif` / `xprv` / `minikey` / `electrum-phrase`
//!      (composite-node infra at `from_input.rs`)
//!
//! Whole-value sentinel only: `--ms1 @env:VAR` resolves; `--ms1 prefix@env:VAR`
//! is treated as a literal (§3.2 grammar lock).
//!
//! Resolution is opt-in per-callsite: non-secret flags (e.g., `--network`,
//! `--template`) treat `@env:VAR` as a literal — this matches the locked rule
//! per §7.0.d ("sentinel resolution applies ONLY at the 6 secret-flag surfaces").
//!
//! Empty-string env-var (`VAR=""`) preserves v0.25.1 watch-only sentinel
//! semantics: substituted value is `""` and proceeds through downstream
//! validation (e.g., `validate_flag_hrp("--ms1", "ms", "")` early-returns Ok).
//!
//! ## Placement note
//!
//! Plan-doc §1.1 cites placement in `secrets.rs`, but `crates/mnemonic-toolkit/
//! src/secrets.rs` is part of the publicly-exposed `mnemonic_toolkit` library
//! (see `lib.rs:61`) while `ToolkitError` is binary-private (`main.rs:9`
//! `mod error;`). A `pub(crate)` resolver returning `Result<String,
//! ToolkitError>` cannot live in the library half. This module is the
//! binary-private home; the public `is_valid_posix_env_var_name` predicate
//! is duplicated nowhere — it's an internal helper here. The locked
//! `flag_is_secret` predicate stays in the library at `secrets.rs` (its
//! GUI-side mirror gate is unaffected).

use crate::error::{EnvVarMissingReason, ToolkitError};

/// SPEC_wallet_import_v0_26_0.md §3.2 — resolve a `@env:<VAR>` sentinel to the
/// env-var's value, or pass through a literal string.
///
/// - If `value` starts with `"@env:"`, parse the suffix as a POSIX env-var
///   name (regex `[A-Z_][A-Z0-9_]*`):
///   - Invalid suffix → `EnvVarMissing { reason: InvalidName }` (exit 1).
///   - Valid suffix but `std::env::var(VAR)` fails → `EnvVarMissing
///     { reason: Unset }` (exit 1).
///   - Valid suffix + var set → `Ok(value)`. Empty-string value is preserved
///     (v0.25.1 watch-only sentinel semantics).
/// - Otherwise return `Ok(value.to_string())` — literal passthrough.
///
/// `flag_name` is the long-form flag (e.g., `"--ms1"`) used to disambiguate
/// the error template across the 6 secret-flag surfaces. Callers pass the
/// argv-side flag spelling; for slot subkeys, pass the composite form (e.g.,
/// `"--slot @0.phrase="`).
pub(crate) fn resolve_env_var_sentinel(
    value: &str,
    flag_name: &str,
) -> Result<String, ToolkitError> {
    if let Some(varname) = value.strip_prefix("@env:") {
        if !is_valid_posix_env_var_name(varname) {
            return Err(ToolkitError::EnvVarMissing {
                flag: flag_name.to_string(),
                var: varname.to_string(),
                reason: EnvVarMissingReason::InvalidName,
            });
        }
        std::env::var(varname).map_err(|_| ToolkitError::EnvVarMissing {
            flag: flag_name.to_string(),
            var: varname.to_string(),
            reason: EnvVarMissingReason::Unset,
        })
    } else {
        Ok(value.to_string())
    }
}

/// POSIX env-var name: first char `[A-Z_]`, remaining chars `[A-Z0-9_]`.
/// Empty string is invalid. Lower-case letters and non-ASCII are rejected by
/// design — we deliberately do NOT accept lower-case names because doing so
/// would surprise users who type `@env:passphrase` expecting case-insensitive
/// behavior (POSIX env-var names are case-sensitive but the convention is
/// upper-case). See SPEC §3.2 grammar lock.
fn is_valid_posix_env_var_name(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_uppercase() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literal_value_passes_through() {
        assert_eq!(
            resolve_env_var_sentinel("plain-value", "--ms1").unwrap(),
            "plain-value"
        );
    }

    #[test]
    fn empty_literal_passes_through() {
        // v0.25.1 watch-only sentinel — empty-string literal is legal.
        assert_eq!(resolve_env_var_sentinel("", "--ms1").unwrap(), "");
    }

    #[test]
    fn literal_at_prefix_without_env_passes_through() {
        // Whole-value-sentinel discipline: not a `@env:` prefix → literal.
        assert_eq!(
            resolve_env_var_sentinel("@something", "--ms1").unwrap(),
            "@something"
        );
    }

    #[test]
    fn literal_with_partial_env_prefix_passes_through() {
        // `prefix@env:VAR` is NOT a whole-value sentinel.
        assert_eq!(
            resolve_env_var_sentinel("prefix@env:VAR", "--ms1").unwrap(),
            "prefix@env:VAR"
        );
    }

    #[test]
    fn env_resolves_when_set() {
        // Per-test unique varnames prevent cross-test interference even when
        // cargo test schedules these in a shared process via threaded harness.
        let varname = "MNEMONIC_TEST_ENV_SENTINEL_SET_OK";
        std::env::set_var(varname, "resolved-value");
        let got = resolve_env_var_sentinel(&format!("@env:{varname}"), "--ms1").unwrap();
        std::env::remove_var(varname);
        assert_eq!(got, "resolved-value");
    }

    #[test]
    fn env_unset_returns_err_unset() {
        let varname = "MNEMONIC_TEST_ENV_SENTINEL_NEVER_SET_X1Y2Z3";
        std::env::remove_var(varname);
        let err = resolve_env_var_sentinel(&format!("@env:{varname}"), "--ms1").unwrap_err();
        match err {
            ToolkitError::EnvVarMissing { flag, var, reason } => {
                assert_eq!(flag, "--ms1");
                assert_eq!(var, varname);
                assert_eq!(reason, EnvVarMissingReason::Unset);
            }
            other => panic!("unexpected variant {other:?}"),
        }
    }

    #[test]
    fn env_empty_string_value_preserves_watch_only_sentinel() {
        // v0.25.1 invariant: `VAR=""` → empty-string passthrough so the
        // ms1 watch-only sentinel still fires downstream.
        let varname = "MNEMONIC_TEST_ENV_SENTINEL_EMPTY_OK";
        std::env::set_var(varname, "");
        let got = resolve_env_var_sentinel(&format!("@env:{varname}"), "--ms1").unwrap();
        std::env::remove_var(varname);
        assert_eq!(got, "");
    }

    #[test]
    fn invalid_name_lowercase_rejected() {
        let err = resolve_env_var_sentinel("@env:lowercase", "--ms1").unwrap_err();
        match err {
            ToolkitError::EnvVarMissing { reason, var, .. } => {
                assert_eq!(reason, EnvVarMissingReason::InvalidName);
                assert_eq!(var, "lowercase");
            }
            other => panic!("unexpected variant {other:?}"),
        }
    }

    #[test]
    fn invalid_name_starts_with_digit_rejected() {
        let err = resolve_env_var_sentinel("@env:1FOO", "--ms1").unwrap_err();
        match err {
            ToolkitError::EnvVarMissing { reason, .. } => {
                assert_eq!(reason, EnvVarMissingReason::InvalidName);
            }
            other => panic!("unexpected variant {other:?}"),
        }
    }

    #[test]
    fn invalid_name_with_space_rejected() {
        let err = resolve_env_var_sentinel("@env:FOO BAR", "--ms1").unwrap_err();
        match err {
            ToolkitError::EnvVarMissing { reason, .. } => {
                assert_eq!(reason, EnvVarMissingReason::InvalidName);
            }
            other => panic!("unexpected variant {other:?}"),
        }
    }

    #[test]
    fn invalid_name_empty_rejected() {
        let err = resolve_env_var_sentinel("@env:", "--ms1").unwrap_err();
        match err {
            ToolkitError::EnvVarMissing { reason, var, .. } => {
                assert_eq!(reason, EnvVarMissingReason::InvalidName);
                assert_eq!(var, "");
            }
            other => panic!("unexpected variant {other:?}"),
        }
    }

    #[test]
    fn underscore_leading_name_accepted() {
        let varname = "_MNEMONIC_TEST_ENV_LEADING_UNDERSCORE";
        std::env::set_var(varname, "v");
        let got = resolve_env_var_sentinel(&format!("@env:{varname}"), "--ms1").unwrap();
        std::env::remove_var(varname);
        assert_eq!(got, "v");
    }

    #[test]
    fn digits_in_tail_accepted() {
        let varname = "MNEMONIC_TEST_ENV_SENTINEL_DIGIT_42";
        std::env::set_var(varname, "v42");
        let got = resolve_env_var_sentinel(&format!("@env:{varname}"), "--ms1").unwrap();
        std::env::remove_var(varname);
        assert_eq!(got, "v42");
    }

    #[test]
    fn message_template_for_unset() {
        let err = ToolkitError::EnvVarMissing {
            flag: "--ms1".to_string(),
            var: "MY_VAR".to_string(),
            reason: EnvVarMissingReason::Unset,
        };
        assert_eq!(
            err.message(),
            "--ms1: env-var MY_VAR referenced by sentinel is not set"
        );
        assert_eq!(err.exit_code(), 1);
    }

    #[test]
    fn message_template_for_invalid_name() {
        let err = ToolkitError::EnvVarMissing {
            flag: "--passphrase".to_string(),
            var: "1bad".to_string(),
            reason: EnvVarMissingReason::InvalidName,
        };
        assert_eq!(err.message(), "--passphrase: invalid env-var name `1bad`");
        assert_eq!(err.exit_code(), 1);
    }
}
