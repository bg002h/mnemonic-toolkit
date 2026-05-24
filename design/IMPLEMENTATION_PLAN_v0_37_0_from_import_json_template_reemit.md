# v0.37.0 — `export-wallet --from-import-json` template auto-derive — Implementation Plan

> **For agentic workers:** execute task-by-task; each task is TDD (test → run-fail → impl → run-pass → commit). Per-phase opus reviewer-loop to 0C/0I before advancing (project R0 gate). Stage paths explicitly (no `git add -A`).

**Goal:** Make `export-wallet --from-import-json <envelope>` re-emit to the four template-requiring formats (sparrow/coldcard/jade/electrum) by deriving `--template` from the envelope's parsed descriptor.

**Architecture:** A new `template_from_descriptor` (descriptor → `CliTemplate`, unambiguous) + a `format_requires_template` predicate; `run_from_import_json` injects the derived template (and fixes `threshold_user_supplied`) only for template-requiring formats, leaving passthrough formats at `template: None`.

**Tech Stack:** Rust, miniscript 13, clap-derive. Source SHA basis `36e6bfa`. Spec: `design/BRAINSTORM_v0_37_0_from_import_json_template_reemit.md` (R0 GREEN). SemVer **MINOR → v0.37.0**.

**Branch:** `git checkout -b v0.37.0-from-import-json-template-reemit` off `master` (do NOT work on master).

**Plan R0 status:** ✅ **GREEN (0C/0I)** — opus architect verified every embedded code block compiles against real types + the 34/6 truth table + RED→GREEN. 4 Minors (M1–M4) folded. Review at `design/agent-reports/v0_37_0-plan-r0-review.md`.

---

## File map
- **Modify** `crates/mnemonic-toolkit/src/wallet_export/mod.rs` — add `template_from_descriptor` (after `script_type_from_descriptor`, ~:248).
- **Modify** `crates/mnemonic-toolkit/src/cmd/export_wallet.rs` — add `format_requires_template`; in `run_from_import_json` compute `derived_template` (after the `:629` taproot refusal) and edit the `:666`/`:671` `EmitInputs` fields.
- **Modify** `crates/mnemonic-toolkit/tests/cli_export_wallet_from_import_json.rs` — unit tests for `template_from_descriptor` are lib-level (see Task 1 note); rewrite `p11c` (`:841`), `p11a` refusal cell (`:611`), Cell 3 (`:96`), `REFUSAL_STDERR_PATTERNS` (`:814`); add round-trip success cells.
- **Modify** `docs/manual/src/45-foreign-formats.md` + `docs/manual/src/40-cli-reference/41-mnemonic.md` — recipe/prose lockstep.
- **Modify** `CHANGELOG.md`, `crates/mnemonic-toolkit/Cargo.toml`, `Cargo.lock`, `scripts/install.sh`, `design/FOLLOWUPS.md` — release prep.

---

## Phase 0 — RED (tests first)

### Task 0.1: Unit tests for `template_from_descriptor`
`template_from_descriptor` is `pub(crate)`, so its unit tests live **in-module** under `#[cfg(test)] mod tests` in `wallet_export/mod.rs` (the bin crate's lib-side; these run under `cargo test -p mnemonic-toolkit`). 

- [ ] **Step 1: Write failing tests.** Append to the existing `#[cfg(test)] mod` in `crates/mnemonic-toolkit/src/wallet_export/mod.rs` (or add one if absent):

```rust
#[cfg(test)]
mod template_from_descriptor_tests {
    use super::*;
    use miniscript::{Descriptor as MsDescriptor, DescriptorPublicKey};
    use std::str::FromStr;

    fn t(desc: &str) -> Result<crate::template::CliTemplate, crate::error::ToolkitError> {
        let d = MsDescriptor::<DescriptorPublicKey>::from_str(desc).unwrap();
        template_from_descriptor(&d)
    }

    // Two real account xpubs for multisig fixtures (mainnet).
    const X1: &str = "[b8688df1/48'/0'/0'/2']xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX/<0;1>/*";
    const X2: &str = "[28645006/48'/0'/0'/2']xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6/<0;1>/*";
    const WPKH: &str = "wpkh([b8688df1/84'/0'/0']xpub6BosfCnifzxcFwrSzQiqu2DBVTshkCXacvNsWGYJVVhhawA7d4R5WSWGFNbi8Aw6ZRc1brxMyWMzG3DSSSSoekkudhUd9yLb6qx39T9nMdj/0/*)#g8l27w39";

    #[test] fn wpkh_to_bip84() { assert_eq!(t(WPKH).unwrap(), crate::template::CliTemplate::Bip84); }
    #[test] fn pkh_to_bip44() {
        let d = format!("pkh([b8688df1/44'/0'/0']{})", "xpub6BosfCnifzxcFwrSzQiqu2DBVTshkCXacvNsWGYJVVhhawA7d4R5WSWGFNbi8Aw6ZRc1brxMyWMzG3DSSSSoekkudhUd9yLb6qx39T9nMdj/0/*");
        assert_eq!(t(&d).unwrap(), crate::template::CliTemplate::Bip44);
    }
    #[test] fn shwpkh_to_bip49() {
        let d = format!("sh(wpkh([b8688df1/49'/0'/0']{}))", "xpub6BosfCnifzxcFwrSzQiqu2DBVTshkCXacvNsWGYJVVhhawA7d4R5WSWGFNbi8Aw6ZRc1brxMyWMzG3DSSSSoekkudhUd9yLb6qx39T9nMdj/0/*");
        assert_eq!(t(&d).unwrap(), crate::template::CliTemplate::Bip49);
    }
    #[test] fn wsh_sortedmulti_to_wsh_sortedmulti() {
        let d = format!("wsh(sortedmulti(2,{X1},{X2}))");
        assert_eq!(t(&d).unwrap(), crate::template::CliTemplate::WshSortedMulti);
    }
    #[test] fn wsh_multi_to_wsh_multi() {
        let d = format!("wsh(multi(2,{X1},{X2}))");
        assert_eq!(t(&d).unwrap(), crate::template::CliTemplate::WshMulti);
    }
    #[test] fn shwsh_sortedmulti_to_sh_wsh_sortedmulti() {
        let d = format!("sh(wsh(sortedmulti(2,{X1},{X2})))");
        assert_eq!(t(&d).unwrap(), crate::template::CliTemplate::ShWshSortedMulti);
    }
    #[test] fn shwsh_multi_to_sh_wsh_multi() {
        let d = format!("sh(wsh(multi(2,{X1},{X2})))");
        assert_eq!(t(&d).unwrap(), crate::template::CliTemplate::ShWshMulti);
    }
    #[test] fn sortedmulti_not_misread_as_multi() {
        // Guard: "sortedmulti(" contains "multi(" — must NOT resolve to WshMulti.
        let d = format!("wsh(sortedmulti(2,{X1},{X2}))");
        assert_ne!(t(&d).unwrap(), crate::template::CliTemplate::WshMulti);
    }
    #[test] fn sh_bare_multi_errs() {
        let d = format!("sh(sortedmulti(2,{X1},{X2}))");
        assert!(t(&d).is_err(), "legacy bare P2SH multisig has no template");
    }
}
```

- [ ] **Step 2: Run, confirm fail.** `cargo test -p mnemonic-toolkit template_from_descriptor_tests 2>&1 | head -30` → FAIL (compile error: `template_from_descriptor` not found). This is the intended RED.
- [ ] **Step 3: Commit.** `git add crates/mnemonic-toolkit/src/wallet_export/mod.rs && git commit -m "test(v0.37.0): RED — template_from_descriptor unit tests"` (+ trailer).

### Task 0.2: Rewrite the C1 inverted matrix + add success cells
- [ ] **Step 1: Edit the test file.** In `crates/mnemonic-toolkit/tests/cli_export_wallet_from_import_json.rs`:

(a) Add the jade-singlesig pattern to `REFUSAL_STDERR_PATTERNS` (`:814-822`):
```rust
    // v0.37.0: jade singlesig refusal (template IS set on the from-import-json path).
    "emits multisig wallet config only",
```

(b) Add a `SINGLESIG_SOURCES` const near `ALL_SOURCES` (`:563`):
```rust
/// Sources whose happy-path fixture is a singlesig wallet (→ bip84).
/// The other 5 sources are wsh(sortedmulti) multisig (→ wsh-sortedmulti).
const SINGLESIG_SOURCES: &[&str] = &["bitcoin-core", "coldcard", "electrum"];
```
**M3:** an identically-named function-local `const SINGLESIG_SOURCES` already exists inside `p11c_green_descriptor_passthrough_…` at `:896` (same value). Delete that inner declaration so the module-level const is the single source of truth (the inner fn then reads the outer one — no behavior change).

(c) Replace the body of `p11c_refusal_matrix_strict_template_only_dests` (`:841-874`) with the §0-equivalence assertion (success expected unless singlesig→{coldcard-multisig,jade}):
```rust
fn p11c_template_only_dest_matrix_post_autoderive() {
    let mut cell_count = 0;
    let mut failures: Vec<String> = Vec::new();
    for src in ALL_SOURCES {
        let is_singlesig = SINGLESIG_SOURCES.contains(src);
        let fixture = fixture_path(happy_path_fixture(src));
        for dest in TEMPLATE_ONLY_DESTS {
            cell_count += 1;
            let expected_refuse = is_singlesig && (*dest == "coldcard-multisig" || *dest == "jade");
            let res = run_export_from_import_envelope(&fixture, src, dest);
            if expected_refuse {
                if res.exit_code == 0 {
                    failures.push(format!("[{src} → {dest}] expected refusal but exit=0"));
                } else if !REFUSAL_STDERR_PATTERNS.iter().any(|p| res.stderr.contains(p)) {
                    failures.push(format!("[{src} → {dest}] refusal stderr unmatched: {}", res.stderr));
                }
            } else if res.exit_code != 0 {
                failures.push(format!("[{src} → {dest}] expected success but exit={}; stderr={}", res.exit_code, res.stderr));
            }
        }
    }
    assert!(failures.is_empty(), "P11C post-auto-derive matrix ({}/{cell_count}): {failures:#?}", failures.len());
    assert_eq!(cell_count, 40, "8×5 = 40 cells");
}
```

(d) Re-point `p11a_helper_returns_nonzero_exit_on_template_only_dest_refusal` (`:611`) to a still-refusing cell — `bitcoin-core` (singlesig) → `coldcard-multisig`:
```rust
    let res = run_export_from_import_envelope(&fixture_path(happy_path_fixture("bitcoin-core")), "bitcoin-core", "coldcard-multisig");
    assert_ne!(res.exit_code, 0, "singlesig → coldcard-multisig must still refuse");
```

(e) Update Cell 3 (`:96-119`) — `envelope_v0_27_0.json` is `sh(multi)` → P2shMulti → new refusal message. Change the assertion (`:114-117`) to:
```rust
        assert!(
            stderr.contains("legacy bare P2SH") || stderr.contains("has no export-wallet template"),
            "format {fmt} must refuse sh(multi) envelope with the P2shMulti template-derivation message; got: {stderr}"
        );
```

- [ ] **Step 2: Run, confirm fail.** `cargo test -p mnemonic-toolkit --test cli_export_wallet_from_import_json 2>&1 | tail -30` → the rewritten cells FAIL against current `master` behavior (every template-only dest still refuses pre-fix). Intended RED.
- [ ] **Step 3: Commit.** `git add crates/mnemonic-toolkit/tests/cli_export_wallet_from_import_json.rs && git commit -m "test(v0.37.0): RED — rewrite p11c/p11a/Cell-3 matrix for template auto-derive"` (+ trailer).

### Task 0.3: Round-trip success cells (§5.2/§5.3)
- [ ] **Step 1:** Add a same-format round-trip test asserting import→re-emit reproduces the source (canonicalized) for representative pairs. Reuse `run_export_from_import_envelope` for the cross-format successes already covered by p11c; add one **same-format** byte-equality cell per script class using a matching `--wallet-name`. Since the existing helper does not thread `--wallet-name`, add a sibling helper `run_export_from_import_envelope_named(fixture, src, dest, wallet_name)` that appends `["--wallet-name", wallet_name]` to the export args, and assert the re-emit succeeds + (for sparrow/electrum JSON) parses + carries the expected descriptor/script-type. **M2 (reconciles spec §0/§5.3):** §0's "byte-identical output once `--wallet-name`/`--account` are matched" remains the conceptual contract, but the *test* deliberately does NOT byte-compare against a direct `--template <derived>` invocation — the from-import-json path reconstructs `ResolvedSlot`s from the envelope's mk1 cards (`json_envelope.rs::envelope_to_resolved_slots`) while the direct path builds them from `--slot @N.xpub=` via `bundle::resolve_slots`; these are different pipelines whose xpub/fingerprint encoding could diverge, so a direct byte-compare risks failing for reasons orthogonal to this feature. The round-trip-against-source equality + the p11c exit-class matrix together lock the contract without that fragility.
- [ ] **Step 2:** Run → FAIL (success cells refuse pre-fix). **Step 3:** Commit.

### Task 0.4: Passthrough byte-identity regression (§5.5)
- [ ] **Step 1:** Add a test asserting bitcoin-core / bip388 / bsms re-emit output is **byte-identical** before and after the change — capture current `master` output as the golden (the `p11b_happy_path_matrix` at `:722` already exercises these 24 descriptor-capable cells; add an explicit byte-snapshot assert for one cell per passthrough format). **Step 2:** Run → PASS on current master (these are unaffected; this guards Phase 1). **Step 3:** Commit.

---

## Phase 1 — GREEN (implementation)

### Task 1.1: Add `template_from_descriptor`
- [ ] **Step 1:** In `crates/mnemonic-toolkit/src/wallet_export/mod.rs`, after `script_type_from_descriptor` (ends ~:248):

```rust
/// SPEC v0.37 §2.2 — map a parsed (non-taproot) `Descriptor` to its `CliTemplate`.
/// Unlike `script_type_from_descriptor`, this preserves the sorted/unsorted
/// multisig distinction (the descriptor carries it), so no inverse ambiguity
/// arises. Used by the `--from-import-json` path. `sortedmulti(` is checked
/// before `multi(` (substring). Taproot is refused upstream on that path; the
/// `Tr(_)` arm is defensive.
pub(crate) fn template_from_descriptor(
    d: &MsDescriptor<DescriptorPublicKey>,
) -> Result<CliTemplate, ToolkitError> {
    use miniscript::descriptor::ShInner;
    use miniscript::Descriptor::*;
    let is_sorted = d.to_string().contains("sortedmulti(");
    match d {
        Pkh(_) => Ok(CliTemplate::Bip44),
        Wpkh(_) => Ok(CliTemplate::Bip84),
        Sh(s) => match s.as_inner() {
            ShInner::Wpkh(_) => Ok(CliTemplate::Bip49),
            ShInner::Wsh(_) => Ok(if is_sorted { CliTemplate::ShWshSortedMulti } else { CliTemplate::ShWshMulti }),
            ShInner::Ms(_) => Err(ToolkitError::BadInput(
                "--from-import-json: legacy bare P2SH multisig (sh(multi)/sh(sortedmulti)) has no export-wallet template; use --format bitcoin-core for descriptor passthrough".into(),
            )),
        },
        Wsh(_) => Ok(if is_sorted { CliTemplate::WshSortedMulti } else { CliTemplate::WshMulti }),
        Tr(_) => Err(ToolkitError::BadInput(
            "--from-import-json: taproot descriptors are refused upstream; template_from_descriptor should not be reached for taproot".into(),
        )),
        Bare(_) => Err(ToolkitError::DescriptorParse(
            "wallet-export descriptor must have a top-level Pkh/Wpkh/Sh/Wsh wrapper".into(),
        )),
    }
}
```
(`CliTemplate`, `MsDescriptor`, `DescriptorPublicKey`, `ToolkitError` are already imported in this module — confirm with `cargo build -p mnemonic-toolkit`.) **M1 note:** the `Bare(_)` arm uses `DescriptorParse` (not the spec §2.2 table's `BadInput`) to mirror `script_type_from_descriptor`'s existing `Bare` handling; it is doubly-unreachable on this path (`script_type_from_descriptor` runs first at `:628` and rejects `Bare` before `template_from_descriptor` is called), so the variant choice is cosmetic.

- [ ] **Step 2:** `cargo test -p mnemonic-toolkit template_from_descriptor_tests` → PASS. **Step 3:** Commit `git add …/wallet_export/mod.rs`.

### Task 1.2: Add `format_requires_template` + wire into `run_from_import_json`
- [ ] **Step 1a:** In `crates/mnemonic-toolkit/src/cmd/export_wallet.rs` (module scope, near the `CliExportFormat` enum at `:22`):

```rust
/// SPEC v0.37 §2.3 — formats whose file-import surface refuses a bare
/// descriptor and requires a `--template`. On the `--from-import-json` path
/// these receive a template derived from the envelope descriptor; all others
/// keep `template: None`. Exhaustive (no `_` arm) so a new variant forces a decision.
fn format_requires_template(f: CliExportFormat) -> bool {
    use CliExportFormat::*;
    match f {
        Sparrow | Coldcard | ColdcardMultisig | Jade | Electrum => true,
        BitcoinCore | Bip388 | Bsms | Green | Specter => false,
    }
}
```

- [ ] **Step 1b:** In `run_from_import_json`, immediately before `let inputs = EmitInputs {` (`:661`), add:

```rust
    // SPEC v0.37 §2.3 — derive the template from the envelope descriptor for
    // template-requiring formats (taproot already refused above, §2.4).
    let derived_template: Option<CliTemplate> = if format_requires_template(args.format) {
        Some(crate::wallet_export::template_from_descriptor(&parsed_ms)?)
    } else {
        None
    };
```

- [ ] **Step 1c:** In that `EmitInputs` literal, change `template: None,` (`:666`) → `template: derived_template,` and `threshold_user_supplied: false,` (`:671`) → `threshold_user_supplied: threshold.is_some(),`.

- [ ] **Step 2:** `cargo test -p mnemonic-toolkit --test cli_export_wallet_from_import_json` → all rewritten + new cells PASS; passthrough byte-identity PASS. Then `cargo test -p mnemonic-toolkit` (full) + `cargo clippy --all-targets -- -D warnings`. **Step 3:** Commit `git add …/cmd/export_wallet.rs`.

---

## Phase 2 — Manual lockstep

- [ ] **Step 1:** In `docs/manual/src/45-foreign-formats.md`, strip `--template`/`--threshold` from the 5 recipes (head+token lines per spec §3 table: `:313/:314`, `:481/:482`, `:564/:565`, `:639/:640`, `:752/:753`) so they read `mnemonic export-wallet --from-import-json envelope.json --format <F> [> out]`. **M4:** the coldcard-multisig recipe is a **3-line** fenced command (`:564` head, `:565` `--format … --template … --threshold … \`, `:566` `> coldcard_ms_re.txt`) — when stripping, fold into a valid single continuation (e.g. `--format coldcard > coldcard_ms_re.txt`) so no dangling `\` remains; same for jade `:639-641` if it spans 3 lines. Update the `:577` coldcard-multisig prose and the `:347` general prose ("requires a recognized `--template`…") to describe auto-derivation. **Leave `:352-357` (taproot round-trip note) unchanged.** Re-grep the chapter: `grep -n "from-import-json" docs/manual/src/45-foreign-formats.md | head` and confirm no residual `--template` continuation lines.
- [ ] **Step 2:** In `docs/manual/src/40-cli-reference/41-mnemonic.md:669`, add to the `--from-import-json` row: "for file-import formats (sparrow/coldcard/jade/electrum) the `--template` is auto-derived from the envelope descriptor (the user still cannot pass `--template` — it remains mutually exclusive)."
- [ ] **Step 3:** `make -C docs/manual lint MNEMONIC_BIN=$(pwd)/target/debug/mnemonic MD_BIN=true MS_BIN=true MK_BIN=true` → passes. **Step 4:** Commit `git add docs/manual/src/45-foreign-formats.md docs/manual/src/40-cli-reference/41-mnemonic.md`.

---

## Phase 3 — Release prep

- [ ] **Step 1:** `crates/mnemonic-toolkit/Cargo.toml` version `0.36.4 → 0.37.0`; run `cargo build -p mnemonic-toolkit` to refresh `Cargo.lock`.
- [ ] **Step 2:** Prepend a `## mnemonic-toolkit [0.37.0] — 2026-05-24` section to `CHANGELOG.md` (SemVer-MINOR; describe the auto-derive + the closed FOLLOWUP + the manual lockstep + the 4-round R0).
- [ ] **Step 3:** `scripts/install.sh:32` self-pin `mnemonic-toolkit-v0.36.4 → mnemonic-toolkit-v0.37.0`.
- [ ] **Step 4:** `design/FOLLOWUPS.md` — flip `export-wallet-from-import-json-template-format-reemit` `Status: open → resolved` (note the v0.37.0 commit). Leave `manual-prose-command-execution-gate` open (Cycle B).
- [ ] **Step 5:** `git add` each path explicitly; commit `release(toolkit): mnemonic-toolkit v0.37.0 — export-wallet --from-import-json template auto-derive` (+ trailer).
- [ ] **Step 6:** Per-phase opus reviews persisted to `design/agent-reports/v0_37_0-phase-{0,1,2,3}-*.md` before each fold-and-commit; end-of-cycle review to 0C/0I before tag.

---

## Self-review checklist
- Spec coverage: §2.2 (Task 1.1), §2.3 (Task 1.2), §2.5 threshold (Task 1.2 Step 1c), §2.6 coldcard-multisig (p11c refuse cells), §3 manual (Phase 2), §5.1–5.5 (Phase 0). ✓
- Type consistency: `CliTemplate` variants match `template.rs:15`; `format_requires_template` exhaustive over the 10 `CliExportFormat` variants. ✓
- No placeholders: all code blocks complete. The only deferred decision is the same-format round-trip helper shape (Task 0.3) — spelled out as `run_export_from_import_envelope_named`.
