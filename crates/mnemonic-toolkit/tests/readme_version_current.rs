//! Anti-decay guard (v0.36.3): BOTH READMEs — the repo-root GitHub-landing
//! README and the crate's published `readme=` (`crates/mnemonic-toolkit/README.md`,
//! `Cargo.toml`) — must carry the current toolkit-version marker. The READMEs
//! silently decayed v0.8.0 → v0.36.2 (28 versions) for lack of this gate; the
//! manual stayed current only because its 6-stage lint gated it.
//!
//! Marker form (one place the release bump touches per file):
//!   <!-- toolkit-version: 0.36.3 -->
//!
//! Scope (deliberate): this guards the single version STRING, killing the
//! status-line decay. It does NOT police the feature narrative / subcommand
//! inventory — those are kept low-drift by pointing each README at the manual
//! (the CLI-reference single-source-of-truth) + CHANGELOG (version history).
//!
//! Path note: this test runs only via the repo's own `cargo test -p
//! mnemonic-toolkit` (the toolkit is git+tag-only, never a crates.io build), so
//! `CARGO_MANIFEST_DIR/../../README.md` (repo root) is always present.

use std::fs;
use std::path::Path;

#[test]
fn both_readmes_carry_current_version_marker() {
    let want = env!("CARGO_PKG_VERSION"); // = Cargo.toml version at compile time
    let marker = format!("<!-- toolkit-version: {want} -->");
    // crate-dir README (the published `readme=`) + repo-root README.
    for rel in ["README.md", "../../README.md"] {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
        let body =
            fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        assert!(
            body.contains(&marker),
            "{} must carry `{marker}` (the READMEs decayed to v0.8.0 once for lack \
             of this gate); update the README status line + marker in lockstep with \
             the crate version",
            path.display(),
        );
    }
}
