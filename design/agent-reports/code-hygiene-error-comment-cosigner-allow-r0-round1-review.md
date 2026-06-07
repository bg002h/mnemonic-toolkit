# R0 Architect Review — code-hygiene-error-comment-cosigner-allow — Round 1

> Persisted verbatim from the opus `feature-dev:code-architect` agent
> (`agentId: ab3db86419e81dead`). Had Read/Glob/Grep; verified against source.

---

## VERDICT: 0 Critical / 0 Important / 1 Minor — GREEN, cleared for implementation.

### Item 1 — 1b will build warning-clean (definitive): YES
`CosignerKeyInfo` is used in non-test production type positions across 4 modules: `parse_descriptor.rs:869` (`DescriptorBinding.cosigners: Vec<CosignerKeyInfo>`), `:1227`, `:1249`; `cmd/bundle.rs:1337`, `:1519`; `cmd/verify_bundle.rs:757`, `:936`; `synthesize.rs:231`. A used item cannot be `dead_code`-flagged. Removal of the `:218` allow is warning-clean. (Crate is hybrid lib+bin; `pub type CosignerKeyInfo` is lib public surface — wouldn't be flagged regardless; usage evidence settles it independently.)

### Item 2 — 1a reword accurate
Both layers typed: `bundle.rs:433` `slots[i].path == slots[j].path`; `parse_descriptor.rs:1212` `cs[i].path == cs[j].path`. The xpub leg uses `.to_string()` (string eq) — see Minor 1. No test asserts the `Bip388Distinctness` doc-comment text (`cli_bundle_origin_path_canon.rs:114` is a standalone test comment, not an assertion). No `readme_version_current` guard exists.

### Item 3 — SemVer: no-bump ff-merge to master
Compiled `mnemonic` binary is byte-identical (doc-comment reword + removal of an inert `#[allow]` → zero codegen). No consumer affected (GUI lib-pin / schema_mirror / manual lint untouched). Same "no observable change" basis as prior test-only no-bump commits. **Recommendation: no-bump ff-merge.** PATCH+tag harmless but no-bump is principled.

### Item 4 — Scope/safety clean
No `#[allow(dead_code)]` on `synthesize_descriptor:229`. The `:218` allow is isolated. No test asserts the attribute or the comment text. No `readme_version_current` guard.

### Item 5 — Re-scope sound
`synthesize_descriptor:229` has no allow (the `:218` allow is on `CosignerKeyInfo:219`). Corrected target right. Cycle 2 shipped no doc falsehood (chapter correctly calls synthesize_descriptor "live").

### Minor 1 — align the reworded comment to the twin (folded)
SPEC said "typed `(xpub, DerivationPath)`" which elides the xpub `.to_string()` leg. Match `bundle.rs:423-428`: `(xpub.to_string(), path)` typed-`DerivationPath`. *(Folded into SPEC §1a.)*

### Verified clean
1. `error.rs:13-16` stale text confirmed. 2. `:218` allow on `CosignerKeyInfo:219`, not `synthesize_descriptor:229`. 3. `synthesize_descriptor` called at `:826` from `synthesize_unified` (CLI entrypoint) — reachable. 4. `CosignerKeyInfo` used in 4 source files. 5. No comment-text/attribute test. 6. Hybrid lib+bin.
