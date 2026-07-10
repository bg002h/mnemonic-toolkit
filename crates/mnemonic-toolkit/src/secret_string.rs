//! A serialize-transparent secret string that zeroizes on drop.
//!
//! Wraps [`zeroize::Zeroizing<String>`] so every copy (struct field, clone,
//! propagated `format!`) scrubs its heap buffer when dropped. [`Serialize`] is
//! transparent (`serialize_str` on the inner `&str`) so a `--json` field typed
//! `SecretString` emits BYTE-IDENTICALLY to a plain `String` — no wire-shape
//! change. [`Display`] / [`Deref`] keep text-path rendering unchanged.
//!
//! Used for derived private-key material that is intentionally emitted to
//! stdout (silent-payment scan/spend priv, nostr WIF) but must not linger in
//! the heap after the command returns. Best-effort caveat: the EMITTED bytes
//! (stdout / OS pipe / terminal) and the secp256k1 source keys are out of
//! scope — the same allocator-residue limit documented elsewhere in the crate.
//!
//! [`Serialize`]: serde::Serialize
//! [`Display`]: std::fmt::Display
//! [`Deref`]: std::ops::Deref

use zeroize::Zeroizing;

/// A `String` whose heap buffer is zeroized on drop, serialized transparently.
#[derive(Clone)]
pub struct SecretString(Zeroizing<String>);

impl SecretString {
    /// Wrap an owned `String` (e.g. a `hex::encode(secret_bytes())` or a WIF).
    pub fn new(s: String) -> Self {
        Self(Zeroizing::new(s))
    }
}

impl std::ops::Deref for SecretString {
    type Target = str;
    fn deref(&self) -> &str {
        &self.0
    }
}

/// Plain (non-constant-time) structural equality. `SecretString` equality is
/// used ONLY in tests (`assert_eq!` of parse output vs an expected literal)
/// and in `SlotInput::is_stdin_sentinel`'s public `"-"` sentinel check — there
/// is no auth / timing boundary where a compare leaks anything (cycle-14 D2).
/// A constant-time compare here would be cargo-cult; the plain `String` `eq`
/// is correct and keeps the zero `subtle`-dependency. Required so a struct
/// `#[derive(PartialEq, Eq)]` keeps compiling with a `SecretString` field.
impl PartialEq for SecretString {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for SecretString {}

/// Cycle G (`repair-engine-outcome-zeroization`) — compare a `SecretString`
/// against a bare `str` / `&str` literal. Test-ergonomics only (the repair
/// engine's `assert_eq!(outcome.corrected_chunks[i], "…")` sites); no
/// production comparison relies on either impl (production code that needs
/// to compare a `SecretString`'s content derefs explicitly, e.g. `&**c ==
/// expected`). Same plain-structural rationale as `impl PartialEq for
/// SecretString` above — no auth/timing boundary crosses these comparisons.
impl PartialEq<str> for SecretString {
    fn eq(&self, other: &str) -> bool {
        self.0.as_str() == other
    }
}

impl PartialEq<&str> for SecretString {
    fn eq(&self, other: &&str) -> bool {
        self.0.as_str() == *other
    }
}

impl std::fmt::Display for SecretString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Length-only Debug — never render the secret into logs/panics.
impl std::fmt::Debug for SecretString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SecretString(<{} chars redacted>)", self.0.len())
    }
}

impl serde::Serialize for SecretString {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::SecretString;

    /// T-B1 — Serialize is transparent: a `SecretString` field emits the SAME
    /// JSON as a plain `String` field (no `--json` wire-shape change).
    #[test]
    fn serializes_byte_identically_to_string() {
        assert_eq!(
            serde_json::to_string(&SecretString::new("deadbeef".to_string())).unwrap(),
            serde_json::to_string("deadbeef").unwrap(),
        );

        #[derive(serde::Serialize)]
        struct Secret {
            wif: Option<SecretString>,
        }
        #[derive(serde::Serialize)]
        struct Plain {
            wif: Option<String>,
        }
        let s = Secret {
            wif: Some(SecretString::new("Kx...wif".to_string())),
        };
        let p = Plain {
            wif: Some("Kx...wif".to_string()),
        };
        assert_eq!(
            serde_json::to_string(&s).unwrap(),
            serde_json::to_string(&p).unwrap(),
        );
    }

    /// T-B2 — Display/Deref render the value verbatim (text-path unchanged).
    #[test]
    fn display_and_deref_are_transparent() {
        let s = SecretString::new("abc123".to_string());
        assert_eq!(format!("{s}"), "abc123");
        assert_eq!(&*s, "abc123");
        assert_eq!(format!("prefix{s}"), "prefixabc123");
    }

    /// Debug must NOT leak the secret (length-only redaction).
    #[test]
    fn debug_redacts_the_secret() {
        let s = SecretString::new("supersecretwif".to_string());
        let dbg = format!("{s:?}");
        assert!(
            !dbg.contains("supersecretwif"),
            "Debug leaked the secret: {dbg}"
        );
        assert!(
            dbg.contains("redacted"),
            "Debug should mark redaction: {dbg}"
        );
    }

    /// T2 (cycle-14, L22) — plain (non-constant-time) `PartialEq`/`Eq`.
    /// Equality is test-only + the public `"-"` sentinel: no auth/timing
    /// boundary (see BRAINSTORM D2), so a structural `String` compare is
    /// correct. Required so `SlotInput`'s `#[derive(PartialEq, Eq)]` keeps
    /// compiling once `SlotInput.value: SecretString`.
    #[test]
    fn partial_eq_and_eq_are_structural() {
        assert_eq!(
            SecretString::new("a".to_string()),
            SecretString::new("a".to_string()),
        );
        assert_ne!(
            SecretString::new("a".to_string()),
            SecretString::new("b".to_string()),
        );
        // Used in equality-bearing collections / asserts; Eq is total.
        fn _assert_eq<T: Eq>() {}
        _assert_eq::<SecretString>();
    }

    /// T2 (cycle-14) — Debug-redaction proves the chosen `SecretString`
    /// avoids option-(a)'s leak: a raw `Zeroizing<String>` derives a
    /// NON-redacting tuple-struct Debug (`Zeroizing("secret")`), which would
    /// surface the secret in `assert_eq!` failure output. `SecretString`'s
    /// length-only Debug never does. (Distinct from `debug_redacts_the_secret`
    /// in that it pins the equality-failure-print path specifically.)
    #[test]
    fn eq_failure_debug_does_not_leak() {
        let a = SecretString::new("topsecretphrase".to_string());
        let b = SecretString::new("differentvalue".to_string());
        assert_ne!(a, b);
        // The string that an `assert_eq!(a, b)` panic would print:
        let printed = format!("left: {a:?}, right: {b:?}");
        assert!(
            !printed.contains("topsecretphrase") && !printed.contains("differentvalue"),
            "equality-failure Debug leaked a secret: {printed}"
        );
    }

    /// Cycle G (`repair-engine-outcome-zeroization`) SPEC §1/§4.3 —
    /// `PartialEq<str>` / `PartialEq<&str>` ergonomics, added so the
    /// repair-engine's `assert_eq!(outcome.corrected_chunks[i], "…")` test
    /// sites keep compiling once `corrected_chunks` is `Vec<SecretString>`.
    /// Exercises BOTH impls (the bare-`str` comparison via a deref'd
    /// operand, and the `&str` comparison via a literal).
    #[test]
    fn partial_eq_str_and_ref_str() {
        let s = SecretString::new("abc123".to_string());
        // PartialEq<&str> — the common `assert_eq!(secret, "literal")` shape.
        assert_eq!(s, "abc123");
        assert_ne!(s, "xyz789");
        // PartialEq<str> — an unsized bare-`str` operand.
        assert!(s == *"abc123");
        assert!(s != *"xyz789");
    }

    /// M6 (Cycle G) SPEC §4.7 — a `Vec<SecretString>` field serializes
    /// byte-identically to a `Vec<String>` field. Extends
    /// `serializes_byte_identically_to_string` (T-B1, scalar) to the
    /// Vec-of-secrets shape actually used by the repair engine's wire types
    /// (`RepairOutcome.corrected_chunks` / `RepairJson.corrected_chunks`).
    #[test]
    fn vec_serializes_byte_identically_to_vec_string() {
        let secrets = vec![
            SecretString::new("ms1aaa".to_string()),
            SecretString::new("ms1bbb".to_string()),
        ];
        let plain = vec!["ms1aaa".to_string(), "ms1bbb".to_string()];
        assert_eq!(
            serde_json::to_string(&secrets).unwrap(),
            serde_json::to_string(&plain).unwrap(),
        );
    }
}
