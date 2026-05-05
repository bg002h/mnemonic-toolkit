# v0.1 audit for v0.2 extension

**Date:** 2026-05-05
**Subject commit:** `8cf8348` (HEAD of master at audit time)
**Reviewer:** opus pre-v0.2 audit (per `audit_before_extending` memory rule)
**v0.2 scope being prepared:** multisig templates, `--account`, `--xpub`-input multisig, `--privacy-preserving`, `--self-check` (K-of-N share encoding deferred)

## Verdict

- **Critical** (must fix before v0.2 brainstorm): 0
- **Important** (must fix as Phase 0 fixup before v0.2 SPEC): **2**
  - `bundle-mismatch-card-static-str-constraint` (also in FOLLOWUPS.md; meta-plan flagged)
  - `error-allow-comments-staleness` (consequence of IMP-1; same-commit fix)
- **Low** (defer to v0.2 FOLLOWUPS scope or beyond): 4
- **Informational** (no action): 6

Cleared to proceed to v0.2 brainstorm after the 2 Important fixups land in a single Phase 0 commit on master.

## A. SPEC drift

Spot-checked SPEC §6.2 / §6.3 / §5.3 / §5.4 / §6.6 / §4.1 / §4.6 / §4.7 against shipped code at HEAD `8cf8348`. **No drift.** All field shapes, exit-code routings, JSON field orders, byte-exact mode-violation strings, derivation flow, typed-struct construction, and cross-binding invariants match SPEC verbatim. The 9-element `checks` array in `verify_bundle::run_full` and `watch_only_checks` matches §5.4 ordering exactly (pinned by `assert_spec_order` test).

One subtlety: the §4.7 invariant 1 debug-assert (`policy_id_stubs[0] == policy_id.as_bytes()[..4]`) is tautological by construction — `stub` is assigned from `policy_id.as_bytes()[..4]` and immediately passed as `policy_id_stubs[0]`. Pre-existing; not drift. Recorded as LOW-1.

## B. Latent bugs

### B.1 Unreachable / unsafe patterns

- `debug_assert!` (release-elided) on the §4.7 invariants. Phase 2 r1 flagged debug-vs-release split as L-4. Pre-existing; v0.2 multisig should consider promoting to `assert!` once invariants apply across multiple stubs.
- `template.derivation_path().expect(...)` is genuinely unreachable (paths are compile-time constants).
- No `unwrap()` calls in non-test code. `From<ms_codec::Error>` uses `unwrap_or("<non-utf8>")`.

### B.2 Edge-case stubs

- **LOW-2 `dead-inner-guard-bundle-watch-only`** (`cmd/bundle.rs:200`): redundant `--xpub`-requires-`--master-fingerprint` guard inside `bundle_watch_only` is unreachable because the mode-violation pre-check at `cmd/bundle.rs:93` rejects the same condition earlier with exit 2 + byte-exact §6.6 text. The dead guard would emit `BadInput` (exit 1) with different text — a future-refactor inconsistency risk. Not currently triggered.
- `synthesize_full` and `synthesize_watch_only` hard-code `n: 1` and `PathDeclPaths::Shared`. v0.2 multisig will replace these wholesale (not an extension). Expected per SPEC §4.6.2; flagged for v0.2 implementer awareness.

### B.3 Test corpus gaps (LOW-3 `friendly-mapper-unit-test-gaps`)

`friendly.rs` unit tests cover only 3 of the ~70 friendly-mapper match arms:
- `friendly_bip39`: 1 of 5 variants (`UnknownWord`); 4 untested (`BadEntropyBitCount`, `BadWordCount`, `InvalidChecksum`, `AmbiguousLanguages`).
- `friendly_bitcoin`: 0 of 3 variants.
- `friendly_ms_codec`: 1 of 9 (`WrongHrp`).
- `friendly_mk_codec`: 1 of 22 (`PathTooDeep`).
- `friendly_md_codec`: 0 of 41.

Integration tests (Phase 5) likely exercise some paths end-to-end, but unit isolation is thin. v0.2 will add new error paths through these mappers. Recommend Phase E of v0.2 implementation plan adds a parametric unit test exercising every friendly-mapper variant.

### B.4 `#[allow(dead_code)]` inventory

All 6 narrow allows verified as genuinely reserved-for-later, none accidentally dead:
- `error.rs:19` `ModeViolation.{mode, flag}` — read by `details()` for §5.5 JSON output.
- `error.rs:27` `BundleMismatch` variant — reserved for runtime emission in v0.2.
- `error.rs:161` `kind()` — §5.5 JSON envelope.
- `error.rs:209` `details()` — §5.5 JSON envelope.
- `error.rs:292` `pub type Result<T>` — convenience alias (comment incorrectly says "in-crate"; see IMP-2).
- `format.rs:32` `chunk_mk1` — reserved alias for mk-codec grouping helper.

## C. Design ambiguities affecting v0.2

### C.1 IMP-1: `bundle-mismatch-card-static-str-constraint`

**File:** `crates/mnemonic-toolkit/src/error.rs:28-31`

**Current shape:**

```rust
BundleMismatch {
    card: &'static str,
    message: String,
},
```

v0.2 wires `BundleMismatch` as a live runtime error for the first time. Multisig bundles emit per-cosigner identifiers like `"mk1[0]"`, `"mk1[1]"` that cannot be `&'static str`. Changing this AFTER v0.2 SPEC drafts §6.6 multisig BundleMismatch rows forces a mid-cycle breaking field-type change.

**Remediation:** change `card: &'static str` → `card: String`. Update test construction sites at `exit_code_table_per_variant` and `kind_strings_stable` to pass `"mk1".into()`. The `details()` arm (`json!({ "card": card })`) is fine either way. One-commit change; `BundleMismatch` is `#[non_exhaustive]` so this is not a public-API break.

### C.2 IMP-2: `error-allow-comments-staleness`

**File:** `crates/mnemonic-toolkit/src/error.rs:27-28` and `292`

**Comment A (`BundleMismatch`):** `"Constructed by integration tests in Phase 5; reserved for runtime emission once verify-bundle's optional-mismatch reporter wires up."` Becomes inaccurate once v0.2 wires the variant as a live error path. Updating in the same commit as IMP-1 keeps the doc comment in sync with the runtime behavior.

**Comment B (`Result<T>`):** `"reserved for in-crate use"` is inaccurate — the type is `pub`, available to external callers.

**Remediation:** Replace both comments with accurate descriptions:
- `BundleMismatch`: `Exit-4 verify-bundle mismatch variant; card identifies the mismatching card (e.g., "mk1", "md1", or "mk1[N]" for multisig cosigner N).`
- `Result<T>`: `Convenience alias; exported for downstream-crate use.`

### C.3 `Bundle.mk1: Vec<String>` reshape (v0.2 SPEC question, not Phase 0)

Multisig bundles emit per-cosigner mk1 strings. v0.2 SPEC must pick one of:
- A: `mk1: Vec<Vec<String>>` (outer per-cosigner, inner per-chunk) — backwards-incompatible JSON shape.
- B: `mk1: Vec<String>` flat with chunk-set-id boundaries — fragile.
- C: `mk1: Vec<MkCard>` with `MkCard { cosigner_idx, chunks }` — most explicit.

This is a brainstorm Q for v0.2 (Q9 / Q12 in the meta-plan), not a Phase 0 fixup.

### C.4 `template::wrapper_node()` return type

Returns `Node` (single root). Multisig descriptors fit because `Node { tag: Tag::Wsh, body: Body::Children(vec![Node{tag:Tag::SortedMulti, body:Body::Variable{k, children}}]) }` is still a single-root tree. **No return-type change needed for v0.2.**

### C.5 `PathDeclPaths::Divergent` for v0.2 `--account`

`build_descriptor` always emits `PathDeclPaths::Shared` in v0.1. v0.2 multi-cosigner with per-cosigner accounts will emit `PathDeclPaths::Divergent(paths)`. md-codec API is ready (variant exists, encoder/decoder both handle it).

## D. Cross-repo breakage risks

All v0.2 sibling-API consumers are present and shaped correctly at the pinned tags:

| API | Crate / version | Status |
|---|---|---|
| `md_codec::Tag::{Multi, SortedMulti, MultiA, SortedMultiA}` | md-codec v0.16.1 | ✓ tags 0x06–0x09 in primary 5-bit space |
| `md_codec::Body::Variable { k, children }` | md-codec v0.16.1 | ✓ in `tree.rs`, exercised by `read_node` |
| `md_codec::PathDeclPaths::Divergent(Vec<OriginPath>)` | md-codec v0.16.1 | ✓ in `origin_path.rs`, encoder + decoder support |
| `md_codec::Tag::Wsh` (multisig wrapper) | md-codec v0.16.1 | ✓ tag 0x02 |
| `mk_codec::KeyCard::new(stubs, None, ...)` | mk-codec v0.2.1 | ✓ accepts `Option<Fingerprint> = None`; bytecode-header bit 2 cleared |
| `mk_codec::encode_with_chunk_set_id` (deterministic CSI) | mk-codec v0.2.1 | ✓ used at v0.1; no version drift |

**No cross-repo blockers for any of the 5 v0.2 features.**

## E. FOLLOWUPS sweep — 14 items (12 open, 2 resolved)

Disposition for v0.2:

| Short-id | Tier | v0.2 disposition | Rationale |
|---|---|---|---|
| `spec-5-5-kind-enum-gap` | v0.1-nice-to-have | Defer past v0.2 | SPEC-prose-only; no code impact. |
| `mk-codec-chunked-visual-grouping-helper` | cross-repo | Defer past v0.2 | Depends on mk-codec exposing render helper; no toolkit blocker. |
| `plan-spike-md-codec-filler-bug` | v0.1-nice-to-have | Defer past v0.2 | Plan source stale; no code impact. |
| `plan-trezor-24-fingerprint-stale` | v0.1-nice-to-have | Defer past v0.2 | Plan source stale; no code impact. |
| `friendly-mk-codec-mixedcase-wording` | v0.1-nice-to-have | Defer past v0.2 | Cosmetic wording diff; not byte-exact-pinned. |
| `bundle-emit-bypasses-chunk-mk1-alias` | v0.1-nice-to-have | **Resolve in v0.2** | v0.2 reshape of mk1 output is ideal time to fix the call site. |
| `watch-only-stderr-warning-suborder` | v0.1-nice-to-have | Defer past v0.2 | SPEC-ambiguous; not pinned by fixtures. |
| `spec-2-2-2-vs-5-4-checks-count-prose` | v0.1-nice-to-have | Defer past v0.2 | SPEC-internal inconsistency; behavior correct. |
| `bundle-mismatch-card-static-str-constraint` | v0.2 | **Phase 0 mandatory fixup (IMP-1)** | See §C.1 above. |
| `verify-bundle-text-mode-trailing-space` | v0.1-nice-to-have | Defer past v0.2 | Cosmetic; text mode unpinned. |
| `error-allow-comments-staleness` | v0.1-nice-to-have | **Phase 0 mandatory fixup (IMP-2)** | Bundled with IMP-1; doc-comment accuracy. |
| `cli-watch-only-test-hardcodes-fingerprint` | v0.1-nice-to-have | Defer past v0.2 | Two-place edit risk only on test-vector swap. |
| `changelog-sha-pin-no-reproduction-command` | v0.1-nice-to-have | Defer past v0.2 | Doc clarity gap; verifiers can re-derive. |
| `cli-mode-violations-byte-exact-naming` | v0.1-nice-to-have | Defer past v0.2 | Test-naming cosmetic. |

## F. Cargo.toml + tooling readiness

- Workspace has 1 member (`crates/mnemonic-toolkit`); resolver = "2"; current.
- Sibling deps pinned to git tags matching SPEC §10.3 exactly: `ms-codec @ ms-codec-v0.1.0`, `mk-codec @ mk-codec-v0.2.1`, `md-codec @ md-codec-v0.16.1`. **No version drift.**
- v0.2 adds **no new crate dependencies** for any of the 5 locked features. All needed APIs exist at the pinned sibling versions.
- **LOW-4 `hex-dep-unused`**: `hex = "0.4"` declared in `[dependencies]` but no `use hex` in any non-test source. Inert. Could be removed but no urgency (and the user's `feedback_dont_drop_reserved_deps` rule applies — defer to user confirmation).

## Low findings (deferred)

| Short-id | Where | Disposition |
|---|---|---|
| LOW-1 `dead-assert-tautological` | `synthesize.rs:99` `debug_assert_eq!(&card.policy_id_stubs[0], &stub)` always true by construction | v0.2 multisig should re-design the assertion to loop over all stubs |
| LOW-2 `dead-inner-guard-bundle-watch-only` | `cmd/bundle.rs:200` redundant `--xpub` guard | v0.2 SPEC review of mode-dispatch; non-blocking |
| LOW-3 `friendly-mapper-unit-test-gaps` | `friendly.rs::tests` — 3 of ~70 arms covered | v0.2 Phase E adds parametric variant test |
| LOW-4 `hex-dep-unused` | `Cargo.toml:27` | User confirmation needed before removal (`feedback_dont_drop_reserved_deps`) |

## Phase 0 actions before v0.2 brainstorm

1. **Apply IMP-1 + IMP-2 in a single fixup commit on `mnemonic-toolkit` master.** File touched: `crates/mnemonic-toolkit/src/error.rs`. Verify `cargo test -p mnemonic-toolkit && cargo clippy -p mnemonic-toolkit --all-targets -- -D warnings && cargo fmt --check -p mnemonic-toolkit` all clean.
2. **Update `design/FOLLOWUPS.md`:** mark `bundle-mismatch-card-static-str-constraint` and `error-allow-comments-staleness` as resolved with the fixup commit SHA. Add `bundle-emit-bypasses-chunk-mk1-alias` annotation noting v0.2-resolve disposition. Add LOW-1..LOW-4 as new entries at v0.2-nice-to-have or external tier.
3. **Cleared to proceed to Phase 1 brainstorm.**
