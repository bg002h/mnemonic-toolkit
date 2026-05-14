//! Trezor SLIP-0039 hierarchical K-of-N Shamir Secret Sharing.
//!
//! See `design/SPEC_slip39_v0_13_0.md` for the contract. Phase 1a lands
//! the math primitives only (GF(256) field arithmetic + Lagrange
//! interpolation). Encryption pipeline (Feistel + PBKDF2) ships at
//! P1b. Share encoding (RS1024 + wordlist + bit-packing + parse/render)
//! + the public `slip39_split` / `slip39_combine` surface ships at P1c.
//!
//! Library-local `Slip39Error` per the v0.11.0 / v0.12.0 precedent;
//! tracked under FOLLOWUP `library-error-and-language-surface-promotion`
//! for the future crate-shape unification with `ToolkitError`.

pub mod gf256;
pub mod lagrange;
