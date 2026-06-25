//! README↔KAT parity guard (P5, doc-fidelity output-fidelity program): the
//! crate README quotes a literal `mnemonic derive-child … --application dice`
//! output (`README.md:185`, `# → 1,0,0,2,0,1,5,5,2,4`). That COPY of the value
//! is otherwise ungated, so a bad README edit could silently drift it away from
//! the real binary output. This test pins the README's copy against the same
//! literal the binary-side KATs assert.
//!
//! Source of truth for the value (a BIP-85 v1.3.0 §"DICE" reference vector,
//! path `m/83696968'/89101'/6'/10'/0'`):
//!   - `tests/cli_derive_child.rs:692` — end-to-end CLI assert
//!     (`assert_eq!(stdout, "1,0,0,2,0,1,5,5,2,4\n")`).
//!   - `src/bip85.rs:401` — unit-level assert
//!     (`assert_eq!(&*rolls, "1,0,0,2,0,1,5,5,2,4")`).
//! The value is not exposed as a `pub const`, so the literal is duplicated here
//! with those two cross-citations as the canonical pin. If the KAT value ever
//! legitimately changes, update all three sites in lockstep.
//!
//! Path note: mirrors `readme_version_current.rs` — runs only via the repo's own
//! `cargo test -p mnemonic-toolkit`, so `CARGO_MANIFEST_DIR/README.md` (the
//! published `readme=`) is always present.

use std::fs;
use std::path::Path;

/// The dice KAT as it must appear, verbatim, in the crate README.
/// Cross-cited source of truth: `tests/cli_derive_child.rs:692` + `src/bip85.rs:401`.
const DICE_KAT: &str = "1,0,0,2,0,1,5,5,2,4";

#[test]
fn readme_dice_kat_matches_canonical_value() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("README.md");
    let body = fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    assert!(
        body.contains(DICE_KAT),
        "{} must quote the dice KAT `{DICE_KAT}` (the canonical BIP-85 v1.3.0 §\"DICE\" \
         reference vector pinned at tests/cli_derive_child.rs:692 + src/bip85.rs:401); \
         a README edit drifted the documented output away from the real binary value",
        path.display(),
    );
}
