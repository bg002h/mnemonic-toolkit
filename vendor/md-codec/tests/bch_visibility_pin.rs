//! Compile-time visibility pin for md-codec public BCH surface.
//!
//! Promoted in v0.34.0 to authorize downstream-codec consumers
//! (toolkit + ms-codec) per plan §2.B.1. The six symbols below were
//! promoted from bare-private or `pub(crate)` to `pub`:
//!
//! - `GEN_REGULAR`                   (bch.rs:7)   bare-private → pub
//! - `MD_REGULAR_CONST`              (bch.rs:17)  bare-private → pub
//! - `polymod_run`                   (bch.rs:34)  bare-private → pub
//! - `hrp_expand`                    (bch.rs:43)  bare-private → pub
//! - `bch_create_checksum_regular`   (bch.rs:57)  pub(crate)   → pub
//! - `bch_verify_regular`            (bch.rs:70)  pub(crate)   → pub
//!
//! Catches accidental visibility regression: if a future refactor
//! drops `pub` on any of these (or on `pub mod bch;` in lib.rs) this
//! file stops compiling.
//!
//! ## Intentionally NOT promoted (per plan Q3 lock)
//!
//! The following remain bare-private inside `bch.rs` because B.2's
//! vendored `bch_decode.rs` re-declares them locally and external
//! consumers have no use for them:
//!
//! - `POLYMOD_INIT`   (bch.rs:19)
//! - `REGULAR_SHIFT`  (bch.rs:20)
//! - `REGULAR_MASK`   (bch.rs:21)
//! - `polymod_step`   (bch.rs:23)
//!
//! These cannot be exercised by an integration test (bare-private
//! items aren't reachable from outside the crate); this comment block
//! documents the deliberate non-promotion so future PRs don't drift.

use md_codec::bch::{
    GEN_REGULAR, MD_REGULAR_CONST, bch_create_checksum_regular, bch_verify_regular, hrp_expand,
    polymod_run,
};

#[test]
fn promoted_symbols_compile() {
    // Touch each promoted symbol to ensure the import resolves and the
    // type signatures match the public-API contract.
    let _gen: [u128; 5] = GEN_REGULAR;
    let _target: u128 = MD_REGULAR_CONST;
    let _residue: u128 = polymod_run(&[]);
    let _expanded: Vec<u8> = hrp_expand("md");
    let _checksum: [u8; 13] = bch_create_checksum_regular("md", &[]);
    let _ok: bool = bch_verify_regular("md", &[0u8; 13]);
}
