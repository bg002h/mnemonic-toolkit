# BRAINSTORM — cycle-7 = M8 (build-descriptor extra-derivation-suffix → silent wrong subtree) + L23 (ecies zero-scalar panic)

**Status:** DESIGN ONLY — no code. Feeds the mandatory opus-architect **R0 loop to 0C/0I** before any implementation.
**Workstream:** `WS-DESCBUILD` (`design/PLAN_constellation_bughunt_fix_program.md`).
**Findings:** **M8** (`w3-tk-descbuild-key-extra-path-suffix-silent`) — GENUINE funds-loss-on-legitimate-input; **L23** (`w3-tk-electrum-crypto-01`) — latent robustness, co-located fold-in.
**Scope:** toolkit-only. No sibling-codec surface. No registry publish.

---

## 1. Source-SHA table (re-grep-verified)

**Recon SHA:** `8d2fe505` (toolkit 0.64.0).
**Current `origin/master` at spec-author time:** `79e3387c` (toolkit 0.64.0).
**Delta `8d2fe505..79e3387c`:** ONE commit — `79e3387c design(cycle6): timelock decay-ordering fix trail + bughunt D-decay marks` (design-doc-only).
**Cycle-7 zone diff `8d2fe505..79e3387c`:** `git diff --stat 8d2fe505 79e3387c -- crates/mnemonic-toolkit/src/descriptor_builder/ crates/mnemonic-toolkit/src/electrum_crypto.rs` = **EMPTY**. All recon citations are byte-stable; lines below re-verified against `79e3387c`.

| # | Symbol / claim | File:line (`79e3387c`) | Verified |
|---|---|---|---|
| C1 | `MULTIPATH_SUFFIX = "/<0;1>/*"` const + account-level-input doc | `descriptor_builder/ir.rs:21-23` | ✅ exact |
| C2 | `with_multipath(key) = format!("{key}{MULTIPATH_SUFFIX}")` (blind literal concat) | `descriptor_builder/ir.rs:218-220` | ✅ exact |
| C3 | `render_keys` maps every key through `with_multipath` | `descriptor_builder/ir.rs:222-228` | ✅ exact |
| C4 | `check_secret_key` — strips `[origin]` via `rsplit(']')`, screens xprv-prefix ONLY | `descriptor_builder/gate.rs:347-361` | ✅ (recon `:347`; body `:347-361`) |
| C5 | `validate_fields` calls `check_secret_key` for Pk/Pkh (`:235`) AND each Multi/Sortedmulti key (`:240`) | `descriptor_builder/gate.rs:233-252` | ✅ exact |
| C6 | `validate_with_allow` step-1 runs `validate_fields(&doc.root, ...)` for ALL intake | `descriptor_builder/gate.rs:163-164` | ✅ exact |
| C7 | `DiagnosticKind` enum — gate-step-ordered (NOT alphabetical); `SecretKey` is a step-1 kind | `descriptor_builder/gate.rs:90-119` | ✅ exact |
| C8 | `DiagnosticKind::as_str` — the stable `--json` discriminant | `descriptor_builder/gate.rs:122-135` | ✅ exact |
| C9 | `field_diag(path, msg)` → `DiagnosticKind::SchemaField` (generic step-1 field-error ctor) | `descriptor_builder/gate.rs:679-686` | ✅ exact |
| C10 | Preset (`--key`) intake → `SpecDoc{root: archetype-lowered tree}` → `gate::validate_with_allow` | `cmd/build_descriptor.rs:282-287` | ✅ exact |
| C11 | `--key` doc: "lowered tree flows through the SAME validation gate as `--spec`" | `cmd/build_descriptor.rs:57` | ✅ exact |
| C12 | Spec (`--spec` JSON) intake → `SpecDoc::parse` → `gate::validate_with_allow` | `cmd/build_descriptor.rs:322-326` | ✅ exact |
| C13 | gate failure → `emit_diagnostics(...)` → **exit 2** (doc `:6`, `:46`) | `cmd/build_descriptor.rs:279,300,329,364-371` | ✅ exact |
| C14 | L23: `Scalar::from_be_bytes(*privkey).map_err(|_| InvalidScalar)?` accepts zero | `electrum_crypto.rs:345` | ✅ exact |
| C15 | L23: `.mul_tweak(&secp, &scalar).expect("…never the identity")` panics on zero tweak | `electrum_crypto.rs:349-351` | ✅ (recon `:350-351`) |
| C16 | `EciesDecryptError::InvalidScalar` variant ALREADY exists (no new variant needed) | `electrum_crypto.rs:247` | ✅ exact |
| C17 | Safe sole caller `derive_storage_eckey` rejects zero scalar | `electrum_crypto.rs:309-310` | ✅ exact |
| C18 | `ecies_decrypt_message` is `pub fn` (library-reachable) | `electrum_crypto.rs:321` | ✅ exact |
| C19 | toolkit version `0.64.0` | `crates/mnemonic-toolkit/Cargo.toml:3` | ✅ exact |

**Load-bearing external-protocol fact (recon, verified against PRIMARY source — pinned miniscript `parse_xkey_deriv`):** a FIXED derivation index before a `<a;b>` BIP-389 multipath token parses cleanly as a valid `MultiXPub`; a wildcard `*` before any further segment is rejected with `InvalidWildcardInDerivationPath`. This asymmetry IS the M8 bug. (Recon §"Per-finding verification" item 3-4; pinned parser `…/95fdd1c/src/descriptor/key.rs:1146+,1213-1221`, err variant `:471/:490`.) Re-verification of the parser source is NOT re-done here — the recon already verified it against authoritative source text per the project's external-protocol-fact rule; the spec builds on it.

---

## 2. Finding summary (both REPRODUCE)

### M8 — `build-descriptor` accepts a key carrying an extra derivation suffix → silently derives a deeper (wrong) subtree
**Class A (wrong-address), toolkit, MED → genuine funds-loss-on-legitimate-input.**

The renderer (`ir.rs::with_multipath`, C2) **owns** the BIP-388 `/<0;1>/*` receive/change/index suffix and unconditionally literal-concats it onto every key (C3). The documented contract (C1) is that inputs are bare account-level `[fp/path]xpub` strings — but **nothing enforces it**. The only step-1 key screen, `check_secret_key` (C4), checks ONLY for an extended-PRIVATE-key prefix; it does NOT reject a key whose xpub body carries its own trailing `/`-segments.

So a key like `[fp/48h/0h/0h/2h]xpub.../5` is taken verbatim through both intake paths (C10/C11 preset; C12 spec), rendered `…xpub.../5/<0;1>/*` (C2), and **accepted by `Descriptor::from_str`** at gate step 2 — because `5` before `<0;1>` is a legal fixed index (load-bearing fact above). The descriptor type-checks, passes `sanity_check`, and emits — but derives the `…/5` subtree, NOT the account-level key the user intended → a DIFFERENT (wrong) address set. The trailing-`*` case (`…xpub.../*` → `…/*/<0;1>/*`) IS caught (`InvalidWildcardInDerivationPath`); the fixed-index-prefix case is NOT. **That asymmetry is the bug.** No descriptor-builder code path performs a trailing-segment check (recon grep-verified across `ir/gate/archetype/mod/schema.rs`).

**REPRODUCE verdict: YES** — confirmed structurally end-to-end against the pinned parser (recon §M8 items 1-5).

### L23 — `ecies_decrypt_message` panics (not `InvalidScalar`) on a zero private scalar
**Class E (panic-DoS), toolkit, LOW — latent (NOT CLI-reachable).**

`Scalar::from_be_bytes` only range-checks `< n`, so a zero scalar passes the `InvalidScalar` guard at C14. Then `mul_tweak` returns `Err(InvalidTweak)` for the zero tweak and hits `.expect(...)` (C15) → **panic** instead of a typed `EciesDecryptError::InvalidScalar`. The sole in-tree caller `derive_storage_eckey` rejects zero (C17), so the panic is unreachable from the CLI today — a robustness item for a future / downstream-library caller of the `pub fn` (C18). `InvalidScalar` already exists (C16), so the fix needs no new variant.

**REPRODUCE verdict: YES** (latent) — confirmed (recon §L23).

---

## 3. Per-finding fix design

### 3.1 M8 — fail-closed reject of a post-origin-strip xpub body carrying extra `/`-segments

**Decision (funds-safety, fail-closed): REJECT.** Honoring `…/5` would let an undocumented in-key path silently override the account-level contract the renderer owns. The only funds-safe behavior is to refuse and tell the user to supply a bare account-level `[origin]xpub`. (Recon §M8 "Decision"; program's "emitters/intake must refuse rather than silently transform" discipline.)

**Fix-site: `descriptor_builder/gate.rs::check_secret_key` (C4), extended in place.** It already strips the `[origin]` prefix (`let key_part = key.rsplit(']').next().unwrap_or(key)`), runs at step 1 for EVERY key node (Pk/Pkh + each Multi/Sortedmulti key, C5), and is reached by BOTH intake paths through the single `validate_fields` recursion (C6) that BOTH `--key` preset (C10/C11) and `--spec` JSON (C12) funnel into. **One guard here covers every intake path.** (If R0 prefers separation-of-concerns, an equivalent is a sibling `check_key_path_suffix` helper called from the same two `validate_fields` arms — same coverage, same step; the in-place extension is the minimal diff and is the recommended form.)

**Guard predicate (decision-complete):** after the existing `[origin]`-strip, the xpub body must NOT contain a `/`. Concretely — reject when `key_part.contains('/')`. Rationale:
- After `rsplit(']')`, `key_part` is the post-bracket remainder: for a well-formed account-level input it is exactly the bare `xpub…`/`tpub…`/SLIP-132 body with **no** `/`. A descriptor `[origin]` (the only legitimate pre-key `/`-bearing token) lives INSIDE the brackets and is stripped by `rsplit(']')`, so a legitimate account-level key's `key_part` has zero `/`.
- ANY `/` in `key_part` means the key carries its own trailing derivation path (`…/5`, `…/5/6`, `…/<0;1>`, `…/*`, `…/0h`) — exactly the over-derivation M8 describes. Reject all of them: the renderer owns the entire post-account suffix, so a key bringing ANY derivation tail is ambiguous and must fail closed.
- **Ordering vs the xprv screen (C4):** keep the existing xprv check; add the suffix check as a sibling step-1 condition in the SAME `check_secret_key` (both are "the key string is structurally wrong for a watch-only account-level cosigner"). If a key is BOTH an xprv AND suffixed, emitting the xprv diagnostic first is fine (xprv is the higher-severity secret-leak refusal); R0 may pick either order — both are step-1, both refuse, exit 2. **Recommended:** emit the xprv diagnostic when `is_xprv`, ELSE the suffix diagnostic — so a single key never double-reports.
- **Edge — bracketless key with no `]`:** `rsplit(']').next().unwrap_or(key)` returns the whole key; a bare `xpub.../5` (no `[origin]`) still has `key_part = "xpub.../5"` which `.contains('/')` → rejected. Covered.
- **Edge — empty `key_part`** (malformed `…]` with nothing after): `.contains('/')` is false → not rejected here; it falls through to step-2 `from_str`, which refuses it (no regression — this is not the M8 class).

**Diagnostic-kind decision: REUSE `DiagnosticKind::SchemaField` (NO new kind, ZERO `--json` wire-shape delta).**
- `field_diag(path, message)` (C9) is the EXISTING generic step-1 field-error constructor that emits `DiagnosticKind::SchemaField` — already used in `validate_fields` for threshold/empty-key errors (C5 neighborhood). The M8 suffix-reject is a step-1 field-shape error of exactly that family, so emitting it via `field_diag(...)` adds **no new `as_str` discriminant string** (C8 unchanged) → **no `--json` wire-shape change** → no GUI self-update burden, nothing for the paired-PR rule to track. This matches the prompt's stated preference (reuse if a fitting kind exists) and cycle-6's `DiagnosticParam`-reuse precedent.
- **Rejected alternative — a NEW `DiagnosticKind::KeyPathSuffix`** (the recon's tentative suggestion). It would be semantically tidy but is a `--json` wire-shape delta (new discriminant string) that GUI consumers must self-update (NOT schema_mirror-gated — a lagging, ungated surface). The `SecretKey` kind exists as its own discriminant because the xprv refusal is a distinct secret-LEAK class with bespoke downstream handling; the suffix-reject has no such distinct-handling need — a node-addressed `SchemaField` message is sufficient and strictly cheaper. **Decision: reuse `SchemaField`.** (If R0 judges the suffix-reject a distinct-enough funds-safety class to warrant its own discriminant à la `SecretKey`, the new-kind path is available — append after `SecretKey` per the gate-step ordering, C7, NOT alphabetically — but the spec's recommendation, for minimal churn, is reuse.)

**Error message (decision-complete, NEVER echoes the key — matches the C4 no-leak discipline):**
> `"{kind} key carries an extra derivation path; build-descriptor accepts only a bare account-level key ([fp/path]xpub…) — the builder appends the /<0;1>/* receive/change suffix itself"`

Node-addressed via the `path` arg (`root`, or `root.multi.keys[i]`), exactly like the existing field diagnostics. No `flag` (matches `field_diag`).

**Funds-safety property:** an over-derived / over-long key FAILS CLOSED (exit 2, no descriptor emitted), never silently re-targets the subtree.

### 3.2 L23 — zero-scalar reject (typed error, not panic)

**Fix-site: `electrum_crypto.rs::ecies_decrypt_message`, at the scalar check (C14).** Add an explicit zero-scalar reject BEFORE `mul_tweak` (C15):
> `if privkey.iter().all(|&b| b == 0) { return Err(EciesDecryptError::InvalidScalar); }`

placed immediately before (or folding into) line `:345`. This mirrors the existing `derive_storage_eckey` zero-guard (C17), reuses the existing `InvalidScalar` variant (C16) — **no new error variant, no wire/CLI change** — and makes the `.expect(...)` (C15) provably unreachable for the zero case (its comment becomes accurate: the scalar is now guaranteed `[1, n-1]`). The `.expect` stays as the defensive invariant-assertion for the prime-order-group fact (a non-identity-result invariant a future change should still uphold).

**Alternative (rejected — less explicit):** `map_err` the `mul_tweak` result to `InvalidScalar` instead of `.expect`. The explicit pre-reject (chosen) localizes the failure at the input boundary and keeps the `.expect` as a true invariant guard; it is the cleaner, recon-preferred form (recon §L23 option (a)).

**Firewalling:** L23 is a separate sub-item — a different file, a different test, no shared code with M8. The branch touches `descriptor_builder/{ir,gate}.rs` (M8) + `electrum_crypto.rs` (L23) — both in WS-DESCBUILD per the program (`PLAN…:429-430`).

---

## 4. SemVer / lockstep / oracle

- **SemVer: toolkit MINOR → 0.65.0.** M8 newly REJECTS previously-accepted input (fail-closed behavior-tightening) — counted FORMAL, MINOR pre-1.0 (program classification clause 1). L23 is internal robustness (no public-surface change) and rides the same bump. **No registry publish** (toolkit is not on crates.io). Renumber if another in-flight cycle lands first (e.g. own-account → 0.65.x).
- **`schema_mirror` (GUI flag-NAME gate): NOT triggered.** No clap flag / subcommand / dropdown-value add/remove/rename — the intake surface (`--key`, `--spec`, `--archetype`, …) is unchanged; only validation tightens. (CLAUDE.md "GUI schema-mirror coverage": the gate is clap flag-NAMES + dropdown enums only.)
- **`--json` wire-shape: UNCHANGED** under the chosen reuse-of-`SchemaField` decision (§3.1) — no new `DiagnosticKind::as_str` discriminant, so no GUI `--json` self-update / paired-PR item. (Had a new kind been chosen, it would be a manual GUI self-update — NOT schema_mirror-gated — per CLAUDE.md "Scope of the gate". The reuse decision eliminates this entirely.)
- **Manual mirror (`docs/manual/src/40-cli-reference/41-mnemonic.md`): NOT gate-triggered** (no `--help`/flag-set change; the flag-coverage lint `docs/manual/tests/lint.sh` checks flag presence, unaffected). A one-line behavioral note ("`build-descriptor` accepts only bare account-level `[fp/path]xpub`; the builder owns the `/<0;1>/*` suffix") is good-practice and MAY be added in the same PR, but is not required by any gate.
- **Oracle (`tests/bitcoind_differential.rs`): N/A — do NOT block.** The Class-A oracle is a `bundle → restore` round-trip keyed on a given `descriptor` string; it does **NOT invoke `build-descriptor`**, so it never exercises M8's intake. The M8 fix is a REFUSAL (no descriptor emitted), so the oracle would not exercise the fixed path either; the funds-safety property ("refuse the ambiguous suffix") is pinned directly by the unit/CLI refusal tests (§5). The program's future "NEW build-descriptor oracle row" (`PLAN…:350-351`) is an optional adjacent enhancement, not a cycle-7 gate.
- **Sibling-codec companion: NONE.** Both findings are pure toolkit; no md/mk/ms surface touched. No `design/FOLLOWUPS.md` companion-mirror needed.

---

## 5. Tests (TDD, RED-first)

All M8 CLI tests live in `crates/mnemonic-toolkit/tests/cli_build_descriptor.rs`; gate-unit tests in the `gate.rs` `#[cfg(test)]` module; L23 in the `electrum_crypto.rs` `#[cfg(test)]` module. The recon confirmed a clean baseline — no existing test passes a key with a trailing index suffix (all fixtures are origin-only / bare xpubs).

### M8 — the core funds-safety pins (RED today → GREEN after the guard)
| # | Test | Today | After fix |
|---|---|---|---|
| T1 | `--key '[fp/84h/0h/0h]xpub.../5'` via a single-sig **preset** (`--archetype`) → **REJECTED, exit 2**, diagnostic `kind=schema_field`, message names the extra-derivation-path, key NOT echoed | ACCEPTED, emits `…/5/<0;1>/*` (wrong subtree) | **exit 2, refused** |
| T2 | A `--spec` JSON `PolicyNode::Pk` whose key is `xpub.../5` (and a `Multi.keys[i]` variant) → **REJECTED, exit 2**, same `schema_field` diagnostic | ACCEPTED (wrong subtree) | **exit 2, refused** |
| T3 (positive control) | A NORMAL account-level key `[fp/84h/0h/0h]xpub…` (bare body, no trailing derivation) via `--key` AND via `--spec` → **STILL BUILDS** (exit 0, descriptor emitted) | builds | builds (no over-rejection) |
| T4 (asymmetry pin) | A key already ending in `*` (`xpub.../*`) → **STILL rejected** (now by the new step-1 suffix guard rather than the step-2 `InvalidWildcardInDerivationPath`; exit 2 either way) | rejected at step 2 | rejected at step 1 (assert exit 2; the test pins the refusal, tolerant of which step) |
| T5 (multi-segment) | `xpub.../5/6` and `xpub.../0h` → **REJECTED, exit 2** (any trailing path tail) | ACCEPTED (wrong subtree) | **exit 2, refused** |
| T6 (no-leak) | The M8 refusal message does NOT contain the xpub body (assert the secret/key bytes are absent from stdout+stderr) | n/a | message names the path-issue only |

Both T1 (preset) and T2 (spec) are mandatory — they pin that the SINGLE guard covers BOTH intake paths (C6/C10/C12).

### L23 — direct unit test (the `pub fn` is the test surface)
| # | Test | Today | After fix |
|---|---|---|---|
| T7 | `ecies_decrypt_message(<valid BIE1 blob>, &[0u8; 32])` → **`Err(EciesDecryptError::InvalidScalar)`**, NOT a panic | panics at `.expect` (C15) | typed `Err(InvalidScalar)` |
| T8 (regression) | the existing Electrum `test_decrypt_message` vector still passes (the zero-guard does not perturb the valid path) | passes | passes |

(L23 needs a blob that reaches the scalar step — a syntactically valid BIE1 envelope ≥85 bytes with a parseable ephemeral pubkey — so `ecies_decrypt_message` gets past the length/magic/pubkey gates to the `mul_tweak`; the existing test vector's blob can be reused as the input, varying only the `privkey` arg to all-zero.)

---

## 6. FOLLOWUP slugs

| slug | repo | status after cycle-7 |
|---|---|---|
| `w3-tk-descbuild-key-extra-path-suffix-silent` (M8) | toolkit | **FIXED** in 0.65.0 — flip status in the shipping commit |
| `w3-tk-electrum-crypto-01` (L23) | toolkit | **FIXED** in 0.65.0 — flip status in the shipping commit |
| `tk-build-descriptor-oracle-row` (NEW, optional) | toolkit | OPEN — adjacent enhancement: a `bundle…`-independent `build-descriptor` human+canonical oracle row (program `PLAN…:350-351`) to cover the M8 *accept* (bare key → correct address) path empirically. NOT a cycle-7 gate. |

WS-DECAY adjacency (`descriptor_builder/archetype.rs:305-317`, BIP-68-unit-normalization) shares the `descriptor_builder/` file zone but is a SEPARATE finding — NOT in cycle-7. Note for the implementer: if WS-DECAY is scheduled, serialize the two branches or flag the file-zone overlap to avoid a merge conflict (recon obs #5; `PLAN…:431-433`).

---

## 7. Resolved decisions (NO open questions)

| # | Decision | Resolution | Why |
|---|---|---|---|
| D1 | M8 honor-`/5` vs reject | **REJECT (fail-closed)** | renderer owns `/<0;1>/*`; honoring an in-key path silently overrides the account-level contract → wrong subtree. Only funds-safe option. |
| D2 | M8 fix-site | **`gate.rs::check_secret_key` (extend in place)** | already strips `[origin]`, runs step-1 for every key node, reached by BOTH intake paths via the single `validate_fields` recursion. One guard covers everything. |
| D3 | M8 guard predicate | **reject when post-`[origin]`-strip `key_part.contains('/')`** | a legitimate account-level key's post-bracket body has zero `/`; ANY `/` ⇒ a trailing derivation tail (the M8 class). Catches `/5`, `/5/6`, `/0h`, `/<0;1>`, `/*`. |
| D4 | M8 diagnostic kind | **REUSE `DiagnosticKind::SchemaField` via `field_diag`** | zero new `as_str` discriminant ⇒ zero `--json` wire-shape delta ⇒ no GUI self-update / paired-PR burden. Matches cycle-6 reuse precedent; the suffix-reject is a step-1 field-shape error of that family. |
| D5 | new diagnostic kind? | **NO** | a new `KeyPathSuffix` would be an ungated `--json` wire delta for no distinct-handling benefit. (Available as a fallback if R0 wants a distinct funds-safety discriminant — append after `SecretKey` per gate-step order, NOT alphabetically.) |
| D6 | M8 message leaks the key? | **NO** — message names the path-issue only, never echoes the key | matches the existing C4 no-leak discipline; pinned by T6. |
| D7 | xprv + suffix collision ordering | **emit xprv diagnostic if `is_xprv`, ELSE suffix** | a single key never double-reports; xprv (secret-leak) takes precedence. Both step-1, both exit 2. |
| D8 | L23 fix | **explicit zero-scalar reject before `mul_tweak`** → `Err(InvalidScalar)` | mirrors `derive_storage_eckey`'s guard; reuses the existing variant (no new variant); makes the `.expect` invariant provably hold. |
| D9 | L23 new error variant? | **NO** — `EciesDecryptError::InvalidScalar` already exists (C16) | no wire/CLI change. |
| D10 | SemVer | **MINOR → 0.65.0**, no registry publish | M8 newly rejects previously-accepted input (FORMAL, MINOR pre-1.0); L23 rides the bump. |
| D11 | `schema_mirror` lockstep | **NOT triggered** | no clap flag/subcommand/dropdown change. |
| D12 | `--json` wire-shape / GUI self-update | **NOT triggered** (under D4 reuse) | no new `DiagnosticKind` discriminant. |
| D13 | Manual mirror | **NOT gate-triggered**; optional one-line behavioral note | no `--help`/flag-set change. |
| D14 | `bitcoind_differential` oracle | **N/A — do NOT block** | oracle never invokes `build-descriptor`; M8 fix is a refusal (no descriptor). Refusal pinned by unit/CLI tests. |
| D15 | sibling-codec companion | **NONE** | both findings pure toolkit. |
| D16 | over-rejection risk | **controlled by T3** (normal account-level key still builds, both paths) | `key_part.contains('/')` is true ONLY for keys with a trailing derivation tail; a bare `[origin]xpub` has none. |

---

## 8. Mandatory R0 gate

This is a brainstorm spec — **DESIGN ONLY, no code**. Per CLAUDE.md Conventions (first bullet): before ANY implementation (writing code or dispatching an implementer subagent), this spec MUST pass an opus-architect **R0 review and converge to 0 Critical / 0 Important** — fold findings → persist the review verbatim to `design/agent-reports/` → re-dispatch → repeat until GREEN. The reviewer-loop continues after every fold (folds can introduce drift). No implementation, phase advance, tag, or ship while any Critical or Important finding is open.
