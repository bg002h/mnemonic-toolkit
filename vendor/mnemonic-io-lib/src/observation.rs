//! **What was measured, as types** — the vocabulary half of the seam.
//!
//! Two defects filed on 2026-08-26, in two repos, are this module's whole
//! argument:
//!
//! - **F-259** — `me sysw wipe --fill zeros` on a terminal exited 2 saying
//!   *"this payload is BEARER"* about a 65,536-byte zeros image the code itself
//!   declared carries no secret. The fact rode the `allow_world_readable`
//!   **`bool`**, and the terminal arm never consults that parameter.
//! - **F-260** — `mt encode` refused stdout mode 0620 saying its permissions
//!   *"grant read to group or others"*. `0620 & 0o044 == 0`: no read bit is set
//!   outside owner.
//!
//! **Both are messages hard-coded to a rule's NAME rather than derived from the
//! observation.** A message computed from the observed mode cannot say "read"
//! about a write-only mode. A payload kind carried as a TYPE cannot be read as
//! a permission override — which is F-259 exactly: one `bool` meant *"the
//! operator accepts file-permission risk"* to the flag and *"this payload is
//! not secret"* to `wipe`.
//!
//! **But the type is a convenience, and the test is the gate.** A probe built
//! `WriteBlock::Terminal(PayloadKind)` in its strongest form — message derived
//! from the carried kind — and then re-wrote F-259 by changing one pattern to
//! `WriteBlock::Terminal(_)`. Clean `cargo build`, clean
//! `cargo clippy --all-targets`, 391/391 tests passing, and the pty printed
//! *"this payload is BEARER"* at exit 2 once more.
//!
//! > **A type stops a value being CONFUSED for another value. It cannot stop a
//! > value being IGNORED.**
//!
//! What actually catches it is a pty assertion on the EMITTED WORDS, with a
//! positive control, mutation-checked in both directions —
//! `tests/terminal_destination.rs`.
//!
//! **Nothing here names a record `Class`.** Deciding what a string IS is `me`'s
//! job. A `PayloadKind` is the CALLER's declaration about bytes it just built,
//! not a verdict read off a record's shape, which is why it can live on the
//! shared side of the seam at all.

/// What the bytes about to be written are, as far as exposure goes.
///
/// **Its own parameter, never a `bool` shared with a policy flag.** F-259 is
/// what happens when this fact travels in the `allow_world_readable` seat: the
/// two facts are read by different arms of the same decision, and the arm that
/// needs this one never looks at that one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PayloadKind {
    /// Whoever can read these bytes can use what is in them — a container
    /// holding key material, or a signed transaction anyone could broadcast.
    Bearer,
    /// Measured to hold nothing: a 65,536-byte `random`/`zeros`/`ones` fill
    /// image, whose purpose is to DESTROY a payload. The opposite of bearer.
    ///
    /// It is still refused at a terminal — 64 KB of binary in a scrollback is
    /// worth refusing whatever the secrecy — but the refusal must say what is
    /// true of it, and "BEARER" is not.
    CarriesNoSecret,
}

impl PayloadKind {
    /// Does exposing these bytes expose anything?
    ///
    /// The **only** question the world-readable-file gate may ask of a kind.
    /// Named as a question about exposure rather than as `is_bearer` so a
    /// caller cannot read it as "is this the Bearer variant" and then reuse it
    /// for a decision it does not answer.
    pub fn exposure_matters(&self) -> bool {
        matches!(self, PayloadKind::Bearer)
    }
}
