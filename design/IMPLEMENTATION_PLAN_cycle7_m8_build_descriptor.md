# IMPLEMENTATION PLAN — cycle-7 = M8 (build-descriptor extra-derivation-suffix → silent wrong subtree) + L23 (ecies zero-scalar panic)

**Status:** DESIGN ONLY — this is the implementation plan-doc. It MUST pass the mandatory opus-architect **R0 loop to 0C/0I** before ANY code is written. No implementation, phase advance, tag, or ship while a Critical or Important finding is open.
**Spec (R0-GREEN 0C/0I):** `design/BRAINSTORM_cycle7_m8_build_descriptor.md` — phased faithfully here. Resolved decisions D1–D16 are NOT re-decided; this plan references them.
**Spec R0 review (folded below):** `design/agent-reports/cycle7-spec-r0-round1-review.md` — both Minors incorporated (Minor-1 → Phase 1 test T1b; Minor-2 → Phase 1 impl note on the `path` arg threading).
**Workstream:** `WS-DESCBUILD` (`design/PLAN_constellation_bughunt_fix_program.md`).

---

## 0. Source SHA + execution model

- **Source SHA (plan basis, prompt-pinned):** `8d2fe505` (toolkit 0.64.0). Current `origin/master` is `79e3387c` (also 0.64.0) — ONE design-doc-only commit ahead; the cycle-7 file-zone (`descriptor_builder/`, `electrum_crypto.rs`) diff `8d2fe505..79e3387c` is **empty**. All citations below are re-grep-verified against `8d2fe505` at plan-author time and resolve exactly.
- **Worktree off `origin/master`.** Single implementer in a fresh worktree branched off `origin/master`. NOT parallel re-implementations.
- **Per-phase gate (run the WHOLE package, NOT `--lib` — the M8 CLI tests live in the BIN target's `tests/` dir):**
  - `cargo test -p mnemonic-toolkit` (full package suite: bin unit tests + `tests/cli_build_descriptor.rs` + all integration tests).
  - `cargo clippy --all-targets -- -D warnings`.
  - **NO `cargo fmt`** (mlock g6 fmt-exemption discipline; never `cargo fmt` this repo).
- **TDD, RED-first per phase:** write the failing test(s) first, confirm RED for the right reason, then the minimal impl to GREEN. Each phase ends GREEN on the full per-phase gate before the next phase starts.
- **Pinned miniscript:** 13.0.0 @ `95fdd1c` (unchanged). The M8 protocol-fact (fixed index before `<a;b>` parses; wildcard-before-segment rejected) was primary-source-verified in recon; not re-verified here per the external-protocol-fact rule.

### Live-line citation table (re-verified against `8d2fe505`)

| # | Symbol / claim | File:line (`8d2fe505`) |
|---|---|---|
| L1 | `check_secret_key(key, path, kind, out)` — fn def | `descriptor_builder/gate.rs:347` |
| L2 | `let key_part = key.rsplit(']').next().unwrap_or(key);` | `gate.rs:348` |
| L3 | `is_xprv` prefix screen → emits inline `Diagnostic{ kind: SecretKey, … }` | `gate.rs:349-359` (`kind: SecretKey` at `:353`) |
| L4 | `validate_fields` Pk/Pkh → `check_secret_key(k, path, …)` | `gate.rs:235` |
| L5 | `validate_fields` Multi/Sortedmulti → per-key `check_secret_key(key, &format!("{path}.{}.keys[{i}]", node.kind()), …)` | `gate.rs:240-244` (path string `:242`) |
| L6 | `validate_with_allow` step-1 runs `validate_fields(&doc.root, …)` (sole pre-emission gate) | `gate.rs:163-164` |
| L7 | `child_paths` recurses AndV/OrD/OrI/OrB/Andor/Thresh/Wrap (nested keys reach `check_secret_key`) | `gate.rs:646` |
| L8 | `DiagnosticKind::as_str` — `SchemaField => "schema_field"` (`:124`), `SecretKey => "secret_key"` (`:132`) | `gate.rs:121-135` |
| L9 | `field_diag(path, message)` → `Diagnostic{ kind: SchemaField, flag: None }` | `gate.rs:679-686` |
| L10 | Preset (`--key`) intake → `SpecDoc{ root }` → `validate_with_allow`; flag-provenance via `archetype::resolve_flag(def, &d.node_path, d.kind)` | `cmd/build_descriptor.rs:282-298` (resolve_flag `:298`) |
| L11 | Spec (`--spec`) intake → `SpecDoc::parse` → `validate_with_allow` | `cmd/build_descriptor.rs:323-326` |
| L12 | gate fail → `emit_diagnostics(…)` → `return Ok(2)` (exit 2) | `cmd/build_descriptor.rs:279-280,300-301,329-330` |
| L13 | `resolve_flag(def, node_path, kind)` — kind-aware longest-prefix; `KEY="--key"` (`:91`), `THRESHOLD="--threshold"` (`:92`) | `descriptor_builder/archetype.rs:395`; consts `:91-92`; `(SecretKey→--key)` test `:757-758` |
| L14 | `Scalar::from_be_bytes(*privkey).map_err(|_| InvalidScalar)?` accepts zero | `electrum_crypto.rs:345` |
| L15 | `.mul_tweak(&secp, &scalar).expect("…never the identity")` panics on zero tweak | `electrum_crypto.rs:350-351` |
| L16 | `EciesDecryptError::InvalidScalar` variant ALREADY exists (Display arm `:278`) | `electrum_crypto.rs:247` |
| L17 | Safe sole caller `derive_storage_eckey` zero-guard `if scalar.iter().all(|&b| b == 0) { return Err(InvalidScalar) }` | `electrum_crypto.rs:309-310` |
| L18 | `ecies_decrypt_message` is `pub fn` | `electrum_crypto.rs:321` |
| L19 | `BIE1_KAT1` valid-blob test const (reuse for T7/T8) | `electrum_crypto.rs:668` |
| L20 | existing `ecies_decrypt_message_electrum_kat_short_vectors` regression test | `electrum_crypto.rs:700-707` |
| L21 | toolkit version `0.64.0` | `crates/mnemonic-toolkit/Cargo.toml:3` |
| L22 | README markers `<!-- toolkit-version: 0.64.0 -->` | `README.md:13`, `crates/mnemonic-toolkit/README.md:9` |
| L23c | install.sh self-pin `mnemonic-toolkit-v0.64.0` | `scripts/install.sh:32` |
| L24 | fuzz Cargo.lock toolkit version `0.64.0` | `fuzz/Cargo.lock:575` |
| L25 | CHANGELOG head `## mnemonic-toolkit [0.64.0] — 2026-06-21` | `CHANGELOG.md:9` |
| L26 | bughunt report M8 checkbox `### - [ ] M8 ·` | `design/agent-reports/constellation-bughunt-2026-06-20.md:721` |
| L27 | bughunt report L23 checkbox `### - [ ] L23 ·` | `design/agent-reports/constellation-bughunt-2026-06-20.md:830` |
| L28 | `readme_version_current.rs` auto-gates README markers vs `CARGO_PKG_VERSION` (no hand-edit drift) | `crates/mnemonic-toolkit/tests/readme_version_current.rs:24-28` |
| L29 | test fixtures K1–K5 (`[fp/path]xpub`), XPUB (`:307`), WIF (`:306`), RAWHEX (`:311`) | `tests/cli_build_descriptor.rs:29-33,306-311` |

---

## Phase order + disjointness

| Phase | Scope | Files touched | Depends on |
|---|---|---|---|
| **P1 — M8 guard** | fail-closed reject of a post-origin-strip xpub body carrying extra `/`-segments | `descriptor_builder/gate.rs` (+ `tests/cli_build_descriptor.rs`, `gate.rs` `#[cfg(test)]`) | — |
| **P2 — L23** | zero-scalar typed reject (not panic) | `electrum_crypto.rs` (+ its `#[cfg(test)]`) | — |
| **P3 — ship** | version bump + FOLLOWUP/report tick | `Cargo.toml`, both READMEs, `scripts/install.sh`, `fuzz/Cargo.lock`, `CHANGELOG.md`, bughunt report | P1, P2 GREEN |

**Disjointness:** P1 touches ONLY `descriptor_builder/gate.rs` + its tests; P2 touches ONLY `electrum_crypto.rs` + its tests. No shared symbols, no shared test module — P1 and P2 are firewalled (different file, different test, no shared code; spec §3.2 "Firewalling"). P3 touches only version-site / report files. The spec's chosen M8 fix-site is `gate.rs::check_secret_key` extended in place (D2); note the spec §3.1 mentions `ir.rs` as the renderer that OWNS the suffix, but the **fix is in `gate.rs` only** — `ir.rs` is NOT edited (the guard refuses before render).

**Merge-conflict note (WS-DECAY adjacency, spec §6):** WS-DECAY (`descriptor_builder/archetype.rs:305-317` BIP-68 normalization) shares the `descriptor_builder/` file zone but a DIFFERENT file (`archetype.rs`) than P1's `gate.rs`. If WS-DECAY is in flight, serialize or flag the zone overlap (recon obs #5). Cycle-7 itself does not edit `archetype.rs`.

---

## Phase 1 — M8 guard (`gate.rs::check_secret_key`)

**Goal (D1–D7, D16):** an account-level cosigner key carrying an extra in-key derivation tail (`…xpub.../5`, `/5/6`, `/0h`, `/<0;1>`, `/*`) FAILS CLOSED (exit 2, no descriptor emitted) instead of being silently rendered to a wrong (`…/5/<0;1>/*`) subtree. One guard in `check_secret_key` covers BOTH intake paths (preset + spec) and EVERY key-bearing node (Pk/Pkh + each Multi/Sortedmulti key, nested anywhere) because `validate_fields` → `check_secret_key` is the unique pre-emission gate (L4–L7; R0 review "no bypass" conclusion).

### P1 RED-first tests (write first)

All M8 CLI tests in `crates/mnemonic-toolkit/tests/cli_build_descriptor.rs`; gate-unit assertions may also go in `gate.rs` `#[cfg(test)]`. Recon confirmed a clean baseline (no existing fixture passes a trailing-suffix key — L29: K1–K5/XPUB are all bare `[fp/path]xpub` with zero post-`]` `/`), so the new tests are genuinely RED today (accepted-with-wrong-subtree) → GREEN after the guard.

| # | Test | Today (RED) | After fix (GREEN) |
|---|---|---|---|
| **T1** | `--key '[fp/84h/0h/0h]xpub.../5'` via a single-sig **preset** (`--archetype`) → **exit 2**, diagnostic `kind=schema_field`, message names the extra-derivation-path, key body NOT echoed | accepted, renders `…/5/<0;1>/*` (wrong subtree) | exit 2, refused |
| **T1b** *(Minor-1, plan-R0 I-1 corrected)* | The **preset** M8 refusal carries flag-provenance via `resolve_flag(def, node_path, SchemaField)` — assert the ACTUAL per-archetype annotation: **single-key archetypes → `flag=--key`**; **quorum archetypes (kofn-recovery / tiered-recovery / decaying-multisig) → `flag=--threshold`** (the `("root.<quorum>[0]", Some(SchemaField), THRESHOLD)` provenance override wins the `max_by_key((prefix.len(), k.is_some()))` tiebreak — because the M8 reject reuses `SchemaField` kind, NOT `SecretKey`; the xprv reject escapes via its `SecretKey` kind, which the threshold override's kind-filter excludes). **This is a provenance-system artifact, NOT a defect: the diagnostic `path` (`root.<quorum>[0].multi.keys[i]`) and `message` correctly identify the offending key, and the exit-2 refusal fires regardless of the `flag` annotation — funds-safety is unaffected.** Assert BOTH: a single-sig preset → `flag=--key`; a quorum preset (e.g. decaying-multisig) → `flag=--threshold` (the test must cover a quorum archetype, not just single-key, else it masks the real behavior). | n/a (key accepted) | per-archetype `flag` as above; path+message name the key; exit 2 |
| **T2** | A `--spec` JSON `PolicyNode::Pk` whose key is `xpub.../5`, AND a `Multi.keys[i]` variant with `xpub.../5` → **exit 2**, same `schema_field` diagnostic; for the Multi case assert the diagnostic `node_path` is the `{path}.{kind}.keys[i]` form (e.g. `root.multi.keys[0]`) — Minor-2 path-fidelity pin | accepted (wrong subtree) | exit 2, refused |
| **T3** *(positive control / over-rejection guard, D16)* | A NORMAL account-level key `[fp/84h/0h/0h]xpub…` (bare body, no trailing derivation) via `--key` AND via `--spec` → **STILL BUILDS** (exit 0, descriptor emitted) | builds | builds (no over-rejection) |
| **T4** *(asymmetry pin)* | A key ending in `*` (`xpub.../*`) → **STILL refused, exit 2** (now caught at step-1 by the new suffix guard rather than step-2 `InvalidWildcardInDerivationPath`). Assert exit 2; tolerant of WHICH step refuses. | refused at step 2 | refused at step 1 |
| **T5** *(multi-segment + hardened)* | `xpub.../5/6` and `xpub.../0h` → **exit 2** (any trailing path tail) | accepted (wrong subtree) | exit 2, refused |
| **T5b** *(recursion coverage)* | A key with an extra suffix nested under `and_v` / `thresh` / `andor` (a deeper subtree node, reached via `child_paths` L7) → **exit 2** | accepted (wrong subtree) | exit 2, refused |
| **T6** *(no-leak, D6)* | The M8 refusal message + full stdout/stderr does NOT contain the xpub body bytes (assert the key body string is absent) | n/a | message names the path-issue only |

T1 (preset) and T2 (spec, incl. Multi) are BOTH mandatory — they pin that the SINGLE guard covers BOTH intake paths (L6/L10/L11). T5b pins nested-node coverage (L7).

### P1 implementation (after RED confirmed)

In `gate.rs::check_secret_key` (L1, `:347-361`), AFTER the existing `[origin]`-strip (L2) + xprv-prefix screen (L3), add a sibling step-1 condition (D2, D3, D7):

- Predicate (D3): after `let key_part = key.rsplit(']').next().unwrap_or(key);`, reject when `key_part.contains('/')`. Rationale (spec §3.1): a legitimate account-level key's post-bracket body has ZERO `/` (the only legitimate pre-key `/`-bearing token, the `[origin]` path, lives INSIDE the brackets and is stripped); ANY `/` is a smuggled derivation tail (the M8 class). Catches `/5`, `/5/6`, `/0h`, `/<0;1>`, `/*`.
- Ordering (D7): emit the xprv diagnostic when `is_xprv`, **ELSE** the suffix diagnostic — `if is_xprv { … } else if key_part.contains('/') { … }`. A single key never double-reports; xprv (secret-leak) takes precedence. Both are step-1, both exit 2.
- Diagnostic kind (D4, D5 — REUSE, zero `--json` delta): emit the suffix reject via the EXISTING `field_diag(path, message)` helper (L9), which constructs `DiagnosticKind::SchemaField` + `flag: None`. **NO new `DiagnosticKind`** → no new `as_str` discriminant (L8 unchanged) → no `--json` wire-shape delta → no GUI self-update / paired-PR burden.
- **Minor-2 (path-fidelity) — load-bearing:** the existing xprv arm builds its `Diagnostic{…}` struct INLINE (it needs `kind: SecretKey`). The NEW suffix arm instead calls `field_diag(path, msg)`. The `path` passed to `field_diag` MUST be the SAME node-addressed `path` arg already threaded into `check_secret_key` — for a Multi/Sortedmulti key that is the `{path}.{kind}.keys[i]` string built at the `validate_fields` `:240-244` call-site (L5), NOT a bare `path` and NOT a hardcoded one. Since `check_secret_key`'s `path` parameter already carries that node-addressed value for every call-site (Pk/Pkh: `path` L4; Multi key: `keys[i]` form L5), simply passing `path` through to `field_diag(path, …)` is correct — but the implementer MUST verify the arg is `path` (the fn param), exactly as the xprv arm uses `node_path: path.to_string()`. T2's `root.multi.keys[0]` assertion pins this.
- Error message (D6, no-leak — NEVER echoes the key): the message names the path-issue + the contract, e.g.:
  > `"{kind} key carries an extra derivation path; build-descriptor accepts only a bare account-level key ([fp/path]xpub…) — the builder appends the /<0;1>/* receive/change suffix itself"`
  No `flag` set by `field_diag` (it hardcodes `flag: None`); the preset path's `resolve_flag` then annotates the flag downstream (L10/L13) — **`--key` for single-key archetypes, `--threshold` for quorum archetypes** (a provenance-override artifact of reusing `SchemaField` kind; the path+message correctly name the key, exit-2 fires regardless — see T1b, plan-R0 I-1). The diagnostic `path`/`message` are the load-bearing user signal; the `flag` annotation is secondary.
- Edges (spec §3.1): bracketless bare `xpub.../5` (no `]`) → `key_part = "xpub.../5"`, `.contains('/')` true → rejected (covered). Empty `key_part` (malformed `…]` with nothing after) → `.contains('/')` false → falls through to step-2 `from_str` refusal (no regression; not the M8 class).

### P1 gate
`cargo test -p mnemonic-toolkit` GREEN (T1–T6 + T1b/T5b pass; T3 + all pre-existing build_descriptor / gate tests still pass — no over-rejection regression) + `cargo clippy --all-targets -- -D warnings` clean.

---

## Phase 2 — L23 (ecies zero-scalar → typed error, not panic)

**Goal (D8, D9):** `ecies_decrypt_message` returns a typed `EciesDecryptError::InvalidScalar` on a zero private scalar instead of panicking at the `.mul_tweak(...).expect(...)`. Latent — **NOT CLI-reachable today**: the sole in-tree caller `derive_storage_eckey` already rejects zero (L17), so the panic is unreachable from the CLI; this is robustness for a future / downstream-library caller of the `pub fn` (L18). Note this explicitly in the CHANGELOG/report.

### P2 RED-first tests (write first)

In `electrum_crypto.rs` `#[cfg(test)]`:

| # | Test | Today (RED) | After fix (GREEN) |
|---|---|---|---|
| **T7** | `ecies_decrypt_message(BIE1_KAT1, &[0u8; 32])` → `Err(EciesDecryptError::InvalidScalar)`, NOT a panic. (Reuse the valid `BIE1_KAT1` blob (L19) so the call gets past the base64 / length / magic / ephemeral-pubkey gates to the scalar step; vary only the `privkey` arg to all-zero.) | panics at `.expect` (L15) | typed `Err(InvalidScalar)` |
| **T8** *(regression)* | the existing `ecies_decrypt_message_electrum_kat_short_vectors` KAT (L20) still passes (zero-guard does not perturb the valid path) | passes | passes |

### P2 implementation (after RED confirmed)

In `ecies_decrypt_message` (L18, `:321`), at the scalar check (L14, `:345`): add an explicit zero-scalar reject BEFORE `mul_tweak` (L15), mirroring `derive_storage_eckey`'s guard (L17, D8):
```rust
if privkey.iter().all(|&b| b == 0) {
    return Err(EciesDecryptError::InvalidScalar);
}
```
placed immediately before (or folded into) line `:345`. Reuses the EXISTING `InvalidScalar` variant (L16, D9) — **no new variant, no wire/CLI change.** The `.mul_tweak(...).expect(...)` (L15) STAYS as the defensive prime-order-group invariant assertion (now provably holds: the scalar is guaranteed `[1, n-1]`); its comment becomes accurate. (Chosen over `map_err`-ing the `mul_tweak` result — spec §3.2 / D8: the explicit pre-reject localizes failure at the input boundary and keeps `.expect` a true invariant guard.)

### P2 gate
`cargo test -p mnemonic-toolkit` GREEN (T7/T8 pass; all pre-existing electrum_crypto tests pass) + `cargo clippy --all-targets -- -D warnings` clean.

---

## Phase 3 — ship (version bump + report tick)

**Depends on P1 + P2 GREEN.** No publish (toolkit is not on crates.io). No schema_mirror / manual / sibling-codec lockstep (confirmed below).

### Version bump `0.64.0 → 0.65.0` (D10; MINOR — M8 newly rejects previously-accepted input = FORMAL, MINOR pre-1.0; L23 rides the bump)

Edit ALL of (re-grep at impl time — lines are `8d2fe505` snapshots):
1. `crates/mnemonic-toolkit/Cargo.toml:3` — `version = "0.65.0"` (L21).
2. `README.md:13` — `<!-- toolkit-version: 0.65.0 -->` (L22).
3. `crates/mnemonic-toolkit/README.md:9` — `<!-- toolkit-version: 0.65.0 -->` (L22). *(Both README markers are auto-gated by `readme_version_current.rs` (L28) against `CARGO_PKG_VERSION` — a missed marker fails `cargo test`; the per-phase gate catches it.)*
4. `scripts/install.sh:32` — `mnemonic-toolkit-v0.65.0` self-pin (L23c).
5. `fuzz/Cargo.lock:575` — toolkit dep `version = "0.65.0"` (L24). *(Edit the lock entry directly or regenerate; this is a known silent-drift site — `project_toolkit_release_ritual_version_sites`.)*
6. `CHANGELOG.md` — prepend a `## mnemonic-toolkit [0.65.0] — <date>` section above the `:9` `[0.64.0]` head (L25). Note both findings; flag L23 as a **latent / not-CLI-reachable** robustness fix.

### FOLLOWUP / report tick (in the shipping commit — status-discipline: flip in the SAME commit)

The two slugs live ONLY in the bughunt report (no `design/FOLLOWUPS.md` entry exists for either — grep-confirmed). Tick both checkboxes there (re-grep the current line numbers at impl time — they were `:721` / `:830` at `8d2fe505`, L26/L27):
- `design/agent-reports/constellation-bughunt-2026-06-20.md` — M8 row (`### - [ ] M8 ·` → `### - [x] M8 ·`), mark FIXED in 0.65.0 (slug `w3-tk-descbuild-key-extra-path-suffix-silent`).
- same file — L23 row (`### - [ ] L23 ·` → `### - [x] L23 ·`), mark FIXED in 0.65.0 (slug `w3-tk-electrum-crypto-01`).

### Lockstep confirmations (NONE triggered — D11–D15)

- **`schema_mirror` (GUI flag-NAME gate): NOT triggered.** No clap flag / subcommand / dropdown-value add/remove/rename — only validation tightens (`--key`/`--spec`/`--archetype` surface unchanged). (D11)
- **`--json` wire-shape: UNCHANGED.** Reuse of `SchemaField` (D4) adds no new `DiagnosticKind::as_str` discriminant → no GUI self-update / paired-PR item. (D12)
- **Manual mirror (`docs/manual/src/40-cli-reference/41-mnemonic.md`): NOT gate-triggered** (no `--help`/flag-set change; `docs/manual/tests/lint.sh` checks flag presence only). An optional one-line behavioral note MAY be added but is not gate-required. (D13)
- **`tests/bitcoind_differential.rs` oracle: N/A — do NOT block.** It is a `bundle→restore` round-trip keyed on a given descriptor string; it never invokes `build-descriptor`. The M8 fix is a refusal (no descriptor emitted); the refusal is pinned directly by P1's unit/CLI tests. (D14)
- **Sibling-codec companion (md/mk/ms): NONE.** Both findings are pure toolkit; no sibling surface touched; no `design/FOLLOWUPS.md` companion-mirror. (D15)

### P3 gate
Full `cargo test -p mnemonic-toolkit` GREEN (incl. `readme_version_current` + any version-marker tests) + `cargo clippy --all-targets -- -D warnings` clean. Stage paths EXPLICITLY (no `git add -A`); the slug-flip lands in the shipping commit.

---

## Mandatory post-implementation gate (non-deferrable)

After P1–P3 GREEN, an **independent adversarial whole-diff execution review** over the entire cycle-7 diff (this is distinct from the plan-doc R0 — it catches implementation-introduced regressions TDD misses). Persist the review verbatim to `design/agent-reports/cycle7-<phase/whole-diff>-review.md` BEFORE any fold. Funds-focus checklist:
1. **No key-intake bypass** — the M8 guard sits on the unique pre-emission gate; every key-bearing node (Pk/Pkh + each Multi/Sortedmulti key, nested under and_v/thresh/andor/wrap) reaches it; no render-to-emit path skips `validate_fields`.
2. **No over-rejection of a legit key** — `key_part.contains('/')` is true ONLY for the trailing-derivation class; a bare `[origin]xpub` (K1–K5/XPUB) still builds via BOTH intake paths (T3).
3. **No key leak** — the M8 refusal message + all stdout/stderr never echo the xpub body (T6).
4. **L23 no panic** — zero scalar yields a typed `Err(InvalidScalar)`, never a `.expect` panic; the valid KAT path is unperturbed (T7/T8).
5. **No `--json` / schema_mirror / manual drift** — confirm no new `DiagnosticKind` discriminant slipped in; no clap-surface change.

If Agent-API dispatch fails mid-session, flag it explicitly and defer the formal review to API recovery — never silently substitute inline self-review.

---

## R0 gate (this plan-doc)

DESIGN ONLY — no code. Per CLAUDE.md Conventions: this plan-doc MUST pass an opus-architect **R0 review and converge to 0 Critical / 0 Important** BEFORE any implementation. Fold findings → persist the review verbatim to `design/agent-reports/` → re-dispatch → repeat until GREEN (the reviewer-loop continues after every fold). No code, phase advance, tag, or ship while any Critical or Important finding is open.
