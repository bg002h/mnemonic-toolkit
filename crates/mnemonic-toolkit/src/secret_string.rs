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
        let s = Secret { wif: Some(SecretString::new("Kx...wif".to_string())) };
        let p = Plain { wif: Some("Kx...wif".to_string()) };
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
        assert!(!dbg.contains("supersecretwif"), "Debug leaked the secret: {dbg}");
        assert!(dbg.contains("redacted"), "Debug should mark redaction: {dbg}");
    }
}
