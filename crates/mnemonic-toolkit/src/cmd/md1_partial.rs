//! Shared helpers for pathless/dead-card partial-decode (md1) — P2.2 / P2.3.
//!
//! A `canonical_origin == None` md1 card whose per-`@N` origin is elided-and-
//! unresolvable (a "dead card") is now rendered as a PARTIAL decode: the
//! always-renderable template + an explicit `origin: «unspecified»` marker +
//! exit 4 (VERIFY-ME), never a silent fake `m/` path. Guiding principle bound:
//! be maximally expressive on output, permissive on input — BOUNDED by never
//! silently misrepresent.
//!
//! The marker text + stderr note are duplicated VERBATIM from md-cli's
//! `crates/md-cli/src/cmd/partial.rs` (`ORIGIN_UNSPECIFIED_MARKER` /
//! `emit_partial_stderr_note`). md-cli holds them `pub(crate)`, so they cannot
//! be imported across the crate boundary — keep the two copies byte-identical
//! (the SPEC "cross-binary parity" contract: a human reads exactly the same
//! bytes on `md` and `mnemonic`).

/// Text-form marker (stdout, partial only) printed in addition to the
/// always-renderable template when the decoded md1 descriptor carries at least
/// one unresolved-origin `@N`. Byte-identical to md-cli's
/// `ORIGIN_UNSPECIFIED_MARKER`.
pub(crate) const ORIGIN_UNSPECIFIED_MARKER: &str =
    "origin: \u{ab}unspecified \u{2014} supply on restore\u{bb}";

/// Emit the partial-decode stderr note (partial case only; never on stdout).
/// `unres` is the ascending set of unresolved-origin `@N` indices (non-empty
/// when called). Byte-identical to md-cli's `emit_partial_stderr_note`.
pub(crate) fn emit_partial_stderr_note<W: std::io::Write>(unres: &[u8], w: &mut W) {
    let idxs = unres
        .iter()
        .map(|i| format!("@{i}"))
        .collect::<Vec<_>>()
        .join(", ");
    let _ = writeln!(
        w,
        "note: the origin(s) for {idxs} are unspecified \u{2014} this card shape has no canonical \
         default derivation path and none was supplied explicitly; exit 4 (VERIFY-ME): confirm \
         the intended path out-of-band before restoring funds from this backup"
    );
}
